-- Reverse hide-nondemo.sql: put the moved rows back (parents first) and
-- un-archive the users it archived, then drop the demo_hidden schema.
CREATE OR REPLACE FUNCTION pg_temp.unhide(tbl text) RETURNS void LANGUAGE plpgsql AS $$
BEGIN
    IF to_regclass('demo_hidden.' || tbl) IS NOT NULL THEN
        EXECUTE format('INSERT INTO public.%I SELECT * FROM demo_hidden.%I ON CONFLICT DO NOTHING', tbl, tbl);
    END IF;
END $$;
SELECT pg_temp.unhide(t) FROM unnest(ARRAY[
    'agent_runs', 'mcp_tool_executions', 'ai_requests', 'ai_request_messages', 'ai_request_tool_calls',
    'ai_request_scopes', 'ai_request_attributions', 'ai_request_client_evidence', 'ai_request_payloads',
    'ai_safety_findings', 'mcp_artifacts', 'agent_run_events', 'agent_farm_runs', 'trace_facts',
    'conversation_facts', 'conversation_analyses', 'governance_decisions', 'plugin_usage_events',
    'plugin_session_summaries', 'plugin_usage_daily', 'admin_usage_daily_rollups',
    'managed_installation_receipts', 'managed_consumer_session_bindings']) AS t;
UPDATE users u SET status = a.prior_status, roles = a.prior_roles, archived_at = NULL, archive_reason = NULL
  FROM demo_hidden.archived_users a WHERE a.id = u.id;
DROP SCHEMA demo_hidden CASCADE;
