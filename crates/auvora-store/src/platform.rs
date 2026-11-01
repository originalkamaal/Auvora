//! Platform scope: the only RLS-bypassing access (tenancy-rls.md §3.3).
//!
//! Every acquisition writes one `auvora.platform_audit_events` row in the
//! same transaction. `scripts/check-store-rules` fails CI if
//! `PlatformScope::acquire` is called outside the allowlisted modules.

use auvora_core::id::{Id, Kind};
use auvora_core::time::Clock;
use serde_json::{Value, json};
use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::StoreError;

/// Why platform access is needed. Each reason is recorded in the platform audit chain.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PlatformReason {
    /// Retention purge (ADR-0109).
    RetentionPurge,
    /// Usage metering rollup.
    MeteringRollup,
    /// Moving a project between cells (ADR-0019).
    CellMigration,
    /// Publishing a signed config snapshot (ADR-0009).
    ConfigSnapshotPublish,
    /// Scheduled key rotation sweep.
    KeyRotationSweep,
    /// Claiming jobs across projects for dispatch.
    JobDispatch,
    /// A cross-project data backfill in a migration.
    MigrationBackfill {
        /// Migration name.
        migration: &'static str,
    },
    /// A support-access read by platform staff (ADR-0057); requires a ticket.
    PlatformAdminRead {
        /// Support ticket reference.
        ticket: String,
    },
}

impl PlatformReason {
    /// Stable reason code stored in the audit chain.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::RetentionPurge => "retention_purge",
            Self::MeteringRollup => "metering_rollup",
            Self::CellMigration => "cell_migration",
            Self::ConfigSnapshotPublish => "config_snapshot_publish",
            Self::KeyRotationSweep => "key_rotation_sweep",
            Self::JobDispatch => "job_dispatch",
            Self::MigrationBackfill { .. } => "migration_backfill",
            Self::PlatformAdminRead { .. } => "platform_admin_read",
        }
    }

    fn detail(&self) -> Option<String> {
        match self {
            Self::MigrationBackfill { migration } => Some((*migration).to_owned()),
            Self::PlatformAdminRead { ticket } => Some(ticket.clone()),
            _ => None,
        }
    }
}

/// Who is acting with platform scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorRef {
    /// Actor type, e.g. `worker`, `admin`.
    pub kind: &'static str,
    /// Actor identifier (worker name or admin principal ID).
    pub id: String,
}

/// Connection to the database as `auvora_platform`. Optional at runtime:
/// platform jobs are disabled when no platform URL is configured.
#[derive(Debug, Clone)]
pub struct PlatformStore {
    pool: PgPool,
}

impl PlatformStore {
    /// Connects as the platform role.
    ///
    /// # Errors
    /// [`StoreError::Database`] on connection failure.
    pub async fn connect(url: &str, max_connections: u32) -> Result<Self, StoreError> {
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .connect(url)
            .await?;
        Ok(Self { pool })
    }
}

/// Capability for RLS-bypassing work. The only constructor audits its use.
#[derive(Debug)]
pub struct PlatformScope {
    _sealed: (),
}

impl PlatformScope {
    /// Opens a platform transaction and records why, by whom and when.
    ///
    /// # Errors
    /// [`StoreError::Database`] if the transaction or the audit insert fails.
    pub async fn acquire(
        reason: PlatformReason,
        actor: ActorRef,
        store: &PlatformStore,
        clock: &dyn Clock,
    ) -> Result<PlatformTx, StoreError> {
        let mut tx = store.pool.begin().await?;
        let id = Id::generate(Kind::Event, clock)
            .map_err(|_| StoreError::Invalid("entropy unavailable"))?;
        let actor_json: Value = json!({ "type": actor.kind, "id": actor.id });
        sqlx::query(
            "INSERT INTO auvora.platform_audit_events (id, time, reason, detail, actor) \
             VALUES ($1, TIMESTAMPTZ 'epoch' + $2 * INTERVAL '1 microsecond', $3, $4, $5)",
        )
        .bind(id.uuid())
        .bind(clock.unix_micros())
        .bind(reason.code())
        .bind(reason.detail())
        .bind(actor_json)
        .execute(&mut *tx)
        .await?;
        Ok(PlatformTx {
            tx,
            reason,
            _scope: Self { _sealed: () },
        })
    }
}

/// An open RLS-bypassing transaction. Only functions in this module accept it.
#[derive(Debug)]
pub struct PlatformTx {
    tx: Transaction<'static, Postgres>,
    reason: PlatformReason,
    _scope: PlatformScope,
}

impl PlatformTx {
    /// The recorded reason.
    #[must_use]
    pub const fn reason(&self) -> &PlatformReason {
        &self.reason
    }

    /// Commits the platform work together with its audit record.
    ///
    /// # Errors
    /// [`StoreError::Database`] if the commit fails.
    pub async fn commit(self) -> Result<(), StoreError> {
        self.tx.commit().await.map_err(StoreError::from)
    }
}

/// A job claimed for execution. The handler then runs it in its own project's
/// `TenantTx` (tenancy-rls.md §6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimedJob {
    /// Job ID.
    pub id: Uuid,
    /// Tenant to scope the handler to.
    pub tenant_id: Uuid,
    /// Project to scope the handler to.
    pub project_id: Uuid,
    /// Job kind.
    pub kind: String,
}

/// Claims up to `limit` due jobs across all projects with `FOR UPDATE SKIP LOCKED`.
///
/// # Errors
/// [`StoreError::Database`] on query failure.
pub async fn claim_jobs(
    ptx: &mut PlatformTx,
    worker: &str,
    limit: i64,
    lease_us: i64,
    clock: &dyn Clock,
) -> Result<Vec<ClaimedJob>, StoreError> {
    let rows: Vec<(Uuid, Uuid, Uuid, String)> = sqlx::query_as(
        "WITH due AS ( \
           SELECT id FROM auvora.jobs \
           WHERE state = 'queued' AND run_at <= TIMESTAMPTZ 'epoch' + $1 * INTERVAL '1 microsecond' \
           ORDER BY run_at LIMIT $2 FOR UPDATE SKIP LOCKED) \
         UPDATE auvora.jobs j SET state = 'running', attempts = attempts + 1, locked_by = $3, \
           locked_until = TIMESTAMPTZ 'epoch' + ($1 + $4) * INTERVAL '1 microsecond' \
         FROM due WHERE j.id = due.id \
         RETURNING j.id, j.tenant_id, j.project_id, j.kind",
    )
    .bind(clock.unix_micros())
    .bind(limit)
    .bind(worker)
    .bind(lease_us)
    .fetch_all(&mut *ptx.tx)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(id, tenant_id, project_id, kind)| ClaimedJob {
            id,
            tenant_id,
            project_id,
            kind,
        })
        .collect())
}
