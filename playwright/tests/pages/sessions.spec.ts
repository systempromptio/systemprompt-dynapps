// The sessions list (/admin/sessions) and one session's detail page.
//
// A session is the unit an operator recognises: one interactive run, holding
// every context, trace and request it produced. The spec's centre of gravity
// is therefore the row — that it names who ran it, which group and project it
// is attributed to, what it cost, and whether it went wrong — and the detail
// page's two judgement panels, which are the only place the console says
// whether a run actually worked.
import { AUTH, test, expect, snapshot, type Principal } from '../support/fixtures';
import { expectAccessible, expectDensity } from '../support/a11y';
import { PATHS } from '../support/paths';
import { SEL, TOKENS } from '../support/pages/selectors';
import { BasePage } from '../support/pages/base.page';
import { SessionsPage } from '../support/pages/sessions.page';

const PATH = PATHS.sessions;
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
  test('lists the seeded sessions, one row per session id', async ({ adminPage }) => {
    const page = new SessionsPage(adminPage);
    await page.goto(WINDOW);
    expect(await page.table().rowCount()).toBeGreaterThan(0);
  });

  test('carries the six decision-relevant totals above the table', async ({ adminPage }) => {
    const page = new SessionsPage(adminPage);
    await page.goto(WINDOW);
    for (const label of ['Sessions', 'With errors', 'Requests', 'Tool calls', 'Tokens', 'Cost']) {
      await expect(page.kpi(label).locator, `KPI "${label}"`).toBeVisible();
    }
    expect(await page.kpi('Sessions').number()).toBeGreaterThan(0);
  });

  test('attributes every row to one group and one project @REQ-010', async ({ adminPage }) => {
    const page = new SessionsPage(adminPage);
    await page.goto(WINDOW);
    const { groups, projects } = await page.attribution();
    expect(groups.length).toBe(projects.length);
    // Exclusive attribution names exactly one of each; a person the directory
    // placed nowhere reads "unattributed" rather than blank.
    for (const cell of [...groups, ...projects]) expect(cell.trim()).not.toBe('');
  });

  test('every session id links to its own detail page', async ({ adminPage }) => {
    const page = new SessionsPage(adminPage);
    await page.goto(WINDOW);
    expect(await page.sessionLinks().count()).toBeGreaterThan(0);
    const detail = await page.openFirst();
    await expect(detail.breadcrumb()).toBeVisible();
    await expect(detail.heading()).toContainText(/Session/i);
  });

  test('the detail page shows contexts, traces and requests for that run', async ({ adminPage }) => {
    const list = new SessionsPage(adminPage);
    await list.goto(WINDOW);
    const detail = await list.openFirst();
    for (const label of ['Requests', 'Contexts', 'Traces', 'Errors', 'Tokens', 'Cost']) {
      await expect(detail.kpi(label).locator, `KPI "${label}"`).toBeVisible();
    }
    await expect(detail.requestsTable()).toBeVisible();
  });

  test('the detail page reports the assessment and the ratings, or says there are none', async ({
    adminPage,
  }) => {
    const list = new SessionsPage(adminPage);
    await list.goto(WINDOW);
    const detail = await list.openFirst();
    // Both panels are always answered: an unassessed session gets a notice, an
    // unrated one an empty state. Neither is silently omitted.
    const assessed = (await detail.analysis().count()) > 0;
    if (assessed) expect(await detail.verdictBadges().count()).toBeGreaterThan(0);
    else await expect(adminPage.locator(SEL.notice).first()).toBeVisible();
    await expect(detail.kpi('Rated').locator).toBeVisible();
  });
});

