-- Consolidated schema: Agent farm
--
-- Farms are synced from a Git marketplace (`services/web/config/agent-farms.yaml`);
-- their agents, fixtures and eval suites are projections of the synced commit.
-- A farm run is a projection over one core agent_runs row (the executor owns
-- leases, credentials and events in agent_run_events); the counters here are
-- stamped from it at finish time.
-- Ids are TEXT (UUID strings for runs and eval rows) so the typed id newtypes
-- decode without a cast. Every statement is idempotent: this file is the only
-- place the farm DDL lives.

CREATE TABLE IF NOT EXISTS agent_farms (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    marketplace_id TEXT NOT NULL,
    source_repo TEXT NOT NULL,
    source_ref TEXT NOT NULL DEFAULT 'main',
    source_subdir TEXT NOT NULL DEFAULT 'farm',
    permission_mode TEXT NOT NULL DEFAULT 'acceptEdits',
    timeout_seconds INTEGER NOT NULL DEFAULT 1200,
    max_turns INTEGER NOT NULL DEFAULT 40,
    max_budget_usd DOUBLE PRECISION NOT NULL DEFAULT 5.0,
    model TEXT,
    synced_commit_sha TEXT,
    synced_at TIMESTAMPTZ,
    synced_by TEXT REFERENCES users(id) ON DELETE SET NULL,
    sync_error TEXT,
    owner_user_id TEXT REFERENCES users(id) ON DELETE SET NULL,
    run_as_user_id TEXT REFERENCES users(id) ON DELETE SET NULL,
    project_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS agent_farm_agents (
    farm_id TEXT NOT NULL REFERENCES agent_farms(id) ON DELETE CASCADE,
    id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    model TEXT,
    tools TEXT[] NOT NULL DEFAULT '{}',
    plugin_id TEXT NOT NULL,
    skills TEXT[] NOT NULL DEFAULT '{}',
    entry_prompt TEXT NOT NULL,
    fixture TEXT,
    max_turns INTEGER,
    max_budget_usd DOUBLE PRECISION,
    output_path TEXT,
    output_format TEXT,
    source_path TEXT NOT NULL,
    body TEXT NOT NULL,
    synced_commit_sha TEXT NOT NULL,
    host TEXT NOT NULL DEFAULT 'claude-code'
        CHECK (host IN ('claude-code', 'codex-cli', 'opencode', 'hermes')),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (farm_id, id)
);

CREATE TABLE IF NOT EXISTS agent_farm_fixtures (
    farm_id TEXT NOT NULL REFERENCES agent_farms(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    file_count INTEGER NOT NULL DEFAULT 0,
    bytes BIGINT NOT NULL DEFAULT 0,
    stored_path TEXT NOT NULL,
    synced_commit_sha TEXT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (farm_id, name)
);

CREATE TABLE IF NOT EXISTS agent_farm_runs (
    id TEXT PRIMARY KEY,
    agent_run_id TEXT NOT NULL REFERENCES agent_runs(id) ON DELETE CASCADE,
    farm_id TEXT NOT NULL REFERENCES agent_farms(id) ON DELETE CASCADE,
    agent_id TEXT NOT NULL,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    triggered_via TEXT NOT NULL
        CHECK (triggered_via IN ('dashboard', 'api', 'eval', 'mcp', 'console', 'demo')),
    host TEXT NOT NULL DEFAULT 'claude-code'
        CHECK (host IN ('claude-code', 'codex-cli', 'opencode', 'hermes')),
    status TEXT NOT NULL DEFAULT 'queued'
        CHECK (status IN ('queued', 'preparing', 'running', 'succeeded', 'failed', 'cancelled', 'timed_out')),
    attempt INTEGER NOT NULL DEFAULT 1,
    prompt TEXT NOT NULL,
    fixture TEXT,
    model TEXT,
    permission_mode TEXT NOT NULL,
    max_turns INTEGER NOT NULL,
    max_budget_usd DOUBLE PRECISION NOT NULL,
    timeout_seconds INTEGER NOT NULL,
    source_commit_sha TEXT,
    context_id TEXT,
    error_class TEXT,
    cost_microdollars BIGINT NOT NULL DEFAULT 0,
    event_count INTEGER NOT NULL DEFAULT 0,
    tool_call_count INTEGER NOT NULL DEFAULT 0,
    gateway_request_count INTEGER NOT NULL DEFAULT 0,
    eval_result_id TEXT,
    started_at TIMESTAMPTZ,
    finished_at TIMESTAMPTZ,
    cancel_requested_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_agent_farm_runs_status ON agent_farm_runs(status);
CREATE INDEX IF NOT EXISTS idx_agent_farm_runs_user ON agent_farm_runs(user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_agent_farm_runs_agent ON agent_farm_runs(farm_id, agent_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_agent_farm_runs_created ON agent_farm_runs(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_agent_farm_runs_agent_run ON agent_farm_runs(agent_run_id);

CREATE TABLE IF NOT EXISTS agent_farm_eval_suites (
    id TEXT PRIMARY KEY,
    farm_id TEXT NOT NULL REFERENCES agent_farms(id) ON DELETE CASCADE,
    agent_id TEXT NOT NULL,
    source_path TEXT NOT NULL,
    case_count INTEGER NOT NULL DEFAULT 0,
    synced_commit_sha TEXT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (farm_id, agent_id)
);

CREATE TABLE IF NOT EXISTS agent_farm_eval_cases (
    suite_id TEXT NOT NULL REFERENCES agent_farm_eval_suites(id) ON DELETE CASCADE,
    id TEXT NOT NULL,
    prompt TEXT,
    fixture TEXT,
    timeout_secs INTEGER,
    assertions JSONB NOT NULL DEFAULT '[]',
    judge_rubric TEXT,
    judge_threshold DOUBLE PRECISION,
    PRIMARY KEY (suite_id, id)
);

CREATE TABLE IF NOT EXISTS agent_farm_eval_runs (
    id TEXT PRIMARY KEY,
    suite_id TEXT NOT NULL REFERENCES agent_farm_eval_suites(id) ON DELETE CASCADE,
    farm_id TEXT NOT NULL REFERENCES agent_farms(id) ON DELETE CASCADE,
    agent_id TEXT NOT NULL,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'queued'
        CHECK (status IN ('queued', 'running', 'completed', 'failed', 'cancelled')),
    started_at TIMESTAMPTZ,
    finished_at TIMESTAMPTZ,
    passed INTEGER NOT NULL DEFAULT 0,
    failed INTEGER NOT NULL DEFAULT 0,
    errored INTEGER NOT NULL DEFAULT 0,
    mean_score DOUBLE PRECISION,
    cost_microdollars BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    triggered_via TEXT NOT NULL DEFAULT 'console'
        CHECK (triggered_via IN ('dashboard', 'api', 'eval', 'mcp', 'console', 'demo'))
);

CREATE INDEX IF NOT EXISTS idx_agent_farm_eval_runs_suite ON agent_farm_eval_runs(suite_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_agent_farm_eval_runs_agent ON agent_farm_eval_runs(farm_id, agent_id, created_at DESC);

CREATE TABLE IF NOT EXISTS agent_farm_eval_results (
    id TEXT PRIMARY KEY,
    eval_run_id TEXT NOT NULL REFERENCES agent_farm_eval_runs(id) ON DELETE CASCADE,
    case_id TEXT NOT NULL,
    run_id TEXT REFERENCES agent_farm_runs(id) ON DELETE SET NULL,
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'running', 'passed', 'failed', 'errored')),
    assertions JSONB NOT NULL DEFAULT '[]',
    judge_score DOUBLE PRECISION,
    judge_rationale TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (eval_run_id, case_id)
);

-- One row per user who has launched a run: the "By person" leaderboard.
CREATE OR REPLACE VIEW agent_farm_runs_by_user AS
SELECT r.user_id,
       COALESCE(NULLIF(u.display_name, ''), u.email, r.user_id) AS display_name,
       COUNT(*)::BIGINT AS runs,
       COUNT(*) FILTER (WHERE r.status = 'succeeded')::BIGINT AS succeeded,
       COUNT(*) FILTER (WHERE r.status IN ('failed', 'timed_out'))::BIGINT AS failed,
       COUNT(*) FILTER (WHERE r.status IN ('queued', 'preparing', 'running'))::BIGINT AS active,
       COUNT(*) FILTER (WHERE r.triggered_via IN ('dashboard', 'console', 'demo'))::BIGINT AS dashboard_runs,
       COUNT(*) FILTER (WHERE r.triggered_via IN ('api', 'mcp'))::BIGINT AS api_runs,
       COUNT(*) FILTER (WHERE r.triggered_via = 'eval')::BIGINT AS eval_runs,
       COALESCE(SUM(r.cost_microdollars), 0)::BIGINT AS cost_microdollars,
       MODE() WITHIN GROUP (ORDER BY r.agent_id) AS favourite_agent,
       MAX(r.created_at) AS last_run_at
FROM agent_farm_runs r
LEFT JOIN users u ON u.id = r.user_id
GROUP BY r.user_id, u.display_name, u.email;
