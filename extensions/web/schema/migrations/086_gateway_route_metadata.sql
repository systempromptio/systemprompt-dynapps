-- Human-facing metadata on the mirrored gateway routes. The console shows a
-- route by its name in place of the generated id, and the file renderer
-- writes both back into services/ai/gateway.yaml, so without these columns
-- the first console save would strip a name the operator declared there.
ALTER TABLE gateway_routes ADD COLUMN IF NOT EXISTS name TEXT;
ALTER TABLE gateway_routes ADD COLUMN IF NOT EXISTS description TEXT;
