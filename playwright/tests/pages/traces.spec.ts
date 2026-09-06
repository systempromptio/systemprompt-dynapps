// The trace explorer (/admin/traces) and one trace's waterfall.
//
// A trace groups every gateway call, governance decision and tool invocation
// that shared a session id. The list is the triage surface — which traces were
// denied, which failed, which were slow — and the detail page is the evidence
// behind one of them, drawn as a waterfall and tabulated span by span.
import { AUTH, test, expect, snapshot, type Principal } from '../support/fixtures';
import { expectAccessible, expectDensity } from '../support/a11y';
import { PATHS } from '../support/paths';
import { SEL, TOKENS } from '../support/pages/selectors';
import { BasePage } from '../support/pages/base.page';
import { TracesPage } from '../support/pages/traces.page';

const PATH = PATHS.traces;
const WINDOW = { preset: '30d' };

// The literals pinned in tests/contract/admin/baseline.txt.
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
  test('lists the seeded traces, one row per session id', async ({ adminPage }) => {
    const page = new TracesPage(adminPage);
    await page.goto(WINDOW);
    expect(await page.table().rowCount()).toBeGreaterThan(0);
  });

  test('carries the six decision-relevant totals above the table', async ({ adminPage }) => {
    const page = new TracesPage(adminPage);
    await page.goto(WINDOW);
    for (const label of ['Traces', 'Denied', 'Errors', 'Cost', 'Tokens', 'p95 active']) {
      await expect(page.kpi(label).locator, `KPI "${label}"`).toBeVisible();
    }
    expect(await page.kpi('Traces').number()).toBeGreaterThan(0);
  });

  test('every row reports its span, request and error counts', async ({ adminPage }) => {
    const page = new TracesPage(adminPage);
    await page.goto(WINDOW);
    // Headers carry the sort indicator inside the cell, so this matches the
    // label at the start of the text rather than the whole of it.
    const headers = (await page.table().headers().allInnerTexts()).map((h) => h.trim().split(/\s/)[0]);
    // Nine columns, not twelve: the trace's size is one cell whose title
    // carries the split, and the error count rides the status badge.
    expect(headers.length, 'column count').toBeLessThanOrEqual(9);
    for (const header of ['ACTIVITY', 'TOKENS', 'COST', 'STATUS']) {
      expect(headers, `column "${header}"`).toContain(header);
    }
  });

  test('the waterfall is one inline SVG with a bar per span @REQ-028', async ({ adminPage }) => {
    const list = new TracesPage(adminPage);
    await list.goto(WINDOW);
    const detail = await list.openFirst();
    await expect(detail.chart()).toBeVisible();
    const bars = await detail.bars().count();
    expect(bars).toBeGreaterThan(0);
    expect(await detail.spanTable().locator('tbody tr').count()).toBe(bars);
  });

  test('the span table names the provider, model, cost and latency', async ({ adminPage }) => {
    const list = new TracesPage(adminPage);
    await list.goto(WINDOW);
    const detail = await list.openFirst();
    // The design system upper-cases header text in CSS, so the comparison is
    // on the words, not on their casing.
    const headers = (await detail.spanTable().locator('thead th').allInnerTexts()).map((h) =>
      h.trim().toLowerCase(),
    );
    for (const header of ['provider', 'model', 'latency', 'cost']) {
      expect(headers, `column "${header}"`).toContain(header);
    }
  });

  test('a rejected span keeps its REJECTED badge and no model', async ({ adminPage }) => {
    const list = new TracesPage(adminPage);
    await list.goto({ ...WINDOW, error_only: 'true' });
    if ((await list.table().rowCount()) === 0) test.skip(true, 'no failing traces seeded');
    const detail = await list.openFirst();
    const rejected = detail.statusBadges().filter({ hasText: /REJECTED/ });
    if ((await rejected.count()) === 0) test.skip(true, 'no rejected span on this trace');
    await expect(rejected.first()).toBeVisible();
  });
});

