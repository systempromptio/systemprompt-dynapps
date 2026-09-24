-- Provision the systemprompt database on the customer's PostgreSQL primary.
-- Run ONCE as the administrative user (OCI: the `postgres`/ADMIN user of the
-- DB system) with the application password bound:
--   psql "$ADMIN_URL" -v pw="$APP_DB_PASSWORD" -f provision.sql
-- Idempotent: every step is guarded, re-running is safe.
--
-- Requirements this proves: PostgreSQL 16+, and the two extensions the
-- platform needs. pgvector is not required.

SELECT 'CREATE ROLE systemprompt LOGIN PASSWORD ' || quote_literal(:'pw')
WHERE NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'systemprompt') \gexec

SELECT 'CREATE DATABASE systemprompt OWNER systemprompt'
WHERE NOT EXISTS (SELECT 1 FROM pg_database WHERE datname = 'systemprompt') \gexec

\connect systemprompt

-- Extensions need elevated rights, which is why this runs as the admin user
-- and the application role never needs them.
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- The application role owns the schema it migrates.
GRANT ALL ON SCHEMA public TO systemprompt;
ALTER DATABASE systemprompt SET timezone TO 'UTC';

SELECT version();
SELECT extname, extversion FROM pg_extension
WHERE extname IN ('uuid-ossp', 'pgcrypto') ORDER BY extname;
