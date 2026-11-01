//! `auvora.audit_events`: append-only, hash-chained per project (ADR-0039).
//!
//! `hash(n) = SHA-256(hash(n-1) || canonical_json(event n))`, with 32 zero
//! bytes before the first event. The chain head row in
//! `auvora.audit_chain_heads` is locked for each append, so `seq` is gapless.
//!
//! Event bodies must not contain PII (ADR-0039 §7) or floating-point numbers
//! (canonical JSON here is RFC 8785 for integer-only documents).

use auvora_core::id::{Id, Kind};
use auvora_core::time::Clock;
use aws_lc_rs::digest;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::{StoreError, TenantTx};

/// Length of a chain hash (SHA-256).
pub const HASH_LEN: usize = 32;

/// An audit event to append.
#[derive(Debug, Clone)]
pub struct NewAuditEvent<'a> {
    /// `security`, `admin`, `activity` or `system`.
    pub category: &'a str,
    /// Dotted action name, e.g. `principal.created`.
    pub action: &'a str,
    /// `success`, `failure` or `denied`.
    pub outcome: &'a str,
    /// Who acted (`{type, id, act_chain}`); IDs only, no PII.
    pub actor: Value,
    /// What was acted on (`{type, id}`); IDs only, no PII.
    pub subject: Value,
    /// Extra non-PII context.
    pub metadata: Value,
}

/// A stored audit event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditEvent {
    /// Event ID.
    pub id: Id,
    /// Owning tenant.
    pub tenant_id: Uuid,
    /// Owning project (the chain).
    pub project_id: Id,
    /// Position in the project's chain, starting at 1.
    pub seq: i64,
    /// Event time, microseconds since the Unix epoch.
    pub time_us: i64,
    /// Chain hash after this event.
    pub hash: Vec<u8>,
    /// Category.
    pub category: String,
    /// Action.
    pub action: String,
    /// Outcome.
    pub outcome: String,
    /// Actor.
    pub actor: Value,
    /// Subject.
    pub subject: Value,
    /// Metadata.
    pub metadata: Value,
}

/// Why a chain failed verification.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ChainError {
    /// A `seq` is missing or out of order.
    #[error("gap or reordering at seq {0}")]
    Gap(i64),
    /// The recomputed hash doesn't match the stored one.
    #[error("hash mismatch at seq {0}")]
    HashMismatch(i64),
    /// The body contains a value that can't be canonicalized (e.g. a float).
    #[error("non-canonical body at seq {0}")]
    NonCanonical(i64),
}

/// Appends an event to the scope's project chain, in the caller's transaction.
///
/// # Errors
/// [`StoreError::ProjectScopeRequired`] under a tenant scope;
/// [`StoreError::Invalid`] if the body contains floats.
pub async fn append(
    tx: &mut TenantTx,
    event: &NewAuditEvent<'_>,
    clock: &dyn Clock,
) -> Result<AuditEvent, StoreError> {
    let project_id = tx.project_id()?;
    let tenant_id = tx.scope().tenant_id();

    sqlx::query("INSERT INTO auvora.audit_chain_heads (tenant_id, project_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
        .bind(tenant_id)
        .bind(project_id)
        .execute(tx.conn())
        .await?;
    let (last_seq, last_hash): (i64, Vec<u8>) = sqlx::query_as(
        "SELECT last_seq, last_hash FROM auvora.audit_chain_heads WHERE project_id = $1 FOR UPDATE",
    )
    .bind(project_id)
    .fetch_one(tx.conn())
    .await?;

    let stored = AuditEvent {
        id: Id::generate(Kind::Event, clock)
            .map_err(|_| StoreError::Invalid("entropy unavailable"))?,
        tenant_id,
        project_id: Id::from_uuid(Kind::Project, project_id),
        seq: last_seq
            .checked_add(1)
            .ok_or(StoreError::Invalid("audit seq overflow"))?,
        time_us: clock.unix_micros(),
        hash: Vec::new(),
        category: event.category.to_owned(),
        action: event.action.to_owned(),
        outcome: event.outcome.to_owned(),
        actor: event.actor.clone(),
        subject: event.subject.clone(),
        metadata: event.metadata.clone(),
    };
    let hash = chain_hash(&last_hash, &stored)
        .ok_or(StoreError::Invalid("audit body must not contain floats"))?;
    let stored = AuditEvent { hash, ..stored };

    sqlx::query(
        "INSERT INTO auvora.audit_events \
           (id, tenant_id, project_id, seq, time, hash, category, action, outcome, actor, subject, metadata) \
         VALUES ($1, $2, $3, $4, TIMESTAMPTZ 'epoch' + $5 * INTERVAL '1 microsecond', $6, $7, $8, $9, $10, $11, $12)",
    )
    .bind(stored.id.uuid())
    .bind(tenant_id)
    .bind(project_id)
    .bind(stored.seq)
    .bind(stored.time_us)
    .bind(&stored.hash)
    .bind(&stored.category)
    .bind(&stored.action)
    .bind(&stored.outcome)
    .bind(&stored.actor)
    .bind(&stored.subject)
    .bind(&stored.metadata)
    .execute(tx.conn())
    .await?;
    sqlx::query(
        "UPDATE auvora.audit_chain_heads SET last_seq = $2, last_hash = $3 WHERE project_id = $1",
    )
    .bind(project_id)
    .bind(stored.seq)
    .bind(&stored.hash)
    .execute(tx.conn())
    .await?;
    Ok(stored)
}

type Row = (
    Uuid,
    Uuid,
    Uuid,
    i64,
    i64,
    Vec<u8>,
    String,
    String,
    String,
    Value,
    Value,
    Value,
);

/// Reads the scope's chain from `from_seq` (inclusive), in order.
///
/// # Errors
/// [`StoreError::ProjectScopeRequired`] under a tenant scope.
pub async fn list(
    tx: &mut TenantTx,
    from_seq: i64,
    limit: i64,
) -> Result<Vec<AuditEvent>, StoreError> {
    let project_id = tx.project_id()?;
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT id, tenant_id, project_id, seq, (extract(epoch FROM time) * 1000000)::bigint, hash, \
                category, action, outcome, actor, subject, metadata \
         FROM auvora.audit_events WHERE project_id = $1 AND seq >= $2 ORDER BY seq LIMIT $3",
    )
    .bind(project_id)
    .bind(from_seq)
    .bind(limit)
    .fetch_all(tx.conn())
    .await?;
    Ok(rows
        .into_iter()
        .map(
            |(
                id,
                tenant_id,
                project_id,
                seq,
                time_us,
                hash,
                category,
                action,
                outcome,
                actor,
                subject,
                metadata,
            )| {
                AuditEvent {
                    id: Id::from_uuid(Kind::Event, id),
                    tenant_id,
                    project_id: Id::from_uuid(Kind::Project, project_id),
                    seq,
                    time_us,
                    hash,
                    category,
                    action,
                    outcome,
                    actor,
                    subject,
                    metadata,
                }
            },
        )
        .collect())
}

