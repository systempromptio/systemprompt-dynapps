-- Synthetic PostgreSQL 18 log shape for measuring migration 112.
-- Run only in a disposable database with the application schema installed.
DROP INDEX IF EXISTS idx_logs_timestamp_id;
DROP INDEX IF EXISTS idx_logs_level_timestamp_id;
INSERT INTO logs(timestamp,level,module,message,metadata,trace_id,context_id)
SELECT now() - (n % 864000) * interval '1 second',
       (ARRAY['INFO','INFO','INFO','WARN','ERROR'])[1 + n % 5],
       'fixture.module.' || n % 24,
       repeat('Synthetic log event. ', 12),
       json_build_object('sequence',n,'elapsed_ms',n % 1000)::text,
       'fixture-trace-' || n % 100000,
       'fixture-context-' || n % 10000
FROM generate_series(1,1000000) AS n;
ANALYZE logs;
CREATE INDEX idx_logs_timestamp_id ON logs(timestamp DESC,id DESC);
CREATE INDEX idx_logs_level_timestamp_id ON logs(level,timestamp DESC,id DESC);
