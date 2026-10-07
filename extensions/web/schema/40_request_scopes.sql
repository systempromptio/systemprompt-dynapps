-- Cost attribution stamped at request time.
--
-- Spend used to be attributed by joining `ai_requests` to each person's
-- *current* primary group and project, so moving someone silently rewrote
-- their history. This table pins the answer per request: an `AFTER INSERT`
-- statement trigger on core's `ai_requests` copies the primary group and project from
-- `user_scope_defaults` the moment the row lands, and nothing later touches it.
-- `source` says who decided — `primary` is the person's default; `header` is
-- the caller's `x-systemprompt-scope-<dimension>` override; `api_key` is the
-- value the calling API key is bound to.
--
-- Gateway requests carry core's own attribution: `ai_request_attributions`
-- rows (one per registered dimension) written in the same transaction as the
-- request row, after it. A second statement trigger on that table overwrites
-- the default-derived project and group with core's values, so the gateway's
-- decision wins and `user_scope_defaults` is only the fallback for a request
-- core did not attribute (or a dimension it left out). Core's `default`
-- source is the provider's first value, which is the person's primary, so it
-- maps to `primary`. Both triggers fire inside the inserting transaction, so
-- the row is still pinned at request time.
--
-- Extension-only: `ai_requests` is core's and has no scope column, and the
-- insert is a plain statement on the write pool, so the trigger resolves the
-- scope itself from each new row's `user_id`. The body is wrapped so that a
-- fault here warns and returns; it never rolls back the core insert.
-- Established databases are backfilled by migration 078.

CREATE TABLE IF NOT EXISTS ai_request_scopes (
    request_id TEXT PRIMARY KEY REFERENCES ai_requests(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL,
    group_id TEXT REFERENCES groups(id) ON DELETE SET NULL,
    project_id TEXT REFERENCES projects(id) ON DELETE SET NULL,
    source TEXT NOT NULL DEFAULT 'primary' CHECK (source IN ('primary', 'header', 'api_key')),
    resolved_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_ai_request_scopes_group
    ON ai_request_scopes(group_id);
CREATE INDEX IF NOT EXISTS idx_ai_request_scopes_project
    ON ai_request_scopes(project_id);
CREATE INDEX IF NOT EXISTS idx_ai_request_scopes_user
    ON ai_request_scopes(user_id);

-- A row is written for every request, including one no primary covers: a
-- NULL group and project is the durable record that the request was
-- unattributed when it happened, which is what lets the remainder bucket be
-- read from this table rather than re-derived. Only a user actor is resolved;
-- a job or MCP actor carries its owner's id and is never a person's spend.
CREATE OR REPLACE FUNCTION request_scope_stamp_ai_requests()
RETURNS TRIGGER AS $$
BEGIN
    BEGIN
        -- TG_LEVEL: the installer swaps function bodies before migrations run,
        -- so a database mid-upgrade still fires the row-form trigger into this
        -- body until the declarative phase replaces it below.
        IF TG_LEVEL = 'ROW' THEN
            INSERT INTO ai_request_scopes (request_id, user_id, group_id, project_id, source)
            SELECT NEW.id, NEW.user_id, d.primary_group_id, d.primary_project_id, 'primary'
              FROM (SELECT 1) one
              LEFT JOIN user_scope_defaults d
                ON NEW.actor_kind = 'user' AND d.user_id = NEW.user_id
            ON CONFLICT (request_id) DO NOTHING;
            RETURN NEW;
        END IF;
        INSERT INTO ai_request_scopes (request_id, user_id, group_id, project_id, source)
        SELECT r.id, r.user_id, d.primary_group_id, d.primary_project_id, 'primary'
          FROM new_rows r
          LEFT JOIN user_scope_defaults d
            ON r.actor_kind = 'user' AND d.user_id = r.user_id
        ON CONFLICT (request_id) DO NOTHING;
    EXCEPTION WHEN OTHERS THEN
        RAISE WARNING 'request_scope_stamp_ai_requests failed: %', SQLERRM;
    END;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Statement-level: one INSERT ... SELECT over the rows the statement added,
-- not one lookup and insert per request. The row form is dropped first
-- because a statement trigger with a transition table is a new definition,
-- not a replacement.
DROP TRIGGER IF EXISTS request_scope_stamp_ai_requests_trg ON ai_requests;
CREATE OR REPLACE TRIGGER request_scope_stamp_ai_requests_stmt
    AFTER INSERT ON ai_requests
    REFERENCING NEW TABLE AS new_rows
    FOR EACH STATEMENT
    EXECUTE FUNCTION request_scope_stamp_ai_requests();

-- A value core names that is not a live project or group (deleted between the
-- request and this statement) leaves that column at its default-derived value.
CREATE OR REPLACE FUNCTION request_scope_stamp_attributions()
RETURNS TRIGGER AS $$
BEGIN
    BEGIN
        WITH pivot AS (
            SELECT a.request_id,
                   max(a.value) FILTER (WHERE a.dimension = 'project') AS project_value,
                   max(a.source) FILTER (WHERE a.dimension = 'project') AS project_source,
                   max(a.value) FILTER (WHERE a.dimension = 'group') AS group_value,
                   max(a.source) FILTER (WHERE a.dimension = 'group') AS group_source
              FROM new_rows a
             WHERE a.dimension IN ('project', 'group')
             GROUP BY a.request_id
        ), resolved AS (
            SELECT pv.request_id, q.user_id, g.id AS group_id, p.id AS project_id,
                   CASE COALESCE(
                            CASE WHEN p.id IS NOT NULL THEN pv.project_source END,
                            CASE WHEN g.id IS NOT NULL THEN pv.group_source END)
                        WHEN 'header' THEN 'header'
                        WHEN 'api_key' THEN 'api_key'
                        ELSE 'primary'
                   END AS source
              FROM pivot pv
              JOIN ai_requests q ON q.id = pv.request_id AND q.actor_kind = 'user'
              LEFT JOIN projects p ON p.id = pv.project_value
              LEFT JOIN groups g ON g.id = pv.group_value
             WHERE p.id IS NOT NULL OR g.id IS NOT NULL
        )
        INSERT INTO ai_request_scopes (request_id, user_id, group_id, project_id, source)
        SELECT request_id, user_id, group_id, project_id, source FROM resolved
        ON CONFLICT (request_id) DO UPDATE
           SET group_id = COALESCE(EXCLUDED.group_id, ai_request_scopes.group_id),
               project_id = COALESCE(EXCLUDED.project_id, ai_request_scopes.project_id),
               source = EXCLUDED.source;
    EXCEPTION WHEN OTHERS THEN
        RAISE WARNING 'request_scope_stamp_attributions failed: %', SQLERRM;
    END;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE TRIGGER request_scope_stamp_attributions_stmt
    AFTER INSERT ON ai_request_attributions
    REFERENCING NEW TABLE AS new_rows
    FOR EACH STATEMENT
    EXECUTE FUNCTION request_scope_stamp_attributions();
