// Idempotent e2e seed: principals plus a deterministic 14-day analytics trail.
//
// Every row this script owns has an `e2e-` id prefix or an `@e2e.local` email;
// `--reset` deletes exactly those rows (children before parents) and nothing
// else — never TRUNCATE, never a developer's data. Safe to run repeatedly:
// every statement upserts.
//
// Column shapes mirror tests/contract/admin/src/seed.rs and the declarative
// schema under extensions/web/schema/ — if a seed insert breaks, diff against
// those files first.
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { Client } from 'pg';

const REPO = join(__dirname, '..', '..');

// ai_requests.context_id is NOT NULL; core's sentinel stands in for "no
// known context" (same constant as tests/contract/admin/src/seed.rs).
const LEGACY_CONTEXT_ID = '00000000-0000-0000-0000-4c4547414359';

// Why: core's session middleware attests that a cookie names a session the
// server issued *to that user* — a token with an unknown session_id is treated
// as stale and silently replaced with an anonymous one. The e2e principals
// therefore need real `user_sessions` rows, and the minted JWT must carry the
// matching session id.
export const E2E_SESSIONS: Record<string, string> = {
  'e2e-platform-admin': 'e2e-session-platform-admin',
  'e2e-admin': 'e2e-session-admin',
  'e2e-developer': 'e2e-session-developer',
  'e2e-business-user': 'e2e-session-business-user',
  'e2e-unassigned': 'e2e-session-unassigned',
  'e2e-pm': 'e2e-session-pm',
  'e2e-user': 'e2e-session-user',
};

export const E2E = {
  admin: { id: 'e2e-admin', email: 'e2e-admin@e2e.local' },
  platformAdmin: { id: 'e2e-platform-admin', email: 'e2e-platform-admin@e2e.local' },
  developer: { id: 'e2e-developer', email: 'e2e-developer@e2e.local' },
  businessUser: { id: 'e2e-business-user', email: 'e2e-business-user@e2e.local' },
  unassigned: { id: 'e2e-unassigned', email: 'e2e-unassigned@e2e.local' },
  pm: { id: 'e2e-pm', email: 'e2e-pm@e2e.local' },
  user: { id: 'e2e-user', email: 'e2e-user@e2e.local' },
  members: [1, 2, 3, 4, 5, 6, 7].map((n) => ({
    id: `e2e-member-${n}`,
    email: `e2e-member-${n}@e2e.local`,
  })),
  victim: { id: 'e2e-victim', email: 'e2e-victim@e2e.local' },
  groups: {
    commerce: 'commerce-devs',
    europe: 'europe-devs',
    business: 'business-users',
  },
  projects: { storefront: 'storefront', checkout: 'checkout', payments: 'payments' },
  marketplaces: {
    commerce: 'astound-commerce-dev',
    europe: 'astound-europe-dev',
    business: 'astound-business',
    baseline: 'enterprise-demo',
  },
};

// Every group/project row this seed writes. `id` doubles as the slug the admin
// URLs use (/admin/access/groups/commerce-devs).
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
];

type Source = 'adfs' | 'manual';

// (user, group, source, source_ad_group). A user absent from this table has no
// group_members row and is therefore in the DERIVED `unassigned` group (the
// `user_groups` view in migration 041), which is what the Unassigned page and
// the assign-to-group flow are built on.
const GROUP_MEMBERSHIPS: {
  user: string;
  group: string;
  source: Source;
  adGroup: string | null;
}[] = [
  { user: 'e2e-developer', group: 'commerce-devs', source: 'adfs', adGroup: 'Systemprompt-Commerce' },
  { user: 'e2e-user', group: 'commerce-devs', source: 'manual', adGroup: null },
  { user: 'e2e-member-1', group: 'commerce-devs', source: 'adfs', adGroup: 'Systemprompt-Commerce' },
  { user: 'e2e-member-2', group: 'commerce-devs', source: 'adfs', adGroup: 'Systemprompt-Commerce' },
  { user: 'e2e-member-3', group: 'commerce-devs', source: 'adfs', adGroup: 'Systemprompt-Commerce-Dev' },
  { user: 'e2e-pm', group: 'europe-devs', source: 'manual', adGroup: null },
  { user: 'e2e-member-4', group: 'europe-devs', source: 'adfs', adGroup: 'Systemprompt-Europe' },
  { user: 'e2e-member-5', group: 'europe-devs', source: 'adfs', adGroup: 'Systemprompt-Europe' },
  { user: 'e2e-business-user', group: 'business-users', source: 'adfs', adGroup: 'Systemprompt-Business' },
  { user: 'e2e-member-6', group: 'business-users', source: 'adfs', adGroup: 'Systemprompt-Business' },
  { user: 'e2e-member-7', group: 'business-users', source: 'manual', adGroup: null },
];

