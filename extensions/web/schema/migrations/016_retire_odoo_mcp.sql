-- Retire the custom Odoo MCP server. Odoo is reached through the hosted
-- `dynapps-odoo` server with each person's own OAuth grant, so nothing reads
-- a linked Odoo account or an `odoo_api_key` machine credential.
--
-- The linked-account table is dropped; its key rows go with it. The machine
-- credential kind and the `odoo_destructive` role are removed with their rows
-- and the CHECKs narrowed in place, and `odoo_writer` takes the role's place:
-- the grant that lets a caller's dynapps-odoo write tools pass governance.
-- `connector_credential_audit` stays; service accounts still write to it.
-- Same DDL as schema/23_groups_projects.sql,
-- schema/53_connector_machine_credentials.sql and
-- schema/54_connector_credential_audit.sql.

DROP TABLE IF EXISTS odoo_user_links CASCADE;

DELETE FROM mcp_connector_machine_credentials WHERE kind = 'odoo_api_key';
ALTER TABLE mcp_connector_machine_credentials
    DROP CONSTRAINT IF EXISTS mcp_connector_machine_credentials_kind_check;
ALTER TABLE mcp_connector_machine_credentials
    ADD CONSTRAINT mcp_connector_machine_credentials_kind_check CHECK (kind IN (
        'github_app'));

DELETE FROM user_manual_roles WHERE role = 'odoo_destructive';
ALTER TABLE user_manual_roles
    DROP CONSTRAINT IF EXISTS user_manual_roles_role_check;
ALTER TABLE user_manual_roles
    ADD CONSTRAINT user_manual_roles_role_check CHECK (role IN (
        'platform_admin', 'admin', 'developer', 'user', 'project_manager',
        'knowledge_worker', 'super_admin', 'odoo_writer'));
