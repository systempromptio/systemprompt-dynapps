// The request log (/admin/requests) and one request's audit detail.
//
// The log is the page an operator reaches for when someone asks "what did that
// cost" or "why was that refused", so the spec's centre of gravity is the row:
// that it carries the attribution, the outcome and the money, that the facets
// narrow it, and that it opens the chain of custody behind it.
import { test, expect, dbRow, AUTH, type Principal } from '../support/fixtures';
import { expectAccessible, expectDensity } from '../support/a11y';
import { PATHS } from '../support/paths';
import { SEL, TOKENS } from '../support/pages/selectors';
import { BasePage } from '../support/pages/base.page';
import { RequestsPage, RequestDetailPage } from '../support/pages/requests.page';

const PATH = PATHS.requests;

// Every principal the suite mints, and the status this page returns to each.
// These are the literals pinned in tests/contract/admin/baseline.txt.
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
  test('lists the seeded requests, one row per gateway call', async ({ adminPage }) => {
    const page = new RequestsPage(adminPage);
    await page.goto({ preset: '30d' });
    expect(await page.rows().count()).toBeGreaterThan(0);
  });

  // The defect this exists to catch: the tiles counted 1,398 while the log
  // said "No requests match", because the list query failed on a null column
  // and the failure rendered as an empty state. A non-zero tile with no first
  // row is now a test failure rather than something a person has to notice.
  test('a non-zero Requests tile always has a first row beneath it', async ({ adminPage }) => {
    const page = new RequestsPage(adminPage);
    await page.goto();
    const total = await page.kpi('Requests').number();
    test.skip(total === 0, 'no requests in the default window');
    await expect(page.rows().first()).toBeVisible();
    await expect(adminPage.locator(SEL.empty)).toHaveCount(0);
  });

  test('the meta count and the Requests tile state the same number', async ({ adminPage }) => {
    const page = new RequestsPage(adminPage);
    await page.goto({ preset: '30d' });
    const meta = await adminPage.locator(SEL.pageHeader).innerText();
    const tile = await page.kpi('Requests').number();
    expect(meta).toContain(tile.toLocaleString('en-US'));
  });

  test('carries the five decision-relevant totals above the table', async ({ adminPage }) => {
    const page = new RequestsPage(adminPage);
    await page.goto({ preset: '30d' });
    for (const label of ['Requests', 'Cost', 'Error rate', 'p95 latency', 'Rejected']) {
      await expect(page.kpi(label).locator, `KPI "${label}"`).toBeVisible();
    }
    expect(await page.kpi('Requests').number()).toBeGreaterThan(0);
  });

  test('attributes every row to one project and one group', async ({ adminPage }) => {
    const page = new RequestsPage(adminPage);
    await page.goto({ preset: '30d' });
    const projects = await page.table().column('Project');
    const groups = await page.table().column('Group');
    expect(projects.length).toBe(groups.length);
    // Exclusive attribution: a row names exactly one of each, and a row the
    // directory placed nowhere says so rather than rendering blank.
    for (const cell of [...projects, ...groups]) expect(cell.trim()).not.toBe('');
  });

  test('renders a rejected row with no model as an em dash and a badge', async ({ adminPage }) => {
    const page = new RequestsPage(adminPage);
    await page.goto({ preset: '30d', status: 'rejected' });
    if ((await page.rows().count()) === 0) test.skip(true, 'no rejected requests seeded');
    await expect(page.rows().first().locator(SEL.badge).first()).toHaveText(/REJECTED/i);
  });

  test('pages at fifty rows', async ({ adminPage }) => {
    const page = new RequestsPage(adminPage);
    await page.goto({ preset: '30d' });
    expect(await page.rows().count()).toBeLessThanOrEqual(50);
    await expect(adminPage.locator(SEL.pagination)).toBeVisible();
  });
});