const PROJECT_MEMBERSHIPS: {
  user: string;
  project: string;
  source: Source;
  adGroup: string | null;
}[] = [
  { user: 'e2e-developer', project: 'storefront', source: 'adfs', adGroup: 'Systemprompt-Project-Storefront' },
  { user: 'e2e-member-1', project: 'storefront', source: 'adfs', adGroup: 'Systemprompt-Project-Storefront' },
  { user: 'e2e-member-2', project: 'storefront', source: 'adfs', adGroup: 'Systemprompt-Project-Storefront' },
  { user: 'e2e-member-3', project: 'checkout', source: 'adfs', adGroup: 'Systemprompt-Project-Checkout' },
  { user: 'e2e-pm', project: 'payments', source: 'manual', adGroup: null },
  { user: 'e2e-member-4', project: 'payments', source: 'adfs', adGroup: 'Systemprompt-Project-Payments' },
  { user: 'e2e-member-5', project: 'payments', source: 'adfs', adGroup: 'Systemprompt-Project-Payments' },
];

// roles per principal; `manual` roles additionally land in user_manual_roles so
// the role editor can tell a dashboard grant from a directory-held one.
const PRINCIPALS: { id: string; email: string; roles: string[]; manualRoles: string[] }[] = [
  { ...E2E.platformAdmin, roles: ['platform_admin', 'admin', 'user'], manualRoles: ['platform_admin'] },
  { ...E2E.admin, roles: ['admin', 'user'], manualRoles: [] },
  { ...E2E.developer, roles: ['developer', 'user'], manualRoles: [] },
  { ...E2E.businessUser, roles: ['user'], manualRoles: [] },
  { ...E2E.unassigned, roles: ['user'], manualRoles: [] },
  { ...E2E.pm, roles: ['project_manager', 'user'], manualRoles: ['project_manager'] },
  { ...E2E.user, roles: ['user'], manualRoles: [] },
  { ...E2E.victim, roles: ['user'], manualRoles: [] },
  ...E2E.members.map((m) => ({ ...m, roles: ['user'], manualRoles: [] as string[] })),
];

export function databaseUrl(): string {
  if (process.env.E2E_DATABASE_URL) return process.env.E2E_DATABASE_URL;
  const secrets = JSON.parse(
    readFileSync(join(REPO, '.systemprompt', 'profiles', 'local', 'secrets.json'), 'utf8'),
  );
  if (!secrets.database_url) throw new Error('no database_url in local profile secrets.json');
  return secrets.database_url;
}

async function upsertUser(db: Client, id: string, email: string, roles: string[]) {
  // users.name is unique; the email doubles as the name, as production
  // provisioning does (see tests/contract/admin/src/seed.rs::insert_user).
  const r = await db.query(
    `INSERT INTO users (id, name, email, display_name, status, email_verified, roles)
     VALUES ($1, $2, $2, $3, 'active', true, $4)
     ON CONFLICT (id) DO UPDATE SET roles = EXCLUDED.roles, status = 'active' RETURNING id`,
    [id, email, email.split('@')[0], roles],
  );
  if (r.rowCount !== 1) throw new Error(`upsertUser(${id}) affected ${r.rowCount} rows`);
}

