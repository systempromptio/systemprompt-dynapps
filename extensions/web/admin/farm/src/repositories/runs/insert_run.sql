INSERT INTO agent_farm_runs (id, farm_id, agent_id, user_id, triggered_via, status, prompt,
                             fixture, model, permission_mode, max_turns, max_budget_usd,
                             timeout_seconds, source_commit_sha, eval_result_id)
VALUES ($1, $2, $3, $4, $5, 'queued', $6, $7, $8, $9, $10, $11, $12, $13, $14)
RETURNING id AS "id: FarmRunId", farm_id AS "farm_id: FarmId", agent_id AS "agent_id: FarmAgentId",
       user_id AS "user_id: UserId", triggered_via, status, prompt, fixture, model,
       permission_mode, max_turns, max_budget_usd, timeout_seconds, source_commit_sha, run_dir,
       gateway_session_id, claude_session_id, context_id AS "context_id: ContextId", api_key_id,
       pid, started_at, finished_at, duration_ms, num_turns, input_tokens, output_tokens,
       cost_microdollars, result_subtype, exit_code, error, diff_summary, eval_result_id,
       cancel_requested_at, created_at, updated_at