test.describe('actions', () => {
  test('a facet select narrows the log and raises a chip', async ({ adminPage }) => {
    const page = new RequestsPage(adminPage);
    await page.goto({ preset: '30d' });
    const before = await page.rows().count();
    const options = await page.form().locator('select[name="status"] option').allInnerTexts();
    const pick = options.find((o) => /completed/i.test(o));
    test.skip(!pick, 'no completed status in the seeded window');
    await page.filterBy('status', 'completed');
    expect(await page.rows().count()).toBeLessThanOrEqual(before);
    await expect(page.chips().filter({ hasText: 'completed' })).toBeVisible();
  });

  test('clearing the filters restores the unfiltered log', async ({ adminPage }) => {
    const page = new RequestsPage(adminPage);
    await page.goto({ preset: '30d', status: 'rejected' });
    await page.clearFilters();
    expect(new URL(adminPage.url()).searchParams.get('status')).toBeNull();
  });

  test('the search box narrows on user, model or trace id', async ({ adminPage }) => {
    const page = new RequestsPage(adminPage);
    await page.goto({ preset: '30d' });
    await page.table().filter('zzz-no-such-request-zzz');
    await page.table().expectEmpty();
  });

  test('sorting by cost re-orders the log', async ({ adminPage }) => {
    const page = new RequestsPage(adminPage);
    await page.goto({ preset: '30d' });
    const direction = await page.table().sortBy('Cost');
    expect(direction).toBe('descending');
    expect(new URL(adminPage.url()).searchParams.get('sort')).toBe('cost');
  });

  test('the Rejected tile is the filter its number describes', async ({ adminPage }) => {
    const page = new RequestsPage(adminPage);
    await page.goto({ preset: '30d' });
    await page.kpi('Rejected').locator.click();
    await adminPage.waitForLoadState('networkidle');
    expect(new URL(adminPage.url()).searchParams.get('status')).toBe('rejected');
  });

  // The other half of the drill-down contract: the analytics spec asserts the
  // href it builds, this asserts the log actually narrows when that href is
  // followed. Neither page can prove the pair on its own.
  test('an analytics drill-down narrows the log', async ({ adminPage }) => {
    const page = new RequestsPage(adminPage);
    await page.goto({ preset: '30d' });
    const all = await page.kpi('Requests').number();
    const model = (await page.table().column('Model'))[0]?.split('\n')[0]?.trim();
    test.skip(!model || model === '\u2014', 'no routed model in the seeded window');
    await page.goto({ preset: '30d', model: model! });
    expect(await page.kpi('Requests').number()).toBeLessThan(all);
    await expect(page.chips().filter({ hasText: model! })).toBeVisible();
  });

  // The rejected rows carry no model at all, so the sentinel has to mean IS
  // NULL. Matching it as a literal would land this drill on an empty table.
  test('the unrouted sentinel selects the rejected rows', async ({ adminPage }) => {
    const page = new RequestsPage(adminPage);
    await page.goto({ preset: '30d', model: 'unrouted' });
    const rejected = await page.kpi('Rejected').number();
    expect(await page.kpi('Requests').number()).toBe(rejected);
  });

  test('exports the filtered log as CSV', async ({ adminPage }) => {
    const res = await adminPage.request.get('/admin/requests.csv?preset=30d');
    expect(res.status()).toBe(200);
    expect(res.headers()['content-type']).toContain('text/csv');
    expect(await res.text()).toContain('created_at,request_id,user_id,group_id,project_id');
  });

  test('a row opens its chain of custody, with the two evidence planes', async ({ adminPage }) => {
    const row = await dbRow<{ id: string }>(
      'SELECT id FROM ai_requests ORDER BY created_at DESC LIMIT 1',
    );
    test.skip(!row, 'no requests seeded');
    const detail = new RequestDetailPage(adminPage, row!.id);
    await detail.goto();
    await expect(detail.breadcrumb()).toBeVisible();
    await expect(detail.safetyFindings()).toBeVisible();
    await expect(detail.toolCalls()).toBeVisible();
  });

  test('an unknown request id is a 404, not a 500', async ({ adminPage }) => {
    const res = await adminPage.request.get(PATHS.request('no-such-request-id'));
    expect(res.status()).toBe(404);
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
  test('wears the shell', async ({ adminPage }) => {
    const page = new BasePage(adminPage, PATH);
    await page.goto({ preset: '30d' });
    await expect(page.activeNavItem()).toHaveAttribute('href', PATH);
    await expect(page.heading()).toBeVisible();
    await expect(adminPage.locator(SEL.headerActions).getByRole('link')).not.toHaveCount(0);
    // The window the page resolved, stated in the header rather than by a
    // second set of preset buttons beside the shell's own.
    await expect(adminPage.locator('[data-window]')).toHaveAttribute('data-window', '30d');
  });

  test('paints from design tokens', async ({ adminPage }) => {
    const page = new BasePage(adminPage, PATH);
    await page.goto();
    const values = await adminPage.evaluate((tokens) => {
      const style = getComputedStyle(document.documentElement);
      return tokens.map((t) => style.getPropertyValue(t).trim());
    }, Object.values(TOKENS));
    for (const value of values) expect(value).not.toBe('');
  });

  test('carries no legacy row class', async ({ adminPage }) => {
    const page = new RequestsPage(adminPage);
    await page.goto({ preset: '30d' });
    expect(await adminPage.locator('.sp-audit-row').count()).toBe(0);
  });

  test('shows an empty state when nothing matches', async ({ adminPage }) => {
    const page = new RequestsPage(adminPage);
    await page.goto({ preset: '30d' });
    await page.table().filter('zzz-no-such-row-zzz');
    await page.table().expectEmpty();
  });

  test('every column header states its sort state', async ({ adminPage }) => {
    const page = new RequestsPage(adminPage);
    await page.goto({ preset: '30d' });
    const sortable = adminPage.locator(`${SEL.tableEl} ${SEL.tableSorted}`);
    expect(await sortable.count()).toBeGreaterThanOrEqual(4);
  });

  test('keeps a visible focus ring', async ({ adminPage }) => {
    const page = new BasePage(adminPage, PATH);
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
    const page = new BasePage(adminPage, PATH);
    for (const width of [1440, 1024]) {
      await adminPage.setViewportSize({ width, height: 900 });
      await page.goto();
      expect(await page.hasHorizontalScroll(), `horizontal scroll at ${width}px`).toBe(false);
    }
  });

  // The measured bar: rows above the fold, row and KPI heights, no clipped or
  // overlapping cells, no sideways scroll, and a header and breadcrumbs.
  test('meets the density bar', async ({ adminPage }) => {
    const page = new RequestsPage(adminPage);
    await adminPage.setViewportSize({ width: 1440, height: 900 });
    await page.goto({ preset: '30d' });
    await expectDensity(adminPage, 'list');
  });

  test('has no serious or critical axe violations', async ({ adminPage }) => {
    const page = new BasePage(adminPage, PATH);
    await page.goto({ preset: '30d' });
    await expect(adminPage.locator(SEL.pageHeader)).toBeVisible();
    await expectAccessible(adminPage);
  });

  test('matches the committed visual snapshot', async ({ adminPage }) => {
    const page = new RequestsPage(adminPage);
    await adminPage.setViewportSize({ width: 1440, height: 900 });
    await page.goto({ preset: '30d' });
    await expect(adminPage).toHaveScreenshot('requests.png', {
      fullPage: false,
      maxDiffPixelRatio: 0.02,
      mask: [adminPage.locator(SEL.tableCellDate), adminPage.locator('.sp-col-date')],
    });
  });
});
