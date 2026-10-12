SELECT id AS "id: FarmRunId", agent_run_id AS "agent_run_id: AgentRunId", farm_id AS "farm_id: FarmId",
       agent_id AS "agent_id: FarmAgentId", user_id AS "user_id: UserId", triggered_via, host, status,
       attempt, prompt, fixture, model, permission_mode, max_turns, max_budget_usd, timeout_seconds,
       source_commit_sha, context_id AS "context_id: ContextId", error_class, cost_microdollars,
       event_count, tool_call_count, gateway_request_count, eval_result_id, started_at, finished_at,
       cancel_requested_at, created_at, updated_at
FROM agent_farm_runs
WHERE ($1::text IS NULL OR farm_id = $1)
  AND ($2::text IS NULL OR agent_id = $2)
ORDER BY created_at DESC, id
LIMIT $3
