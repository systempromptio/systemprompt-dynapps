-- Core gateway attribution feeds ai_request_scopes.
--
-- `source` gains `api_key`: a value the calling API key is bound to is a
-- per-key decision, neither the person's primary nor a per-request header,
-- and folding it into either would misreport who chose the scope. Existing
-- rows are all `primary` or `header`, so the widened check validates without
-- a rewrite. The trigger on ai_request_attributions that writes these values
-- is declarative (schema/40_request_scopes.sql); no backfill: requests made
-- before core attributed them keep the default-derived scope they were pinned
-- to when they happened.

ALTER TABLE ai_request_scopes DROP CONSTRAINT IF EXISTS ai_request_scopes_source_check;
ALTER TABLE ai_request_scopes ADD CONSTRAINT ai_request_scopes_source_check
    CHECK (source IN ('primary', 'header', 'api_key'));
