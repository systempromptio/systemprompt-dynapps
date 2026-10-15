-- One task-completion report per terminal farm run, written by the runner's
-- observer: the final message, the files the run wrote, the Odoo writes its
-- MCP calls made, and the eval verdict. Same DDL as schema/52_agent_farm.sql.

CREATE TABLE IF NOT EXISTS agent_farm_run_reports (
    run_id TEXT PRIMARY KEY REFERENCES agent_farm_runs(id) ON DELETE CASCADE,
    client_id TEXT REFERENCES clients(id) ON DELETE SET NULL,
    completed BOOLEAN NOT NULL DEFAULT false,
    summary TEXT,
    final_message TEXT,
    outputs JSONB NOT NULL DEFAULT '[]',
    odoo_writes JSONB NOT NULL DEFAULT '[]',
    eval_passed BOOLEAN,
    eval_score DOUBLE PRECISION,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
