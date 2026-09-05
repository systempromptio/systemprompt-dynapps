// Warn mode, end to end, on real gateway traffic (REQ-030/033/036/041).
//
// All three enforcement planes on an inference request are configured to warn
// on this deployment: the governance chain (`governance.mode: warn`), the
// safety scanners (`safety.mode: warn`) and the quota windows
// (`quota_mode: warn`). "Nothing blocks" is only true if it is true on the
// wire, so each case here sends a request that every plane would have refused
// under enforce, asserts it came back 200 from the mock upstream, and then
// asserts the refusal-that-was-not is on disk: a `governance_decisions` row
// with `decision = 'warn'` naming the policy, or an `ai_safety_findings` row
// with `blocked = false`. The last case reads the operator surface that
// renders those rows, because a warn nobody can see is a warn that never
// gets tuned.
//
// The quota case pre-fills the caller's hourly bucket to the ceiling through
// the same table the gateway reserves against, then resets it, so it never
// waits for six hundred real calls and never leaves the principal exhausted
// for the specs running beside it.
import { Client } from 'pg';
import { databaseUrl } from '../../setup/seed';
import type { Page } from '@playwright/test';
import { test, expect } from '../support/fixtures';

// Why: every case shares one principal, and the quota case fills that
// principal's live bucket. Run in parallel, the clean-prompt case would see the
// quota warn the other case provoked. Serial keeps each case's rows its own.
test.describe.configure({ mode: 'serial' });

const PRINCIPAL = 'e2e-platform-admin';
const SESSION = 'e2e-session-platform-admin';
const REQUEST_ID_HEADER = 'x-systemprompt-request-id';

// 32 random bytes as base64: no vendor prefix, unreadable when decoded, over
// the entropy ceiling. The shape the backstop exists to catch.
const PREFIXLESS_KEY = 'oJXyD5OVZQz5OAuO2yJKaySKHpJOj9CuLhqUkqMwXxg';
const SSN = '219-09-9999';



async function issuePat(page: Page): Promise<{ id: string; secret: string }> {
  const issued = await page.request.post('/admin/devices/pats', {
    data: { name: `e2e-warn-${Date.now()}` },
  });
  expect(issued.status()).toBe(200);
  return (await issued.json()) as { id: string; secret: string };
}

async function governedCall(page: Page, pat: string, prompt: string) {
  const res = await page.request.post('/v1/messages', {
    headers: {
      authorization: `Bearer ${pat}`,
      'x-session-id': SESSION,
      'anthropic-version': '2023-06-01',
    },
    data: { model: 'e2e-mock-sonnet', max_tokens: 128, messages: [{ role: 'user', content: prompt }] },
  });
  const requestId = res.headers()[REQUEST_ID_HEADER];
  return { res, requestId };
}

async function withDb<T>(fn: (db: Client) => Promise<T>): Promise<T> {
  const db = new Client({ connectionString: databaseUrl() });
  await db.connect();
  try {
    return await fn(db);
  } finally {
    await db.end();
  }
}

interface DecisionRow {
  decision: string;
  policy: string;
  reason: string;
}

// Why: the gateway keys the chain's idempotency on the ai-request id, and
// writes it as the audit blob's call_id, so that — not the session — is the
// join back from a response to its decision rows.
async function decisionsFor(requestId: string): Promise<DecisionRow[]> {
  return withDb(async (db) => {
    const r = await db.query<DecisionRow>(
      `SELECT decision, policy, reason FROM governance_decisions
        WHERE evaluated_rules->>'call_id' = $1 ORDER BY created_at`,
      [requestId],
    );
    return r.rows;
  });
}

async function findingsFor(requestId: string): Promise<{ category: string; blocked: boolean }[]> {
  return withDb(async (db) => {
    const r = await db.query<{ category: string; blocked: boolean }>(
      `SELECT category, blocked FROM ai_safety_findings WHERE ai_request_id = $1`,
      [requestId],
    );
    return r.rows;
  });
}

function hourWindowStart(): Date {
  const secs = Math.floor(Date.now() / 1000);
  return new Date(Math.floor(secs / 3600) * 3600 * 1000);
}

// Why: `requests` is set, not incremented, so a re-run after a crash starts
// from the same place; the gateway's own reservation adds the one that tips
// it over the 600-request ceiling. The gateway may already own this window's
// row under its own id, so the row is updated in place when it exists and
// inserted only when it does not.
async function fillHourlyBucket(userId: string, requests: number): Promise<void> {
  await withDb(async (db) => {
    const start = hourWindowStart();
    const updated = await db.query(
      `UPDATE ai_quota_buckets SET requests = $3, updated_at = NOW()
        WHERE subject_kind = 'user' AND subject_id = $1 AND window_seconds = 3600
          AND window_start = $2`,
      [userId, start, requests],
    );
    if (updated.rowCount === 0) {
      await db.query(
        `INSERT INTO ai_quota_buckets
           (id, subject_id, subject_kind, window_seconds, window_start, requests)
         VALUES ($1, $2, 'user', 3600, $3, $4)
         ON CONFLICT DO NOTHING`,
        [`e2e-bucket-${userId}-${start.getTime()}`, userId, start, requests],
      );
    }
  });
}

