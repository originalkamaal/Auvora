//! `auvora.jobs`: project-level background jobs. Enqueued in a tenant
//! transaction; claimed across projects only with a `PlatformScope`
//! (`crate::platform::claim_jobs`).

use auvora_core::id::{Id, Kind};
use serde_json::Value;
use uuid::Uuid;

use crate::{StoreError, TenantTx};

/// A job row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Job {
    /// Job ID (`job_`).
    pub id: Id,
    /// Owning project.
    pub project_id: Uuid,
    /// Job kind (handler name).
    pub kind: String,
    /// Handler input.
    pub payload: Value,
    /// `queued`, `running`, `done` or `failed`.
    pub state: String,
}

/// Enqueues a job to run at `run_at_us` (microseconds since the Unix epoch).
///
/// # Errors
/// [`StoreError::ProjectScopeRequired`] under a tenant scope.
pub async fn enqueue(
    tx: &mut TenantTx,
    id: Id,
    kind: &str,
    payload: &Value,
    run_at_us: i64,
) -> Result<Job, StoreError> {
    let project_id = tx.project_id()?;
    let tenant_id = tx.scope().tenant_id();
    sqlx::query(
        "INSERT INTO auvora.jobs (id, tenant_id, project_id, kind, payload, run_at) \
         VALUES ($1, $2, $3, $4, $5, TIMESTAMPTZ 'epoch' + $6 * INTERVAL '1 microsecond')",
    )
    .bind(id.uuid())
    .bind(tenant_id)
    .bind(project_id)
    .bind(kind)
    .bind(payload)
    .bind(run_at_us)
    .execute(tx.conn())
    .await?;
    Ok(Job {
        id,
        project_id,
        kind: kind.to_owned(),
        payload: payload.clone(),
        state: "queued".to_owned(),
    })
}

/// Reads a job.
///
/// # Errors
/// [`StoreError::NotFound`] if it isn't in the scope's project.
pub async fn get(tx: &mut TenantTx, id: Uuid) -> Result<Job, StoreError> {
    let project_id = tx.project_id()?;
    let row: Option<(Uuid, Uuid, String, Value, String)> =
        sqlx::query_as("SELECT id, project_id, kind, payload, state FROM auvora.jobs WHERE project_id = $1 AND id = $2")
            .bind(project_id)
            .bind(id)
            .fetch_optional(tx.conn())
            .await?;
    let (id, project_id, kind, payload, state) = row.ok_or(StoreError::NotFound)?;
    Ok(Job {
        id: Id::from_uuid(Kind::Job, id),
        project_id,
        kind,
        payload,
        state,
    })
}
