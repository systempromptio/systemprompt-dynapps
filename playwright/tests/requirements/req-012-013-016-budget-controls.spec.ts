// REQ-012 (soft budget alerts), REQ-013 (hard budget cutoffs), REQ-016
// (scheduled cost digests).
//
// Acceptance criteria: REQ-012 "a configurable soft threshold can be set for
// an agreed scope and triggers an alert without blocking production traffic";
// REQ-013 "a configurable hard threshold … blocks further eligible AI
// consumption when the limit is reached"; REQ-016 "recurring cost/budget
// digests can be scheduled and delivered automatically".
//
// Enforcement proof for all three lives in the Rust tier
// (tests/integration/gateway): the 429 at the hard cap, the non-blocking
// Slack alert on the soft crossing, and the digest job's Slack delivery
// cannot be driven honestly from a browser session. What the browser proves:
// the seeded plan's soft ($30) and hard ($50) thresholds are the ones the
// admin surface displays — "the same numbers the gateway enforces", as the
// page itself puts it — and the digest job is registered on the scheduler
// rather than being a run-it-by-hand script.
import { test, expect, apiAs } from '../support/fixtures';

test.describe('@REQ-012 @REQ-013 soft and hard thresholds surfaced together', () => {
  // The configurable soft/hard thresholds were attributes of an organization
  // PLAN, and the plan/seat/organization spine was removed in migration 041.
  // Nothing in the admin UI now displays a budget ceiling, so the browser-side
  // proof that "the display and the enforced number agree" has no surface to
  // stand on. The enforcement itself (429 at the hard cap, non-blocking alert
  // at the soft one) remains covered in tests/integration/gateway.
  //
  // Reinstate as soon as a group- or project-scoped budget lands: the shape of
  // these assertions (a meter carrying the cap, a distinct soft tick, a
  // warnings caption naming the threshold kind) is what to rebuild them from.
  test.fixme('@REQ-013 the seeded hard cap renders as the meter ceiling', async () => {});

  test.fixme('@REQ-012 the soft threshold is drawn as its own tick, distinct from the cap', async () => {});

  test.fixme('@REQ-012 soft-cap crossings are recorded with their threshold kind', async () => {});
});

test.describe('@REQ-016 scheduled cost digest', () => {
  test('@REQ-016 the cost_digest job is registered on the scheduler', async ({
    request,
  }) => {
    // Delivery is Slack-side (Rust tier). Scheduling is the browser-provable
    // half: the job exists as a scheduler entry, so the digest recurs without
    // anyone remembering to run it — the requirement's whole point.
    const res = await request.get('/api/public/admin/jobs', apiAs(request, 'admin'));
    expect(res.status()).toBe(200);
    expect(JSON.stringify(await res.json())).toContain('cost_digest');
  });

  test('@REQ-016 the scheduler surface refuses a non-admin', async ({ request }) => {
    const res = await request.get('/api/public/admin/jobs', apiAs(request, 'user'));
    expect(res.status()).toBe(403);
  });
});
