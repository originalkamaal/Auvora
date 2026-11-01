//! `auvora.users`: project-level, 1:1 with a `user` principal.

use serde_json::Value;
use uuid::Uuid;

use crate::{StoreError, TenantTx};

/// A user row (profile fields arrive with `auvora-authn`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    /// The principal this user extends.
    pub principal_id: Uuid,
    /// Owning project.
    pub project_id: Uuid,
    /// Frontend-readable metadata (≤ 8 KB, ADR-0076).
    pub public_metadata: Value,
}

/// Creates the user row for an existing `user` principal.
///
/// # Errors
/// [`StoreError::Database`] if the principal isn't in the scope's project.
pub async fn create(tx: &mut TenantTx, principal_id: Uuid) -> Result<User, StoreError> {
    let project_id = tx.project_id()?;
    let tenant_id = tx.scope().tenant_id();
    sqlx::query(
        "INSERT INTO auvora.users (tenant_id, project_id, principal_id) VALUES ($1, $2, $3)",
    )
    .bind(tenant_id)
    .bind(project_id)
    .bind(principal_id)
    .execute(tx.conn())
    .await?;
    Ok(User {
        principal_id,
        project_id,
        public_metadata: Value::Object(serde_json::Map::new()),
    })
}

/// Reads a user.
///
/// # Errors
/// [`StoreError::NotFound`] if it isn't in the scope's project.
pub async fn get(tx: &mut TenantTx, principal_id: Uuid) -> Result<User, StoreError> {
    let project_id = tx.project_id()?;
    let row: Option<(Uuid, Uuid, Value)> = sqlx::query_as(
        "SELECT principal_id, project_id, public_metadata FROM auvora.users WHERE project_id = $1 AND principal_id = $2",
    )
    .bind(project_id)
    .bind(principal_id)
    .fetch_optional(tx.conn())
    .await?;
    let (principal_id, project_id, public_metadata) = row.ok_or(StoreError::NotFound)?;
    Ok(User {
        principal_id,
        project_id,
        public_metadata,
    })
}

/// Replaces a user's public metadata.
///
/// # Errors
/// [`StoreError::NotFound`] if the user isn't in the scope's project;
/// [`StoreError::Database`] if the document exceeds 8 KB.
pub async fn set_public_metadata(
    tx: &mut TenantTx,
    principal_id: Uuid,
    metadata: &Value,
) -> Result<(), StoreError> {
    let project_id = tx.project_id()?;
    let done = sqlx::query(
        "UPDATE auvora.users SET public_metadata = $3, version = version + 1 WHERE project_id = $1 AND principal_id = $2",
    )
    .bind(project_id)
    .bind(principal_id)
    .bind(metadata)
    .execute(tx.conn())
    .await?;
    if done.rows_affected() == 0 {
        return Err(StoreError::NotFound);
    }
    Ok(())
}
