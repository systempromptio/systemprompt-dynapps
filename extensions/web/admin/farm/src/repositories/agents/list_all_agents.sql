SELECT farm_id AS "farm_id: FarmId", id AS "id: FarmAgentId", name, description, model, tools,
       plugin_id AS "plugin_id: PluginId", skills, entry_prompt, fixture, max_turns,
       max_budget_usd, output_path, output_format, source_path, body, synced_commit_sha, host,
       updated_at
FROM agent_farm_agents
ORDER BY farm_id, name, id
