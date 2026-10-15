-- Retire the `salesforce_client_credentials` and `atlassian_api_token`
-- machine credential kinds: the Salesforce and Atlassian servers are gone, so
-- no broker call can spend them. Their rows are deleted and the CHECK is
-- narrowed in place. Same DDL as schema/53_connector_machine_credentials.sql.

DELETE FROM mcp_connector_machine_credentials
    WHERE kind IN ('salesforce_client_credentials', 'atlassian_api_token');
ALTER TABLE mcp_connector_machine_credentials
    DROP CONSTRAINT IF EXISTS mcp_connector_machine_credentials_kind_check;
ALTER TABLE mcp_connector_machine_credentials
    ADD CONSTRAINT mcp_connector_machine_credentials_kind_check CHECK (kind IN (
        'github_app', 'odoo_api_key'));
