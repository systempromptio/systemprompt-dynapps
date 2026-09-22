-- C12: conversation_facts vs a fresh refresh_conversation_facts(NULL), inside a rolled-back transaction
BEGIN;
CREATE TEMP TABLE cf_before AS SELECT * FROM conversation_facts;
SELECT refresh_conversation_facts(NULL);
\echo == C12 rows before / after refresh
SELECT (SELECT count(*) FROM cf_before) before, (SELECT count(*) FROM conversation_facts) after;
\echo == C12 per-column differences after refresh
SELECT count(*) AS compared,
  count(*) FILTER (WHERE b.request_count IS DISTINCT FROM a.request_count) request_count,
  count(*) FILTER (WHERE b.turn_count IS DISTINCT FROM a.turn_count) turn_count,
  count(*) FILTER (WHERE b.cost_microdollars IS DISTINCT FROM a.cost_microdollars) cost,
  count(*) FILTER (WHERE b.input_tokens IS DISTINCT FROM a.input_tokens OR b.output_tokens IS DISTINCT FROM a.output_tokens) tokens,
  count(*) FILTER (WHERE b.first_at IS DISTINCT FROM a.first_at OR b.last_at IS DISTINCT FROM a.last_at) first_last,
  count(*) FILTER (WHERE b.tool_calls_intended IS DISTINCT FROM a.tool_calls_intended) intended,
  count(*) FILTER (WHERE b.tool_calls_executed IS DISTINCT FROM a.tool_calls_executed) executed,
  count(*) FILTER (WHERE b.gov_allow IS DISTINCT FROM a.gov_allow OR b.gov_warn IS DISTINCT FROM a.gov_warn OR b.gov_deny IS DISTINCT FROM a.gov_deny) gov,
  count(*) FILTER (WHERE b.hook_event_count IS DISTINCT FROM a.hook_event_count) hook_events,
  count(*) FILTER (WHERE b.skill_invocations IS DISTINCT FROM a.skill_invocations) skills
FROM cf_before b JOIN conversation_facts a USING (context_id);
\echo == C12 what the facts actually hold (sums) — the link-plane columns
SELECT count(*) contexts, sum(request_count) requests, sum(tool_calls_intended) intended, sum(tool_calls_executed) executed, sum(tool_calls_failed) failed,
       sum(artifact_count) artifacts, sum(gov_allow) gov_allow, sum(gov_warn) gov_warn, sum(gov_deny) gov_deny, sum(hook_event_count) hook_events, sum(prompt_count) prompts, sum(skill_invocations) skills,
       count(*) FILTER (WHERE client_session_id IS NULL) no_client_session
FROM conversation_facts;
\echo == C12 contexts sharing one client_session_id (hook/gov counts re-counted per context)
SELECT n AS contexts_per_client_session, count(*) client_sessions FROM (SELECT client_session_id, count(*) n FROM conversation_facts WHERE client_session_id IS NOT NULL GROUP BY 1) x GROUP BY 1 ORDER BY 1;
\echo == C12 over-count: sum of per-context hook_event_count vs distinct hook events for shared client sessions
WITH shared AS (SELECT client_session_id FROM conversation_facts WHERE client_session_id IS NOT NULL GROUP BY 1 HAVING count(*) > 1)
SELECT sum(f.hook_event_count) AS summed_over_contexts,
       (SELECT count(*) FROM plugin_usage_events p WHERE p.session_id IN (SELECT client_session_id FROM shared)) AS actual_events,
       sum(f.gov_allow + f.gov_warn + f.gov_deny) AS summed_gov,
       (SELECT count(*) FROM governance_decisions g WHERE g.session_id IN (SELECT client_session_id FROM shared)) AS actual_gov_by_session
FROM conversation_facts f WHERE f.client_session_id IN (SELECT client_session_id FROM shared);
ROLLBACK;
