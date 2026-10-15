SELECT id AS "id: FarmId", name, description, marketplace_id AS "marketplace_id: MarketplaceId",
       source_repo, source_ref, source_subdir, permission_mode, timeout_seconds, max_turns,
       max_budget_usd, model, synced_commit_sha, synced_at, synced_by, sync_error,
       owner_user_id, run_as_user_id AS "run_as_user_id: UserId", run_as_mode, project_id AS "project_id: ProjectId", created_at, updated_at,
       client_id AS "client_id: ClientId"
FROM agent_farms
ORDER BY name, id
