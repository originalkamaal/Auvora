-- Users (1:1 with principals) and environment groups (domain-model.md §3).

CREATE TABLE auvora.environment_groups (
  id         uuid PRIMARY KEY,
  tenant_id  uuid NOT NULL REFERENCES auvora.tenants (id),
  name       text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (tenant_id, name)
);
COMMENT ON COLUMN auvora.environment_groups.id         IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.environment_groups.tenant_id  IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.environment_groups.name       IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.environment_groups.created_at IS 'pii=none;export=false';
SELECT auvora.enable_tenant_rls('auvora.environment_groups', 'tenant');

-- Expand step (ADR-0025): projects gain an optional environment group.
ALTER TABLE auvora.projects ADD COLUMN env_group_id uuid NULL REFERENCES auvora.environment_groups (id);
COMMENT ON COLUMN auvora.projects.env_group_id IS 'pii=none;export=false';

CREATE TABLE auvora.users (
  tenant_id             uuid NOT NULL,
  project_id            uuid NOT NULL,
  principal_id          uuid NOT NULL,
  profile               jsonb NOT NULL DEFAULT '{}'::jsonb,
  profile_schema_version integer NOT NULL DEFAULT 1,
  public_metadata       jsonb NOT NULL DEFAULT '{}'::jsonb CHECK (octet_length(public_metadata::text) <= 8192),
  private_metadata      jsonb NOT NULL DEFAULT '{}'::jsonb CHECK (octet_length(private_metadata::text) <= 8192),
  unsafe_metadata       jsonb NOT NULL DEFAULT '{}'::jsonb CHECK (octet_length(unsafe_metadata::text) <= 8192),
  ial                   smallint NOT NULL DEFAULT 1 CHECK (ial BETWEEN 1 AND 3),
  password_changed_at   timestamptz NULL,
  mfa_enrolled          boolean NOT NULL DEFAULT false,
  terms_accepted        jsonb NOT NULL DEFAULT '{}'::jsonb,
  version               bigint NOT NULL DEFAULT 1,
  PRIMARY KEY (project_id, principal_id),
  FOREIGN KEY (project_id, principal_id) REFERENCES auvora.principals (project_id, id) ON DELETE CASCADE
);
COMMENT ON COLUMN auvora.users.tenant_id              IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.users.project_id             IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.users.principal_id           IS 'pii=indirect;export=true';
COMMENT ON COLUMN auvora.users.profile                IS 'pii=direct;export=true';
COMMENT ON COLUMN auvora.users.profile_schema_version IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.users.public_metadata        IS 'pii=direct;export=true';
COMMENT ON COLUMN auvora.users.private_metadata       IS 'pii=direct;export=true';
COMMENT ON COLUMN auvora.users.unsafe_metadata        IS 'pii=direct;export=true';
COMMENT ON COLUMN auvora.users.ial                    IS 'pii=none;export=true';
COMMENT ON COLUMN auvora.users.password_changed_at    IS 'pii=none;export=true';
COMMENT ON COLUMN auvora.users.mfa_enrolled           IS 'pii=none;export=true';
COMMENT ON COLUMN auvora.users.terms_accepted         IS 'pii=none;export=true';
COMMENT ON COLUMN auvora.users.version                IS 'pii=none;export=false';
SELECT auvora.enable_tenant_rls('auvora.users', 'project');
