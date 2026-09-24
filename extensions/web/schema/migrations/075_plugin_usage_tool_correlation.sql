-- @supersedes-checksum: 630eb2fb5d76acc4
-- Promote the tool-call correlation keys on plugin_usage_events.
--
-- A hook event carried the client's tool_use_id only inside metadata, so
-- every join to the decision that governed the call, the execution that ran
-- it, and the artifact it produced went through a JSON path or a time
-- window. The key is its own indexed column now, with the hook's trace (the
-- session) and the execution the artifact ingest resolved beside it. The
-- schema views (skill_invocation_events, analysis_skill_events) are re-created
-- from schema/05 and schema/28 at boot and read the column.

ALTER TABLE plugin_usage_events ADD COLUMN IF NOT EXISTS tool_use_id TEXT;
ALTER TABLE plugin_usage_events ADD COLUMN IF NOT EXISTS trace_id TEXT;
ALTER TABLE plugin_usage_events ADD COLUMN IF NOT EXISTS mcp_execution_id TEXT;

-- Why: the ownership trigger re-asserts the owner on every UPDATE, and
-- historical rows can belong to accounts that no longer exist; a backfill of
-- two derived columns changes no ownership and must not fail on them. The
-- migration runner suspends it (and every other row trigger on the table)
-- for this migration; the named DISABLE/ENABLE pair it shipped with is gone.
UPDATE plugin_usage_events
SET tool_use_id = NULLIF(metadata->>'tool_use_id', ''),
    trace_id = session_id
WHERE tool_use_id IS NULL
  AND event_type IN ('PreToolUse', 'PostToolUse', 'PostToolUseFailure')
  AND metadata ? 'tool_use_id';

CREATE INDEX IF NOT EXISTS idx_plugin_usage_tool_use_id ON plugin_usage_events(tool_use_id) WHERE tool_use_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_plugin_usage_execution ON plugin_usage_events(mcp_execution_id) WHERE mcp_execution_id IS NOT NULL;
