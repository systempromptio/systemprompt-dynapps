-- Remove every row the vet-group demo seed owns. Runs inside seed.sh's
-- transaction with systemprompt.audit_maintenance = on, so the audit guards
-- accept the deletes of rows inside their retention floors. Ownership is
-- recognised by the demo domain (users), the `vg-` prefix (groups, projects,
-- clients), the farm id and the `[vetgroup-demo]` justification tag.
CREATE TEMP TABLE vet_users ON COMMIT DROP AS
    SELECT id FROM users WHERE email LIKE '%@vetgroup-demo.example';

DELETE FROM conversation_context_aliases a
 USING conversation_session_contexts c
 WHERE c.context_id = a.context_id AND c.user_id IN (SELECT id FROM vet_users);
DELETE FROM conversation_facts WHERE user_id IN (SELECT id FROM vet_users);
DELETE FROM trace_facts WHERE trace_id IN (
    SELECT trace_id FROM ai_requests WHERE user_id IN (SELECT id FROM vet_users));
DELETE FROM admin_usage_daily_rollups WHERE user_id IN (SELECT id FROM vet_users);
DELETE FROM agent_farms WHERE id = 'vetgroup-ops';
DELETE FROM ai_requests WHERE user_id IN (SELECT id FROM vet_users);
DELETE FROM mcp_tool_executions WHERE user_id IN (SELECT id FROM vet_users);
DELETE FROM governance_decisions WHERE user_id IN (SELECT id FROM vet_users);
DELETE FROM plugin_usage_events WHERE user_id IN (SELECT id FROM vet_users);
DELETE FROM access_control_rules WHERE justification LIKE '[vetgroup-demo]%';
DELETE FROM clients WHERE id IN ('vg-group-finance', 'vg-clinics-be');
DELETE FROM groups WHERE id LIKE 'vg-%';
DELETE FROM projects WHERE id LIKE 'vg-%';
DELETE FROM users WHERE id IN (SELECT id FROM vet_users);
