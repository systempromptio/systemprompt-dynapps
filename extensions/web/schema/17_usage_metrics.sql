-- Usage metrics captured from Claude Code hook telemetry and the gateway,
-- feeding the admin analytics dashboard.
--
-- These are web-owned tables. They reference core-owned users only by TEXT id
-- (the plugin_usage_events precedent) — no FK onto core tables.

-- Per-user-per-day rollup joining the hook plane (sessions, prompts) and
-- the gateway plane (ai_requests tokens + cost), so the
-- dashboard reads one narrow table instead of re-aggregating raw events.
-- Recomputed idempotently by the usage_daily_rollup job for a trailing window.
CREATE TABLE IF NOT EXISTS admin_usage_daily_rollups (
    user_id TEXT NOT NULL,
    date DATE NOT NULL,
    sessions_count INT NOT NULL DEFAULT 0,
    prompts BIGINT NOT NULL DEFAULT 0,
    tool_uses BIGINT NOT NULL DEFAULT 0,
    errors BIGINT NOT NULL DEFAULT 0,
    ai_requests_count BIGINT NOT NULL DEFAULT 0,
    input_tokens BIGINT NOT NULL DEFAULT 0,
    output_tokens BIGINT NOT NULL DEFAULT 0,
    cost_microdollars BIGINT NOT NULL DEFAULT 0,
    -- The user's primary group and project when the row was rolled up, so a
    -- container's history keeps the attribution it had rather than the one
    -- today's membership would imply.
    group_id TEXT,
    project_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, date)
);
CREATE INDEX IF NOT EXISTS idx_admin_usage_rollups_date
    ON admin_usage_daily_rollups(date DESC);
CREATE INDEX IF NOT EXISTS idx_admin_usage_rollups_group
    ON admin_usage_daily_rollups(group_id, date DESC);
CREATE INDEX IF NOT EXISTS idx_admin_usage_rollups_project
    ON admin_usage_daily_rollups(project_id, date DESC);

-- Persisted usage anomalies, one row per metric per hourly window (see
-- migrations/032_usage_anomalies.sql for the rationale). Written by the
-- usage_anomaly job, read by the spend dashboard.
CREATE TABLE IF NOT EXISTS usage_anomalies (
    metric TEXT NOT NULL CHECK (metric IN ('requests', 'cost', 'errors')),
    window_start TIMESTAMPTZ NOT NULL,
    observed BIGINT NOT NULL,
    baseline BIGINT NOT NULL,
    detected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (metric, window_start)
);

CREATE INDEX IF NOT EXISTS idx_usage_anomalies_detected
    ON usage_anomalies(detected_at DESC);