test.describe('@REQ-041 @REQ-030 governance chain in warn mode on the wire', () => {
  test('@REQ-041 a prefixless high-entropy key is answered, and recorded as a secret_scan warn', async ({
    platformAdminPage: page,
  }) => {
    const pat = await issuePat(page);
    const { res, requestId } = await governedCall(
      page,
      pat.secret,
      `rotate the deploy token ${PREFIXLESS_KEY} before Friday`,
    );
    // Under enforce this was a 403 from secret_scan's entropy backstop — the
    // exact denial that was blocking ordinary developer traffic.
    expect(res.status()).toBe(200);
    expect(requestId).toBeTruthy();
    const reply = (await res.json()) as { content: { text: string }[] };
    expect(reply.content[0].text).toContain('Mock reply');

    const rows = await decisionsFor(requestId);
    const warn = rows.find((r) => r.policy === 'secret_scan');
    expect(warn, `expected a secret_scan row, got ${JSON.stringify(rows)}`).toBeTruthy();
    expect(warn?.decision).toBe('warn');
    expect(warn?.reason).toMatch(/secret detected/i);
    expect(rows.some((r) => r.decision === 'deny')).toBe(false);

    await page.request.delete(`/admin/devices/pats/${pat.id}`);
  });

  test('@REQ-041 a clean prompt is an allow, not a warn — warn mode does not relabel clean traffic', async ({
    platformAdminPage: page,
  }) => {
    const pat = await issuePat(page);
    const { res, requestId } = await governedCall(page, pat.secret, 'summarise the release notes');
    expect(res.status()).toBe(200);

    const rows = await decisionsFor(requestId);
    expect(rows.length).toBeGreaterThan(0);
    expect(rows.every((r) => r.decision === 'allow')).toBe(true);

    await page.request.delete(`/admin/devices/pats/${pat.id}`);
  });
});

test.describe('@REQ-030 @REQ-036 safety scanners in warn mode on the wire', () => {
  test('@REQ-036 an SSN in a request is answered, and its finding is recorded with blocked = false', async ({
    platformAdminPage: page,
  }) => {
    const pat = await issuePat(page);
    const { res, requestId } = await governedCall(
      page,
      pat.secret,
      `the applicant's SSN is ${SSN}, draft the verification letter`,
    );
    // pii_ssn is in block_categories: under enforce this was a 400.
    expect(res.status()).toBe(200);

    const findings = await findingsFor(requestId);
    const ssn = findings.find((f) => f.category === 'pii_ssn');
    expect(ssn, `expected a pii_ssn finding, got ${JSON.stringify(findings)}`).toBeTruthy();
    expect(ssn?.blocked).toBe(false);
    expect(findings.every((f) => f.blocked === false)).toBe(true);

    await page.request.delete(`/admin/devices/pats/${pat.id}`);
  });
});

test.describe('@REQ-033 quota windows in warn mode on the wire', () => {
  test('@REQ-033 an exhausted hourly window is answered, and recorded as a quota warn', async ({
    platformAdminPage: page,
  }) => {
    const pat = await issuePat(page);
    await fillHourlyBucket(PRINCIPAL, 600);
    try {
      const { res, requestId } = await governedCall(page, pat.secret, 'one more after the ceiling');
      // Under enforce this was a 429 with retry-after.
      expect(res.status()).toBe(200);

      const rows = await decisionsFor(requestId);
      const warn = rows.find((r) => r.policy === 'quota');
      expect(warn, `expected a quota row, got ${JSON.stringify(rows)}`).toBeTruthy();
      expect(warn?.decision).toBe('warn');
      expect(warn?.reason).toMatch(/quota exceeded for user window 3600s/);
    } finally {
      await fillHourlyBucket(PRINCIPAL, 0);
      await page.request.delete(`/admin/devices/pats/${pat.id}`);
    }
  });
});

test.describe('@REQ-026 warns are visible where an operator would tune them', () => {
  test('@REQ-026 the governance warnings page lists the policies that warned in this run', async ({
    platformAdminPage: page,
  }) => {
    // Produce one warn of each kind first so the page has this run's rows.
    const pat = await issuePat(page);
    await fillHourlyBucket(PRINCIPAL, 600);
    try {
      const { res } = await governedCall(page, pat.secret, `token ${PREFIXLESS_KEY}`);
      expect(res.status()).toBe(200);
    } finally {
      await fillHourlyBucket(PRINCIPAL, 0);
      await page.request.delete(`/admin/devices/pats/${pat.id}`);
    }

    await page.goto('/admin/governance?preset=24h');
    await expect(page.locator('main')).toContainText('Governance');
    await expect(page.locator('main')).toContainText('secret_scan');
    await expect(page.locator('main')).toContainText('quota');

    const csv = await page.request.get('/admin/governance/warnings.csv?preset=24h');
    expect(csv.status()).toBe(200);
    expect(await csv.text()).toContain('secret_scan');
  });
});
