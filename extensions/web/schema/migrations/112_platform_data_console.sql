-- @cost: rows=2000000 measured=4.157s triggers=live
-- PostgreSQL 18 isolated benchmark: 1,000,000 synthetic log rows; fixture
-- tests/fixtures/schema/platform-data-console-cost.sql. Two indexes write
-- 2,000,000 keys. Timestamp/ID index: 4156.583 ms; level/timestamp/ID:
-- 2198.939 ms. Existing database triggers remain enabled.
-- Platform Logs / SQL console indexes and confirmation receipts.
-- Durable deletion receipts and short-lived, actor-bound preview tickets.
CREATE TABLE IF NOT EXISTS platform_data_clear_previews (
    id UUID PRIMARY KEY,
    actor_id TEXT NOT NULL,
    selection JSONB NOT NULL,
    plan JSONB NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL DEFAULT (now() + interval '10 minutes')
);
CREATE INDEX IF NOT EXISTS idx_platform_clear_previews_expiry ON platform_data_clear_previews(expires_at);
CREATE TABLE IF NOT EXISTS platform_data_clear_audit (
    id UUID PRIMARY KEY,
    actor_id TEXT NOT NULL,
    selected_table TEXT NOT NULL,
    selection JSONB NOT NULL,
    effects JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_platform_clear_audit_time ON platform_data_clear_audit(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_logs_timestamp_id ON logs(timestamp DESC, id DESC);
CREATE INDEX IF NOT EXISTS idx_logs_level_timestamp_id ON logs(level, timestamp DESC, id DESC);
