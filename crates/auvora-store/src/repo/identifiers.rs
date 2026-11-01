//! `auvora.identifiers`: project-level table; uniqueness is per project (ADR-0075).

use auvora_core::id::{Id, Kind};
use uuid::Uuid;

use crate::{StoreError, TenantTx};

/// An identifier to insert. Normalization and blind indexing happen before
/// the store (ADR-0072 §6, ADR-0075); the store only persists the results.
#[derive(Debug, Clone)]
pub struct NewIdentifier<'a> {
    /// New identifier ID (`idn_`).
    pub id: Id,
    /// Owning principal (same project).
    pub principal_id: Uuid,
    /// `email`, `phone`, `username` or `external`.
    pub kind: &'a str,
    /// Keyed blind index of the normalized value.
    pub value_normalized_bidx: &'a [u8],
    /// Display form (plaintext only when encryption at rest is off).
    pub value_display: Option<&'a str>,
}

/// An identifier row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identifier {
    /// Identifier ID.
    pub id: Id,
    /// Owning project.
    pub project_id: Id,
    /// Owning principal (raw UUID; its prefix depends on the principal kind).
    pub principal_id: Uuid,
    /// Identifier type.
    pub kind: String,
    /// Keyed blind index of the normalized value.
    pub value_normalized_bidx: Vec<u8>,
}

type Row = (Uuid, Uuid, Uuid, String, Vec<u8>);

fn from_row((id, project_id, principal_id, kind, bidx): Row) -> Identifier {
    Identifier {
        id: Id::from_uuid(Kind::Identifier, id),
        project_id: Id::from_uuid(Kind::Project, project_id),
        principal_id,
        kind,
        value_normalized_bidx: bidx,
    }
}

/// Creates an identifier.
///
/// # Errors
/// [`StoreError::Conflict`] if the value already exists in the project;
/// [`StoreError::Database`] if the principal isn't in the project.
pub async fn create(tx: &mut TenantTx, new: &NewIdentifier<'_>) -> Result<Identifier, StoreError> {
    let project_id = tx.project_id()?;
    let tenant_id = tx.scope().tenant_id();
    sqlx::query(
        "INSERT INTO auvora.identifiers (id, tenant_id, project_id, principal_id, type, value_normalized_bidx, value_display) \
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(new.id.uuid())
    .bind(tenant_id)
    .bind(project_id)
    .bind(new.principal_id)
    .bind(new.kind)
    .bind(new.value_normalized_bidx)
    .bind(new.value_display)
    .execute(tx.conn())
    .await?;
    Ok(Identifier {
        id: new.id,
        project_id: Id::from_uuid(Kind::Project, project_id),
        principal_id: new.principal_id,
        kind: new.kind.to_owned(),
        value_normalized_bidx: new.value_normalized_bidx.to_vec(),
    })
}

/// Looks up an identifier by type and blind index.
///
/// # Errors
/// [`StoreError::NotFound`] if no such identifier exists in the project.
pub async fn find_by_value(
    tx: &mut TenantTx,
    kind: &str,
    bidx: &[u8],
) -> Result<Identifier, StoreError> {
    let project_id = tx.project_id()?;
    let row: Option<Row> = sqlx::query_as(
        "SELECT id, project_id, principal_id, type, value_normalized_bidx FROM auvora.identifiers \
         WHERE project_id = $1 AND type = $2 AND value_normalized_bidx = $3",
    )
    .bind(project_id)
    .bind(kind)
    .bind(bidx)
    .fetch_optional(tx.conn())
    .await?;
    row.map(from_row).ok_or(StoreError::NotFound)
}

/// Lists a principal's identifiers.
///
/// # Errors
/// [`StoreError::ProjectScopeRequired`] under a tenant-level scope.
pub async fn list_for_principal(
    tx: &mut TenantTx,
    principal_id: Uuid,
) -> Result<Vec<Identifier>, StoreError> {
    let project_id = tx.project_id()?;
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT id, project_id, principal_id, type, value_normalized_bidx FROM auvora.identifiers \
         WHERE project_id = $1 AND principal_id = $2 ORDER BY id",
    )
    .bind(project_id)
    .bind(principal_id)
    .fetch_all(tx.conn())
    .await?;
    Ok(rows.into_iter().map(from_row).collect())
}

/// Deletes an identifier.
///
/// # Errors
/// [`StoreError::NotFound`] if it isn't in the scope's project.
pub async fn delete(tx: &mut TenantTx, id: Uuid) -> Result<(), StoreError> {
    let project_id = tx.project_id()?;
    let done = sqlx::query("DELETE FROM auvora.identifiers WHERE project_id = $1 AND id = $2")
        .bind(project_id)
        .bind(id)
        .execute(tx.conn())
        .await?;
    if done.rows_affected() == 0 {
        return Err(StoreError::NotFound);
    }
    Ok(())
}
