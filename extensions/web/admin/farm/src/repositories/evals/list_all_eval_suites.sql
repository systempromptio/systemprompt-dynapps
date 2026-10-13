SELECT id AS "id: EvalSuiteId", farm_id AS "farm_id: FarmId", agent_id AS "agent_id: FarmAgentId",
       source_path, case_count, synced_commit_sha, updated_at
FROM agent_farm_eval_suites
ORDER BY farm_id, agent_id, id