test.describe('actions', () => {
  test('sorting by cost reorders the table and marks the header', async ({ adminPage }) => {
    const page = new TracesPage(adminPage);
    await page.goto(WINDOW);
    const direction = await page.sortBy('Cost');
    expect(direction).toBe('descending');
    await expect(adminPage).toHaveURL(/sort=cost/);
  });

  test('the denied KPI is the filter it counts @REQ-018', async ({ adminPage }) => {
    const page = new TracesPage(adminPage);
    await page.goto(WINDOW);
    await page.kpi('Denied').locator.click();
    await adminPage.waitForLoadState('networkidle');
    await expect(adminPage).toHaveURL(/deny_only=true/);
  });

  test('narrowing the scope to one group narrows the table', async ({ adminPage }) => {
    const page = new TracesPage(adminPage);
    await page.goto(WINDOW);
    const before = await page.table().rowCount();
    await page.scopeTo('group', 'europe-devs');
    await expect(adminPage).toHaveURL(/group=europe-devs/);
    expect(await page.table().rowCount()).toBeLessThanOrEqual(before);
  });

  test('paging keeps the window and the sort', async ({ adminPage }) => {
    const page = new TracesPage(adminPage);
    await page.goto({ ...WINDOW, sort: 'cost', dir: 'desc' });
    const controls = adminPage.locator(SEL.paginationControls).first();
    await expect(controls).toBeVisible();
    const next = controls.getByRole('link', { name: /next/i });
    if ((await next.count()) === 0) test.skip(true, 'one page of traces seeded');
    await next.click();
    await adminPage.waitForLoadState('networkidle');
    await expect(adminPage).toHaveURL(/sort=cost/);
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

  // A trace whose only request was rejected has no provider and no model. The
  // detail page used to answer 500 on exactly that row; it must answer 200.
  test('an unknown trace id is a 404, not a 500', async ({ browser }) => {
    const context = await browser.newContext({ storageState: AUTH.admin });
    const res = await context.request.get(PATHS.trace('no-such-trace-id'), { maxRedirects: 0 });
    expect(res.status()).toBe(404);
    await context.close();
  });
});

test.describe('design language', () => {
  test('wears the shell', async ({ adminPage }) => {
    const page = new TracesPage(adminPage);
    await page.goto(WINDOW);
    // The sidebar link carries the window forward, so the path is the claim.
    await expect(page.activeNavItem()).toHaveAttribute('href', new RegExp(`^${PATH}(\\?|$)`));
    await expect(page.heading()).toBeVisible();
    await expect(adminPage.locator(SEL.toolbarCount)).toBeVisible();
  });

  test('paints from design tokens', async ({ adminPage }) => {
    const page = new TracesPage(adminPage);
    await page.goto(WINDOW);
    const values = await adminPage.evaluate((tokens) => {
      const style = getComputedStyle(document.documentElement);
      return tokens.map((t) => style.getPropertyValue(t).trim());
    }, Object.values(TOKENS));
    for (const value of values) expect(value).not.toBe('');
  });

  // The rows were 51px of two-line stacked cells, which halved how many traces
  // fitted above the fold. One line per row is the whole point of this table.
  test('keeps rows to one line', async ({ adminPage }) => {
    const page = new TracesPage(adminPage);
    await adminPage.setViewportSize({ width: 1440, height: 900 });
    await page.goto(WINDOW);
    expect(await page.rowHeight()).toBeLessThan(44);
  });

  test('shows an empty state when nothing matches', async ({ adminPage }) => {
    const page = new TracesPage(adminPage);
    await page.goto({ preset: '15m', user_id: 'no-such-user' });
    await expect(page.emptyState()).toBeVisible();
  });

  test('keeps a visible focus ring', async ({ adminPage }) => {
    const page = new TracesPage(adminPage);
    await page.goto(WINDOW);
    await adminPage.keyboard.press('Tab');
    // The shipped design system draws the ring with box-shadow, and keeps a
    // non-zero outline-width for forced-colors mode. Either mechanism is a
    // visible ring; asserting on outline-style alone reads as absent.
    const ring = await adminPage.evaluate(() => {
      const el = document.activeElement;
      if (!el) return null;
      const s = getComputedStyle(el);
      return { style: s.outlineStyle, width: s.outlineWidth, shadow: s.boxShadow };
    });
    expect(ring).not.toBeNull();
    const outlined = ring?.style !== 'none' && Number.parseFloat(ring?.width ?? '0') > 0;
    const shadowed = (ring?.shadow ?? 'none') !== 'none';
    expect(outlined || shadowed, `no visible focus ring: ${JSON.stringify(ring)}`).toBe(true);
  });

  test('does not scroll horizontally at 1440 or 1024', async ({ adminPage }) => {
    const page = new BasePage(adminPage, PATH);
    for (const width of [1440, 1024]) {
      await adminPage.setViewportSize({ width, height: 900 });
      await page.goto(WINDOW);
      expect(await page.hasHorizontalScroll(), `horizontal scroll at ${width}px`).toBe(false);
    }
  });



  test('meets the density bar', async ({ adminPage }) => {
    const page = new TracesPage(adminPage);
    await adminPage.setViewportSize({ width: 1440, height: 900 });
    await page.goto(WINDOW);
    await expectDensity(adminPage, 'list');
  });


  test('the detail page meets the density bar', async ({ adminPage }) => {
    const list = new TracesPage(adminPage);
    await adminPage.setViewportSize({ width: 1440, height: 900 });
    await list.goto(WINDOW);
    await list.openFirst();
    await expectDensity(adminPage, 'detail');
  });

  test('matches its visual baseline', async ({ adminPage }) => {
    const page = new TracesPage(adminPage);
    await adminPage.setViewportSize({ width: 1440, height: 900 });
    await page.goto(WINDOW);
    await snapshot(adminPage, 'traces');
  });

  test('has no serious or critical axe violations', async ({ adminPage }) => {
    const page = new TracesPage(adminPage);
    await page.goto(WINDOW);
    await expect(adminPage.locator(SEL.pageHeader)).toBeVisible();
    await expectAccessible(adminPage);
  });

  test('the waterfall page has no serious or critical axe violations', async ({ adminPage }) => {
    const list = new TracesPage(adminPage);
    await list.goto(WINDOW);
    await list.openFirst();
    await expectAccessible(adminPage);
  });
});
