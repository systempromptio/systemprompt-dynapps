-- The system group. Nothing else: every other group and project, and every
-- directory mapping, comes from `services/web/config/groups.yaml` through the
-- boot sync (an empty groups plane is seeded from code; a populated one only
-- reports drift), so a customer's sets never live in a seed.
--
-- Insert-if-absent, so an operator's rename survives every boot. `unassigned`
-- is load-bearing: the `group_memberships` view names it as the home of every
-- user with no `group_members` row, and the groups page treats it as a system
-- row that cannot be deleted. The sync's emptiness probe excludes system
-- groups, so this row alone does not stop the YAML seeding.
INSERT INTO groups (id, name, description, is_system, source)
VALUES ('unassigned', 'Unassigned', 'Signed in through the directory but mapped to no group. Membership is derived: any user with no group row is here.', true, 'system')
ON CONFLICT (id) DO NOTHING;
