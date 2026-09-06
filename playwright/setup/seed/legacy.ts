// The two pre-existing analytics trails, kept verbatim in shape because the
// requirements evidence pack and the group-page specs were written against
// their exact numbers.
//
// The 14-day trail is per-member and drives the requirements screenshots; the
// 30-day trail is group-shaped and gives each group a distinct model mix, cost
// band and volume. They use disjoint id prefixes (e2e-req / e2e-greq) so the
// declarative dataset in traffic.ts can never collide with either.
import type { Client } from 'pg';
import { E2E } from './principals';
import { LEGACY_CONTEXT_ID } from './traffic';

// Deterministic per-member, per-day request counts: enough shape for trends
// (weekday hump) without randomness, so charts and visual baselines are stable.
const LEGACY_MODELS = ['claude-opus-5', 'claude-sonnet-5', 'claude-haiku-4-5'];

export async function seedAnalyticsTrail(db: Client) {
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
        const model = LEGACY_MODELS[(day + r + mi) % LEGACY_MODELS.length];
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
         ON CONFLICT DO NOTHING`,
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
    group: E2E.groups.europe,
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
    plugin: 'astound-europe-storefront',
    tools: ['list_skills', 'get_user_report'],
  },
  {
    group: E2E.groups.india,
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
    plugin: 'astound-india-sfcore',
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
    skills: ['meeting_summary', 'client_presentation', 'stakeholder_update'],
    plugin: 'astound-cowork-deliverables',
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

export async function seedGroupTraffic(db: Client) {
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
     ON CONFLICT DO NOTHING`,
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
