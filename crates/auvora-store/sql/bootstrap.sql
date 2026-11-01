-- Auvora database bootstrap (tenancy-rls.md §2). Run ONCE per database by a
-- superuser (or a role with CREATEROLE + BYPASSRLS rights), before the first
-- `auvora migrate`. Idempotent. Passwords are set separately by the operator:
--   ALTER ROLE auvora_owner    LOGIN PASSWORD '…';
--   ALTER ROLE auvora_app      LOGIN PASSWORD '…';
--   ALTER ROLE auvora_platform LOGIN PASSWORD '…';

DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'auvora_owner') THEN
    CREATE ROLE auvora_owner NOLOGIN NOSUPERUSER NOBYPASSRLS NOCREATEROLE NOCREATEDB;
  END IF;
  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'auvora_app') THEN
    CREATE ROLE auvora_app NOLOGIN NOSUPERUSER NOBYPASSRLS NOCREATEROLE NOCREATEDB;
  END IF;
  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'auvora_platform') THEN
    CREATE ROLE auvora_platform NOLOGIN NOSUPERUSER BYPASSRLS NOCREATEROLE NOCREATEDB;
  END IF;
  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'auvora_retention') THEN
    CREATE ROLE auvora_retention NOLOGIN NOSUPERUSER NOBYPASSRLS NOCREATEROLE NOCREATEDB;
  END IF;
END $$;

-- Platform acts as the app plus global tables; retention is reached only via platform.
GRANT auvora_app TO auvora_platform;
GRANT auvora_retention TO auvora_platform;

-- The owner holds the schema; migrations (sqlx ledger included) live in it.
CREATE SCHEMA IF NOT EXISTS auvora AUTHORIZATION auvora_owner;
ALTER ROLE auvora_owner SET search_path = auvora;
REVOKE ALL ON SCHEMA auvora FROM PUBLIC;
GRANT USAGE ON SCHEMA auvora TO auvora_app, auvora_platform;
