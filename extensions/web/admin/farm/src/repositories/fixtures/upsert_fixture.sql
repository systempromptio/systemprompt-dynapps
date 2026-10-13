INSERT INTO agent_farm_fixtures (farm_id, name, file_count, bytes, stored_path, synced_commit_sha)
VALUES ($1, $2, $3, $4, $5, $6)
ON CONFLICT (farm_id, name) DO UPDATE SET
    file_count = EXCLUDED.file_count,
    bytes = EXCLUDED.bytes,
    stored_path = EXCLUDED.stored_path,
    synced_commit_sha = EXCLUDED.synced_commit_sha,
    updated_at = NOW()
