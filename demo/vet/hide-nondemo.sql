-- Hide every non-demo identity's usage so lists show only the fictional vet
-- group. Rows are MOVED, not dropped: each is copied into schema demo_hidden
-- (same table name) and then deleted, in one transaction with
-- systemprompt.audit_maintenance on. unhide-nondemo.sql puts them back.
-- Non-demo users other than the instance admin are archived (archived_at),
-- the same status/archived_at pair the console's own archive action sets, so
-- the user index skips them; the admin login is untouched.
CREATE SCHEMA IF NOT EXISTS demo_hidden;
CREATE TEMP TABLE nd_users ON COMMIT DROP AS
    SELECT id FROM users WHERE email NOT LIKE '%@vetgroup-demo.example';
CREATE TEMP TABLE nd_requests ON COMMIT DROP AS
    SELECT id, trace_id FROM ai_requests WHERE user_id IN (SELECT id FROM nd_users);
CREATE TEMP TABLE nd_exec ON COMMIT DROP AS
    SELECT mcp_execution_id FROM mcp_tool_executions WHERE user_id IN (SELECT id FROM nd_users);
CREATE TEMP TABLE nd_runs ON COMMIT DROP AS
    SELECT id FROM agent_runs WHERE user_id IN (SELECT id FROM nd_users);

CREATE OR REPLACE FUNCTION pg_temp.hide(tbl text, pred text) RETURNS void LANGUAGE plpgsql AS $$
BEGIN
    IF to_regclass('demo_hidden.' || tbl) IS NULL THEN
        EXECUTE format('CREATE TABLE demo_hidden.%I AS SELECT * FROM public.%I WITH NO DATA', tbl, tbl);
    END IF;
    EXECUTE format('INSERT INTO demo_hidden.%I SELECT * FROM public.%I WHERE %s', tbl, tbl, pred);
    EXECUTE format('DELETE FROM public.%I WHERE %s', tbl, pred);
END $$;

-- children first so their copies exist before parent deletes cascade
SELECT pg_temp.hide('ai_request_messages', 'request_id IN (SELECT id FROM nd_requests)');
SELECT pg_temp.hide('ai_request_tool_calls', 'request_id IN (SELECT id FROM nd_requests)');
SELECT pg_temp.hide('ai_request_scopes', 'request_id IN (SELECT id FROM nd_requests)');
SELECT pg_temp.hide('ai_request_attributions', 'request_id IN (SELECT id FROM nd_requests)');
SELECT pg_temp.hide('ai_request_client_evidence', 'ai_request_id IN (SELECT id FROM nd_requests)');
SELECT pg_temp.hide('ai_request_payloads', 'ai_request_id IN (SELECT id FROM nd_requests)');
SELECT pg_temp.hide('ai_safety_findings', 'ai_request_id IN (SELECT id FROM nd_requests)');
SELECT pg_temp.hide('mcp_artifacts', 'mcp_execution_id IN (SELECT mcp_execution_id FROM nd_exec)');
SELECT pg_temp.hide('agent_run_events', 'run_id IN (SELECT id FROM nd_runs)');
SELECT pg_temp.hide('agent_farm_runs', 'agent_run_id IN (SELECT id FROM nd_runs)');
SELECT pg_temp.hide('trace_facts', 'trace_id IN (SELECT trace_id FROM nd_requests)');
SELECT pg_temp.hide('conversation_analyses', 'user_id IN (SELECT id FROM nd_users)');
SELECT pg_temp.hide('conversation_facts', 'user_id IN (SELECT id FROM nd_users)');
SELECT pg_temp.hide('ai_requests', 'id IN (SELECT id FROM nd_requests)');
SELECT pg_temp.hide('mcp_tool_executions', 'mcp_execution_id IN (SELECT mcp_execution_id FROM nd_exec)');
SELECT pg_temp.hide('governance_decisions', 'user_id IN (SELECT id FROM nd_users)');
SELECT pg_temp.hide('plugin_usage_events', 'user_id IN (SELECT id FROM nd_users)');
SELECT pg_temp.hide('plugin_session_summaries', 'user_id IN (SELECT id FROM nd_users)');
SELECT pg_temp.hide('plugin_usage_daily', 'user_id IN (SELECT id FROM nd_users)');
SELECT pg_temp.hide('admin_usage_daily_rollups', 'user_id IN (SELECT id FROM nd_users)');
SELECT pg_temp.hide('managed_consumer_session_bindings', 'receipt_id IN (SELECT id FROM managed_installation_receipts WHERE consumer_id IN (SELECT id FROM nd_users))');
SELECT pg_temp.hide('managed_installation_receipts', 'consumer_id IN (SELECT id FROM nd_users)');
SELECT pg_temp.hide('agent_runs', 'id IN (SELECT id FROM nd_runs)');

-- archive the people (not the instance admin, who signs in to run the demo)
CREATE TABLE IF NOT EXISTS demo_hidden.archived_users (id text PRIMARY KEY, prior_status text NOT NULL,
    prior_roles text[] NOT NULL);
INSERT INTO demo_hidden.archived_users
    SELECT id, status, roles FROM users WHERE id IN (SELECT id FROM nd_users) AND archived_at IS NULL
      AND 'admin' <> ALL(roles) AND email <> 'ed@systemprompt.io'
    ON CONFLICT DO NOTHING;
-- The console roster filters only the anonymous role, not archived rows, so
-- the hidden accounts also carry that role until unhide restores prior_roles.
UPDATE users SET status = 'deleted', roles = array_append(roles, 'anonymous'), archived_at = now(), archive_reason = 'hidden for vet demo (demo/vet/hide-nondemo.sql)'
 WHERE id IN (SELECT id FROM demo_hidden.archived_users) AND archived_at IS NULL;
