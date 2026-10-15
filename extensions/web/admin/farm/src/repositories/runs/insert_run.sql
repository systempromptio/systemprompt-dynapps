INSERT INTO agent_farm_runs (id, agent_run_id, farm_id, agent_id, user_id, triggered_via, host,
                             status, attempt, prompt, fixture, model, permission_mode, max_turns,
                             max_budget_usd, timeout_seconds, source_commit_sha, eval_result_id, client_id)
VALUES ($1, $2, $3, $4, $5, $6, $7, 'queued', $8, $9, $10, $11, $12, $13, $14, $15, $16, $17,
        COALESCE($18, (SELECT client_id FROM agent_farms WHERE id = $3)))
RETURNING id AS "id: FarmRunId", agent_run_id AS "agent_run_id: AgentRunId", farm_id AS "farm_id: FarmId",
       agent_id AS "agent_id: FarmAgentId", user_id AS "user_id: UserId", triggered_via, host, status,
       attempt, prompt, fixture, model, permission_mode, max_turns, max_budget_usd, timeout_seconds,
       source_commit_sha, context_id AS "context_id: ContextId", error_class, cost_microdollars,
       event_count, tool_call_count, gateway_request_count, eval_result_id, started_at, finished_at,
       cancel_requested_at, created_at, updated_at, client_id AS "client_id: ClientId"
