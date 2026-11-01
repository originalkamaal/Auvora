//! `auvora.environment_groups`: tenant-level; links dev/staging/prod projects.

use auvora_core::id::{Id, Kind};
use uuid::Uuid;

use crate::{StoreError, TenantTx};

/// An environment group.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvironmentGroup {
    /// Group ID (`env_`).
    pub id: Id,
    /// Owning tenant.
    pub tenant_id: Uuid,
    /// Name, unique within the tenant.
    pub name: String,
}

/// Creates a group in the scope's tenant.
///
/// # Errors
/// [`StoreError::Conflict`] on a duplicate name within the tenant.
pub async fn create(tx: &mut TenantTx, id: Id, name: &str) -> Result<EnvironmentGroup, StoreError> {
    let tenant_id = tx.scope().tenant_id();
    sqlx::query("INSERT INTO auvora.environment_groups (id, tenant_id, name) VALUES ($1, $2, $3)")
        .bind(id.uuid())
        .bind(tenant_id)
        .bind(name)
        .execute(tx.conn())
        .await?;
    Ok(EnvironmentGroup {
        id,
        tenant_id,
        name: name.to_owned(),
    })
}

/// Lists the tenant's groups.
///
/// # Errors
/// [`StoreError::Database`] on query failure.
pub async fn list(tx: &mut TenantTx) -> Result<Vec<EnvironmentGroup>, StoreError> {
    let tenant_id = tx.scope().tenant_id();
    let rows: Vec<(Uuid, Uuid, String)> =
        sqlx::query_as("SELECT id, tenant_id, name FROM auvora.environment_groups WHERE tenant_id = $1 ORDER BY id")
            .bind(tenant_id)
            .fetch_all(tx.conn())
            .await?;
    Ok(rows
        .into_iter()
        .map(|(id, tenant_id, name)| EnvironmentGroup {
            id: Id::from_uuid(Kind::EnvironmentLink, id),
            tenant_id,
            name,
        })
        .collect())
}
