-- `plugin_session_summaries.plugin_id` was never written: the ingestion
-- upsert listed every other column and left the plugin at NULL for every
-- session, so nothing keyed on it. Every hook event carries the plugin
-- whose hooks reported it (the session-global governance owner), so the
-- summary takes it from the events. Same function, one column added; the
-- backfill below fills the sessions already summarised.
CREATE OR REPLACE FUNCTION drain_ingestion_outbox(batch_size integer)
RETURNS bigint LANGUAGE plpgsql AS $$
DECLARE event_ids text[]; affected bigint;
BEGIN
    SELECT array_agg(event_id) INTO event_ids FROM (
        SELECT event_id FROM ingestion_outbox WHERE processed_at IS NULL ORDER BY created_at
        LIMIT least(greatest(batch_size, 1), 10000) FOR UPDATE SKIP LOCKED
    ) pending;
    IF event_ids IS NULL THEN RETURN 0; END IF;
    PERFORM pg_advisory_xact_lock(hashtextextended('ingestion:' || session_id, 0))
    FROM (SELECT DISTINCT session_id FROM plugin_usage_events WHERE id = ANY(event_ids) ORDER BY session_id) owners;
    INSERT INTO plugin_session_summaries(id,session_id,user_id,total_events,tool_uses,prompts,errors,
        content_input_bytes,content_output_bytes,started_at,subagent_spawns,user_prompts,automated_actions,plugin_id)
    SELECT 'sess_' || e.session_id,e.session_id,e.user_id,count(*),
        count(*) FILTER(WHERE event_type IN ('PostToolUse','PostToolUseFailure')),
        count(*) FILTER(WHERE event_type = 'UserPromptSubmit'),
        count(*) FILTER(WHERE event_type = 'PostToolUseFailure'),
        coalesce(sum(content_input_bytes),0),coalesce(sum(content_output_bytes),0),
        min(e.created_at),
        count(*) FILTER(WHERE event_type='SubagentStop'),
        count(*) FILTER(WHERE event_type='UserPromptSubmit' AND coalesce(metadata->>'agent_id','')=''),
        count(*) FILTER(WHERE event_type IN ('PostToolUse','PostToolUseFailure') AND coalesce(metadata->>'agent_id','')<>''),
        max(e.plugin_id)
    FROM plugin_usage_events e WHERE (e.user_id,e.session_id) IN (
        SELECT user_id,session_id FROM plugin_usage_events WHERE id = ANY(event_ids))
    GROUP BY e.user_id,e.session_id
    ON CONFLICT(session_id) DO UPDATE SET total_events=EXCLUDED.total_events,
        tool_uses=EXCLUDED.tool_uses,prompts=EXCLUDED.prompts,errors=EXCLUDED.errors,
        content_input_bytes=EXCLUDED.content_input_bytes,content_output_bytes=EXCLUDED.content_output_bytes,
        subagent_spawns=EXCLUDED.subagent_spawns,
        user_prompts=EXCLUDED.user_prompts,automated_actions=EXCLUDED.automated_actions,
        plugin_id=coalesce(EXCLUDED.plugin_id,plugin_session_summaries.plugin_id),updated_at=now()
    WHERE plugin_session_summaries.user_id=EXCLUDED.user_id;
    UPDATE ingestion_outbox SET processed_at=now() WHERE event_id=ANY(event_ids);
    GET DIAGNOSTICS affected = ROW_COUNT;
    RETURN affected;
END $$;

UPDATE plugin_session_summaries s
SET plugin_id = e.plugin_id
FROM (
    SELECT session_id, max(plugin_id) AS plugin_id
    FROM plugin_usage_events
    WHERE plugin_id IS NOT NULL
    GROUP BY session_id
) e
WHERE e.session_id = s.session_id
  AND s.plugin_id IS NULL;
