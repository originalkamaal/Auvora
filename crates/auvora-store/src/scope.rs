use auvora_core::id::{Id, IdError, Kind};
use uuid::Uuid;

/// Who a transaction acts for (tenancy-rls.md §3.2).
///
/// A scope is built only from a resolved request context: host/issuer
/// resolution on the data plane, or an authorized admin target on the
/// control plane (tenancy-rls.md §6). It never comes from request input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TenantScope {
    tenant_id: Uuid,
    project_id: Option<Uuid>,
}

impl TenantScope {
    /// A project-level scope: project tables and the owning tenant's rows.
    ///
    /// # Errors
    /// [`IdError::WrongKind`] if `tenant` isn't a tenant ID or `project` isn't a project ID.
    pub fn project(tenant: Id, project: Id) -> Result<Self, IdError> {
        Ok(Self {
            tenant_id: expect_kind(tenant, Kind::Tenant)?,
            project_id: Some(expect_kind(project, Kind::Project)?),
        })
    }

    /// A tenant-level scope: tenant tables only. Project tables error under it.
    ///
    /// # Errors
    /// [`IdError::WrongKind`] if `tenant` isn't a tenant ID.
    pub fn tenant(tenant: Id) -> Result<Self, IdError> {
        Ok(Self {
            tenant_id: expect_kind(tenant, Kind::Tenant)?,
            project_id: None,
        })
    }

    /// The tenant this scope belongs to.
    #[must_use]
    pub const fn tenant_id(&self) -> Uuid {
        self.tenant_id
    }

    /// The project, for project-level scopes.
    #[must_use]
    pub const fn project_id(&self) -> Option<Uuid> {
        self.project_id
    }
}

fn expect_kind(id: Id, expected: Kind) -> Result<Uuid, IdError> {
    if id.kind() == expected {
        Ok(id.uuid())
    } else {
        Err(IdError::WrongKind {
            expected,
            found: id.kind(),
        })
    }
}
