UPDATE agent_farms
SET synced_commit_sha = $2, synced_at = NOW(), synced_by = $3, sync_error = NULL,
    updated_at = NOW()
WHERE id = $1
