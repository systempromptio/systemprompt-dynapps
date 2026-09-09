-- Executive role grants must work through the same manual-role editor as other roles.
ALTER TABLE user_manual_roles DROP CONSTRAINT IF EXISTS user_manual_roles_role_check;
ALTER TABLE user_manual_roles ADD CONSTRAINT user_manual_roles_role_check
    CHECK (role IN ('platform_admin','admin','developer','user','project_manager','knowledge_worker','super_admin'));

-- Correct the rollout project identifier without deleting memberships or history.
-- Any conflicting unique rows abort the transaction rather than silently discard data.
INSERT INTO projects (id, name, description, source, created_at, updated_at)
SELECT 'business-implementation', name, description, source, created_at, CURRENT_TIMESTAMP
FROM projects WHERE id = 'business-imeplmentation'
ON CONFLICT (id) DO NOTHING;

UPDATE project_members SET project_id = 'business-implementation'
WHERE project_id = 'business-imeplmentation';
UPDATE project_ad_mappings SET project_id = 'business-implementation'
WHERE project_id = 'business-imeplmentation';
UPDATE user_scope_defaults SET primary_project_id = 'business-implementation'
WHERE primary_project_id = 'business-imeplmentation';
UPDATE admin_usage_daily_rollups SET project_id = 'business-implementation'
WHERE project_id = 'business-imeplmentation';
UPDATE access_control_rules SET rule_value = 'business-implementation'
WHERE rule_type = 'project' AND rule_value = 'business-imeplmentation';

-- Optional MCP extensions may not be installed on every instance.
DO $$
DECLARE
    relation_name TEXT;
BEGIN
    FOREACH relation_name IN ARRAY ARRAY[
        'requirements_workflows', 'knowledge_drive_files', 'knowledge_drive_document_projects'
    ] LOOP
        IF to_regclass(relation_name) IS NOT NULL THEN
            EXECUTE format('UPDATE %I SET project_id = $1 WHERE project_id = $2', relation_name)
                USING 'business-implementation', 'business-imeplmentation';
        END IF;
    END LOOP;
    IF to_regclass('knowledge_document_grants') IS NOT NULL THEN
        UPDATE knowledge_document_grants SET subject_value = 'business-implementation'
        WHERE subject_kind = 'project' AND subject_value = 'business-imeplmentation';
    END IF;
END $$;

DELETE FROM projects WHERE id = 'business-imeplmentation';
