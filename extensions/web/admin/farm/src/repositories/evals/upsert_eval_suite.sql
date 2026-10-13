INSERT INTO agent_farm_eval_suites (id, farm_id, agent_id, source_path, case_count, synced_commit_sha)
VALUES ($1, $2, $3, $4, $5, $6)
ON CONFLICT (id) DO UPDATE SET
    farm_id = EXCLUDED.farm_id,
    agent_id = EXCLUDED.agent_id,
    source_path = EXCLUDED.source_path,
    case_count = EXCLUDED.case_count,
    synced_commit_sha = EXCLUDED.synced_commit_sha,
    updated_at = NOW()
