/// Errors returned by the store.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum StoreError {
    /// The runtime database role could bypass row-level security.
    ///
    /// The server refuses to start in this case (tenancy-rls.md §2, rule 1).
    #[error("unsafe database role: {0}")]
    UnsafeRole(&'static str),

    /// A row that the caller asked for doesn't exist in the current scope.
    ///
    /// Rows in other projects are indistinguishable from missing rows.
    #[error("not found")]
    NotFound,

    /// A uniqueness constraint was violated (e.g. a duplicate identifier).
    #[error("conflict: {constraint}")]
    Conflict {
        /// Name of the violated constraint.
        constraint: String,
    },

    /// The operation needs a project-level scope but the transaction is tenant-level.
    #[error("operation requires a project scope")]
    ProjectScopeRequired,

    /// Any other database error.
    #[error("database error: {0}")]
    Database(#[source] sqlx::Error),
}

impl From<sqlx::Error> for StoreError {
    fn from(err: sqlx::Error) -> Self {
        if let sqlx::Error::Database(db) = &err
            && db.is_unique_violation()
        {
            return Self::Conflict {
                constraint: db.constraint().unwrap_or("unknown").to_owned(),
            };
        }
        Self::Database(err)
    }
}