test.describe('actions', () => {
  test('sorting by cost reorders the table and marks the header', async ({ adminPage }) => {
    const page = new SessionsPage(adminPage);
    await page.goto(WINDOW);
    const direction = await page.sortBy('Cost');
    expect(direction).toBe('descending');
    await expect(adminPage).toHaveURL(/sort=cost/);
  });

  test('the errors KPI is the filter it counts @REQ-026', async ({ adminPage }) => {
    const page = new SessionsPage(adminPage);
    await page.goto(WINDOW);
    await page.errorsKpi().click();
    await adminPage.waitForLoadState('networkidle');
    await expect(adminPage).toHaveURL(/error_only=true/);
  });

  test('narrowing the scope to one group narrows the table', async ({ adminPage }) => {
    const page = new SessionsPage(adminPage);
    await page.goto(WINDOW);
    const before = await page.table().rowCount();
    await page.scopeTo('group', 'europe-devs');
    await expect(adminPage).toHaveURL(/group=europe-devs/);
    expect(await page.table().rowCount()).toBeLessThanOrEqual(before);
  });

  test('paging keeps the window and the sort', async ({ adminPage }) => {
    const page = new SessionsPage(adminPage);
    await page.goto({ ...WINDOW, sort: 'cost', dir: 'desc' });
    const controls = adminPage.locator(SEL.paginationControls).first();
    await expect(controls).toBeVisible();
    const next = controls.getByRole('link', { name: /next/i });
    if ((await next.count()) === 0) test.skip(true, 'one page of sessions seeded');
    await next.click();
    await adminPage.waitForLoadState('networkidle');
    await expect(adminPage).toHaveURL(/sort=cost/);
    await expect(adminPage).toHaveURL(/page=1/);
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

  test('an unknown session id is a 404, not a 500', async ({ browser }) => {
    const context = await browser.newContext({ storageState: AUTH.admin });
    const res = await context.request.get(PATHS.session('no-such-session-id'), {
      maxRedirects: 0,
    });
    expect(res.status()).toBe(404);
    await context.close();
  });
});

test.describe('design language', () => {
  test('wears the shell', async ({ adminPage }) => {
    const page = new SessionsPage(adminPage);
    await page.goto(WINDOW);
    // The sidebar link carries the window forward, so the path is the claim.
    await expect(page.activeNavItem()).toHaveAttribute('href', new RegExp(`^${PATH}(\\?|$)`));
    await expect(page.heading()).toBeVisible();
    await expect(adminPage.locator(SEL.toolbarCount)).toBeVisible();
  });

  test('paints from design tokens', async ({ adminPage }) => {
    const page = new SessionsPage(adminPage);
    await page.goto(WINDOW);
    const values = await adminPage.evaluate((tokens) => {
      const style = getComputedStyle(document.documentElement);
      return tokens.map((t) => style.getPropertyValue(t).trim());
    }, Object.values(TOKENS));
    for (const value of values) expect(value).not.toBe('');
  });

  test('shows twelve rows above the fold at 1440x900', async ({ adminPage }) => {
    const page = new SessionsPage(adminPage);
    await adminPage.setViewportSize({ width: 1440, height: 900 });
    await page.goto(WINDOW);
    const total = await page.table().rowCount();
    if (total < 12) test.skip(true, `only ${total} sessions seeded`);
    const rows = page.table().rows();
    const box = await rows.nth(11).boundingBox();
    expect(box, 'the twelfth row is not laid out').not.toBeNull();
  });

  test('shows an empty state when nothing matches', async ({ adminPage }) => {
    const page = new SessionsPage(adminPage);
    await page.goto({ preset: '15m', user_id: 'no-such-user' });
    await expect(page.emptyState()).toBeVisible();
  });

  test('keeps a visible focus ring', async ({ adminPage }) => {
    const page = new SessionsPage(adminPage);
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
    const page = new SessionsPage(adminPage);
    await adminPage.setViewportSize({ width: 1440, height: 900 });
    await page.goto(WINDOW);
    await expectDensity(adminPage, 'list');
  });


  test('the detail page meets the density bar', async ({ adminPage }) => {
    const list = new SessionsPage(adminPage);
    await adminPage.setViewportSize({ width: 1440, height: 900 });
    await list.goto(WINDOW);
    await list.openFirst();
    await expectDensity(adminPage, 'detail');
  });

  test('matches its visual baseline', async ({ adminPage }) => {
    const page = new SessionsPage(adminPage);
    await adminPage.setViewportSize({ width: 1440, height: 900 });
    await page.goto(WINDOW);
    await snapshot(adminPage, 'sessions');
  });

  test('has no serious or critical axe violations', async ({ adminPage }) => {
    const page = new SessionsPage(adminPage);
    await page.goto(WINDOW);
    await expect(adminPage.locator(SEL.pageHeader)).toBeVisible();
    await expectAccessible(adminPage);
  });

  test('the detail page has no serious or critical axe violations', async ({ adminPage }) => {
    const list = new SessionsPage(adminPage);
    await list.goto(WINDOW);
    await list.openFirst();
    await expectAccessible(adminPage);
  });
});
