// The console landing page: /admin.
//
// The overview is the only page every console user meets, so its four blocks
// assert on what it claims rather than on that it drew: the KPI strip's six
// figures, a liveness state per configured MCP server, and two spend boards
// that include the unattributed bucket rather than dropping it.
import { test, expect, AUTH, type Principal } from '../support/fixtures';
import { expectAccessible, expectDensity } from '../support/a11y';
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
    // The design system uppercases KPI labels in CSS, and innerText reports
    // the rendered casing, so the comparison is case-insensitive.
    const labels = (await page.kpiLabels()).map((l) => l.toUpperCase());
    for (const kpi of KPIS) expect(labels).toContain(kpi.toUpperCase());
    // Every tile states what the previous window held. Whether it also states a
    // percentage depends on whether that window carried enough traffic to
    // compare against, so the assertion is on the comparison, not the arrow.
    const subs = await adminPage.locator(`${SEL.kpiGrid} ${SEL.kpiSub}`).allInnerTexts();
    const comparisons = subs.filter((t) => /vs previous/.test(t));
    expect(comparisons.length).toBeGreaterThanOrEqual(KPIS.length);
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
        principal === 'anon' ? {} : { storageState: AUTH[principal] },
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

  // Tabbed to rather than focused programmatically: the ring is a
  // :focus-visible indicator, and .focus() does not always satisfy that
  // pseudo-class. The shipped ring is a two-stop box-shadow, not the outline
  // in the reset — several components replace the outline with it — so this
  // reads the shadow.
  //
  // It walks to the window control rather than to a KPI tile on purpose: a
  // linked KPI tile currently shows NO focus indicator, because .sp-kpi--link
  // sets box-shadow for its card and that wins over the shared anchor focus
  // rule. That is a design-system defect in 05-kpi.css, reported to the design
  // system's owner; asserting the ring here would either fail or quietly
  // record the defect as correct.
  test('keeps a visible focus ring on its window control', async ({ adminPage }) => {
    const page = new OverviewPage(adminPage);
    await page.goto();
    let ring: { onRange: boolean; shadow: string } | null = null;
    for (let i = 0; i < 60; i += 1) {
      await adminPage.keyboard.press('Tab');
      ring = await adminPage.evaluate(() => {
        const el = document.activeElement;
        if (!el) return null;
        return {
          onRange: el.classList.contains('sp-p-overview__range'),
          shadow: getComputedStyle(el).boxShadow,
        };
      });
      if (ring?.onRange) break;
    }
    expect(ring?.onRange, 'never reached the window control by tabbing').toBe(true);
    expect(ring?.shadow, 'the focused control paints no ring').not.toBe('none');
    expect(ring?.shadow).toMatch(/0px 0px 0px 4px/);
  });

  // The measured density bar: rows above the fold, row and KPI heights,
  // clipped or overlapping cells, sideways scroll, and the shell's own
  // furniture. The overview is a list-shaped page — six tiles and three
  // tables — so it is held to the list bar.
  test('meets the measured density bar', async ({ adminPage }) => {
    await adminPage.setViewportSize({ width: 1440, height: 900 });
    const page = new OverviewPage(adminPage);
    await page.goto();
    await expectDensity(adminPage, 'list');
  });

  test('does not scroll horizontally at 1440 or 1024', async ({ adminPage }) => {
    const page = new OverviewPage(adminPage);
    for (const width of [1440, 1024]) {
      await adminPage.setViewportSize({ width, height: 900 });
      await page.goto();
      expect(await page.hasHorizontalScroll(), `horizontal scroll at ${width}px`).toBe(false);
    }
  });

  // No pixel baseline. The overview is built out of live relative times — a
  // heartbeat age, an anomaly window — and out of text whose wrapping changes
  // with them, so the page's height moves between two runs minutes apart and a
  // full-page comparison reports the whole lower half as changed. The
  // committed screenshot under pages/__screenshots__/ is kept as the visual
  // record of the page; it is not a gate, because a gate that fails on the
  // clock teaches everyone to re-record it without looking.

  test('has no serious or critical axe violations', async ({ adminPage }) => {
    const page = new OverviewPage(adminPage);
    await page.goto();
    await expect(adminPage.locator(SEL.pageHeader)).toBeVisible();
    await expectAccessible(adminPage);
  });
});
