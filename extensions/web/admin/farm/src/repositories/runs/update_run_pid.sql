UPDATE agent_farm_runs SET pid = $2, updated_at = NOW() WHERE id = $1
