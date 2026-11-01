//! `auvora.principals`: project-level table.

use auvora_core::id::{Id, Kind};
use uuid::Uuid;

use crate::{StoreError, TenantTx};

/// The principal kinds (domain-model.md §3, ADR-0047 vocabulary).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrincipalKind {
    /// Human user.
    User,
    /// Auvora-managed non-human principal.
    ServiceAccount,
    /// Externally attested workload.
    Workload,
    /// AI agent.
    Agent,
    /// Anonymous (guest) user.
    Anonymous,
}

impl PrincipalKind {
    /// Every kind.
    pub const ALL: [Self; 5] = [
        Self::User,
        Self::ServiceAccount,
        Self::Workload,
        Self::Agent,
        Self::Anonymous,
    ];

    /// Database value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::ServiceAccount => "service_account",
            Self::Workload => "workload",
            Self::Agent => "agent",
            Self::Anonymous => "anonymous",
        }
    }

    /// External ID kind for this principal kind (ADR-0018).
    #[must_use]
    pub const fn id_kind(self) -> Kind {
        match self {
            Self::User => Kind::User,
            Self::ServiceAccount => Kind::ServiceAccount,
            Self::Workload => Kind::Workload,
            Self::Agent => Kind::Agent,
            Self::Anonymous => Kind::AnonymousUser,
        }
    }

    fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|k| k.as_str() == s)
    }
}

/// A principal row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Principal {
    /// External ID; its prefix follows the kind.
    pub id: Id,
    /// Owning project.
    pub project_id: Id,
    /// Principal kind.
    pub kind: PrincipalKind,
    /// `active`, `suspended` or `deleted`.
    pub status: String,
}

type Row = (Uuid, Uuid, String, String);

fn from_row((id, project_id, kind, status): Row) -> Result<Principal, StoreError> {
    let kind = PrincipalKind::parse(&kind).ok_or(StoreError::NotFound)?;
    Ok(Principal {
        id: Id::from_uuid(kind.id_kind(), id),
        project_id: Id::from_uuid(Kind::Project, project_id),
        kind,
        status,
    })
}

/// Creates a principal in the scope's project.
///
/// # Errors
/// [`StoreError::ProjectScopeRequired`] under a tenant-level scope.
pub async fn create(
    tx: &mut TenantTx,
    id: Id,
    kind: PrincipalKind,
) -> Result<Principal, StoreError> {
    let project_id = tx.project_id()?;
    let tenant_id = tx.scope().tenant_id();
    sqlx::query(
        "INSERT INTO auvora.principals (id, tenant_id, project_id, kind) VALUES ($1, $2, $3, $4)",
    )
    .bind(id.uuid())
    .bind(tenant_id)
    .bind(project_id)
    .bind(kind.as_str())
    .execute(tx.conn())
    .await?;
    Ok(Principal {
        id: Id::from_uuid(kind.id_kind(), id.uuid()),
        project_id: Id::from_uuid(Kind::Project, project_id),
        kind,
        status: "active".to_owned(),
    })
}

/// Reads a principal by ID.
///
/// # Errors
/// [`StoreError::NotFound`] if it isn't in the scope's project.
pub async fn get(tx: &mut TenantTx, id: Uuid) -> Result<Principal, StoreError> {
    let project_id = tx.project_id()?;
    let row: Option<Row> = sqlx::query_as(
        "SELECT id, project_id, kind, status FROM auvora.principals WHERE project_id = $1 AND id = $2",
    )
    .bind(project_id)
    .bind(id)
    .fetch_optional(tx.conn())
    .await?;
    row.map_or(Err(StoreError::NotFound), from_row)
}

/// Lists the project's principals, oldest first.
///
/// # Errors
/// [`StoreError::ProjectScopeRequired`] under a tenant-level scope.
pub async fn list(tx: &mut TenantTx) -> Result<Vec<Principal>, StoreError> {
    let project_id = tx.project_id()?;
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT id, project_id, kind, status FROM auvora.principals WHERE project_id = $1 ORDER BY id",
    )
    .bind(project_id)
    .fetch_all(tx.conn())
    .await?;
    rows.into_iter().map(from_row).collect()
}

/// Sets a principal's status (`active`, `suspended` or `deleted`).
///
/// # Errors
/// [`StoreError::NotFound`] if it isn't in the scope's project.
pub async fn set_status(tx: &mut TenantTx, id: Uuid, status: &str) -> Result<(), StoreError> {
    let project_id = tx.project_id()?;
    let done = sqlx::query(
        "UPDATE auvora.principals SET status = $3, version = version + 1 WHERE project_id = $1 AND id = $2",
    )
    .bind(project_id)
    .bind(id)
    .bind(status)
    .execute(tx.conn())
    .await?;
    if done.rows_affected() == 0 {
        return Err(StoreError::NotFound);
    }
    Ok(())
}

/// Hard-deletes a principal and (by cascade) its identifiers.
///
/// # Errors
/// [`StoreError::NotFound`] if it isn't in the scope's project.
pub async fn delete(tx: &mut TenantTx, id: Uuid) -> Result<(), StoreError> {
    let project_id = tx.project_id()?;
    let done = sqlx::query("DELETE FROM auvora.principals WHERE project_id = $1 AND id = $2")
        .bind(project_id)
        .bind(id)
        .execute(tx.conn())
        .await?;
    if done.rows_affected() == 0 {
        return Err(StoreError::NotFound);
    }
    Ok(())
}
