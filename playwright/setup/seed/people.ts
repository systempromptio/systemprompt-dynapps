// Groups, projects, memberships and the marketplace entitlement rules the
// access surfaces are asserted against.
import type { Client } from 'pg';
import { E2E } from './principals';

type Source = 'adfs' | 'manual';

// `id` doubles as the slug the admin URLs use (/admin/groups/commerce-devs).
const GROUPS: { id: string; name: string; description: string; adGroups: string[] }[] = [
  {
    id: E2E.groups.commerce,
    name: 'Commerce devs',
    description: 'Storefront and checkout engineers.',
    adGroups: ['Systemprompt-Commerce', 'Systemprompt-Commerce-Dev'],
  },
  {
    id: E2E.groups.europe,
    name: 'Europe devs',
    description: 'EU-resident engineering, payments platform.',
    adGroups: ['Systemprompt-Europe'],
  },
  {
    id: E2E.groups.business,
    name: 'Business users',
    description: 'Non-engineering staff on the business plugin.',
    adGroups: ['Systemprompt-Business'],
  },
];

const PROJECTS: { id: string; name: string; adGroups: string[] }[] = [
  { id: 'storefront', name: 'Storefront', adGroups: ['Systemprompt-Project-Storefront'] },
  { id: 'checkout', name: 'Checkout', adGroups: ['Systemprompt-Project-Checkout'] },
  { id: 'payments', name: 'Payments', adGroups: ['Systemprompt-Project-Payments'] },
  { id: 'platform', name: 'Platform', adGroups: ['Systemprompt-Project-Platform'] },
];

// A user absent from this table has no group_members row and is therefore in
// the DERIVED `unassigned` group (the user_groups view in migration 041), which
// is what the Unassigned page and the assign-to-group flow are built on.
//
// e2e-member-3 is deliberately in TWO groups: without one such person, nothing
// distinguishes "counted once against a primary group" from "counted in full in
// every group", which is the whole point of user_scope_defaults. It is member-3
// rather than a principal because the group-detail specs pin exactly which
// principals each group holds.
const GROUP_MEMBERSHIPS: { user: string; group: string; source: Source; adGroup: string | null }[] = [
  { user: 'e2e-developer', group: 'commerce-devs', source: 'adfs', adGroup: 'Systemprompt-Commerce' },
  { user: 'e2e-user', group: 'commerce-devs', source: 'manual', adGroup: null },
  { user: 'e2e-member-1', group: 'commerce-devs', source: 'adfs', adGroup: 'Systemprompt-Commerce' },
  { user: 'e2e-member-2', group: 'commerce-devs', source: 'adfs', adGroup: 'Systemprompt-Commerce' },
  { user: 'e2e-member-3', group: 'commerce-devs', source: 'adfs', adGroup: 'Systemprompt-Commerce-Dev' },
  { user: 'e2e-member-3', group: 'europe-devs', source: 'adfs', adGroup: 'Systemprompt-Europe' },
  { user: 'e2e-pm', group: 'europe-devs', source: 'manual', adGroup: null },
  { user: 'e2e-member-4', group: 'europe-devs', source: 'adfs', adGroup: 'Systemprompt-Europe' },
  { user: 'e2e-member-5', group: 'europe-devs', source: 'adfs', adGroup: 'Systemprompt-Europe' },
  { user: 'e2e-business-user', group: 'business-users', source: 'adfs', adGroup: 'Systemprompt-Business' },
  { user: 'e2e-member-6', group: 'business-users', source: 'adfs', adGroup: 'Systemprompt-Business' },
  { user: 'e2e-member-7', group: 'business-users', source: 'manual', adGroup: null },
];

const PROJECT_MEMBERSHIPS: { user: string; project: string; source: Source; adGroup: string | null }[] = [
  { user: 'e2e-developer', project: 'storefront', source: 'adfs', adGroup: 'Systemprompt-Project-Storefront' },
  { user: 'e2e-member-1', project: 'storefront', source: 'adfs', adGroup: 'Systemprompt-Project-Storefront' },
  { user: 'e2e-member-2', project: 'storefront', source: 'adfs', adGroup: 'Systemprompt-Project-Storefront' },
  { user: 'e2e-member-3', project: 'checkout', source: 'adfs', adGroup: 'Systemprompt-Project-Checkout' },
  { user: 'e2e-pm', project: 'payments', source: 'manual', adGroup: null },
  { user: 'e2e-member-4', project: 'payments', source: 'adfs', adGroup: 'Systemprompt-Project-Payments' },
  { user: 'e2e-member-5', project: 'payments', source: 'adfs', adGroup: 'Systemprompt-Project-Payments' },
  { user: 'e2e-member-6', project: 'platform', source: 'manual', adGroup: null },
  { user: 'e2e-business-user', project: 'platform', source: 'adfs', adGroup: 'Systemprompt-Project-Platform' },
];

