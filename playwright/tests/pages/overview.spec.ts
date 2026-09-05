// The console landing page: /admin.
//
// The overview is the only page every console user meets, so its four blocks
// assert on what it claims rather than on that it drew: the KPI strip's six
// figures, a liveness state per configured MCP server, and two spend boards
// that include the unattributed bucket rather than dropping it.
import { test, expect, snapshot, type Principal } from '../support/fixtures';
import { expectAccessible } from '../support/a11y';
import { PATHS } from '../support/paths';
import { SEL, TOKENS } from '../support/pages/selectors';
import { OverviewPage } from '../support/pages/overview.page';

const PATH = PATHS.overview;

// The six figures the strip carries, in the order it renders them.
const KPIS = ['Requests', 'Spend', 'Active people', 'p95 latency', 'Error rate', 'Denied'];

// Every principal the suite mints, and the status /admin returns to each.
// These are the literals in tests/contract/admin/baseline.txt: 200 renders,
// 303 bounces a non-console user to their profile, 307 sends anonymous to the
// login page carrying the target.
const ACCESS: { principal: Principal | 'anon'; status: number }[] = [
  { principal: 'platformAdmin', status: 200 },
  { principal: 'admin', status: 200 },
  { principal: 'pm', status: 200 },
  { principal: 'developer', status: 303 },
  { principal: 'businessUser', status: 303 },
  { principal: 'unassigned', status: 303 },
  { principal: 'user', status: 303 },
  { principal: 'anon', status: 307 },
];

test.describe('renders', () => {
  test('carries the six headline figures, each with a delta', async ({ adminPage }) => {
    const page = new OverviewPage(adminPage);
    await page.goto();
    const labels = await page.kpiLabels();
    for (const kpi of KPIS) expect(labels).toContain(kpi);
    expect(await adminPage.locator('.sp-delta').count()).toBeGreaterThanOrEqual(KPIS.length);
  });

  test('gives every configured MCP server a liveness state', async ({ adminPage }) => {
    const page = new OverviewPage(adminPage);
    await page.goto();
    const rows = page.rowsIn('MCP servers');
    expect(await rows.count()).toBeGreaterThan(0);
    // The seed heartbeats `systemprompt` within the hour, so it answers.
    await expect(page.mcpState('systemprompt')).toHaveText(/alive|stale/i);
  });

  test('ranks projects and groups by spend and keeps the unattributed bucket', async ({
    adminPage,
  }) => {
    const page = new OverviewPage(adminPage);
    await page.goto({ preset: '30d' });
    for (const board of ['Top projects by spend', 'Top groups by spend']) {
      const rows = page.rowsIn(board);
      expect(await rows.count(), board).toBeGreaterThan(0);
      expect(await rows.count(), board).toBeLessThanOrEqual(5);
    }
    await expect(page.section('Top groups by spend')).toContainText(/Unattributed|Unassigned/);
  });

  test('shows twelve decision-relevant facts above the fold', async ({ adminPage }) => {
    await adminPage.setViewportSize({ width: 1440, height: 900 });
    const page = new OverviewPage(adminPage);
    await page.goto();
    const visible = await adminPage.evaluate(() => {
      const inFold = (el: Element) => {
        const r = el.getBoundingClientRect();
        return r.top < 900 && r.bottom > 0;
      };
      const tiles = [...document.querySelectorAll('.sp-kpi')].filter(inFold).length;
      const rows = [...document.querySelectorAll('.sp-table__el tbody tr')].filter(inFold).length;
      return tiles + rows;
    });
    expect(visible).toBeGreaterThanOrEqual(12);
  });
});

test.describe('actions', () => {
  test('the range control moves the window and marks itself current', async ({ adminPage }) => {
    const page = new OverviewPage(adminPage);
    await page.goto();
    await page.rangeLink('7d').click();
    await expect(adminPage).toHaveURL(/preset=7d/);
    await expect(page.activeRange()).toHaveText('7d');
    await expect(page.heading()).toBeVisible();
  });

  test('the denied tile opens the request log filtered to the rows it counted', async ({
    adminPage,
  }) => {
    const page = new OverviewPage(adminPage);
    await page.goto();
    await adminPage.locator(SEL.kpi).filter({ hasText: 'Denied' }).first().click();
    await expect(adminPage).toHaveURL(/\/admin\/requests\?status=rejected/);
  });

  test('a non-console user is sent to their profile rather than refused', async ({ userPage }) => {
    await userPage.goto(PATH);
    await expect(userPage).toHaveURL(/\/admin\/profile/);
  });
});

test.describe('authorization', () => {
  for (const { principal, status } of ACCESS) {
    test(`${principal} gets ${status}`, async ({ browser }) => {
      const context = await browser.newContext(
        principal === 'anon' ? {} : { storageState: `.auth/${principal}.json` },
      );
      const res = await context.request.get(PATH, { maxRedirects: 0 });
      expect(res.status()).toBe(status);
      await context.close();
    });
  }
});

test.describe('design language', () => {
  test('wears the shell and marks Overview as the current page', async ({ adminPage }) => {
    const page = new OverviewPage(adminPage);
    await page.goto();
    await expect(page.heading()).toHaveText('Overview');
    await expect(page.activeNavItem()).toHaveAttribute('href', PATH);
  });

  test('paints from design tokens', async ({ adminPage }) => {
    const page = new OverviewPage(adminPage);
    await page.goto();
    const values = await adminPage.evaluate((tokens) => {
      const style = getComputedStyle(document.documentElement);
      return tokens.map((t) => style.getPropertyValue(t).trim());
    }, Object.values(TOKENS));
    for (const value of values) expect(value).not.toBe('');
  });

  test('every section states a title', async ({ adminPage }) => {
    const page = new OverviewPage(adminPage);
    await page.goto();
    const titles = await adminPage.locator('.sp-section__title').allInnerTexts();
    expect(titles.length).toBeGreaterThanOrEqual(4);
    for (const title of titles) expect(title.trim()).not.toBe('');
  });

  test('keeps a visible focus ring', async ({ adminPage }) => {
    const page = new OverviewPage(adminPage);
    await page.goto();
    await adminPage.keyboard.press('Tab');
    const ring = await adminPage.evaluate(() => {
      const el = document.activeElement;
      if (!el) return null;
      const s = getComputedStyle(el);
      return { style: s.outlineStyle, width: s.outlineWidth };
    });
    expect(ring).not.toBeNull();
    expect(ring?.style).not.toBe('none');
    expect(Number.parseFloat(ring?.width ?? '0')).toBeGreaterThan(0);
  });

  test('does not scroll horizontally at 1440 or 1024', async ({ adminPage }) => {
    const page = new OverviewPage(adminPage);
    for (const width of [1440, 1024]) {
      await adminPage.setViewportSize({ width, height: 900 });
      await page.goto();
      expect(await page.hasHorizontalScroll(), `horizontal scroll at ${width}px`).toBe(false);
    }
  });

  test('matches the committed visual baseline', async ({ adminPage }) => {
    const page = new OverviewPage(adminPage);
    await adminPage.setViewportSize({ width: 1440, height: 900 });
    await page.goto();
    await snapshot(adminPage, 'overview');
  });

  test('has no serious or critical axe violations', async ({ adminPage }) => {
    const page = new OverviewPage(adminPage);
    await page.goto();
    await expect(adminPage.locator(SEL.pageHeader)).toBeVisible();
    await expectAccessible(adminPage);
  });
});
