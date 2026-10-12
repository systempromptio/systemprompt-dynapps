UPDATE agent_farm_runs SET claude_session_id = $2, updated_at = NOW() WHERE id = $1
