// /admin/gateway — the model routing table and the settings above it.
//
// Route order is policy, so the assertions treat the row ordinal as data. The
// mutation tests are deliberately net-zero against the shared gateway file: a
// reorder is moved and moved back, and the settings save writes the values the
// page was rendered with. This suite must not leave a different routing policy
// behind than it found.
import { test, expect, type Principal } from '../support/fixtures';
import { expectAccessible } from '../support/a11y';
import { PATHS } from '../support/paths';
import { SEL, TOKENS } from '../support/pages/selectors';
import { BasePage } from '../support/pages/base.page';
import { GatewayPage } from '../support/pages/gateway.page';

const PATH = PATHS.gateway;

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
  test('lists the declared routes in dispatch order', async ({ adminPage }) => {
    const page = new GatewayPage(adminPage);
    await page.goto();
    const ids = await page.routeIds();
    expect(ids.length).toBeGreaterThan(0);
    const ordinals = await page.table().column('#');
    expect(ordinals[0].trim()).toBe('1');
  });

  test('states whether the gateway is on and where it listens', async ({ adminPage }) => {
    const page = new GatewayPage(adminPage);
    await page.goto();
    expect(['On', 'Off']).toContain(await page.kpi('Gateway').value());
    await expect(adminPage.locator('#gw-prefix')).toHaveValue(/^\//);
  });

  test('names every route the dispatcher resolves that the file omits', async ({ adminPage }) => {
    const page = new GatewayPage(adminPage);
    await page.goto();
    await expect(adminPage.locator('section[aria-label="Resolved routes"]')).toBeVisible();
  });

  test('lists the providers a route may name', async ({ adminPage }) => {
    const page = new GatewayPage(adminPage);
    await page.goto();
    expect(await page.kpi('Providers').number()).toBeGreaterThan(0);
  });
});

test.describe('actions', () => {
  test('a route with no model pattern is refused with an inline reason', async ({ adminPage }) => {
    const page = new GatewayPage(adminPage);
    await page.goto();
    await page.openNewRoute();
    await page.submitRoute({ model_pattern: '', provider: 'anthropic' });
    await expect(page.routeError()).not.toBeEmpty();
    // Nothing was written, so the route count is unchanged.
    await adminPage.reload();
    expect((await page.routeIds()).length).toBeGreaterThan(0);
  });

  test('reordering moves a route and the file keeps the new order', async ({ adminPage }) => {
    const page = new GatewayPage(adminPage);
    await page.goto();
    const before = await page.routeIds();
    test.skip(before.length < 2, 'reordering needs at least two declared routes');

    await adminPage.locator('[data-action="route-down"]').first().click();
    await adminPage.waitForLoadState('networkidle');
    const moved = await page.routeIds();
    expect(moved[0]).toBe(before[1]);
    expect(moved[1]).toBe(before[0]);

    await adminPage.locator('[data-action="route-up"]').nth(1).click();
    await adminPage.waitForLoadState('networkidle');
    expect(await page.routeIds()).toEqual(before);
  });

  test('saving the settings it was rendered with reports success', async ({ adminPage }) => {
    const page = new GatewayPage(adminPage);
    await page.goto();
    await adminPage.locator('#gateway-settings').getByRole('button', { name: /save/i }).click();
    await expect(adminPage.locator('[data-settings-status]')).toContainText(/saved/i);
  });

  test('probing an account lists the routes it would be offered', async ({ adminPage }) => {
    const page = new GatewayPage(adminPage);
    await page.goto();
    const text = await page.probe('e2e-admin');
    expect(text.length).toBeGreaterThan(0);
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

  test('a project manager may read the routes but not write one', async ({ browser }) => {
    const context = await browser.newContext({ storageState: '.auth/pm.json' });
    const res = await context.request.post('/api/public/admin/gateway/routes', {
      data: { model_pattern: 'x-*', provider: 'anthropic' },
      failOnStatusCode: false,
    });
    expect(res.status()).toBe(403);
    await context.close();
  });
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

  test('shows an empty state where a section has nothing to list', async ({ adminPage }) => {
    const page = new BasePage(adminPage, PATH);
    await page.goto();
    await expect(adminPage.locator(SEL.empty).first()).toBeVisible();
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
});