// Migration 041 replaced `user_profile_ext.project` and the
// `adfs_group_memberships` side table with `groups` / `group_members` /
// `projects` / `project_members`. If those tables are missing, the DB has not
// been migrated and every group assertion downstream would fail obscurely, so
// fail here with the migration named.
async function requireGroupsSchema(db: Client) {
  const tables = [
    'groups',
    'group_members',
    'group_ad_mappings',
    'projects',
    'project_members',
    'project_ad_mappings',
    'user_manual_roles',
    'user_groups',
  ];
  const { rows } = await db.query<{ present: string }>(
    `SELECT c.relname AS present FROM pg_class c
      JOIN pg_namespace n ON n.oid = c.relnamespace
      WHERE n.nspname = 'public' AND c.relname = ANY($1)`,
    [tables],
  );
  const present = new Set(rows.map((r) => r.present));
  const missing = tables.filter((t) => !present.has(t));
  if (missing.length > 0) {
    throw new Error(
      `e2e seed: migration 041_groups_projects.sql has not been applied — missing ${missing.join(', ')}. ` +
        `Build and restart the server (migrations run at startup), then re-run the seed.`,
    );
  }
}

async function seedGroupsAndProjects(db: Client) {
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

async function seedMemberships(db: Client) {
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

async function seedPrincipals(db: Client) {
  for (const p of PRINCIPALS) {
    await upsertUser(db, p.id, p.email, p.roles);
    try {
      await db.query(
        `INSERT INTO user_profile_ext (user_id) VALUES ($1) ON CONFLICT (user_id) DO NOTHING`,
        [p.id],
      );
    } catch (e) {
      throw new Error(`profile row for ${p.id} (${p.email}): ${(e as Error).message} :: ${(e as { detail?: string }).detail ?? ""}`);
    }
  }
  await db.query(`DELETE FROM user_manual_roles WHERE user_id LIKE 'e2e-%'`);
  for (const p of PRINCIPALS) {
    for (const role of p.manualRoles) {
      await db.query(
        `INSERT INTO user_manual_roles (user_id, role) VALUES ($1, $2) ON CONFLICT DO NOTHING`,
        [p.id, role],
      );
    }
  }

  for (const [userId, sessionId] of Object.entries(E2E_SESSIONS)) {
    await db.query(
      `INSERT INTO user_sessions (session_id, user_id, user_type, expires_at, last_activity_at)
       VALUES ($1, $2, 'registered', NOW() + INTERVAL '7 days', NOW())
       ON CONFLICT (session_id) DO UPDATE
          SET expires_at = NOW() + INTERVAL '7 days',
              last_activity_at = NOW(),
              ended_at = NULL`,
      [sessionId, userId],
    );
  }
}

// Marketplace entitlement the group pages and the bridge manifest are asserted
// against: each group's marketplace is denied by default and allowed to that
// group only; the admin plugin is denied to business users outright.
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
  {
    id: 'e2e-rule-commerce-group',
    type: 'marketplace',
    entity: E2E.marketplaces.commerce,
    ruleType: 'group',
    ruleValue: E2E.groups.commerce,
    access: 'allow',
  },
  {
    id: 'e2e-rule-europe-group',
    type: 'marketplace',
    entity: E2E.marketplaces.europe,
    ruleType: 'group',
    ruleValue: E2E.groups.europe,
    access: 'allow',
  },
  {
    id: 'e2e-rule-business-group',
    type: 'marketplace',
    entity: E2E.marketplaces.business,
    ruleType: 'group',
    ruleValue: E2E.groups.business,
    access: 'allow',
  },
  {
    id: 'e2e-rule-commerce-project',
    type: 'marketplace',
    entity: E2E.marketplaces.commerce,
    ruleType: 'project',
    ruleValue: E2E.projects.storefront,
    access: 'allow',
  },
  {
    id: 'e2e-rule-admin-business-deny',
    type: 'plugin',
    entity: 'astound-admin',
    ruleType: 'group',
    ruleValue: E2E.groups.business,
    access: 'deny',
  },
];

async function seedAccessControl(db: Client) {
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

// Deterministic per-member, per-day request counts: enough shape for trends
// (weekday hump) without randomness, so charts and visual baselines are stable.
const MODELS = ['claude-opus-5', 'claude-sonnet-5', 'claude-haiku-4-5'];

async function seedAnalyticsTrail(db: Client) {
  for (const [mi, m] of E2E.members.slice(0, 3).entries()) {
    for (let day = 0; day < 14; day++) {
      const perDay = 1 + ((day + mi) % 4); // 1..4 requests/day
      const sessionId = `e2e-session-${m.id}-${day}`;
      await db.query(
        `INSERT INTO user_sessions (session_id, user_id) VALUES ($1, $2)
         ON CONFLICT (session_id) DO NOTHING`,
        [sessionId, m.id],
      );
      for (let r = 0; r < perDay; r++) {
        const id = `e2e-req-${m.id}-${day}-${r}`;
        const model = MODELS[(day + r + mi) % MODELS.length];
        // The app's success vocabulary is completed/pending/streaming — anything
        // else counts as an error, so 'success' here would read as a 100% error
        // rate on the dashboard.
        const status = (day + r) % 9 === 0 ? 'failed' : 'completed';
        const cost = 900 + 350 * ((r + mi) % 3); // microdollars
        const latency = 400 + 5200 * (r % 2); // exercises the fast/slow split
        await db.query(
          `INSERT INTO ai_requests (
               id, request_id, user_id, session_id, trace_id, context_id,
               provider, model, input_tokens, output_tokens, tokens_used,
               cost_microdollars, latency_ms, status, actor_kind, actor_id,
               created_at, updated_at)
           VALUES ($1::TEXT, $1::TEXT, $2::TEXT, $3, $4, $5, 'anthropic', $6, 120, 30, 150,
                   $7, $8, $9, 'user', $2::TEXT,
                   NOW() - ($10 || ' days')::interval - ($11 || ' hours')::interval,
                   NOW() - ($10 || ' days')::interval)
           ON CONFLICT DO NOTHING`,
          [id, m.id, sessionId, `e2e-trace-${m.id}-${day}`, LEGACY_CONTEXT_ID,
           model, cost, latency, status, String(day), String(9 + r)],
        );
      }
      const decisionId = `e2e-gov-${m.id}-${day}`;
      const decision = day % 5 === 0 ? 'deny' : 'allow';
      await db.query(
        `INSERT INTO governance_decisions (
             id, user_id, session_id, context_id, tool_name, decision, policy, reason,
             actor_kind, actor_id, created_at)
         VALUES ($1, $2, $3, $4, 'Edit', $5, $6, 'e2e fixture', 'user', $2,
                 NOW() - ($7 || ' days')::interval)
         ON CONFLICT (id) DO NOTHING`,
        [decisionId, m.id, sessionId, LEGACY_CONTEXT_ID, decision,
         decision === 'deny' ? 'tool_blocklist' : 'governance_disabled', String(day)],
      );
      await db.query(
        `INSERT INTO plugin_usage_events (id, user_id, session_id, event_type, tool_name, created_at)
         VALUES ($1, $2, $3, 'claude_code_PostToolUse', 'Edit',
                 NOW() - ($4 || ' days')::interval)
         ON CONFLICT (id) DO NOTHING`,
        [`e2e-evt-${m.id}-${day}`, m.id, sessionId, String(day)],
      );
      // A permission request and the tool use that answers it, so the Usage
      // tab's grant-rate proxy has both halves of its pair. Every third day
      // the request goes unanswered, which is what makes the rate < 100%.
      await db.query(
        `INSERT INTO plugin_usage_events (id, user_id, session_id, event_type, tool_name, created_at)
         VALUES ($1, $2, $3, 'PermissionRequest', $4,
                 NOW() - ($5 || ' days')::interval - INTERVAL '2 minutes')
         ON CONFLICT (id) DO NOTHING`,
        [`e2e-perm-${m.id}-${day}`, m.id, sessionId, 'Bash', String(day)],
      );
      if (day % 3 !== 0) {
        await db.query(
          `INSERT INTO plugin_usage_events (id, user_id, session_id, event_type, tool_name, created_at)
           VALUES ($1, $2, $3, 'PostToolUse', $4,
                   NOW() - ($5 || ' days')::interval - INTERVAL '1 minute')
           ON CONFLICT (id) DO NOTHING`,
          [`e2e-perm-ok-${m.id}-${day}`, m.id, sessionId, 'Bash', String(day)],
        );
      }
      await db.query(
        `INSERT INTO plugin_session_summaries
             (id, session_id, user_id, started_at, tool_uses, prompts, errors,
              model, status, ai_title, total_events)
         VALUES ($1, $2, $3, NOW() - ($4 || ' days')::interval, 3, 2, 0,
                 'claude-opus-5', 'active', $5, 5)
         ON CONFLICT (id) DO NOTHING`,
        [`e2e-sum-${m.id}-${day}`, sessionId, m.id, String(day), `e2e session ${day}`],
      );
      // Code tab sources: daily rollups (commits + AI LOC) and per-tool daily
      // edit operations.
      await db.query(
        `INSERT INTO admin_usage_daily_rollups
             (user_id, date, sessions_count, prompts, tool_uses, errors,
              loc_added_ai, loc_removed_ai, commits_count, commit_insertions,
              commit_deletions, ai_requests_count, input_tokens, output_tokens,
              cost_microdollars)
         VALUES ($1, (NOW() - ($2 || ' days')::interval)::date, 1, 4, 6, 0,
                 $3, $4, $5, $6, $7, $8, 480, 120, $9)
         ON CONFLICT (user_id, date) DO UPDATE SET
             loc_added_ai = EXCLUDED.loc_added_ai,
             commits_count = EXCLUDED.commits_count`,
        [m.id, String(day), 40 + 10 * (day % 3), 8, 1 + (day % 2),
         60 + 12 * (day % 4), 9, perDay, perDay * 1200],
      );
      await db.query(
        `INSERT INTO plugin_usage_daily
             (id, date, event_type, tool_name, user_id, event_count, loc_added, loc_removed)
         VALUES ($1, (NOW() - ($2 || ' days')::interval)::date,
                 'claude_code_PostToolUse', 'Edit', $3, $4, $5, $6)
         ON CONFLICT (id) DO NOTHING`,
        [`e2e-daily-${m.id}-${day}`, String(day), m.id, 6, 40 + 10 * (day % 3), 8],
      );
    }
    // Client-reported statusline snapshot (Usage tab cache/context cards).
    await db.query(
      `INSERT INTO session_cost_snapshots
           (session_id, user_id, model, total_cost_microdollars, context_window_size,
            input_tokens, output_tokens, cache_creation_input_tokens, cache_read_input_tokens)
       VALUES ($1, $2, 'claude-opus-5', 15000, 180000, 5000, 1500, 20000, 45000)
       ON CONFLICT (session_id) DO UPDATE SET updated_at = NOW()`,
      [`e2e-session-${m.id}-0`, m.id],
    );
  }
}


// --- 30-day, group-shaped traffic -------------------------------------------
//
// Separate from the 14-day trail above (which the requirements evidence pack
// depends on) and prefixed `e2e-g` so the two never collide. Each group gets a
// distinct model mix, cost band and volume so the group pages, the model-mix
// share bars and the cost columns are visibly different from each other.
interface GroupTrafficProfile {
  group: string;
  users: string[];
  models: { model: string; provider: string }[];
  costLow: number;
  costHigh: number;
  perDayLow: number;
  perDayHigh: number;
  inputTokens: number;
  outputTokens: number;
  weekdaysOnly: boolean;
  skills: string[];
  plugin: string;
  tools: string[];
}

const TRAFFIC: GroupTrafficProfile[] = [
  {
    group: E2E.groups.commerce,
    users: ['e2e-developer', 'e2e-member-1', 'e2e-member-2', 'e2e-member-3'],
    models: [
      { model: 'claude-opus-5', provider: 'anthropic' },
      { model: 'claude-opus-5', provider: 'anthropic' },
      { model: 'claude-sonnet-5', provider: 'anthropic' },
    ],
    costLow: 4200,
    costHigh: 6800,
    perDayLow: 3,
    perDayHigh: 6,
    inputTokens: 2400,
    outputTokens: 600,
    weekdaysOnly: false,
    skills: ['systematic_debugging', 'code_review', 'git_commit'],
    plugin: 'astound-commerce-dev',
    tools: ['list_skills', 'get_user_report'],
  },
  {
    group: E2E.groups.europe,
    users: ['e2e-pm', 'e2e-member-4', 'e2e-member-5'],
    models: [
      { model: 'gemini-2.5-pro', provider: 'vertex' },
      { model: 'gemini-2.5-flash', provider: 'vertex' },
      { model: 'claude-sonnet-5', provider: 'anthropic' },
    ],
    costLow: 1100,
    costHigh: 2600,
    perDayLow: 2,
    perDayHigh: 4,
    inputTokens: 1800,
    outputTokens: 400,
    weekdaysOnly: false,
    skills: ['systematic_debugging', 'git_commit'],
    plugin: 'astound-europe-dev',
    tools: ['list_skills'],
  },
  {
    group: E2E.groups.business,
    users: ['e2e-business-user', 'e2e-member-6', 'e2e-member-7'],
    models: [{ model: 'claude-haiku-4-5', provider: 'anthropic' }],
    costLow: 250,
    costHigh: 400,
    perDayLow: 1,
    perDayHigh: 2,
    inputTokens: 600,
    outputTokens: 120,
    weekdaysOnly: true,
    skills: ['meeting_summary', 'requirements_brief', 'stakeholder_update'],
    plugin: 'astound-business',
    tools: ['get_activity_report'],
  },
];

function band(low: number, high: number, n: number): number {
  const span = high - low;
  return low + (span === 0 ? 0 : (n * 137) % (span + 1));
}

function span(low: number, high: number, n: number): number {
  return low + (n % (high - low + 1));
}

async function insertRequest(
  db: Client,
  args: {
    id: string;
    userId: string;
    sessionId: string;
    model: string;
    provider: string;
    inputTokens: number;
    outputTokens: number;
    cost: number;
    latency: number;
    status: string;
    day: number;
    hour: number;
  },
) {
  await db.query(
    `INSERT INTO ai_requests (
         id, request_id, user_id, session_id, trace_id, context_id,
         provider, model, input_tokens, output_tokens, tokens_used,
         cost_microdollars, latency_ms, status, actor_kind, actor_id,
         created_at, updated_at)
     VALUES ($1::TEXT, $1::TEXT, $2::TEXT, $3, $4, $5, $6, $7, $8, $9, $10,
             $11, $12, $13, 'user', $2::TEXT,
             NOW() - ($14 || ' days')::interval - ($15 || ' hours')::interval,
             NOW() - ($14 || ' days')::interval)
     ON CONFLICT DO NOTHING`,
    [
      args.id,
      args.userId,
      args.sessionId,
      `e2e-gtrace-${args.userId}-${args.day}`,
      LEGACY_CONTEXT_ID,
      args.provider,
      args.model,
      args.inputTokens,
      args.outputTokens,
      args.inputTokens + args.outputTokens,
      args.cost,
      args.latency,
      args.status,
      String(args.day),
      String(args.hour),
    ],
  );
}

async function seedGroupTraffic(db: Client) {
  for (const profile of TRAFFIC) {
    for (const [ui, userId] of profile.users.entries()) {
      for (let day = 0; day < 30; day++) {
        const weekday = (new Date(Date.now() - day * 86400000).getUTCDay() + 6) % 7 < 5;
        if (profile.weekdaysOnly && !weekday) continue;
        const sessionId = `e2e-gsession-${userId}-${day}`;
        await db.query(
          `INSERT INTO user_sessions (session_id, user_id, user_type, expires_at, last_activity_at, started_at)
           VALUES ($1, $2, 'registered', NOW() + INTERVAL '7 days',
                   NOW() - ($3 || ' days')::interval, NOW() - ($3 || ' days')::interval)
           ON CONFLICT (session_id) DO NOTHING`,
          [sessionId, userId, String(day)],
        );
        const perDay = span(profile.perDayLow, profile.perDayHigh, day + ui);
        for (let r = 0; r < perDay; r++) {
          const mix = profile.models[(day + r + ui) % profile.models.length];
          await insertRequest(db, {
            id: `e2e-greq-${userId}-${day}-${r}`,
            userId,
            sessionId,
            model: mix.model,
            provider: mix.provider,
            inputTokens: profile.inputTokens,
            outputTokens: profile.outputTokens,
            cost: band(profile.costLow, profile.costHigh, day + r + ui),
            latency: 600 + 1800 * ((day + r) % 3),
            status: (day * 7 + r) % 11 === 0 ? 'failed' : 'completed',
            day,
            hour: 9 + (r % 8),
          });
        }
        await seedSkillAndTools(db, profile, userId, sessionId, day, ui);
      }
    }
  }

  // The unassigned principal: light, sporadic traffic so the Unassigned page
  // is not empty but is obviously an outlier next to the groups.
  for (let day = 0; day < 30; day += 3) {
    const sessionId = `e2e-gsession-e2e-unassigned-${day}`;
    await db.query(
      `INSERT INTO user_sessions (session_id, user_id, user_type, expires_at, last_activity_at)
       VALUES ($1, 'e2e-unassigned', 'registered', NOW() + INTERVAL '7 days',
               NOW() - ($2 || ' days')::interval)
       ON CONFLICT (session_id) DO NOTHING`,
      [sessionId, String(day)],
    );
    await insertRequest(db, {
      id: `e2e-greq-e2e-unassigned-${day}-0`,
      userId: 'e2e-unassigned',
      sessionId,
      model: 'claude-haiku-4-5',
      provider: 'anthropic',
      inputTokens: 600,
      outputTokens: 120,
      cost: band(250, 400, day),
      latency: 900,
      status: day % 11 === 0 ? 'failed' : 'completed',
      day,
      hour: 11,
    });
  }
}

// Skill invocations are read by the `skill_invocation_events` view (migration
// 040) from plugin_usage_events where event_type='UserPromptSubmit' and the
// prompt starts `/plugin:skill` — the slash form needs no governance row.
async function seedSkillAndTools(
  db: Client,
  profile: GroupTrafficProfile,
  userId: string,
  sessionId: string,
  day: number,
  ui: number,
) {
  const skill = profile.skills[(day + ui) % profile.skills.length];
  await db.query(
    `INSERT INTO plugin_usage_events
         (id, user_id, session_id, event_type, plugin_id, prompt_preview, created_at)
     VALUES ($1, $2, $3, 'UserPromptSubmit', $4, $5, NOW() - ($6 || ' days')::interval)
     ON CONFLICT (id) DO NOTHING`,
    [
      `e2e-skill-${userId}-${day}`,
      userId,
      sessionId,
      profile.plugin,
      `/${profile.plugin}:${skill}`,
      String(day),
    ],
  );
  await db.query(
    `INSERT INTO plugin_usage_daily
         (id, date, plugin_id, event_type, tool_name, user_id, event_count)
     VALUES ($1, (NOW() - ($2 || ' days')::interval)::date, $3, 'PostToolUse', $4, $5, $6)
     ON CONFLICT (id) DO NOTHING`,
    [
      `e2e-gdaily-${userId}-${day}`,
      String(day),
      profile.plugin,
      profile.tools[day % profile.tools.length],
      userId,
      1 + (day % 3),
    ],
  );
  const toolCalls = (day + ui) % 3;
  for (let n = 0; n < toolCalls; n++) {
    const tool = profile.tools[(day + n) % profile.tools.length];
    await db.query(
      `INSERT INTO mcp_tool_executions
           (mcp_execution_id, tool_name, server_name, started_at, completed_at,
            execution_time_ms, input, output, status, user_id, session_id,
            actor_kind, actor_id, created_at)
       VALUES ($1, $2, 'systemprompt',
               NOW() - ($3 || ' days')::interval,
               NOW() - ($3 || ' days')::interval + INTERVAL '2 seconds',
               $4, '{}', '{}', 'success', $5::TEXT, $6, 'user', $5::TEXT,
               NOW() - ($3 || ' days')::interval)
       ON CONFLICT (mcp_execution_id) DO NOTHING`,
      [`e2e-mcp-${userId}-${day}-${n}`, tool, String(day), 800 + 400 * n, userId, sessionId],
    );
  }
}

async function reset(db: Client) {
  // Children before parents; every predicate is anchored to the e2e prefix.
  //
  // The `groups` and `projects` rows themselves are deliberately NOT deleted:
  // a developer may have hand-assigned a real user to commerce-devs, and the
  // membership rows above (all `e2e-%`) are what this seed actually owns. The
  // group rows are upserted on every run, so they cannot drift.
  const stmts = [
    `DELETE FROM mcp_tool_executions WHERE mcp_execution_id LIKE 'e2e-%'`,
    `DELETE FROM access_control_rules WHERE id LIKE 'e2e-rule-%'`,
    `DELETE FROM access_control_entities WHERE source = 'e2e'`,
    `DELETE FROM group_members WHERE user_id LIKE 'e2e-%'`,
    `DELETE FROM project_members WHERE user_id LIKE 'e2e-%'`,
    `DELETE FROM user_manual_roles WHERE user_id LIKE 'e2e-%'`,
    `DELETE FROM session_cost_snapshots WHERE session_id LIKE 'e2e-%'`,
    `DELETE FROM plugin_usage_daily WHERE id LIKE 'e2e-%'`,
    `DELETE FROM admin_usage_daily_rollups WHERE user_id LIKE 'e2e-%'`,
    `DELETE FROM plugin_session_summaries WHERE id LIKE 'e2e-%'`,
    `DELETE FROM plugin_usage_events WHERE id LIKE 'e2e-%'`,
    `DELETE FROM governance_decisions WHERE id LIKE 'e2e-%'`,
    `DELETE FROM ai_requests WHERE id LIKE 'e2e-%'`,
    `DELETE FROM user_commits WHERE user_id LIKE 'e2e-%'`,
    `DELETE FROM user_sessions WHERE session_id LIKE 'e2e-%'`,
    `DELETE FROM user_profile_ext WHERE user_id LIKE 'e2e-%'`,
    `DELETE FROM users WHERE email LIKE '%@e2e.local'`,
  ];
  for (const s of stmts) await db.query(s);
}

export async function seed(opts: { reset?: boolean } = {}) {
  const db = new Client({ connectionString: databaseUrl() });
  await db.connect();
  try {
    await requireGroupsSchema(db);
    if (opts.reset) await reset(db);
    await seedPrincipals(db);
    await seedGroupsAndProjects(db);
    await seedMemberships(db);
    await seedAccessControl(db);
    await seedAnalyticsTrail(db);
    await seedGroupTraffic(db);
  } finally {
    await db.end();
  }
}

if (require.main === module) {
  seed({ reset: process.argv.includes('--reset') })
    .then(() => console.log('e2e seed complete'))
    .catch((e) => {
      console.error('e2e seed failed:', e.message);
      console.error(e.stack);
      process.exit(1);
    });
}
