-- Principals and identifiers (domain-model.md §3). Both are project-level tables.

CREATE TABLE auvora.principals (
  id           uuid NOT NULL PRIMARY KEY,
  tenant_id    uuid NOT NULL,
  project_id   uuid NOT NULL REFERENCES auvora.projects (id),
  kind         text NOT NULL CHECK (kind IN ('user', 'service_account', 'workload', 'agent', 'anonymous')),
  status       text NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'suspended', 'deleted')),
  legal_hold   boolean NOT NULL DEFAULT false,
  merged_into  uuid NULL,
  pii_dek_id   uuid NULL,
  version      bigint NOT NULL DEFAULT 1,
  created_at   timestamptz NOT NULL DEFAULT now(),
  last_seen_at timestamptz NULL,
  deleted_at   timestamptz NULL,
  -- Target for composite FKs: a child can only reference a parent in its own project.
  UNIQUE (project_id, id),
  FOREIGN KEY (project_id, merged_into) REFERENCES auvora.principals (project_id, id)
);
COMMENT ON COLUMN auvora.principals.id           IS 'pii=indirect;export=true';
COMMENT ON COLUMN auvora.principals.tenant_id    IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.principals.project_id   IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.principals.kind         IS 'pii=none;export=true';
COMMENT ON COLUMN auvora.principals.status       IS 'pii=none;export=true';
COMMENT ON COLUMN auvora.principals.legal_hold   IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.principals.merged_into  IS 'pii=indirect;export=true';
COMMENT ON COLUMN auvora.principals.pii_dek_id   IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.principals.version      IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.principals.created_at   IS 'pii=none;export=true';
COMMENT ON COLUMN auvora.principals.last_seen_at IS 'pii=indirect;export=true';
COMMENT ON COLUMN auvora.principals.deleted_at   IS 'pii=none;export=false';
SELECT auvora.enable_tenant_rls('auvora.principals', 'project');

CREATE TABLE auvora.identifiers (
  id                     uuid NOT NULL PRIMARY KEY,
  tenant_id              uuid NOT NULL,
  project_id             uuid NOT NULL,
  principal_id           uuid NOT NULL,
  type                   text NOT NULL CHECK (type IN ('email', 'phone', 'username', 'external')),
  value_normalized_bidx  bytea NOT NULL,
  value_display          text NULL,
  value_skeleton_bidx    bytea NULL,
  verified_at            timestamptz NULL,
  is_primary             boolean NOT NULL DEFAULT false,
  version                bigint NOT NULL DEFAULT 1,
  created_at             timestamptz NOT NULL DEFAULT now(),
  FOREIGN KEY (project_id, principal_id) REFERENCES auvora.principals (project_id, id) ON DELETE CASCADE,
  -- ADR-0075: uniqueness is per project, never global.
  UNIQUE (project_id, type, value_normalized_bidx)
);
CREATE UNIQUE INDEX identifiers_username_skeleton
  ON auvora.identifiers (project_id, value_skeleton_bidx) WHERE type = 'username';
CREATE UNIQUE INDEX identifiers_one_primary_per_type
  ON auvora.identifiers (project_id, principal_id, type) WHERE is_primary;
COMMENT ON COLUMN auvora.identifiers.id                    IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.identifiers.tenant_id             IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.identifiers.project_id            IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.identifiers.principal_id          IS 'pii=indirect;export=true';
COMMENT ON COLUMN auvora.identifiers.type                  IS 'pii=none;export=true';
COMMENT ON COLUMN auvora.identifiers.value_normalized_bidx IS 'pii=indirect;export=false';
COMMENT ON COLUMN auvora.identifiers.value_display         IS 'pii=direct;export=true';
COMMENT ON COLUMN auvora.identifiers.value_skeleton_bidx   IS 'pii=indirect;export=false';
COMMENT ON COLUMN auvora.identifiers.verified_at           IS 'pii=none;export=true';
COMMENT ON COLUMN auvora.identifiers.is_primary            IS 'pii=none;export=true';
COMMENT ON COLUMN auvora.identifiers.version               IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.identifiers.created_at            IS 'pii=none;export=true';
SELECT auvora.enable_tenant_rls('auvora.identifiers', 'project');
