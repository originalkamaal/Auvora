-- Audit (ADR-0039), outbox (ADR-0008), jobs and the platform audit chain.

-- Monthly range partitions, created ahead by the worker (ADR-0109 §1).
-- SECURITY DEFINER so the runtime role needs no DDL rights; it accepts only
-- the partitioned tables listed here, and partitions get no grants (lint L7).
CREATE FUNCTION auvora.create_partitions(tbl text, months_ahead int) RETURNS void
LANGUAGE plpgsql SECURITY DEFINER SET search_path = pg_catalog, auvora AS $$
DECLARE
  month_start date;
  part text;
BEGIN
  IF tbl NOT IN ('audit_events', 'outbox_events', 'platform_audit_events') THEN
    RAISE EXCEPTION 'create_partitions: % is not a partitioned table', tbl;
  END IF;
  IF months_ahead < 0 OR months_ahead > 24 THEN
    RAISE EXCEPTION 'create_partitions: months_ahead must be 0..24';
  END IF;
  FOR i IN -1..months_ahead LOOP
    month_start := (date_trunc('month', now()) + make_interval(months => i))::date;
    part := format('%s_y%sm%s', tbl, to_char(month_start, 'YYYY'), to_char(month_start, 'MM'));
    IF to_regclass('auvora.' || part) IS NULL THEN
      EXECUTE format('CREATE TABLE auvora.%I PARTITION OF auvora.%I FOR VALUES FROM (%L) TO (%L)',
                     part, tbl, month_start, (month_start + interval '1 month')::date);
      EXECUTE format('REVOKE ALL ON auvora.%I FROM PUBLIC, auvora_app', part);
    END IF;
  END LOOP;
END $$;
REVOKE ALL ON FUNCTION auvora.create_partitions(text, int) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION auvora.create_partitions(text, int) TO auvora_app;

-- Append-only enforcement shared by audit tables (ADR-0039 §2).
CREATE FUNCTION auvora.reject_audit_mutation() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
  RAISE EXCEPTION 'audit records are append-only (ADR-0039)';
END $$;

-- Per-project chain head: serializes seq allocation so the chain is gapless.
CREATE TABLE auvora.audit_chain_heads (
  tenant_id  uuid NOT NULL,
  project_id uuid NOT NULL REFERENCES auvora.projects (id),
  last_seq   bigint NOT NULL DEFAULT 0,
  last_hash  bytea NOT NULL DEFAULT '\x0000000000000000000000000000000000000000000000000000000000000000',
  PRIMARY KEY (project_id)
);
COMMENT ON COLUMN auvora.audit_chain_heads.tenant_id  IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.audit_chain_heads.project_id IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.audit_chain_heads.last_seq   IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.audit_chain_heads.last_hash  IS 'pii=none;export=false';
SELECT auvora.enable_tenant_rls('auvora.audit_chain_heads', 'project');
REVOKE DELETE ON auvora.audit_chain_heads FROM auvora_app;

CREATE TABLE auvora.audit_events (
  id         uuid NOT NULL,
  tenant_id  uuid NOT NULL,
  project_id uuid NOT NULL,
  seq        bigint NOT NULL,
  time       timestamptz NOT NULL,
  hash       bytea NOT NULL,
  category   text NOT NULL CHECK (category IN ('security', 'admin', 'activity', 'system')),
  action     text NOT NULL,
  outcome    text NOT NULL CHECK (outcome IN ('success', 'failure', 'denied')),
  actor      jsonb NOT NULL,
  subject    jsonb NOT NULL,
  metadata   jsonb NOT NULL DEFAULT '{}'::jsonb,
  PRIMARY KEY (project_id, seq, time)
) PARTITION BY RANGE (time);
COMMENT ON COLUMN auvora.audit_events.id         IS 'pii=none;export=true';
COMMENT ON COLUMN auvora.audit_events.tenant_id  IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.audit_events.project_id IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.audit_events.seq        IS 'pii=none;export=true';
COMMENT ON COLUMN auvora.audit_events.time       IS 'pii=none;export=true';
COMMENT ON COLUMN auvora.audit_events.hash       IS 'pii=none;export=true';
COMMENT ON COLUMN auvora.audit_events.category   IS 'pii=none;export=true';
COMMENT ON COLUMN auvora.audit_events.action     IS 'pii=none;export=true';
COMMENT ON COLUMN auvora.audit_events.outcome    IS 'pii=none;export=true';
COMMENT ON COLUMN auvora.audit_events.actor      IS 'pii=indirect;export=true';
COMMENT ON COLUMN auvora.audit_events.subject    IS 'pii=indirect;export=true';
COMMENT ON COLUMN auvora.audit_events.metadata   IS 'pii=none;export=true';
SELECT auvora.enable_tenant_rls('auvora.audit_events', 'project');
REVOKE UPDATE, DELETE ON auvora.audit_events FROM auvora_app;
CREATE TRIGGER audit_events_append_only BEFORE UPDATE OR DELETE ON auvora.audit_events
  FOR EACH ROW EXECUTE FUNCTION auvora.reject_audit_mutation();
