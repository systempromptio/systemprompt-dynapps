-- C19: access control vs sync_state; C20: groups/projects membership; C21: managed_*
\echo == C19 access_control tables and sync state
SELECT (SELECT count(*) FROM access_control_rules) rules, (SELECT count(*) FROM access_control_entities) entities,
       (SELECT string_agg(source||'='||n, ', ') FROM (SELECT source, count(*) n FROM access_control_rules GROUP BY 1) s) rule_sources,
       (SELECT count(*) FROM access_control_rule_validity WHERE valid_until < now()) expired_validity;
SELECT plane, applied_mode, applied_by, applied_at::timestamp(0), declared_hash = applied_hash AS in_sync FROM sync_state ORDER BY 1;
\echo == C19 entity types in DB
SELECT entity_type, count(*) FROM access_control_entities GROUP BY 1 ORDER BY 1;
\echo == C19 rules whose entity is not in access_control_entities
SELECT count(*) FROM access_control_rules r WHERE NOT EXISTS (SELECT 1 FROM access_control_entities e WHERE e.entity_type = r.entity_type AND e.entity_id = r.entity_id);
\echo == C20 groups / projects
SELECT (SELECT count(*) FROM groups) groups, (SELECT count(*) FROM projects) projects, (SELECT count(*) FROM group_members WHERE revoked_at IS NULL) group_memberships, (SELECT count(*) FROM project_members WHERE revoked_at IS NULL) project_memberships,
       (SELECT count(*) FROM users u WHERE NOT EXISTS (SELECT 1 FROM group_members m WHERE m.user_id = u.id AND m.revoked_at IS NULL)) users_in_no_group,
       (SELECT count(*) FROM users u WHERE NOT EXISTS (SELECT 1 FROM project_members m WHERE m.user_id = u.id AND m.revoked_at IS NULL)) users_in_no_project,
       (SELECT count(*) FROM user_scope_defaults d WHERE primary_group_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM groups g WHERE g.id = d.primary_group_id)) defaults_missing_group,
       (SELECT count(*) FROM user_scope_defaults d WHERE primary_project_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM projects p WHERE p.id = d.primary_project_id)) defaults_missing_project,
       (SELECT count(*) FROM (SELECT group_id, user_id FROM group_members WHERE revoked_at IS NULL GROUP BY 1,2 HAVING count(*) > 1) d) dup_group_memberships,
       (SELECT count(*) FROM (SELECT project_id, user_id FROM project_members WHERE revoked_at IS NULL GROUP BY 1,2 HAVING count(*) > 1) d) dup_project_memberships;
\echo == C20 users in more than one group / project (non-exclusive scope attribution)
SELECT (SELECT count(*) FROM (SELECT user_id FROM group_members WHERE revoked_at IS NULL GROUP BY 1 HAVING count(DISTINCT group_id) > 1) x) multi_group_users,
       (SELECT count(*) FROM (SELECT user_id FROM project_members WHERE revoked_at IS NULL GROUP BY 1 HAVING count(DISTINCT project_id) > 1) x) multi_project_users;
\echo == C20 requests whose user is in N groups: scope rows vs requests
SELECT n_groups, count(*) requests FROM (SELECT r.id, (SELECT count(DISTINCT group_id) FROM group_members m WHERE m.user_id = r.user_id AND m.revoked_at IS NULL) n_groups FROM ai_requests r) x GROUP BY 1 ORDER BY 1;
\echo == C21 managed_* tables (row counts; several are empty on this instance)
SELECT relname, n_live_tup FROM pg_stat_user_tables WHERE relname LIKE 'managed\_%' ORDER BY 1;
SELECT (SELECT count(*) FROM managed_consumer_grants WHERE revoked_at IS NULL) active_grants,
       (SELECT count(*) FROM managed_consumer_grants g WHERE revoked_at IS NULL AND NOT EXISTS (SELECT 1 FROM users u WHERE u.id = g.consumer_id)) grants_for_missing_users,
       (SELECT count(*) FROM managed_distribution_outbox WHERE delivered_at IS NULL) undelivered_distribution;
