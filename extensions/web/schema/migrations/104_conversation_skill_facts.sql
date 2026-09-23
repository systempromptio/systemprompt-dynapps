-- The Versions pages move onto the conversation record.
--
-- `conversation_skill_facts` is one row per (conversation, skill), written by
-- refresh_conversation_facts beside the conversation_facts row and kept as
-- long: invocations, failures, and the marketplace version being served when
-- the conversation first invoked the skill. It replaces the
-- conversation_skill_versions view, which recomputed the same thing from raw
-- hook events on every read and so lost every version older than their
-- retention. `conversation_rollup_state` holds the rollup job's watermark,
-- which replaces its overlapping three-minute lookback.
--
-- The rows are filled by the rollup, not here:
--   systemprompt infra jobs run conversation_rollup -p all=true
--
-- Declarative twin: 45_conversation_facts.sql.
CREATE TABLE IF NOT EXISTS conversation_skill_facts (
    context_id TEXT NOT NULL REFERENCES conversation_facts(context_id) ON DELETE CASCADE,
    plugin_id TEXT NOT NULL,
    skill TEXT NOT NULL,
    user_id TEXT NOT NULL,
    marketplace_id TEXT,
    marketplace_hash TEXT,
    invocations BIGINT NOT NULL DEFAULT 0,
    failures BIGINT NOT NULL DEFAULT 0,
    first_invoked_at TIMESTAMPTZ NOT NULL,
    last_invoked_at TIMESTAMPTZ NOT NULL,
    refreshed_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (context_id, plugin_id, skill)
);
CREATE INDEX IF NOT EXISTS idx_conversation_skill_facts_version
    ON conversation_skill_facts(marketplace_id, marketplace_hash, first_invoked_at);
CREATE INDEX IF NOT EXISTS idx_conversation_skill_facts_first
    ON conversation_skill_facts(first_invoked_at);

CREATE TABLE IF NOT EXISTS conversation_rollup_state (
    id BOOLEAN PRIMARY KEY DEFAULT TRUE CHECK (id),
    watermark TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp()
);

DROP VIEW IF EXISTS conversation_skill_versions;
DROP FUNCTION IF EXISTS refresh_conversation_facts_since(TIMESTAMPTZ);
