//! `auvora.tenants`: tenant-level table; a scope can see only its own tenant.

use auvora_core::id::{Id, Kind};

use crate::{StoreError, TenantTx};

/// A tenant row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tenant {
    /// Tenant ID.
    pub id: Id,
    /// Installation-unique slug.
    pub slug: String,
    /// Display name.
    pub name: String,
    /// `active`, `suspended` or `deleted`.
    pub status: String,
}

/// Creates the scope's own tenant. The ID is the scope's tenant ID.
///
/// # Errors
/// [`StoreError::Conflict`] if the tenant already exists.
pub async fn create(tx: &mut TenantTx, slug: &str, name: &str) -> Result<Tenant, StoreError> {
    let id = tx.scope().tenant_id();
    sqlx::query("INSERT INTO auvora.tenants (id, slug, name) VALUES ($1, $2, $3)")
        .bind(id)
        .bind(slug)
        .bind(name)
        .execute(tx.conn())
        .await?;
    Ok(Tenant {
        id: Id::from_uuid(Kind::Tenant, id),
        slug: slug.to_owned(),
        name: name.to_owned(),
        status: "active".to_owned(),
    })
}

/// Reads the scope's own tenant.
///
/// # Errors
/// [`StoreError::NotFound`] if it doesn't exist.
pub async fn get(tx: &mut TenantTx) -> Result<Tenant, StoreError> {
    let tenant_id = tx.scope().tenant_id();
    let row: Option<(uuid::Uuid, String, String, String)> =
        sqlx::query_as("SELECT id, slug, name, status FROM auvora.tenants WHERE id = $1")
            .bind(tenant_id)
            .fetch_optional(tx.conn())
            .await?;
    let (id, slug, name, status) = row.ok_or(StoreError::NotFound)?;
    Ok(Tenant {
        id: Id::from_uuid(Kind::Tenant, id),
        slug,
        name,
        status,
    })
}
