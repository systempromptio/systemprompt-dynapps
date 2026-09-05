// /admin/setup — the four-phase onboarding guide.
//
// Shell note: setup has no sidebar entry of its own; it shares the Settings
// entry (sidebar.hbs marks Settings active for page ids "settings" and
// "setup"), so the design-language block asserts the Settings link is current,
// not a /admin/setup link that does not exist.
import { test, expect, type Principal } from '../support/fixtures';
import { expectAccessible } from '../support/a11y';
import { PATHS } from '../support/paths';
import { SEL, TOKENS } from '../support/pages/selectors';
import { SetupPage } from '../support/pages/setup.page';

const PATH = PATHS.setup;

// Mirrors tests/contract/admin/baseline.txt for GET /admin/setup.
const ACCESS: { principal: Principal | 'anon'; status: number }[] = [
  { principal: 'platformAdmin', status: 200 },
  { principal: 'admin', status: 200 },
  { principal: 'pm', status: 200 },
  { principal: 'developer', status: 200 },
  { principal: 'businessUser', status: 200 },
  { principal: 'unassigned', status: 200 },
  { principal: 'user', status: 200 },
  { principal: 'anon', status: 307 },
];

test.describe('renders', () => {
  test('lists all four phases in order', async ({ developerPage }) => {
    const page = new SetupPage(developerPage);
    await page.goto();
    await expect(page.phases()).toHaveCount(4);
    expect(await page.table().column('#')).toEqual(['1', '2', '3', '4']);
  });

  test('shows progress as KPIs, not prose', async ({ developerPage }) => {
    const page = new SetupPage(developerPage);
    await page.goto();
    await expect(page.kpi('Phases complete').locator).toBeVisible();
    await expect(page.kpi('Plugins reaching you').locator).toBeVisible();
    await expect(page.kpi('Skills in your library').locator).toBeVisible();
  });

  test('gives every phase a status badge', async ({ developerPage }) => {
    const page = new SetupPage(developerPage);
    await page.goto();
    const statuses = await page.table().column('Status');
    expect(statuses).toHaveLength(4);
    for (const status of statuses) {
      expect(['Complete', 'Current', 'Not started']).toContain(status.trim());
    }
  });

  test('marks at most one phase as the one to do next', async ({ developerPage }) => {
    const page = new SetupPage(developerPage);
    await page.goto();
    expect(await page.currentPhase().count()).toBeLessThanOrEqual(1);
  });
});

test.describe('actions', () => {
  test('every phase links to its guide and the actionable ones to the page', async ({
    developerPage,
  }) => {
    const page = new SetupPage(developerPage);
    await page.goto();
    expect(await page.guideLinks().count()).toBeGreaterThanOrEqual(4);
    for (const href of [PATHS.plugins, PATHS.skills]) {
      await expect(developerPage.locator(`.sp-table__el a[href="${href}"]`)).toHaveCount(1);
    }
  });

  test('the header sends the reader to the connect code that phase one needs', async ({
    developerPage,
  }) => {
    const page = new SetupPage(developerPage);
    await page.goto();
    await developerPage.locator(`${SEL.headerActions} a[href="${PATHS.profile}"]`).click();
    await expect(developerPage).toHaveURL(new RegExp(`${PATHS.profile}$`));
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

  test('the guide reads the same for an admin and an ordinary member', async ({
    adminPage,
    userPage,
  }) => {
    await new SetupPage(adminPage).goto();
    await new SetupPage(userPage).goto();
    expect(await adminPage.locator('.sp-table__el tbody tr').count()).toBe(
      await userPage.locator('.sp-table__el tbody tr').count(),
    );
  });
});

test.describe('design language', () => {
  test('wears the shell', async ({ developerPage }) => {
    const page = new SetupPage(developerPage);
    await page.goto();
    await expect(page.activeNavItem()).toHaveAttribute('href', PATHS.settings);
    await expect(page.breadcrumb()).toBeVisible();
    await expect(page.heading()).toBeVisible();
  });

  test('carries no trace of the old bespoke stepper', async ({ developerPage }) => {
    const page = new SetupPage(developerPage);
    await page.goto();
    for (const legacy of ['.sp-setup-stepper', '.sp-setup-phase', '.sp-setup-badge']) {
      expect(await developerPage.locator(legacy).count(), legacy).toBe(0);
    }
  });

  test('paints from design tokens', async ({ developerPage }) => {
    const page = new SetupPage(developerPage);
    await page.goto();
    const values = await developerPage.evaluate((tokens) => {
      const style = getComputedStyle(document.documentElement);
      return tokens.map((t) => style.getPropertyValue(t).trim());
    }, Object.values(TOKENS));
    for (const value of values) expect(value).not.toBe('');
  });

  test('keeps a visible focus ring', async ({ developerPage }) => {
    const page = new SetupPage(developerPage);
    await page.goto();
    await developerPage.keyboard.press('Tab');
    const ring = await developerPage.evaluate(() => {
      const el = document.activeElement;
      if (!el) return null;
      const s = getComputedStyle(el);
      return { style: s.outlineStyle, width: s.outlineWidth };
    });
    expect(ring).not.toBeNull();
    expect(ring?.style).not.toBe('none');
    expect(Number.parseFloat(ring?.width ?? '0')).toBeGreaterThan(0);
  });

  test('does not scroll horizontally at 1440 or 1024', async ({ developerPage }) => {
    const page = new SetupPage(developerPage);
    for (const width of [1440, 1024]) {
      await developerPage.setViewportSize({ width, height: 900 });
      await page.goto();
      expect(await page.hasHorizontalScroll(), `horizontal scroll at ${width}px`).toBe(false);
    }
  });

  test('has no serious or critical axe violations', async ({ developerPage }) => {
    const page = new SetupPage(developerPage);
    await page.goto();
    await expect(developerPage.locator(SEL.pageHeader)).toBeVisible();
    await expectAccessible(developerPage);
  });
});
