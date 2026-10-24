-- `gateway_routes` takes the 0.73 route shape: one `route` JSONB column
-- holding core's `GatewayRoute` serialisation, replacing the per-field
-- columns of the retired `model_pattern -> provider` route.
--
-- The table is a staging copy of `gateway.routes:` in
-- services/ai/gateway.yaml, which the console regenerates from these rows
-- and core loads; `admin config migrate` already rewrote that file, and the
-- old rows (model_pattern, provider, fallbacks, by_scope) have no meaning in
-- the new shape. The table is dropped and recreated empty; the governance
-- bootstrap seeds it from the migrated file on the next start, exactly as on
-- a fresh install.
--
-- Why: lock footprint. One row per declared gateway route (a handful, never
-- more than hundreds); ai_requests is not touched. DROP and CREATE hold
-- ACCESS EXCLUSIVE on this table only, for milliseconds.
DROP TABLE IF EXISTS gateway_routes;

CREATE TABLE gateway_routes (
    id TEXT PRIMARY KEY,
    position INTEGER NOT NULL,
    route JSONB NOT NULL,
    source TEXT NOT NULL DEFAULT 'code' CHECK (source IN ('code', 'dashboard')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_gateway_routes_position ON gateway_routes(position);
