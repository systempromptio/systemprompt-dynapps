-- Extensions for the bundled Postgres in docker-compose.single.yml. Runs once
-- at first start as the superuser (docker-entrypoint-initdb.d). The role and
-- database already exist from POSTGRES_USER / POSTGRES_DB; provision.sql is the
-- equivalent for a database you manage yourself.
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE EXTENSION IF NOT EXISTS vector;
