-- Derived rows the live ingestion path writes per event, rebuilt in bulk for
-- the seeded users: session summaries, daily hook counters, the admin daily
-- rollup and the conversation/trace fact tables.
CREATE TEMP TABLE vet_users2 ON COMMIT DROP AS
    SELECT id FROM users WHERE email LIKE '%@vetgroup-demo.example';

INSERT INTO plugin_session_summaries
    (id, session_id, user_id, plugin_id, started_at, ended_at, total_events, tool_uses, prompts, errors,
     content_input_bytes, content_output_bytes, subagent_spawns, user_prompts, automated_actions,
     client_source, permission_mode, status)
SELECT 'sess_' || e.session_id, e.session_id, e.user_id, 'dynapps-commons', MIN(e.created_at), MAX(e.created_at),
       COUNT(*),
       COUNT(*) FILTER (WHERE e.event_type IN ('PostToolUse', 'PostToolUseFailure')),
       COUNT(*) FILTER (WHERE e.event_type = 'UserPromptSubmit'),
       COUNT(*) FILTER (WHERE e.event_type = 'PostToolUseFailure'),
       SUM(e.content_input_bytes), SUM(e.content_output_bytes), 0,
       COUNT(*) FILTER (WHERE e.event_type = 'UserPromptSubmit'), 0,
       'claude-code', 'default', 'completed'
FROM plugin_usage_events e
WHERE e.user_id IN (SELECT id FROM vet_users2)
GROUP BY e.session_id, e.user_id;

INSERT INTO plugin_usage_daily
    (id, date, user_id, plugin_id, event_type, tool_name, event_count, content_input_bytes,
     content_output_bytes, error_count)
SELECT d || '_' || user_id || '_' || event_type || '_' || COALESCE(tool_name, ''), d, user_id,
       'dynapps-commons', event_type, tool_name, COUNT(*), SUM(content_input_bytes),
       SUM(content_output_bytes), COUNT(*) FILTER (WHERE event_type = 'PostToolUseFailure')
FROM (SELECT (created_at AT TIME ZONE 'UTC')::date AS d, * FROM plugin_usage_events
      WHERE user_id IN (SELECT id FROM vet_users2)) e
GROUP BY d, user_id, event_type, tool_name;

INSERT INTO admin_usage_daily_rollups
    (user_id, date, sessions_count, prompts, tool_uses, errors, ai_requests_count, input_tokens,
     output_tokens, cost_microdollars)
SELECT u.user_id, u.date,
       COALESCE(s.n, 0), COALESCE(h.prompts, 0), COALESCE(h.tool_uses, 0), COALESCE(h.errors, 0),
       COALESCE(r.n, 0), COALESCE(r.inp, 0), COALESCE(r.out, 0), COALESCE(r.cost, 0)
FROM (
    SELECT user_id, (created_at AT TIME ZONE 'UTC')::date AS date FROM ai_requests
     WHERE user_id IN (SELECT id FROM vet_users2)
    UNION
    SELECT user_id, date FROM plugin_usage_daily WHERE user_id IN (SELECT id FROM vet_users2)
) u
LEFT JOIN (SELECT user_id, (started_at AT TIME ZONE 'UTC')::date AS date, COUNT(*)::int AS n
             FROM plugin_session_summaries WHERE user_id IN (SELECT id FROM vet_users2) GROUP BY 1, 2) s
       ON s.user_id = u.user_id AND s.date = u.date
LEFT JOIN (SELECT user_id, date,
                  SUM(event_count) FILTER (WHERE event_type = 'UserPromptSubmit') AS prompts,
                  SUM(event_count) FILTER (WHERE event_type IN ('PostToolUse', 'PostToolUseFailure')) AS tool_uses,
                  SUM(error_count) AS errors
             FROM plugin_usage_daily WHERE user_id IN (SELECT id FROM vet_users2) GROUP BY 1, 2) h
       ON h.user_id = u.user_id AND h.date = u.date
LEFT JOIN (SELECT user_id, (created_at AT TIME ZONE 'UTC')::date AS date, COUNT(*) AS n,
                  SUM(input_tokens) AS inp, SUM(output_tokens) AS out, SUM(cost_microdollars) AS cost
             FROM ai_requests WHERE user_id IN (SELECT id FROM vet_users2) GROUP BY 1, 2) r
       ON r.user_id = u.user_id AND r.date = u.date;

UPDATE admin_usage_daily_rollups r
   SET group_id = d.primary_group_id, project_id = d.primary_project_id
  FROM user_scope_defaults d
 WHERE d.user_id = r.user_id AND r.user_id IN (SELECT id FROM vet_users2);
