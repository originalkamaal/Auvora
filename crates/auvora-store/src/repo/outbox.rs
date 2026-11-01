//! `auvora.outbox_events`: domain events written in the business transaction
//! and delivered later by workers (ADR-0008). Undelivered rows are never dropped.

use auvora_core::id::{Id, Kind};
use auvora_core::time::Clock;
use serde_json::Value;
use uuid::Uuid;

use crate::{StoreError, TenantTx};

/// An outbox event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutboxEvent {
    /// Event ID (`evt_`).
    pub id: Id,
    /// Owning project.
    pub project_id: Uuid,
    /// Versioned event type, e.g. `auvora.user.created.v1` (ADR-0080).
    pub kind: String,
    /// Subject the event is about.
    pub subject: String,
    /// Event payload.
    pub data: Value,
}

/// Enqueues an event in the caller's transaction.
///
/// # Errors
/// [`StoreError::ProjectScopeRequired`] under a tenant scope.
pub async fn enqueue(
    tx: &mut TenantTx,
    kind: &str,
    subject: &str,
    data: &Value,
    clock: &dyn Clock,
) -> Result<OutboxEvent, StoreError> {
    let project_id = tx.project_id()?;
    let tenant_id = tx.scope().tenant_id();
    let id =
        Id::generate(Kind::Event, clock).map_err(|_| StoreError::Invalid("entropy unavailable"))?;
    sqlx::query(
        "INSERT INTO auvora.outbox_events (id, tenant_id, project_id, created_at, type, subject, data, ordering_key) \
         VALUES ($1, $2, $3, TIMESTAMPTZ 'epoch' + $4 * INTERVAL '1 microsecond', $5, $6, $7, $6)",
    )
    .bind(id.uuid())
    .bind(tenant_id)
    .bind(project_id)
    .bind(clock.unix_micros())
    .bind(kind)
    .bind(subject)
    .bind(data)
    .execute(tx.conn())
    .await?;
    Ok(OutboxEvent {
        id,
        project_id,
        kind: kind.to_owned(),
        subject: subject.to_owned(),
        data: data.clone(),
    })
}

/// Lists the project's undelivered events, oldest first.
///
/// # Errors
/// [`StoreError::ProjectScopeRequired`] under a tenant scope.
pub async fn list_undelivered(
    tx: &mut TenantTx,
    limit: i64,
) -> Result<Vec<OutboxEvent>, StoreError> {
    let project_id = tx.project_id()?;
    let rows: Vec<(Uuid, Uuid, String, String, Value)> = sqlx::query_as(
        "SELECT id, project_id, type, subject, data FROM auvora.outbox_events \
         WHERE project_id = $1 AND delivered_at IS NULL ORDER BY created_at, id LIMIT $2",
    )
    .bind(project_id)
    .bind(limit)
    .fetch_all(tx.conn())
    .await?;
    Ok(rows
        .into_iter()
        .map(|(id, project_id, kind, subject, data)| OutboxEvent {
            id: Id::from_uuid(Kind::Event, id),
            project_id,
            kind,
            subject,
            data,
        })
        .collect())
}
