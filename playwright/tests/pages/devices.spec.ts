// Devices & enrolments — /admin/devices.
//
// The page answers one question of four tables: what still holds a credential
// to this platform, and is anything using it. The tests below follow that
// shape — the fleet strip is fleet-wide and must not move with the tab, and
// the revoke path is the only mutation the page offers.
import { AUTH, test, expect, type Principal } from '../support/fixtures';
import { expectAccessible } from '../support/a11y';
import { PATHS } from '../support/paths';
import { SEL, TOKENS } from '../support/pages/selectors';
import { DevicesPage } from '../support/pages/devices.page';

const PATH = PATHS.devices;

// 200 = renders, 303 = redirected to a page they may see. The console roles
// read the fleet; only the two admin roles are offered a revoke button, which
// the actions block covers rather than this table.
const ACCESS: { principal: Principal | 'anon'; status: number }[] = [
  { principal: 'platformAdmin', status: 200 },
  { principal: 'admin', status: 200 },
  { principal: 'pm', status: 200 },
  { principal: 'developer', status: 303 },
  { principal: 'businessUser', status: 303 },
  { principal: 'unassigned', status: 303 },
  { principal: 'user', status: 303 },
  // Why: 307, not the 303 a signed-in non-console user gets. An anonymous
  // request is bounced by the auth layer with its method preserved, before
  // anything has decided whether this page would admit them.
  { principal: 'anon', status: 307 },
];

test.describe('renders', () => {
  test('lists the seeded bridge sessions', async ({ adminPage }) => {
    const page = new DevicesPage(adminPage);
    await page.goto();
    expect(await page.table().rowCount()).toBeGreaterThan(0);
  });

  test('counts active and stale bridges separately', async ({ adminPage }) => {
    const page = new DevicesPage(adminPage);
    await page.goto();
    const active = await page.kpi('Active devices').number();
    const stale = await page.kpi('Stale over 7d').number();
    expect(active).toBeGreaterThan(0);
    expect(stale).toBeGreaterThan(0);
  });

  // The histogram is capped at twelve bars on purpose, so this asserts the
  // relationship rather than equality: every bar is a real version, and a
  // fleet with versions in it draws at least one.
  test('draws a histogram bar for the bridge versions in the fleet', async ({ adminPage }) => {
    const page = new DevicesPage(adminPage);
    await page.goto();
    const versions = await page.kpi('Versions').number();
    const bars = await page.versionBars().count();
    expect(bars).toBeGreaterThan(0);
    expect(bars).toBeLessThanOrEqual(Math.min(versions, 12));
  });

  test('each credential tab renders its own rows', async ({ adminPage }) => {
    const page = new DevicesPage(adminPage);
    for (const tab of ['pats', 'certs', 'links'] as const) {
      await page.gotoTab(tab);
      expect(await page.table().rowCount(), `${tab} tab`).toBeGreaterThan(0);
    }
  });

  test('every row links to the person holding the credential', async ({ adminPage }) => {
    const page = new DevicesPage(adminPage);
    await page.goto();
    const links = page.table().rows().locator('a[href^="/admin/users/"]');
    expect(await links.count()).toBe(await page.table().rowCount());
  });
});

