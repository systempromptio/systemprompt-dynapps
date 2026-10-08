-- Gateway routes carry core's deployment selection and context-window keys.
--
-- Core added a route-level `strategy` (ordered | weighted | least_busy),
-- `weight` and `context_fallbacks`. The console regenerates gateway.yaml's
-- `routes:` from these rows, so a key with no column would be deleted by
-- the next console save. `routing` holds the three as declared (core
-- validates them at boot); NULL is a route that sets none of them. A
-- fallback's own `weight` rides in `fallbacks`, and a scope chain's in
-- `by_scope`. DDL only; no rows rewritten.

ALTER TABLE gateway_routes ADD COLUMN IF NOT EXISTS routing JSONB;
