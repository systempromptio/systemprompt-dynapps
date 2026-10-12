UPDATE agent_farms SET sync_error = $2, updated_at = NOW() WHERE id = $1
