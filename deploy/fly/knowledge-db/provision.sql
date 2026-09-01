-- Provision the knowledge database on the dedicated Fly Postgres app.
-- Run as the superuser through the proxy, with the role password bound:
--   KNOWLEDGE_DB_SUPERUSER_PASSWORD=... KNOWLEDGE_DB_PASSWORD=... just knowledge-db-provision
--   (= deploy/fly/knowledge-db/psql.sh --admin -v pw="$KNOWLEDGE_DB_PASSWORD" -f provision.sql)
-- Idempotent where Postgres allows it; the role and database steps are guarded.

SELECT 'CREATE ROLE knowledge LOGIN PASSWORD ' || quote_literal(:'pw')
WHERE NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'knowledge') \gexec

SELECT 'CREATE DATABASE knowledge OWNER knowledge'
WHERE NOT EXISTS (SELECT 1 FROM pg_database WHERE datname = 'knowledge') \gexec

\connect knowledge

-- Fails here if the image has no pgvector: redeploy from the Dockerfile
-- beside this file and re-run.
CREATE EXTENSION IF NOT EXISTS vector;

SELECT extname, extversion FROM pg_extension WHERE extname = 'vector';
