SELECT farm_id AS "farm_id: FarmId", name, file_count, bytes, stored_path, synced_commit_sha,
       updated_at
FROM agent_farm_fixtures
WHERE farm_id = $1
ORDER BY name
