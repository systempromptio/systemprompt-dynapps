-- Measured gateway overhead (NFR-1.6): one row per UTC day per inbound wire
-- dialect, plus an `all` row, written nightly by the `gateway_overhead_rollup`
-- job from completed, non-synthetic `ai_requests`. Overhead is
-- `latency_ms - upstream_latency_ms` clamped at zero: the time the gateway
-- itself added around the provider call. Percentiles are milliseconds,
-- interpolated (`percentile_cont`). The job keeps the last 400 days.
CREATE TABLE IF NOT EXISTS gateway_overhead_daily (
    date DATE NOT NULL,
    wire_protocol TEXT NOT NULL,
    request_count BIGINT NOT NULL,
    overhead_p50_ms DOUBLE PRECISION NOT NULL,
    overhead_p95_ms DOUBLE PRECISION NOT NULL,
    overhead_p99_ms DOUBLE PRECISION NOT NULL,
    upstream_p50_ms DOUBLE PRECISION NOT NULL,
    upstream_p95_ms DOUBLE PRECISION NOT NULL,
    upstream_p99_ms DOUBLE PRECISION NOT NULL,
    computed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (date, wire_protocol)
);
