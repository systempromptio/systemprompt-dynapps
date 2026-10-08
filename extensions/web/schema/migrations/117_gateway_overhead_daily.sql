-- Nightly gateway overhead percentiles (NFR-1.6). New empty table; no rows
-- rewritten. Same DDL as schema/50_gateway_overhead.sql.
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
