-- Tenants and projects (domain-model.md §3). Both are tenant-level tables.

CREATE TABLE auvora.tenants (
  id          uuid PRIMARY KEY,
  -- Mirrors id so the standard tenant-level policy applies without a hand-written exception.
  tenant_id   uuid GENERATED ALWAYS AS (id) STORED NOT NULL,
  slug        text NOT NULL,
  name        text NOT NULL,
  home_cell   text NOT NULL DEFAULT 'default',
  status      text NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'suspended', 'deleted')),
  legal_hold  boolean NOT NULL DEFAULT false,
  version     bigint NOT NULL DEFAULT 1,
  created_at  timestamptz NOT NULL DEFAULT now(),
  deleted_at  timestamptz NULL
);
COMMENT ON COLUMN auvora.tenants.id         IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.tenants.tenant_id  IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.tenants.slug       IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.tenants.name       IS 'pii=indirect;export=true';
COMMENT ON COLUMN auvora.tenants.home_cell  IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.tenants.status     IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.tenants.legal_hold IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.tenants.version    IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.tenants.created_at IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.tenants.deleted_at IS 'pii=none;export=false';
SELECT auvora.enable_tenant_rls('auvora.tenants', 'tenant');

CREATE TABLE auvora.projects (
  id               uuid PRIMARY KEY,
  tenant_id        uuid NOT NULL REFERENCES auvora.tenants (id),
  slug             text NOT NULL,
  name             text NOT NULL,
  stage            text NOT NULL DEFAULT 'development'
                     CHECK (stage IN ('development', 'staging', 'production', 'custom')),
  issuer           text NOT NULL,
  issuer_locked_at timestamptz NULL,
  home_cell        text NOT NULL DEFAULT 'default',
  config_version   bigint NOT NULL DEFAULT 1,
  settings         jsonb NOT NULL DEFAULT '{}'::jsonb,
  status           text NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'suspended', 'deleted')),
  legal_hold       boolean NOT NULL DEFAULT false,
  version          bigint NOT NULL DEFAULT 1,
  created_at       timestamptz NOT NULL DEFAULT now(),
  deleted_at       timestamptz NULL,
  UNIQUE (tenant_id, slug)
);
-- Installation-wide issuer uniqueness moves to auvora_global.domain_directory with cells (cells.md §4.2).
COMMENT ON COLUMN auvora.projects.id               IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.projects.tenant_id        IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.projects.slug             IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.projects.name             IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.projects.stage            IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.projects.issuer           IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.projects.issuer_locked_at IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.projects.home_cell        IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.projects.config_version   IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.projects.settings         IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.projects.status           IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.projects.legal_hold       IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.projects.version          IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.projects.created_at       IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.projects.deleted_at       IS 'pii=none;export=false';
SELECT auvora.enable_tenant_rls('auvora.projects', 'tenant');
