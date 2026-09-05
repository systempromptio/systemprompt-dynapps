-- The knowledge_worker role: the Cowork entitlement, granted to
-- Systemprompt-Business by the directory and by hand from the role editor.
-- The manual-roles CHECK is dropped and re-added under its generated name so
-- the migration is a no-op on a database that already carries it.
ALTER TABLE user_manual_roles DROP CONSTRAINT IF EXISTS user_manual_roles_role_check;
ALTER TABLE user_manual_roles ADD CONSTRAINT user_manual_roles_role_check
    CHECK (role IN ('platform_admin','admin','developer','user','project_manager','knowledge_worker'));

-- The commerce-devs group is retired: Systemprompt-Commerce now places its
-- members in europe-devs (services/web/config/groups.yaml). Groups are only
-- ever upserted by the loader, so the row has to go here. Members and AD
-- mappings cascade from the group; the ACL rows for the group and for the
-- marketplace and plugin it reached do not, because access_control_* is
-- keyed by string and knows nothing about groups.
DELETE FROM access_control_rules WHERE rule_type = 'group' AND rule_value = 'commerce-devs';
DELETE FROM access_control_rules
    WHERE (entity_type = 'marketplace' OR entity_type = 'plugin') AND entity_id = 'astound-commerce-dev';
DELETE FROM access_control_entities
    WHERE (entity_type = 'marketplace' OR entity_type = 'plugin') AND entity_id = 'astound-commerce-dev';
DELETE FROM groups WHERE id = 'commerce-devs';
