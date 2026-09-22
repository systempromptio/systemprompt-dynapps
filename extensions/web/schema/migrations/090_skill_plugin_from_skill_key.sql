-- A skill invocation was credited to the plugin whose hooks reported it. The
-- session-global governance hooks belong to exactly one plugin, so every
-- marketplace's skills landed on that owner and the Skills page filtered by
-- any other marketplace found nothing. The invoked skill is `<plugin>:<skill>`;
-- its prefix is the owning plugin. Same leading columns, one appended
-- (`reporter_plugin_id`, the hook owner), so dependent views and the
-- feedback functions that select `s.*` stay valid.
CREATE OR REPLACE VIEW analysis_skill_events AS
SELECT s.id, s.user_id, s.session_id,
       COALESCE(NULLIF(split_part(s.skill, ':', 1), ''), s.reporter_plugin_id) AS plugin_id,
       s.skill, s.tool_use_id, s.source, s.invoked_at, s.reporter_plugin_id
FROM (
    SELECT e.id, e.user_id, e.session_id, e.plugin_id AS reporter_plugin_id,
           replace(substring(e.prompt_preview from '^/([A-Za-z0-9._-]+:[A-Za-z0-9._-]+)'), '_', '-') AS skill,
           NULL::text AS tool_use_id, 'slash'::text AS source, e.created_at AS invoked_at
    FROM plugin_usage_events e
    WHERE e.event_type = 'UserPromptSubmit'
      AND e.prompt_preview ~ '^/[A-Za-z0-9._-]+:[A-Za-z0-9._-]+'
    UNION ALL
    SELECT e.id, e.user_id, e.session_id, e.plugin_id,
           replace(e.metadata->'tool_input'->>'skill', '_', '-') AS skill,
           e.tool_use_id, 'tool'::text AS source, e.created_at AS invoked_at
    FROM plugin_usage_events e
    WHERE e.event_type IN ('PostToolUse', 'PostToolUseFailure') AND e.tool_name = 'Skill'
      AND e.metadata->'tool_input'->>'skill' IS NOT NULL
    UNION ALL
    SELECT e.id, e.user_id, e.session_id, e.plugin_id,
           COALESCE(e.metadata->>'skill_ref', 'opencode:' || replace(e.metadata->'tool_input'->>'name', '_', '-')) AS skill,
           e.tool_use_id, 'tool'::text AS source, e.created_at AS invoked_at
    FROM plugin_usage_events e
    WHERE e.event_type IN ('PostToolUse', 'PostToolUseFailure') AND e.tool_name = 'skill'
      AND e.metadata->'tool_input'->>'name' IS NOT NULL
) s;
