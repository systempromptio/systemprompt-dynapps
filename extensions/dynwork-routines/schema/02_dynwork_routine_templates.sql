-- Scheduled task templates: the library an admin publishes for DynWork.
--
-- One row per template. `body` is the template as the desktop receives it
-- (name, description, where, definition, locks, personal); `mandatory` and
-- `plugins`, the anchor plugins whose access rules decide who sees it, are
-- columns because queries filter on them. A draft has never been published
-- and is at revision 0; every publish adds one to `revision` and keeps a copy
-- in `dynwork_routine_template_revisions`, so a published template's row is
-- always its latest revision. `retire_mode` is set exactly when the template
-- is retired.

CREATE TABLE IF NOT EXISTS dynwork_routine_templates (
    id           TEXT PRIMARY KEY CHECK (id ~ '^[A-Za-z0-9_.:-]{1,128}$'),
    revision     BIGINT NOT NULL CHECK (revision >= 0),
    state        TEXT NOT NULL CHECK (state IN ('draft', 'published', 'closed', 'retired')),
    retire_mode  TEXT CHECK (retire_mode IN ('unlink', 'pause', 'remove')),
    mandatory    BOOLEAN NOT NULL,
    plugins      TEXT[] NOT NULL CHECK (cardinality(plugins) > 0),
    source       TEXT NOT NULL CHECK (source IN ('console', 'yaml')),
    body         JSONB NOT NULL,
    created_by   TEXT NOT NULL,
    updated_by   TEXT NOT NULL,
    updated_at   TIMESTAMPTZ NOT NULL,
    CHECK ((state = 'draft') = (revision = 0)),
    CHECK ((state = 'retired') = (retire_mode IS NOT NULL))
);

-- Every published revision as it was published, and who published it: the
-- base of a desktop's three-way merge and the admin's history.
CREATE TABLE IF NOT EXISTS dynwork_routine_template_revisions (
    template_id   TEXT NOT NULL REFERENCES dynwork_routine_templates(id) ON DELETE CASCADE,
    revision      BIGINT NOT NULL CHECK (revision > 0),
    mandatory     BOOLEAN NOT NULL,
    body          JSONB NOT NULL,
    published_by  TEXT NOT NULL,
    published_at  TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (template_id, revision)
);
