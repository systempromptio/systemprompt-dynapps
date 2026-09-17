-- Twin of schema/38_session_evaluation.sql: console-triggered evaluation.
--
-- A session can now be queued by a click (`closed_reason = 'manual'`), and a
-- run records whether the scheduler or an admin asked for it.
ALTER TABLE evaluation_queue DROP CONSTRAINT IF EXISTS evaluation_queue_closed_reason_check;
ALTER TABLE evaluation_queue ADD CONSTRAINT evaluation_queue_closed_reason_check
    CHECK (closed_reason IN ('session_end','quiet','manual'));

ALTER TABLE evaluation_runs ADD COLUMN IF NOT EXISTS trigger TEXT NOT NULL DEFAULT 'automatic'
    CHECK (trigger IN ('automatic','manual'));
ALTER TABLE evaluation_runs ADD COLUMN IF NOT EXISTS requested_by TEXT;
