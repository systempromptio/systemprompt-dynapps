CREATE TABLE IF NOT EXISTS ingestion_session_owners (
    session_id TEXT PRIMARY KEY CHECK (length(session_id) BETWEEN 1 AND 255),
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE OR REPLACE FUNCTION assert_ingestion_owner(session_key text, owner_key text)
RETURNS void LANGUAGE plpgsql AS $$
DECLARE bound_owner text;
BEGIN
    IF session_key IS NULL OR btrim(session_key) = '' OR length(session_key) > 255 THEN
        RAISE EXCEPTION 'Invalid ingestion session' USING ERRCODE = '23514';
    END IF;
    PERFORM pg_advisory_xact_lock(hashtextextended('ingestion:' || session_key, 0));
    IF NOT EXISTS (SELECT 1 FROM users WHERE id = owner_key) THEN
        RAISE EXCEPTION 'Unknown ingestion owner' USING ERRCODE = '23503';
    END IF;
    IF EXISTS (SELECT 1 FROM plugin_usage_events WHERE session_id = session_key AND user_id <> owner_key)
       OR EXISTS (SELECT 1 FROM plugin_session_summaries WHERE session_id = session_key AND user_id <> owner_key) THEN
        RAISE EXCEPTION 'Ingestion session ownership conflict' USING ERRCODE = '23514';
    END IF;
    INSERT INTO ingestion_session_owners(session_id, user_id) VALUES(session_key, owner_key)
    ON CONFLICT (session_id) DO NOTHING;
    SELECT user_id INTO bound_owner FROM ingestion_session_owners WHERE session_id = session_key;
    IF bound_owner <> owner_key THEN
        RAISE EXCEPTION 'Ingestion session ownership conflict' USING ERRCODE = '23514';
    END IF;
END $$;

CREATE OR REPLACE FUNCTION enforce_ingestion_owner()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF TG_OP = 'UPDATE' AND (NEW.user_id <> OLD.user_id OR NEW.session_id <> OLD.session_id) THEN
        RAISE EXCEPTION 'Ingestion ownership is immutable' USING ERRCODE = '23514';
    END IF;
    PERFORM assert_ingestion_owner(NEW.session_id, NEW.user_id);
    RETURN NEW;
END $$;

DROP TRIGGER IF EXISTS ingestion_owner ON plugin_usage_events;
CREATE TRIGGER ingestion_owner BEFORE INSERT OR UPDATE ON plugin_usage_events
FOR EACH ROW EXECUTE FUNCTION enforce_ingestion_owner();
DROP TRIGGER IF EXISTS ingestion_owner ON plugin_session_summaries;
CREATE TRIGGER ingestion_owner BEFORE INSERT OR UPDATE ON plugin_session_summaries
FOR EACH ROW EXECUTE FUNCTION enforce_ingestion_owner();

CREATE OR REPLACE FUNCTION analysis_native_session(metadata_user_id text)
RETURNS text LANGUAGE plpgsql IMMUTABLE AS $$
DECLARE session_key text;
BEGIN
    IF left(btrim(metadata_user_id), 1) = '{' THEN
        session_key := (metadata_user_id::jsonb)->>'session_id';
    ELSE
        session_key := substring(metadata_user_id from '_session_([^[:space:]]+)$');
    END IF;
    RETURN (session_key::uuid)::text;
EXCEPTION WHEN invalid_text_representation THEN RETURN NULL;
END $$;

-- Why: `plugin_id` is the plugin the skill key names. The hook rows carry the
-- plugin whose hooks reported them (one plugin owns the session-global hooks
-- for every user), so that column would credit every marketplace's skills to
-- the hook owner; the invoked skill is `<plugin>:<skill>`, and its prefix is
-- the id the marketplaces declare. `reporter_plugin_id` keeps the hook owner.
CREATE OR REPLACE VIEW analysis_skill_events AS
SELECT s.id, s.user_id, s.session_id,
       COALESCE(NULLIF(split_part(s.skill, ':', 1), ''), s.reporter_plugin_id) AS plugin_id,
       s.skill, s.tool_use_id, s.source, s.invoked_at, s.reporter_plugin_id
FROM (
    SELECT e.id, e.user_id, e.session_id, e.plugin_id AS reporter_plugin_id,
           replace(substring(e.prompt_preview from '^/([A-Za-z0-9._-]+:[A-Za-z0-9._-]+)'), '_', '-') AS skill,
           NULL::text AS tool_use_id, 'slash'::text AS source, e.created_at AS invoked_at
    FROM plugin_usage_events e
    WHERE e.event_type = 'UserPromptSubmit'
      AND e.prompt_preview ~ '^/[A-Za-z0-9._-]+:[A-Za-z0-9._-]+'
    UNION ALL
    SELECT e.id, e.user_id, e.session_id, e.plugin_id,
           replace(e.metadata->'tool_input'->>'skill', '_', '-') AS skill,
           e.tool_use_id, 'tool'::text AS source, e.created_at AS invoked_at
    FROM plugin_usage_events e
    WHERE e.event_type IN ('PostToolUse', 'PostToolUseFailure') AND e.tool_name = 'Skill'
      AND e.metadata->'tool_input'->>'skill' IS NOT NULL
    UNION ALL
    SELECT e.id, e.user_id, e.session_id, e.plugin_id,
           COALESCE(e.metadata->>'skill_ref', 'opencode:' || replace(e.metadata->'tool_input'->>'name', '_', '-')) AS skill,
           e.tool_use_id, 'tool'::text AS source, e.created_at AS invoked_at
    FROM plugin_usage_events e
    WHERE e.event_type IN ('PostToolUse', 'PostToolUseFailure') AND e.tool_name = 'skill'
      AND e.metadata->'tool_input'->>'name' IS NOT NULL
) s;

CREATE TABLE IF NOT EXISTS ingestion_event_receipts (
    dedup_key TEXT PRIMARY KEY,
    payload_digest TEXT NOT NULL,
    accepted_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE OR REPLACE FUNCTION accept_ingestion_delivery()
RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE old_digest text; digest text;
BEGIN
    digest := NEW.metadata->>'_ingestion_digest';
    IF digest IS NOT NULL THEN
        INSERT INTO ingestion_event_receipts(dedup_key, payload_digest) VALUES(NEW.dedup_key, digest)
        ON CONFLICT(dedup_key) DO NOTHING;
        SELECT payload_digest INTO old_digest FROM ingestion_event_receipts WHERE dedup_key = NEW.dedup_key;
        IF old_digest <> digest THEN
            RAISE EXCEPTION 'Conflicting event delivery' USING ERRCODE = '23505';
        END IF;
    END IF;
    RETURN NEW;
END $$;
DROP TRIGGER IF EXISTS ingestion_delivery ON plugin_usage_events;
CREATE TRIGGER ingestion_delivery BEFORE INSERT ON plugin_usage_events FOR EACH ROW EXECUTE FUNCTION accept_ingestion_delivery();
