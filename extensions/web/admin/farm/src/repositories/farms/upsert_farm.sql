INSERT INTO agent_farms (id, name, description, marketplace_id, source_repo, source_ref,
                         source_subdir, permission_mode, timeout_seconds, max_turns,
                         max_budget_usd, model, run_as_user_id, project_id)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
ON CONFLICT (id) DO UPDATE SET
    name = EXCLUDED.name,
    description = EXCLUDED.description,
    marketplace_id = EXCLUDED.marketplace_id,
    source_repo = EXCLUDED.source_repo,
    source_ref = EXCLUDED.source_ref,
    source_subdir = EXCLUDED.source_subdir,
    permission_mode = EXCLUDED.permission_mode,
    timeout_seconds = EXCLUDED.timeout_seconds,
    max_turns = EXCLUDED.max_turns,
    max_budget_usd = EXCLUDED.max_budget_usd,
    model = EXCLUDED.model,
    run_as_user_id = EXCLUDED.run_as_user_id,
    project_id = EXCLUDED.project_id,
    updated_at = NOW()
