//! PostgreSQL persistence with tenant isolation enforced by row-level security.
//!
//! Design: [`docs/specs/tenancy-rls.md`] and ADR-0017.
//!
//! - The connection pool is private. Callers get a [`TenantTx`] from
//!   [`Store::begin`], which sets the tenant/project context as the first
//!   statement of the transaction (`set_config(…, true)`, never session-level).
//! - Every tenant table has `FORCE ROW LEVEL SECURITY` and one
//!   `tenant_isolation` policy. A missing context is an error, never an
//!   unfiltered result.
//! - [`Store::connect`] refuses to run as a superuser, an RLS-bypassing role or
//!   a table owner.
//!
//! [`docs/specs/tenancy-rls.md`]: https://github.com/originalkamaal/Auvora/blob/main/docs/specs/tenancy-rls.md

mod error;
mod scope;
mod store;

pub mod repo;

pub use error::StoreError;
pub use scope::TenantScope;
pub use store::{BOOTSTRAP_SQL, MIGRATOR, Store, TenantTx};
