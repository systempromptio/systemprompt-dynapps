-- Twin of schema/43_evaluation_attribution.sql: score attribution views
-- over evaluation_runs and the evaluation_reports table. Views are
-- CREATE OR REPLACE and the table is IF NOT EXISTS, so re-applying is safe.

CREATE OR REPLACE VIEW evaluation_session_scores AS
SELECT DISTINCT ON (r.session_id, r.criterion)
       r.session_id, r.user_id, r.host, r.criterion, r.id AS run_id, r.status,
       r.score_source, r.score, r.rationale, r.model, r.error,
       r.created_at AS scored_at
FROM evaluation_runs r
ORDER BY r.session_id, r.criterion, (r.status = 'succeeded') DESC, r.created_at DESC;

CREATE OR REPLACE VIEW evaluation_session_scorecard AS
SELECT session_id, user_id,
       MAX(score) FILTER (WHERE criterion = 'intent'  AND status = 'succeeded')::smallint AS intent,
       MAX(score) FILTER (WHERE criterion = 'quality' AND status = 'succeeded')::smallint AS quality,
       MAX(score) FILTER (WHERE criterion = 'speed'   AND status = 'succeeded')::smallint AS speed,
       MAX(score) FILTER (WHERE criterion = 'cost'    AND status = 'succeeded')::smallint AS cost,
       COUNT(*) FILTER (WHERE status = 'succeeded')::int AS scored_criteria,
       COUNT(*) FILTER (WHERE status = 'failed')::int AS failed_criteria,
       MAX(scored_at) AS last_scored_at
FROM evaluation_session_scores
GROUP BY session_id, user_id;

CREATE OR REPLACE VIEW evaluation_session_skills AS
SELECT e.session_id, e.user_id, e.plugin_id, e.skill, o.marketplace_id,
       marketplace_version_at(o.marketplace_id, MIN(e.invoked_at)) AS marketplace_hash,
       COUNT(*)::bigint AS invocations, MIN(e.invoked_at) AS first_invoked_at
FROM analysis_skill_events e
LEFT JOIN service_owned_ids o ON o.kind = 'plugin' AND o.id = e.plugin_id
WHERE e.skill IS NOT NULL AND e.session_id IS NOT NULL
GROUP BY e.session_id, e.user_id, e.plugin_id, e.skill, o.marketplace_id;

CREATE OR REPLACE VIEW evaluation_version_scores AS
SELECT s.marketplace_id, s.marketplace_hash,
       COUNT(DISTINCT s.session_id)::bigint AS sessions,
       COUNT(DISTINCT s.session_id) FILTER (WHERE c.scored_criteria > 0)::bigint AS scored_sessions,
       AVG(c.intent)::float8 AS intent, AVG(c.quality)::float8 AS quality,
       AVG(c.speed)::float8 AS speed, AVG(c.cost)::float8 AS cost,
       MIN(c.last_scored_at) AS first_scored_at, MAX(c.last_scored_at) AS last_scored_at
FROM (SELECT DISTINCT marketplace_id, marketplace_hash, session_id
        FROM evaluation_session_skills WHERE marketplace_id IS NOT NULL) s
LEFT JOIN evaluation_session_scorecard c ON c.session_id = s.session_id
GROUP BY s.marketplace_id, s.marketplace_hash;

CREATE TABLE IF NOT EXISTS evaluation_reports (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    scope_kind TEXT NOT NULL CHECK (scope_kind IN ('global', 'marketplace')),
    scope_id TEXT NOT NULL DEFAULT '',
    window_start TIMESTAMPTZ NOT NULL,
    window_end TIMESTAMPTZ NOT NULL,
    marketplace_hash TEXT,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'generated', 'failed')),
    trigger TEXT NOT NULL DEFAULT 'automatic' CHECK (trigger IN ('automatic', 'manual')),
    requested_by TEXT,
    source_run_count BIGINT NOT NULL DEFAULT 0,
    source_last_run_at TIMESTAMPTZ,
    provider TEXT,
    model TEXT,
    ai_request_id TEXT,
    input_tokens BIGINT,
    output_tokens BIGINT,
    cost_microdollars BIGINT,
    latency_ms INTEGER,
    inputs JSONB NOT NULL DEFAULT '{}'::jsonb,
    findings JSONB,
    attempts INTEGER NOT NULL DEFAULT 0,
    next_attempt TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    lease_token TEXT,
    lease_until TIMESTAMPTZ,
    last_error TEXT,
    generated_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp()
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_evaluation_reports_one_pending
    ON evaluation_reports(scope_kind, scope_id) WHERE status = 'pending';
CREATE INDEX IF NOT EXISTS idx_evaluation_reports_latest
    ON evaluation_reports(scope_kind, scope_id, generated_at DESC) WHERE status = 'generated';
CREATE INDEX IF NOT EXISTS idx_evaluation_reports_pending
    ON evaluation_reports(next_attempt) WHERE status = 'pending';
