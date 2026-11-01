//! `auvora.projects`: tenant-level table; a scope sees its tenant's projects.

use auvora_core::id::{Id, Kind};
use uuid::Uuid;

use crate::{StoreError, TenantTx};

/// A project row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    /// Project ID.
    pub id: Id,
    /// Owning tenant.
    pub tenant_id: Id,
    /// Slug, unique within the tenant.
    pub slug: String,
    /// Display name.
    pub name: String,
    /// OIDC issuer URL.
    pub issuer: String,
}

type Row = (Uuid, Uuid, String, String, String);

fn from_row((id, tenant_id, slug, name, issuer): Row) -> Project {
    Project {
        id: Id::from_uuid(Kind::Project, id),
        tenant_id: Id::from_uuid(Kind::Tenant, tenant_id),
        slug,
        name,
        issuer,
    }
}

/// Creates a project in the scope's tenant.
///
/// # Errors
/// [`StoreError::Conflict`] on a duplicate slug within the tenant.
pub async fn create(
    tx: &mut TenantTx,
    id: Id,
    slug: &str,
    name: &str,
    issuer: &str,
) -> Result<Project, StoreError> {
    let tenant_id = tx.scope().tenant_id();
    sqlx::query("INSERT INTO auvora.projects (id, tenant_id, slug, name, issuer) VALUES ($1, $2, $3, $4, $5)")
        .bind(id.uuid())
        .bind(tenant_id)
        .bind(slug)
        .bind(name)
        .bind(issuer)
        .execute(tx.conn())
        .await?;
    Ok(Project {
        id,
        tenant_id: Id::from_uuid(Kind::Tenant, tenant_id),
        slug: slug.to_owned(),
        name: name.to_owned(),
        issuer: issuer.to_owned(),
    })
}

/// Reads one of the tenant's projects.
///
/// # Errors
/// [`StoreError::NotFound`] if it doesn't exist in this tenant.
pub async fn get(tx: &mut TenantTx, id: Id) -> Result<Project, StoreError> {
    let tenant_id = tx.scope().tenant_id();
    let row: Option<Row> = sqlx::query_as(
        "SELECT id, tenant_id, slug, name, issuer FROM auvora.projects WHERE tenant_id = $1 AND id = $2",
    )
    .bind(tenant_id)
    .bind(id.uuid())
    .fetch_optional(tx.conn())
    .await?;
    row.map(from_row).ok_or(StoreError::NotFound)
}

/// Lists the tenant's projects, oldest first.
///
/// # Errors
/// [`StoreError::Database`] on query failure.
pub async fn list(tx: &mut TenantTx) -> Result<Vec<Project>, StoreError> {
    let tenant_id = tx.scope().tenant_id();
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT id, tenant_id, slug, name, issuer FROM auvora.projects WHERE tenant_id = $1 ORDER BY id",
    )
    .bind(tenant_id)
    .fetch_all(tx.conn())
    .await?;
    Ok(rows.into_iter().map(from_row).collect())
}
