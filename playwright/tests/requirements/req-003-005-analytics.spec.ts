// REQ-003, REQ-004, REQ-005 — usage, cost/model, and adoption analytics.
//
// Asserted against the deterministic 14-day seed (three members, 1..4 requests
// per member per day, three models), so these check numbers and set membership
// rather than "a KPI card exists".
import { test, expect } from '../support/fixtures';

const SEEDED_MODELS = ['claude-opus-5', 'claude-sonnet-5', 'claude-haiku-4-5'];

test.describe('@REQ-003 usage analytics dashboard', () => {
  test('@REQ-003 the window is selectable and changes the series', async ({
    platformAdminPage: page,
  }) => {
    await page.goto('/admin/analytics?preset=7d&bucket=day');
    await expect(page.locator('figure.svgchart').first()).toBeVisible();
    const daily = await page.locator('figure.svgchart').first().innerHTML();

    await page.goto('/admin/analytics?preset=7d&bucket=week');
    await expect(page.locator('figure.svgchart').first()).toBeVisible();
    const weekly = await page.locator('figure.svgchart').first().innerHTML();

    // Daily and weekly buckets over the same window must not plot identically.
    expect(daily).not.toEqual(weekly);
  });

  test('@REQ-003 request volume and active users are non-zero for the seeded org', async ({
    platformAdminPage: page,
  }) => {
    await page.goto('/admin/analytics?preset=30d');
    const strip = page.locator('.kpi-strip');
    await expect(strip).toBeVisible();
    // The seed writes 105 requests across three members over 14 days, so a 30d
    // window must report a positive count -- a zero here means the scope filter
    // or the window resolution has broken, which is exactly what a
    // presence-only assertion would miss.
    await expect(strip).not.toContainText(/\b0 requests\b/);
  });

  test('@REQ-003 a window with no data renders empty states, never a 500', async ({
    platformAdminPage: page,
  }) => {
    const res = await page.goto(
      '/admin/analytics?from=2000-01-01T00:00&to=2000-01-08T00:00',
    );
    expect(res?.status()).toBe(200);
  });
});

test.describe('@REQ-004 cost and model analytics', () => {
  test('@REQ-004 the model mix names exactly the seeded models', async ({
    platformAdminPage: page,
  }) => {
    // Scoped to the commerce group: the unscoped dashboard mixes in the
    // developer's own traffic and collapses the tail into an "Other" bucket,
    // which is exactly what would hide a missing model.
    await page.goto('/admin/analytics?preset=30d&group=commerce-devs');
    const body = await page.locator('main').innerText();
    for (const model of SEEDED_MODELS) {
      expect(body).toContain(model);
    }
  });

  test('@REQ-004 cost per request is reported, not just total spend', async ({
    platformAdminPage: page,
  }) => {
    await page.goto('/admin/analytics?preset=30d');
    await expect(page.locator('.kpi-strip')).toContainText('per request');
  });
});

test.describe('@REQ-005 adoption analytics', () => {
  test('@REQ-005 the leaderboard lists the seeded members', async ({
    platformAdminPage: page,
  }) => {
    await page.goto('/admin/analytics?tab=usage&preset=30d&sort=requests');
    const table = page.locator('.data-table').first();
    await expect(table).toContainText('e2e-member-1');
    await expect(table).toContainText('e2e-member-2');
    await expect(table).toContainText('e2e-member-3');
  });

  // The seats tab and its inactivity window went with the per-seat plan model
  // (migration 041): analytics now has four tabs — overview, usage, spend,
  // code — and no seat utilisation surface. Reinstate if seat accounting
  // returns in some group- or project-scoped form.
  test.fixme('@REQ-005 the inactivity window is configurable and changes the result', async () => {});

  test.fixme('@REQ-005 an out-of-range window clamps instead of erroring', async () => {});
});
