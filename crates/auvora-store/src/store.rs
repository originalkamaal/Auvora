use sqlx::migrate::Migrator;
use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::{PgConnection, Postgres, Transaction};
use uuid::Uuid;

use crate::{StoreError, TenantScope};

/// Database bootstrap: roles and schema. Run once per database by a superuser
/// before the first migration (tenancy-rls.md §2).
pub const BOOTSTRAP_SQL: &str = include_str!("../sql/bootstrap.sql");

/// Embedded migrations. Run them as `auvora_owner` only.
pub static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

/// Runtime access to the database as `auvora_app`.
#[derive(Debug, Clone)]
pub struct Store {
    pool: PgPool,
}

impl Store {
    /// Connects as the runtime role and verifies that it can't bypass RLS.
    ///
    /// # Errors
    /// [`StoreError::UnsafeRole`] if the role is a superuser, has `BYPASSRLS`
    /// or owns tables in the `auvora` schema; [`StoreError::Database`] on
    /// connection failure.
    pub async fn connect(url: &str, max_connections: u32) -> Result<Self, StoreError> {
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .connect(url)
            .await?;
        let store = Self { pool };
        store.check_role_is_safe().await?;
        Ok(store)
    }

    async fn check_role_is_safe(&self) -> Result<(), StoreError> {
        let (superuser, bypass_rls): (bool, bool) = sqlx::query_as(
            "SELECT rolsuper, rolbypassrls FROM pg_roles WHERE rolname = current_user",
        )
        .fetch_one(&self.pool)
        .await?;
        if superuser {
            return Err(StoreError::UnsafeRole("runtime role is a superuser"));
        }
        if bypass_rls {
            return Err(StoreError::UnsafeRole("runtime role has BYPASSRLS"));
        }
        let owned: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM pg_tables WHERE schemaname = 'auvora' AND tableowner = current_user",
        )
        .fetch_one(&self.pool)
        .await?;
        if owned > 0 {
            return Err(StoreError::UnsafeRole(
                "runtime role owns tables in schema auvora",
            ));
        }
        Ok(())
    }

    /// Opens a transaction bound to `scope`.
    ///
    /// The first statement sets the transaction-local context that every RLS
    /// policy reads. Values are bound parameters; the settings revert at
    /// commit or rollback, so pooled connections never carry context.
    ///
    /// # Errors
    /// [`StoreError::Database`] if the transaction can't be started.
    pub async fn begin(&self, scope: &TenantScope) -> Result<TenantTx, StoreError> {
        let mut tx = self.pool.begin().await?;
        let project = scope
            .project_id()
            .map(|p| p.to_string())
            .unwrap_or_default();
        sqlx::query("SELECT set_config('auvora.tenant_id', $1, true), set_config('auvora.project_id', $2, true)")
            .bind(scope.tenant_id().to_string())
            .bind(project)
            .execute(&mut *tx)
            .await?;
        Ok(TenantTx { tx, scope: *scope })
    }
}

/// An open transaction with its tenant scope. The only handle repositories accept.
#[derive(Debug)]
pub struct TenantTx {
    tx: Transaction<'static, Postgres>,
    scope: TenantScope,
}

impl TenantTx {
    /// The scope this transaction acts for.
    #[must_use]
    pub const fn scope(&self) -> &TenantScope {
        &self.scope
    }

    /// Commits the transaction.
    ///
    /// # Errors
    /// [`StoreError::Database`] if the commit fails.
    pub async fn commit(self) -> Result<(), StoreError> {
        self.tx.commit().await.map_err(StoreError::from)
    }

    /// Rolls the transaction back.
    ///
    /// # Errors
    /// [`StoreError::Database`] if the rollback fails.
    pub async fn rollback(self) -> Result<(), StoreError> {
        self.tx.rollback().await.map_err(StoreError::from)
    }

    pub(crate) fn conn(&mut self) -> &mut PgConnection {
        &mut self.tx
    }

    pub(crate) fn project_id(&self) -> Result<Uuid, StoreError> {
        self.scope
            .project_id()
            .ok_or(StoreError::ProjectScopeRequired)
    }
}
