-- The gateway routing table as the console holds it: a plane whose runtime
-- input is still a file core reads at boot.
--
-- `gateway_routes` is the declared `gateway.routes:` sequence of
-- services/ai/gateway.yaml, one row per policy route in file order. Core
-- loads the routes from that file and never reads this table; the extension
-- regenerates the file's `routes:` sequence from these rows after every
-- console write and every sync apply. `route` is the route as core
-- serialises a `GatewayRoute` (id, label, models, when, requires,
-- deployments, strategy, context_fallbacks, alias), so a block the console
-- does not edit round-trips unchanged. `source` says who wrote the row:
-- `code` for the seed and every sync apply, `dashboard` for the
-- /admin/gateway editor.
CREATE TABLE IF NOT EXISTS gateway_routes (
    id TEXT PRIMARY KEY,
    position INTEGER NOT NULL,
    route JSONB NOT NULL,
    source TEXT NOT NULL DEFAULT 'code' CHECK (source IN ('code', 'dashboard')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_gateway_routes_position ON gateway_routes(position);
