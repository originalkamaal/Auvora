-- Tenancy context and RLS helper (tenancy-rls.md §3.1, §4).

-- Fail closed: an unset or reverted setting raises instead of matching nothing.
CREATE FUNCTION auvora.ctx_project_id() RETURNS uuid
  LANGUAGE sql STABLE PARALLEL SAFE
  AS $$ SELECT current_setting('auvora.project_id')::uuid $$;

CREATE FUNCTION auvora.ctx_tenant_id() RETURNS uuid
  LANGUAGE sql STABLE PARALLEL SAFE
  AS $$ SELECT current_setting('auvora.tenant_id')::uuid $$;

GRANT EXECUTE ON FUNCTION auvora.ctx_project_id(), auvora.ctx_tenant_id() TO auvora_app;

-- The only way tenant tables get RLS. Hand-written policies are not allowed (§4).
CREATE FUNCTION auvora.enable_tenant_rls(tbl regclass, level text) RETURNS void
LANGUAGE plpgsql AS $$
BEGIN
  EXECUTE format('ALTER TABLE %s ENABLE ROW LEVEL SECURITY', tbl);
  EXECUTE format('ALTER TABLE %s FORCE ROW LEVEL SECURITY', tbl);
  IF level = 'project' THEN
    EXECUTE format($p$CREATE POLICY tenant_isolation ON %s
      USING      (project_id = (SELECT auvora.ctx_project_id())
              AND tenant_id  = (SELECT auvora.ctx_tenant_id()))
      WITH CHECK (project_id = (SELECT auvora.ctx_project_id())
              AND tenant_id  = (SELECT auvora.ctx_tenant_id()))$p$, tbl);
  ELSIF level = 'tenant' THEN
    EXECUTE format($p$CREATE POLICY tenant_isolation ON %s
      USING      (tenant_id = (SELECT auvora.ctx_tenant_id()))
      WITH CHECK (tenant_id = (SELECT auvora.ctx_tenant_id()))$p$, tbl);
  ELSE
    RAISE EXCEPTION 'unknown tenancy level %', level;
  END IF;
  EXECUTE format('GRANT SELECT, INSERT, UPDATE, DELETE ON %s TO auvora_app', tbl);
END $$;

REVOKE ALL ON FUNCTION auvora.enable_tenant_rls(regclass, text) FROM PUBLIC;
