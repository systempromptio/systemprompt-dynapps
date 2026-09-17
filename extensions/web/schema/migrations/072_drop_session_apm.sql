-- Session APM (actions per minute), its error-adjusted twin and the
-- concurrent-session peak were computed on every `Stop` hook and written to
-- the session summary, but nothing ever read them back: no page, query or
-- export surfaced the three columns. Drop the storage with the code.
ALTER TABLE plugin_session_summaries
    DROP COLUMN IF EXISTS apm,
    DROP COLUMN IF EXISTS eapm,
    DROP COLUMN IF EXISTS peak_concurrent;