test.describe('actions', () => {
  test('the stale tile filters the listing to stale bridges', async ({ adminPage }) => {
    const page = new DevicesPage(adminPage);
    await page.goto();
    const all = await page.table().rowCount();
    const stale = await page.kpi('Stale over 7d').number();
    await page.kpi('Stale over 7d').locator.click();
    await adminPage.waitForLoadState('networkidle');
    expect(await page.table().rowCount()).toBe(stale);
    expect(stale).toBeLessThan(all);
  });

  test('sorting a column flips its direction and reorders the rows', async ({ adminPage }) => {
    const page = new DevicesPage(adminPage);
    await page.goto();
    const first = await page.table().sortBy('Host');
    const second = await page.table().sortBy('Host');
    expect(second).not.toBe(first);
  });

  test('switching tab drops the filter it does not apply to', async ({ adminPage }) => {
    const page = new DevicesPage(adminPage);
    await page.goto();
    await page.kpi('Stale over 7d').locator.click();
    await adminPage.waitForLoadState('networkidle');
    await page.tab('Access tokens').click();
    await adminPage.waitForLoadState('networkidle');
    expect(adminPage.url()).not.toContain('stale=');
  });

  // Oldest first on purpose. The newest active tokens are the bridge
  // principals' real credentials, which the gateway specs authenticate with;
  // the oldest are the fleet-filler rows, which nothing else depends on.
  const OLDEST_ACTIVE_PATS = `${PATHS.devicesTab('pats')}&state=active&sort=created&dir=asc`;

  test('revoking a token takes it out of the active view', async ({ adminPage }) => {
    const page = new DevicesPage(adminPage);
    await adminPage.goto(OLDEST_ACTIVE_PATS);
    const before = await page.table().rowCount();
    await page.revokeFirst();
    await expect(page.toast()).toBeVisible();
    await adminPage.goto(OLDEST_ACTIVE_PATS);
    expect(await page.table().rowCount()).toBe(before - 1);
  });
});

test.describe('authorization', () => {
  for (const { principal, status } of ACCESS) {
    test(`${principal} gets ${status}`, async ({ browser }) => {
      // Why: AUTH, not a path built from the principal's name. The files are
      // kebab-cased and the principals are camel-cased, so a template literal
      // silently misses on exactly the two principals whose names have two
      // words — which is how a matrix loses rows without losing a test.
      const context = await browser.newContext(
        principal === 'anon' ? {} : { storageState: AUTH[principal] },
      );
      const res = await context.request.get(PATH, { maxRedirects: 0 });
      expect(res.status()).toBe(status);
      await context.close();
    });
  }

  test('a project manager reads the fleet but is offered no revoke', async ({ pmPage }) => {
    const page = new DevicesPage(pmPage);
    await page.gotoTab('pats');
    expect(await page.table().rowCount()).toBeGreaterThan(0);
    expect(await page.revokeButtons().count()).toBe(0);
  });
});

test.describe('design language', () => {
  test('wears the shell', async ({ adminPage }) => {
    const page = new DevicesPage(adminPage);
    await page.goto();
    await expect(page.activeNavItem()).toHaveAttribute('href', PATH);
    await expect(page.breadcrumb()).toBeVisible();
    await expect(page.heading()).toBeVisible();
    await expect(page.activeTab()).toHaveText(/Bridge sessions/);
  });

  test('paints from design tokens', async ({ adminPage }) => {
    const page = new DevicesPage(adminPage);
    await page.goto();
    const values = await adminPage.evaluate((tokens) => {
      const style = getComputedStyle(document.documentElement);
      return tokens.map((t) => style.getPropertyValue(t).trim());
    }, Object.values(TOKENS));
    for (const value of values) expect(value).not.toBe('');
  });

  test('shows an empty state when a filter matches nothing', async ({ adminPage }) => {
    const page = new DevicesPage(adminPage);
    await adminPage.goto(`${PATHS.devicesTab('certs')}&state=revoked&page=9`);
    await expect(page.emptyState()).toBeVisible();
  });

  test('keeps a visible focus ring', async ({ adminPage }) => {
    const page = new DevicesPage(adminPage);
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
    const page = new DevicesPage(adminPage);
    for (const width of [1440, 1024]) {
      await adminPage.setViewportSize({ width, height: 900 });
      await page.goto();
      expect(await page.hasHorizontalScroll(), `horizontal scroll at ${width}px`).toBe(false);
    }
  });

  test('has no serious or critical axe violations', async ({ adminPage }) => {
    const page = new DevicesPage(adminPage);
    await page.goto();
    await expect(adminPage.locator(SEL.pageHeader)).toBeVisible();
    await expectAccessible(adminPage);
  });
});
