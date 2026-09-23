-- The audit-event NOTIFY triggers are dropped.
--
-- Every insert into governance_decisions, ai_requests and plugin_usage_events
-- built a JSON payload and called pg_notify('audit_events', …) for an SSE
-- bus that nothing subscribed to: the listener (`audit_event_bus`) had no
-- caller. A NOTIFY also serialises every committing writer on the one
-- notification queue lock.
--
-- Every statement is a DROP ... IF EXISTS, so this is a retirement
-- migration: executed as well as stamped on a fresh extension.
DROP TRIGGER IF EXISTS audit_event_notify_governance_trg ON governance_decisions;
DROP TRIGGER IF EXISTS audit_event_notify_ai_requests_stmt ON ai_requests;
DROP TRIGGER IF EXISTS audit_event_notify_ai_requests_trg ON ai_requests;
DROP TRIGGER IF EXISTS audit_event_notify_plugin_usage_trg ON plugin_usage_events;
DROP FUNCTION IF EXISTS audit_event_notify_governance();
DROP FUNCTION IF EXISTS audit_event_notify_ai_requests();
DROP FUNCTION IF EXISTS audit_event_notify_ai_request(TEXT, TEXT, TEXT, TEXT, TEXT, TEXT, TEXT, TIMESTAMPTZ);
DROP FUNCTION IF EXISTS audit_event_notify_plugin_usage();