// Each group's marketplace is denied by default and allowed to that group only;
// the admin plugin is denied to business users outright.
const ACL_ENTITIES: { type: string; id: string; defaultIncluded: boolean }[] = [
  { type: 'marketplace', id: E2E.marketplaces.commerce, defaultIncluded: false },
  { type: 'marketplace', id: E2E.marketplaces.europe, defaultIncluded: false },
  { type: 'marketplace', id: E2E.marketplaces.business, defaultIncluded: false },
  { type: 'plugin', id: 'astound-admin', defaultIncluded: false },
];

const ACL_RULES: {
  id: string;
  type: string;
  entity: string;
  ruleType: string;
  ruleValue: string;
  access: string;
}[] = [
  { id: 'e2e-rule-commerce-group', type: 'marketplace', entity: E2E.marketplaces.commerce, ruleType: 'group', ruleValue: E2E.groups.commerce, access: 'allow' },
  { id: 'e2e-rule-europe-group', type: 'marketplace', entity: E2E.marketplaces.europe, ruleType: 'group', ruleValue: E2E.groups.europe, access: 'allow' },
  { id: 'e2e-rule-business-group', type: 'marketplace', entity: E2E.marketplaces.business, ruleType: 'group', ruleValue: E2E.groups.business, access: 'allow' },
  { id: 'e2e-rule-commerce-project', type: 'marketplace', entity: E2E.marketplaces.commerce, ruleType: 'project', ruleValue: E2E.projects.storefront, access: 'allow' },
  { id: 'e2e-rule-admin-business-deny', type: 'plugin', entity: 'astound-admin', ruleType: 'group', ruleValue: E2E.groups.business, access: 'deny' },
];

export async function seedGroupsAndProjects(db: Client) {
  for (const g of GROUPS) {
    await db.query(
      `INSERT INTO groups (id, name, description, is_system, source)
       VALUES ($1, $2, $3, false, 'dashboard')
       ON CONFLICT (id) DO UPDATE SET name = EXCLUDED.name, description = EXCLUDED.description`,
      [g.id, g.name, g.description],
    );
    for (const ad of g.adGroups) {
      await db.query(
        `INSERT INTO group_ad_mappings (ad_group, group_id, source)
         VALUES ($1, $2, 'dashboard') ON CONFLICT DO NOTHING`,
        [ad, g.id],
      );
    }
  }
  for (const p of PROJECTS) {
    await db.query(
      `INSERT INTO projects (id, name, source) VALUES ($1, $2, 'dashboard')
       ON CONFLICT (id) DO UPDATE SET name = EXCLUDED.name`,
      [p.id, p.name],
    );
    for (const ad of p.adGroups) {
      await db.query(
        `INSERT INTO project_ad_mappings (ad_group, project_id, source)
         VALUES ($1, $2, 'dashboard') ON CONFLICT DO NOTHING`,
        [ad, p.id],
      );
    }
  }
}

export async function seedMemberships(db: Client) {
  // Delete first so a re-run after an in-test reassignment (the assign-group
  // spec moves e2e-unassigned into business-users) restores the seeded shape.
  await db.query(`DELETE FROM group_members WHERE user_id LIKE 'e2e-%'`);
  await db.query(`DELETE FROM project_members WHERE user_id LIKE 'e2e-%'`);
  for (const m of GROUP_MEMBERSHIPS) {
    await db.query(
      `INSERT INTO group_members (group_id, user_id, source, source_ad_group)
       VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING`,
      [m.group, m.user, m.source, m.adGroup],
    );
  }
  for (const m of PROJECT_MEMBERSHIPS) {
    await db.query(
      `INSERT INTO project_members (project_id, user_id, source, source_ad_group)
       VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING`,
      [m.project, m.user, m.source, m.adGroup],
    );
  }
}

export async function seedAccessControl(db: Client) {
  for (const e of ACL_ENTITIES) {
    await db.query(
      `INSERT INTO access_control_entities (entity_type, entity_id, default_included, source)
       VALUES ($1, $2, $3, 'e2e')
       ON CONFLICT (entity_type, entity_id) DO NOTHING`,
      [e.type, e.id, e.defaultIncluded],
    );
  }
  for (const r of ACL_RULES) {
    await db.query(
      `INSERT INTO access_control_rules
           (id, entity_type, entity_id, rule_type, rule_value, access, justification)
       VALUES ($1, $2, $3, $4, $5, $6, 'e2e fixture')
       ON CONFLICT (entity_type, entity_id, rule_type, rule_value)
       DO UPDATE SET access = EXCLUDED.access`,
      [r.id, r.type, r.entity, r.ruleType, r.ruleValue, r.access],
    );
  }
}

export const PEOPLE_DATASET = { GROUPS, PROJECTS, GROUP_MEMBERSHIPS, PROJECT_MEMBERSHIPS, ACL_RULES };
