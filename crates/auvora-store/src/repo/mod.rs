//! Repositories. Every function takes `&mut TenantTx`; none can reach the pool.
//!
//! Every query also filters by the scope's IDs explicitly (ADR-0017 §6), so
//! RLS is a second, independent barrier rather than the only one. Each public
//! `async fn` here is exercised by the isolation property tests; the coverage
//! gate in `tests/rls_isolation.rs` scans these sources and fails if one isn't.

pub mod identifiers;
pub mod principals;
pub mod projects;
pub mod tenants;
