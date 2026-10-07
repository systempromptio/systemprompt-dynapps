-- Gateway routes follow core's failover model.
--
-- Core replaced a route's single `fallback_provider` / `fallback_upstream_model`
-- with an ordered `fallbacks: [{provider, upstream_model}]` chain and added
-- `by_scope` per-scope chains; a gateway.yaml still naming the old keys is
-- refused at load. The mirrored rows carry the new shape so the console never
-- regenerates a file core refuses: a declared fallback becomes a one-entry
-- chain. gateway_routes holds one row per declared route.

ALTER TABLE gateway_routes ADD COLUMN IF NOT EXISTS fallbacks JSONB NOT NULL DEFAULT '[]'::jsonb;
ALTER TABLE gateway_routes ADD COLUMN IF NOT EXISTS by_scope JSONB;

DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
         WHERE table_schema = current_schema()
           AND table_name = 'gateway_routes'
           AND column_name = 'fallback_provider'
    ) THEN
        UPDATE gateway_routes
           SET fallbacks = jsonb_build_array(jsonb_strip_nulls(jsonb_build_object(
                   'provider', fallback_provider,
                   'upstream_model', fallback_upstream_model)))
         WHERE fallback_provider IS NOT NULL;
    END IF;
END
$$;

ALTER TABLE gateway_routes DROP COLUMN IF EXISTS fallback_provider;
ALTER TABLE gateway_routes DROP COLUMN IF EXISTS fallback_upstream_model;
