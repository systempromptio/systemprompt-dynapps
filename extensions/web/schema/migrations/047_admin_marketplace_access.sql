-- The admin marketplace includes India administrators. Retire only the five
-- exclusions previously owned by groups.yaml; preserve operator-authored rules,
-- the knowledge-bank exclusion, and all role grants.
DELETE FROM access_control_rules
WHERE rule_type = 'group'
  AND rule_value = 'india-devs'
  AND access = 'deny'
  AND justification = 'configured access for members of a group or project'
  AND (entity_type, entity_id) IN (
    ('mcp_server', 'systemprompt'),
    ('plugin', 'astound-admin'),
    ('skill', 'admin_activity_report'),
    ('skill', 'admin_user_report'),
    ('skill', 'systemprompt_cli')
  );
