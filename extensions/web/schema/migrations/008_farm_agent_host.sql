-- A farm agent names the CLI host its runs execute on. schema/52_agent_farm.sql
-- gained agent_farm_agents.host without a migration, so databases installed
-- before it have no such column and farm sync fails writing it. Existing
-- agents ran on Claude Code; they take the column default. Same DDL as
-- schema/52_agent_farm.sql.

ALTER TABLE agent_farm_agents
    ADD COLUMN IF NOT EXISTS host TEXT NOT NULL DEFAULT 'claude-code';
ALTER TABLE agent_farm_agents
    DROP CONSTRAINT IF EXISTS agent_farm_agents_host_check;
ALTER TABLE agent_farm_agents
    ADD CONSTRAINT agent_farm_agents_host_check
        CHECK (host IN ('claude-code', 'codex-cli', 'opencode', 'hermes'));
