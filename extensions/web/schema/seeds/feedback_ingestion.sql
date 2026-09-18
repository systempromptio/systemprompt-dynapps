INSERT INTO feedback_ingestion_backfill(source) VALUES('plugin_usage_events'),('ai_requests') ON CONFLICT DO NOTHING;
INSERT INTO analytics_ingestion_producers(producer,pending_count,oldest_pending_at)
VALUES('astound.feedback',2,clock_timestamp()) ON CONFLICT DO NOTHING;
