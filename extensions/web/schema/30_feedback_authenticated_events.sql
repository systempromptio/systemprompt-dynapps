-- The device a hook event was authenticated from.
--
-- `/hooks/track` writes one row per event that arrived with an enrolled
-- device certificate (`repositories/marketplace/webhook.rs`); the row is
-- immutable. 31_skill_version_receipts.sql reads it to tie a skill
-- invocation to the installation receipt that device holds.
CREATE TABLE IF NOT EXISTS feedback_authenticated_events (
    event_id TEXT PRIMARY KEY REFERENCES plugin_usage_events(id) ON DELETE CASCADE,
    consumer_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    device_id TEXT NOT NULL REFERENCES user_device_certs(id) ON DELETE CASCADE,
    host TEXT NOT NULL,
    native_session_id TEXT NOT NULL,
    authenticated_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp()
);
DROP TRIGGER IF EXISTS feedback_authenticated_events_immutable ON feedback_authenticated_events;
CREATE TRIGGER feedback_authenticated_events_immutable BEFORE UPDATE ON feedback_authenticated_events FOR EACH ROW EXECUTE FUNCTION reject_managed_content_update();
