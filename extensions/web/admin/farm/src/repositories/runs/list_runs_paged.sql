-- $8 is a sort key bound as text and picked by CASE, never interpolated.
SELECT id AS "id: FarmRunId", farm_id AS "farm_id: FarmId", agent_id AS "agent_id: FarmAgentId",
       user_id AS "user_id: UserId", triggered_via, status, prompt, fixture, model,
       permission_mode, max_turns, max_budget_usd, timeout_seconds, source_commit_sha, run_dir,
       gateway_session_id, claude_session_id, context_id AS "context_id: ContextId", api_key_id,
       pid, started_at, finished_at, duration_ms, num_turns, input_tokens, output_tokens,
       cost_microdollars, result_subtype, exit_code, error, diff_summary, eval_result_id,
       cancel_requested_at, created_at, updated_at
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
    CASE WHEN $8 = 'duration_desc' THEN duration_ms END DESC NULLS LAST,
    CASE WHEN $8 = 'cost_desc' THEN cost_microdollars END DESC NULLS LAST,
    CASE WHEN $8 = 'turns_desc' THEN num_turns END DESC NULLS LAST,
    created_at DESC,
    id
LIMIT $9 OFFSET $10