/// Verifies a complete chain read from seq 1, without touching the database.
///
/// # Errors
/// The first [`ChainError`] found.
pub fn verify_chain(events: &[AuditEvent]) -> Result<(), ChainError> {
    let mut prev = vec![0u8; HASH_LEN];
    for (expected_seq, event) in (1i64..).zip(events) {
        if event.seq != expected_seq {
            return Err(ChainError::Gap(expected_seq));
        }
        let hash = chain_hash(&prev, event).ok_or(ChainError::NonCanonical(event.seq))?;
        if hash != event.hash {
            return Err(ChainError::HashMismatch(event.seq));
        }
        prev = hash;
    }
    Ok(())
}

fn chain_hash(prev: &[u8], event: &AuditEvent) -> Option<Vec<u8>> {
    let body = json!({
        "id": event.id.uuid().to_string(),
        "tenant_id": event.tenant_id.to_string(),
        "project_id": event.project_id.uuid().to_string(),
        "seq": event.seq,
        "time_us": event.time_us,
        "category": event.category,
        "action": event.action,
        "outcome": event.outcome,
        "actor": event.actor,
        "subject": event.subject,
        "metadata": event.metadata,
    });
    let mut canonical = Vec::new();
    write_canonical(&body, &mut canonical)?;
    let mut ctx = digest::Context::new(&digest::SHA256);
    ctx.update(prev);
    ctx.update(&canonical);
    Some(ctx.finish().as_ref().to_vec())
}

/// RFC 8785 canonical JSON for documents without floating-point numbers:
/// object keys sorted by UTF-16 code units, no insignificant whitespace.
fn write_canonical(value: &Value, out: &mut Vec<u8>) -> Option<()> {
    match value {
        Value::Null | Value::Bool(_) | Value::String(_) => {
            out.extend(serde_json::to_vec(value).ok()?);
        }
        Value::Number(n) => {
            if !(n.is_i64() || n.is_u64()) {
                return None;
            }
            out.extend(n.to_string().as_bytes());
        }
        Value::Array(items) => {
            out.push(b'[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(b',');
                }
                write_canonical(item, out)?;
            }
            out.push(b']');
        }
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort_by(|a, b| a.encode_utf16().cmp(b.encode_utf16()));
            out.push(b'{');
            for (i, key) in keys.into_iter().enumerate() {
                if i > 0 {
                    out.push(b',');
                }
                out.extend(serde_json::to_vec(key).ok()?);
                out.push(b':');
                write_canonical(map.get(key)?, out)?;
            }
            out.push(b'}');
        }
    }
    Some(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_json_sorts_keys_and_strips_whitespace() {
        let mut out = Vec::new();
        write_canonical(
            &json!({"b": 1, "a": [true, null, "x"], "c": {"z": -2, "y": "é"}}),
            &mut out,
        )
        .unwrap();
        assert_eq!(
            String::from_utf8(out).unwrap(),
            r#"{"a":[true,null,"x"],"b":1,"c":{"y":"é","z":-2}}"#
        );
    }

    #[test]
    fn canonical_json_rejects_floats() {
        assert!(write_canonical(&json!({"x": 1.5}), &mut Vec::new()).is_none());
    }

    #[test]
    fn keys_sort_by_utf16_code_units_not_utf8_bytes() {
        // U+1F600 is the surrogate pair D83D DE00, which sorts before U+FF61 in
        // UTF-16; in UTF-8 (F0… vs EF…) the order would be reversed.
        let mut out = Vec::new();
        write_canonical(&json!({"\u{FF61}": 2, "\u{1F600}": 1}), &mut out).unwrap();
        assert_eq!(
            String::from_utf8(out).unwrap(),
            "{\"\u{1F600}\":1,\"\u{FF61}\":2}"
        );
    }
}
