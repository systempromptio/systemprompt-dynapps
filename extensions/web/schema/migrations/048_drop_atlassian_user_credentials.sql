-- The v1 Atlassian 3LO link (its own OAuth app, a per-user refresh token and
-- the disabled jira/confluence servers gated on it) is retired: every Atlassian
-- read now goes through the hosted connector (mcp_connector_credentials).
-- Drop the table and the rules the retired loader projected. Idempotent.
DROP TABLE IF EXISTS atlassian_user_credentials;

DELETE FROM access_control_rules WHERE rule_type = 'atlassian';

DELETE FROM access_control_entities
WHERE entity_type = 'mcp_server' AND entity_id IN ('jira', 'confluence');