SELECT auvora.create_partitions('audit_events', 3);

CREATE TABLE auvora.outbox_events (
  id           uuid NOT NULL,
  tenant_id    uuid NOT NULL,
  project_id   uuid NOT NULL,
  created_at   timestamptz NOT NULL,
  type         text NOT NULL,
  subject      text NOT NULL,
  data         jsonb NOT NULL,
  ordering_key text NOT NULL,
  delivered_at timestamptz NULL,
  PRIMARY KEY (project_id, id, created_at)
) PARTITION BY RANGE (created_at);
CREATE INDEX outbox_events_undelivered ON auvora.outbox_events (project_id, created_at) WHERE delivered_at IS NULL;
COMMENT ON COLUMN auvora.outbox_events.id           IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.outbox_events.tenant_id    IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.outbox_events.project_id   IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.outbox_events.created_at   IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.outbox_events.type         IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.outbox_events.subject      IS 'pii=indirect;export=false';
COMMENT ON COLUMN auvora.outbox_events.data         IS 'pii=indirect;export=false';
COMMENT ON COLUMN auvora.outbox_events.ordering_key IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.outbox_events.delivered_at IS 'pii=none;export=false';
SELECT auvora.enable_tenant_rls('auvora.outbox_events', 'project');
REVOKE DELETE ON auvora.outbox_events FROM auvora_app;
SELECT auvora.create_partitions('outbox_events', 3);

CREATE TABLE auvora.jobs (
  id           uuid PRIMARY KEY,
  tenant_id    uuid NOT NULL,
  project_id   uuid NOT NULL REFERENCES auvora.projects (id),
  kind         text NOT NULL,
  payload      jsonb NOT NULL DEFAULT '{}'::jsonb,
  state        text NOT NULL DEFAULT 'queued' CHECK (state IN ('queued', 'running', 'done', 'failed')),
  run_at       timestamptz NOT NULL,
  attempts     integer NOT NULL DEFAULT 0,
  locked_by    text NULL,
  locked_until timestamptz NULL,
  created_at   timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX jobs_ready ON auvora.jobs (run_at) WHERE state = 'queued';
COMMENT ON COLUMN auvora.jobs.id           IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.jobs.tenant_id    IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.jobs.project_id   IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.jobs.kind         IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.jobs.payload      IS 'pii=indirect;export=false';
COMMENT ON COLUMN auvora.jobs.state        IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.jobs.run_at       IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.jobs.attempts     IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.jobs.locked_by    IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.jobs.locked_until IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.jobs.created_at   IS 'pii=none;export=false';
SELECT auvora.enable_tenant_rls('auvora.jobs', 'project');

-- Cell-local platform chain: every PlatformScope acquisition is recorded here
-- (tenancy-rls.md §3.3). Platform role only; the app role has no grants.
CREATE TABLE auvora.platform_audit_events (
  id         uuid NOT NULL,
  time       timestamptz NOT NULL,
  reason     text NOT NULL,
  detail     text NULL,
  actor      jsonb NOT NULL,
  PRIMARY KEY (id, time)
) PARTITION BY RANGE (time);
COMMENT ON COLUMN auvora.platform_audit_events.id     IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.platform_audit_events.time   IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.platform_audit_events.reason IS 'pii=none;export=false';
COMMENT ON COLUMN auvora.platform_audit_events.detail IS 'pii=indirect;export=false';
COMMENT ON COLUMN auvora.platform_audit_events.actor  IS 'pii=indirect;export=false';
GRANT SELECT, INSERT ON auvora.platform_audit_events TO auvora_platform;
CREATE TRIGGER platform_audit_events_append_only BEFORE UPDATE OR DELETE ON auvora.platform_audit_events
  FOR EACH ROW EXECUTE FUNCTION auvora.reject_audit_mutation();
SELECT auvora.create_partitions('platform_audit_events', 3);
