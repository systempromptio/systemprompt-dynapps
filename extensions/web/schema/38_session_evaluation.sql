-- Session evaluation: the LLM-as-judge engine. A closed session is queued
-- once per criterion; the session_evaluation job leases queue rows and writes
-- one run per attempt. Runs keep the full prompt and response so the
-- dashboard can show exactly what the judge saw. Twin: migrations/074.

CREATE TABLE IF NOT EXISTS evaluation_queue (
    session_id TEXT NOT NULL,
    criterion TEXT NOT NULL CHECK (criterion IN ('intent','quality','speed','cost')),
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    host TEXT NOT NULL DEFAULT 'unknown',
    closed_at TIMESTAMPTZ NOT NULL,
    closed_reason TEXT NOT NULL CHECK (closed_reason IN ('session_end','quiet')),
    pending BOOLEAN NOT NULL DEFAULT true,
    attempts INT NOT NULL DEFAULT 0,
    next_attempt TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    lease_token TEXT,
    lease_until TIMESTAMPTZ,
    last_error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (session_id, criterion)
);
CREATE INDEX IF NOT EXISTS idx_evaluation_queue_pending ON evaluation_queue(next_attempt) WHERE pending;

CREATE TABLE IF NOT EXISTS evaluation_runs (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    session_id TEXT NOT NULL,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    host TEXT NOT NULL,
    criterion TEXT NOT NULL CHECK (criterion IN ('intent','quality','speed','cost')),
    attempt INT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('succeeded','failed')),
    score_source TEXT NOT NULL CHECK (score_source IN ('judge','metric')),
    score SMALLINT CHECK (score BETWEEN 0 AND 100),
    rationale TEXT,
    evidence JSONB NOT NULL DEFAULT '[]',
    metrics JSONB NOT NULL DEFAULT '{}',
    provider TEXT NOT NULL,
    model TEXT NOT NULL,
    ai_request_id TEXT,
    input_tokens BIGINT,
    output_tokens BIGINT,
    cost_microdollars BIGINT,
    latency_ms INT,
    system_prompt TEXT NOT NULL,
    user_prompt TEXT NOT NULL,
    response TEXT,
    error TEXT,
    config_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp()
);
CREATE INDEX IF NOT EXISTS idx_evaluation_runs_session ON evaluation_runs(session_id, criterion, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_evaluation_runs_created ON evaluation_runs(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_evaluation_runs_user ON evaluation_runs(user_id, created_at DESC);

DROP TRIGGER IF EXISTS ingestion_owner ON evaluation_runs;
CREATE TRIGGER ingestion_owner BEFORE INSERT OR UPDATE ON evaluation_runs
FOR EACH ROW EXECUTE FUNCTION enforce_ingestion_owner();
DROP TRIGGER IF EXISTS feedback_capture ON evaluation_runs;
CREATE TRIGGER feedback_capture AFTER INSERT OR UPDATE OR DELETE ON evaluation_runs FOR EACH ROW EXECUTE FUNCTION capture_feedback_source();
