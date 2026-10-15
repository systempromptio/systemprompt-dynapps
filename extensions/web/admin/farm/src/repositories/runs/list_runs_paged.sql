-- $8 is a sort key bound as text and picked by CASE, never interpolated.
SELECT id AS "id: FarmRunId", agent_run_id AS "agent_run_id: AgentRunId", farm_id AS "farm_id: FarmId",
       agent_id AS "agent_id: FarmAgentId", user_id AS "user_id: UserId", triggered_via, host, status,
       attempt, prompt, fixture, model, permission_mode, max_turns, max_budget_usd, timeout_seconds,
       source_commit_sha, context_id AS "context_id: ContextId", error_class, cost_microdollars,
       event_count, tool_call_count, gateway_request_count, eval_result_id, started_at, finished_at,
       cancel_requested_at, created_at, updated_at, client_id AS "client_id: ClientId"
FROM agent_farm_runs
WHERE ($1::text IS NULL OR farm_id = $1)
  AND ($2::text IS NULL OR agent_id = $2)
  AND ($3::text IS NULL OR user_id = $3)
  AND ($4::text IS NULL OR status = $4)
  AND ($5::text IS NULL OR triggered_via = $5)
  AND ($6::timestamptz IS NULL OR created_at >= $6)
  AND ($7::timestamptz IS NULL OR created_at < $7)
ORDER BY
    CASE WHEN $8 = 'created_asc' THEN created_at END ASC,
    CASE WHEN $8 = 'duration_desc' THEN finished_at - started_at END DESC NULLS LAST,
    CASE WHEN $8 = 'cost_desc' THEN cost_microdollars END DESC NULLS LAST,
    CASE WHEN $8 = 'tool_calls_desc' THEN tool_call_count END DESC NULLS LAST,
    created_at DESC,
    id
LIMIT $9 OFFSET $10
