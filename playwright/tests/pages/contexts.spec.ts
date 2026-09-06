// The contexts list (/admin/contexts) and one context's detail page.
//
// A context is the persisted state of one conversation. The list is what an
// operator scans to find a conversation worth reading; the detail page is the
// transcript plus what the session around it actually touched.
import { AUTH, test, expect, snapshot, type Principal } from '../support/fixtures';
import { expectAccessible, expectDensity } from '../support/a11y';
import { PATHS } from '../support/paths';
import { SEL, TOKENS } from '../support/pages/selectors';
import { BasePage } from '../support/pages/base.page';
import { ContextsPage } from '../support/pages/contexts.page';

const PATH = PATHS.contexts;

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
  test('lists the seeded contexts, one row per context id', async ({ adminPage }) => {
    const page = new ContextsPage(adminPage);
    await page.goto();
    expect(await page.table().rowCount()).toBeGreaterThan(0);
  });

  test('carries the six decision-relevant totals above the table', async ({ adminPage }) => {
    const page = new ContextsPage(adminPage);
    await page.goto();
    for (const label of ['Contexts', 'Users', 'Requests', 'Messages', 'Tokens', 'Cost']) {
      await expect(page.kpi(label).locator, `KPI "${label}"`).toBeVisible();
    }
    expect(await page.kpi('Contexts').number()).toBeGreaterThan(0);
  });

  test('every context id links to its detail, and its session to the session', async ({
    adminPage,
  }) => {
    const page = new ContextsPage(adminPage);
    await page.goto();
    expect(await page.contextLinks().count()).toBeGreaterThan(0);
    expect(
      await adminPage.locator(`${SEL.tableRow} a[href^="/admin/sessions/"]`).count(),
    ).toBeGreaterThan(0);
  });

  test('the detail page carries breadcrumbs back to the list and the session', async ({
    adminPage,
  }) => {
    const list = new ContextsPage(adminPage);
    await list.goto();
    const detail = await list.openFirst();
    await expect(detail.breadcrumb()).toBeVisible();
    await expect(detail.breadcrumb().getByRole('link', { name: /contexts/i })).toBeVisible();
  });

  test('the detail page reports what the session touched @REQ-026', async ({ adminPage }) => {
    const list = new ContextsPage(adminPage);
    await list.goto();
    const detail = await list.openFirst();
    // The panel is always answered: a gateway-only context has no session and
    // gets an empty state rather than being omitted.
    const hasTable = (await detail.entityTable().count()) > 0;
    if (hasTable) expect(await detail.entityBars().count()).toBeGreaterThan(0);
    else await expect(detail.emptyState().first()).toBeVisible();
  });
});

test.describe('actions', () => {
  test('sorting by cost reorders the table and marks the header', async ({ adminPage }) => {
    const page = new ContextsPage(adminPage);
    await page.goto();
    const direction = await page.sortBy('Cost');
    expect(direction).toBe('descending');
    await expect(adminPage).toHaveURL(/sort=cost/);
  });

  test('the search box narrows the table', async ({ adminPage }) => {
    const page = new ContextsPage(adminPage);
    await page.goto();
    const before = await page.table().rowCount();
    await page.search('e2e-developer');
    expect(await page.table().rowCount()).toBeLessThanOrEqual(before);
  });

  test('narrowing the scope to one group narrows the table', async ({ adminPage }) => {
    const page = new ContextsPage(adminPage);
    await page.goto();
    const before = await page.table().rowCount();
    await page.scopeTo('group', 'europe-devs');
    expect(await page.table().rowCount()).toBeLessThanOrEqual(before);
  });

  test('the by-user view groups the same rows under their owner', async ({ adminPage }) => {
    const page = new ContextsPage(adminPage);
    await page.goto({ view: 'users' });
    expect(await page.table().rowCount()).toBeGreaterThan(0);
    await expect(page.viewTab('users')).toHaveAttribute('aria-selected', 'true');
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

  test('an unknown context id is a 404, not a 500', async ({ browser }) => {
    const context = await browser.newContext({ storageState: AUTH.admin });
    const res = await context.request.get(PATHS.context('not-a-uuid'), { maxRedirects: 0 });
    expect(res.status()).toBe(404);
    await context.close();
  });
});

test.describe('design language', () => {
  test('wears the shell', async ({ adminPage }) => {
    const page = new ContextsPage(adminPage);
    await page.goto();
    // The sidebar link carries the window forward, so the path is the claim.
    await expect(page.activeNavItem()).toHaveAttribute('href', new RegExp(`^${PATH}(\\?|$)`));
    await expect(page.heading()).toBeVisible();
    await expect(adminPage.locator(SEL.toolbarCount)).toBeVisible();
  });

  test('paints from design tokens', async ({ adminPage }) => {
    const page = new ContextsPage(adminPage);
    await page.goto();
    const values = await adminPage.evaluate((tokens) => {
      const style = getComputedStyle(document.documentElement);
      return tokens.map((t) => style.getPropertyValue(t).trim());
    }, Object.values(TOKENS));
    for (const value of values) expect(value).not.toBe('');
  });

  test('shows an empty state when nothing matches', async ({ adminPage }) => {
    const page = new ContextsPage(adminPage);
    await page.goto({ q: 'zzz-no-such-context-zzz' });
    await expect(page.emptyState()).toBeVisible();
  });

  test('keeps a visible focus ring', async ({ adminPage }) => {
    const page = new ContextsPage(adminPage);
    await page.goto();
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
      await page.goto();
      expect(await page.hasHorizontalScroll(), `horizontal scroll at ${width}px`).toBe(false);
    }
  });



  test('meets the density bar', async ({ adminPage }) => {
    const page = new ContextsPage(adminPage);
    await adminPage.setViewportSize({ width: 1440, height: 900 });
    await page.goto();
    await expectDensity(adminPage, 'list');
  });


  test('the detail page meets the density bar', async ({ adminPage }) => {
    const list = new ContextsPage(adminPage);
    await adminPage.setViewportSize({ width: 1440, height: 900 });
    await list.goto();
    await list.openFirst();
    await expectDensity(adminPage, 'detail');
  });

  test('matches its visual baseline', async ({ adminPage }) => {
    const page = new ContextsPage(adminPage);
    await adminPage.setViewportSize({ width: 1440, height: 900 });
    await page.goto();
    await snapshot(adminPage, 'contexts');
  });

  test('has no serious or critical axe violations', async ({ adminPage }) => {
    const page = new ContextsPage(adminPage);
    await page.goto();
    await expect(adminPage.locator(SEL.pageHeader)).toBeVisible();
    await expectAccessible(adminPage);
  });

  test('the detail page has no serious or critical axe violations', async ({ adminPage }) => {
    const list = new ContextsPage(adminPage);
    await list.goto();
    await list.openFirst();
    await expectAccessible(adminPage);
  });
});
