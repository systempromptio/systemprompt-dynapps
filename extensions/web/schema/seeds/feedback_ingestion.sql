INSERT INTO feedback_ingestion_backfill(source) VALUES('plugin_usage_events'),('ai_requests'),('session_analyses') ON CONFLICT DO NOTHING;
INSERT INTO analytics_ingestion_producers(producer,pending_count,oldest_pending_at)
VALUES('astound.feedback',3,clock_timestamp()) ON CONFLICT DO NOTHING;
