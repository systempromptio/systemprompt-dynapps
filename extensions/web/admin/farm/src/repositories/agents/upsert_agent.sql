INSERT INTO agent_farm_agents (farm_id, id, name, description, model, tools, plugin_id, skills,
                               entry_prompt, fixture, max_turns, max_budget_usd, output_path,
                               output_format, source_path, body, synced_commit_sha, host)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
ON CONFLICT (farm_id, id) DO UPDATE SET
    name = EXCLUDED.name,
    description = EXCLUDED.description,
    model = EXCLUDED.model,
    tools = EXCLUDED.tools,
    plugin_id = EXCLUDED.plugin_id,
    skills = EXCLUDED.skills,
    entry_prompt = EXCLUDED.entry_prompt,
    fixture = EXCLUDED.fixture,
    max_turns = EXCLUDED.max_turns,
    max_budget_usd = EXCLUDED.max_budget_usd,
    output_path = EXCLUDED.output_path,
    output_format = EXCLUDED.output_format,
    source_path = EXCLUDED.source_path,
    body = EXCLUDED.body,
    synced_commit_sha = EXCLUDED.synced_commit_sha,
    host = EXCLUDED.host,
    updated_at = NOW()
