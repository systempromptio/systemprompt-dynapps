# Migrations held back from boot

`build.rs` discovers `schema/migrations/*.sql` and the runtime applies every one
it finds at startup. A migration in this directory is **not** discovered and
**not** applied.

`034_knowledge_bank.sql` is here because it needs pgvector
(`CREATE EXTENSION IF NOT EXISTS vector`, `vector(1024)`, an HNSW index) and the
production database does not have pgvector available. Applied at boot it fails
with `extension "vector" is not available`, which fails schema installation,
which crash-loops the platform — an optional MCP server's tables taking the
whole deployment down.

Move it back into `schema/migrations/` in the same change that makes pgvector
available on every target this deploys to, and register `SCHEMA_KNOWLEDGE_BANK`
in `src/schemas.rs` alongside it.
