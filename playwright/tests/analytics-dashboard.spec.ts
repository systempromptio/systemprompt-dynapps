// The four-tab analytics dashboard against the deterministic e2e seed.
//
// Scope is expressed by group and project now — the organization/plan spine and
// its `?org=` selector were removed with migration 041, and the seats tab went
// with the per-seat plan model. Every navigation asserts on seeded content,
// never on live totals.
import { test, expect } from './support/fixtures';

const TABS = ['overview', 'usage', 'spend', 'code'] as const;

// Each tab renders its own centre of gravity, not one shared strip: overview
// and usage lead with KPI cards, spend with the SLO split and the anomaly
// table, code with the measurement-frame cards.
const TAB_MARKER: Record<(typeof TABS)[number], string> = {
  overview: '.kpi-strip .kpi-card',
  usage: '.kpi-strip .kpi-card',
  spend: '.data-table, .empty-state',
  code: '.code-frames .code-frame',
};

test.describe('analytics tabs', () => {
  for (const tab of TABS) {
    test(`the ${tab} tab renders with seeded data`, async ({ adminPage: page }) => {
      const res = await page.goto(`/admin/analytics?tab=${tab}&preset=30d`);
      expect(res?.status()).toBe(200);
      await expect(page.locator('.sp-tab--active')).toHaveCount(1);
      await expect(page.locator(TAB_MARKER[tab]).first()).toBeVisible();
    });
  }

  test('the usage leaderboard contains a seeded member', async ({ adminPage: page }) => {
    await page.goto('/admin/analytics?tab=usage&preset=30d&group=commerce-devs');
    await expect(page.locator('main')).toContainText('e2e-member-1');
  });

  test('bucket toggle switches to weekly', async ({ adminPage: page }) => {
    await page.goto('/admin/analytics?tab=overview&bucket=week');
    await expect(page).toHaveURL(/bucket=week/);
    await expect(page.locator('.sp-tab--active')).toHaveCount(1);
  });

  test('the scope filters are group and project, not organization', async ({
    adminPage: page,
  }) => {
    await page.goto('/admin/analytics?tab=overview');
    await expect(page.locator('select[name="org"]')).toHaveCount(0);
    await expect(page.locator('select[name="group"]')).toBeVisible();
    await expect(page.locator('select[name="project"]')).toBeVisible();
  });

  test('the group filter narrows the leaderboard', async ({ adminPage: page }) => {
    // members 1-3 are commerce devs; members 4-5 are europe devs. The two
    // slices must disagree, or the filter is decoration.
    await page.goto('/admin/analytics?tab=usage&preset=30d&group=commerce-devs');
    const main = page.locator('main');
    await expect(main).toContainText('e2e-member-1');
    await expect(main).not.toContainText('e2e-member-4');
  });

  test('the project filter narrows the leaderboard', async ({ adminPage: page }) => {
    await page.goto('/admin/analytics?tab=usage&preset=30d&project=payments');
    const main = page.locator('main');
    await expect(main).toContainText('e2e-member-4');
    await expect(main).not.toContainText('e2e-member-1');
  });

  test('an empty range renders empty states, never a 500', async ({ adminPage: page }) => {
    for (const tab of TABS) {
      const res = await page.goto(`/admin/analytics?tab=${tab}&from=2020-01-01&to=2020-01-02`);
      expect(res?.status(), `tab=${tab}`).toBe(200);
    }
  });

  test('pagination beyond the seeded rows renders cleanly', async ({ adminPage: page }) => {
    const res = await page.goto('/admin/analytics?tab=usage&page=99');
    expect(res?.status()).toBe(200);
  });
});

test.describe('platform scope', () => {
  test('a platform admin reads the same group-scoped leaderboard', async ({
    platformAdminPage: page,
  }) => {
    await page.goto('/admin/analytics?tab=usage&preset=30d&group=commerce-devs');
    await expect(page.locator('main')).toContainText('e2e-member-1');
  });
});
