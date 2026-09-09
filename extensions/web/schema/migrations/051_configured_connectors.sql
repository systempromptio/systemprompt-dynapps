ALTER TABLE mcp_connector_accounts DROP CONSTRAINT IF EXISTS mcp_connector_accounts_provider_check;
ALTER TABLE mcp_connector_credentials DROP CONSTRAINT IF EXISTS mcp_connector_credentials_provider_check;
ALTER TABLE mcp_connector_oauth_states DROP CONSTRAINT IF EXISTS mcp_connector_oauth_states_provider_check;
ALTER TABLE mcp_connector_accounts ADD CONSTRAINT mcp_connector_accounts_provider_check
    CHECK (provider ~ '^[A-Za-z0-9_-]{1,128}$');
ALTER TABLE mcp_connector_credentials ADD CONSTRAINT mcp_connector_credentials_provider_check
    CHECK (provider ~ '^[A-Za-z0-9_-]{1,128}$');
ALTER TABLE mcp_connector_oauth_states ADD CONSTRAINT mcp_connector_oauth_states_provider_check
    CHECK (provider ~ '^[A-Za-z0-9_-]{1,128}$');
