-- OpenCode reports skill use through the bridge plugin as a `skill` tool call
-- carrying `skill_ref` (`plugin:kebab`); count it beside Claude Code's slash
-- and `Skill` tool signals. Same column list, so dependent views stay valid.
CREATE OR REPLACE VIEW analysis_skill_events AS
SELECT e.id, e.user_id, e.session_id, e.plugin_id,
       replace(substring(e.prompt_preview from '^/([A-Za-z0-9._-]+:[A-Za-z0-9._-]+)'), '_', '-') AS skill,
       NULL::text AS tool_use_id, 'slash'::text AS source, e.created_at AS invoked_at
FROM plugin_usage_events e
WHERE e.event_type = 'UserPromptSubmit'
  AND e.prompt_preview ~ '^/[A-Za-z0-9._-]+:[A-Za-z0-9._-]+'
UNION ALL
SELECT e.id, e.user_id, e.session_id, e.plugin_id,
       replace(e.metadata->'tool_input'->>'skill', '_', '-') AS skill,
       e.metadata->>'tool_use_id' AS tool_use_id, 'tool'::text AS source, e.created_at AS invoked_at
FROM plugin_usage_events e
WHERE e.event_type IN ('PostToolUse', 'PostToolUseFailure') AND e.tool_name = 'Skill'
  AND e.metadata->'tool_input'->>'skill' IS NOT NULL
UNION ALL
SELECT e.id, e.user_id, e.session_id, e.plugin_id,
       COALESCE(e.metadata->>'skill_ref', 'opencode:' || replace(e.metadata->'tool_input'->>'name', '_', '-')) AS skill,
       e.metadata->>'tool_use_id' AS tool_use_id, 'tool'::text AS source, e.created_at AS invoked_at
FROM plugin_usage_events e
WHERE e.event_type IN ('PostToolUse', 'PostToolUseFailure') AND e.tool_name = 'skill'
  AND e.metadata->'tool_input'->>'name' IS NOT NULL;
