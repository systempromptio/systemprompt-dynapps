// /admin/mcp — the tool servers, their liveness, and their traffic.
//
// The declaration and the runtime are separate sources that can disagree, and
// the assertions below are written on that seam: a server the catalog declares
// with no session is "Never connected", and a server the runtime has served
// under a name nothing declares still gets a row.
import { test, expect, type Principal } from '../support/fixtures';
import { expectAccessible } from '../support/a11y';
import { PATHS } from '../support/paths';
import { SEL, TOKENS } from '../support/pages/selectors';
import { BasePage } from '../support/pages/base.page';
import { McpDetailPage, McpListPage } from '../support/pages/mcp.page';

const PATH = PATHS.mcp;

// The seed drives traffic through these three names. `atlassian` is
// deliberately not declared in services/mcp, so it exercises the undeclared arm.
const SEEDED = 'systemprompt';
const UNDECLARED = 'atlassian';

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
  test('lists every declared server and every server the runtime has served', async ({
    adminPage,
  }) => {
    const page = new McpListPage(adminPage);
    await page.goto();
    const ids = await page.serverIds();
    expect(ids).toContain(SEEDED);
    expect(ids).toContain(UNDECLARED);
  });

  test('a server carrying seeded traffic reads as alive', async ({ adminPage }) => {
    const page = new McpListPage(adminPage);
    await page.goto();
    await expect(page.row(SEEDED)).toBeVisible();
    expect(await page.statusOf(SEEDED)).toMatch(/alive|stale/i);
  });

  test('the fleet tiles count the declared servers', async ({ adminPage }) => {
    const page = new McpListPage(adminPage);
    await page.goto();
    expect(await page.kpi('Declared').number()).toBeGreaterThan(0);
    expect(await page.kpi('Calls 24h').number()).toBeGreaterThan(0);
  });

  test('the detail page shows tools, calls, sessions and grants', async ({ adminPage }) => {
    const page = new McpDetailPage(adminPage, SEEDED);
    await page.goto();
    await expect(page.heading()).toHaveText(SEEDED);
    expect(await page.toolRows().count()).toBeGreaterThan(0);
    expect(await page.executionRows().count()).toBeGreaterThan(0);
    expect(await page.sessionRows().count()).toBeGreaterThan(0);
  });

  test('an unknown server id is a 404, not a 500', async ({ adminPage }) => {
    const res = await adminPage.goto(PATHS.mcpServer('no-such-server-at-all'));
    expect(res?.status()).toBe(404);
  });
});

test.describe('actions', () => {
  test('filtering narrows the table', async ({ adminPage }) => {
    const page = new McpListPage(adminPage);
    await page.goto();
    const before = await page.table().rowCount();
    await page.table().filter(SEEDED);
    expect(await page.table().rowCount()).toBeLessThan(before);
    await expect(page.row(SEEDED)).toBeVisible();
  });

  test('sorting by calls states the direction and orders the rows by it', async ({
    adminPage,
  }) => {
    const page = new McpListPage(adminPage);
    await page.goto({ sort: 'calls', dir: 'desc' });
    const header = adminPage.locator('th').filter({ hasText: 'Calls 24h' }).first();
    await expect(header).toHaveAttribute('aria-sort', 'descending');
    const calls = (await page.table().column('Calls 24h')).map((c) =>
      Number(c.replace(/[^0-9]/g, '') || '0'),
    );
    expect([...calls].sort((a, b) => b - a)).toEqual(calls);
  });

  test('a sort link keeps the active filter', async ({ adminPage }) => {
    const page = new McpListPage(adminPage);
    await page.goto({ q: SEEDED });
    const href = await adminPage.locator('th a.sp-table__sort').first().getAttribute('href');
    expect(href).toContain(`q=${SEEDED}`);
  });

  test('the call log pages when there is more than one page of calls', async ({ adminPage }) => {
    const page = new McpDetailPage(adminPage, SEEDED);
    await page.goto();
    await expect(adminPage.locator(SEL.paginationInfo).first()).toBeVisible();
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
  test('wears the shell', async ({ adminPage }) => {
    const page = new BasePage(adminPage, PATH);
    await page.goto();
    await expect(page.activeNavItem()).toHaveAttribute('href', PATH);
    await expect(page.breadcrumb()).toBeVisible();
    await expect(page.heading()).toBeVisible();
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

  test('shows an empty state when nothing matches', async ({ adminPage }) => {
    const page = new BasePage(adminPage, PATH);
    await page.goto();
    await page.table().filter('zzz-no-such-server-zzz');
    await page.table().expectEmpty();
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

  test('has no serious or critical axe violations', async ({ adminPage }) => {
    const page = new BasePage(adminPage, PATH);
    await page.goto();
    await expect(adminPage.locator(SEL.pageHeader)).toBeVisible();
    await expectAccessible(adminPage);
  });

  test('the detail page has no serious or critical axe violations', async ({ adminPage }) => {
    const page = new McpDetailPage(adminPage, SEEDED);
    await page.goto();
    await expectAccessible(adminPage);
  });
});
