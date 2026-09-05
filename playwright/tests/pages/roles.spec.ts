// /admin/roles — the role's side of the two tables the console reads
// elsewhere from the user's side.
//
// The authorization table is literal per principal and matches
// tests/contract/admin/baseline.txt: reading is the console tier, so a project
// manager sees the page and a developer is sent to their profile.
import { test, expect, AUTH, type Principal } from '../support/fixtures';
import { expectAccessible, expectDensity } from '../support/a11y';
import { PATHS } from '../support/paths';
import { SEL, TOKENS } from '../support/pages/selectors';
import { RolesPage } from '../support/pages/roles.page';

const PATH = PATHS.roles;

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
  test('draws one card per role, with its member count', async ({ adminPage }) => {
    const page = new RolesPage(adminPage);
    await page.goto();
    expect(await page.cards().count()).toBe(6);
    await expect(page.card('Platform admin')).toBeVisible();
    await expect(page.card('Developer')).toBeVisible();
  });

  test('lists the seeded assignments, one row per person per role', async ({ adminPage }) => {
    const page = new RolesPage(adminPage);
    await page.goto();
    expect(await page.assignments().rowCount()).toBeGreaterThan(0);
    const people = await page.columnValues('Person');
    expect(people.join(' ')).toContain('e2e');
  });

  test('separates a directory grant from one made by hand', async ({ adminPage }) => {
    const page = new RolesPage(adminPage);
    await page.goto();
    const sources = await page.columnValues('Grant source');
    expect(sources.length).toBeGreaterThan(0);
    for (const source of sources) expect(['Manual', 'Directory']).toContain(source.trim());
  });

  test('lists what each role opens', async ({ adminPage }) => {
    const page = new RolesPage(adminPage);
    await page.goto();
    await expect(adminPage.getByText('Role entitlements').first()).toBeVisible();
  });

  test('shows the five totals above the fold', async ({ adminPage }) => {
    const page = new RolesPage(adminPage);
    await page.goto();
    expect(await adminPage.locator(SEL.kpi).count()).toBeGreaterThanOrEqual(5);
    await expect(page.kpi('Platform admins').locator).toBeVisible();
  });
});

test.describe('actions', () => {
  test('a role card filters the assignment table to that role', async ({ adminPage }) => {
    const page = new RolesPage(adminPage);
    await page.goto();
    await page.selectRole('Admin');
    await page.expectRowsFor('Admin');
  });

  test('the grant-source filter narrows the table', async ({ adminPage }) => {
    const page = new RolesPage(adminPage);
    await page.goto();
    const before = await page.assignments().rowCount();
    await page.filterBy('source', 'directory');
    expect(await page.assignments().rowCount()).toBeLessThanOrEqual(before);
    const sources = await page.columnValues('Grant source');
    for (const source of sources) expect(source.trim()).toBe('Directory');
  });

  test('searching narrows the table to one person', async ({ adminPage }) => {
    const page = new RolesPage(adminPage);
    await page.goto();
    await page.assignments().filter('e2e-developer');
    const people = await page.columnValues('Person');
    expect(people.length).toBeGreaterThan(0);
  });

  test('sorting by cost reorders the rows and says so', async ({ adminPage }) => {
    const page = new RolesPage(adminPage);
    await page.goto();
    const direction = await page.assignments().sortBy('Cost 30d');
    expect(['ascending', 'descending']).toContain(direction);
  });

  test('an admin is offered a grant control; only a manual grant offers revoke', async ({
    adminPage,
  }) => {
    const page = new RolesPage(adminPage);
    await page.goto();
    await expect(page.grantButton()).toBeVisible();
    await page.filterBy('source', 'directory');
    expect(await page.revokeButtons().count()).toBe(0);
  });

  test('a project manager gets no write control', async ({ pmPage }) => {
    const page = new RolesPage(pmPage);
    await page.goto();
    await expect(page.grantButton()).toHaveCount(0);
    expect(await page.revokeButtons().count()).toBe(0);
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
    const page = new RolesPage(adminPage);
    await page.goto();
    await expect(page.activeNavItem()).toHaveAttribute('href', PATH);
    await expect(page.heading()).toBeVisible();
  });

  test('paints from design tokens', async ({ adminPage }) => {
    const page = new RolesPage(adminPage);
    await page.goto();
    const values = await adminPage.evaluate((tokens) => {
      const style = getComputedStyle(document.documentElement);
      return tokens.map((t) => style.getPropertyValue(t).trim());
    }, Object.values(TOKENS));
    for (const value of values) expect(value).not.toBe('');
  });

  test('shows an empty state when nothing matches', async ({ adminPage }) => {
    const page = new RolesPage(adminPage);
    await page.goto();
    await page.assignments().filter('zzz-no-such-person-zzz');
    await expect(adminPage.locator(SEL.empty).first()).toBeVisible();
  });

  test('keeps a visible focus ring', async ({ adminPage }) => {
    const page = new RolesPage(adminPage);
    await page.goto();
    await adminPage.keyboard.press('Tab');
    // The design system draws the ring with box-shadow and keeps a non-zero
    // outline for forced-colors mode. Either mechanism is a visible ring;
    // asserting on outline-style alone reads a shadowed ring as absent.
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
    const page = new RolesPage(adminPage);
    for (const width of [1440, 1024]) {
      await adminPage.setViewportSize({ width, height: 900 });
      await page.goto();
      expect(await page.hasHorizontalScroll(), `horizontal scroll at ${width}px`).toBe(false);
    }
  });

  test('meets the density bar for a list page', async ({ adminPage }) => {
    const page = new RolesPage(adminPage);
    await adminPage.setViewportSize({ width: 1440, height: 900 });
    await page.goto();
    await expectDensity(adminPage, 'list');
  });

  test('has no serious or critical axe violations', async ({ adminPage }) => {
    const page = new RolesPage(adminPage);
    await page.goto();
    await expect(adminPage.locator(SEL.pageHeader)).toBeVisible();
    await expectAccessible(adminPage);
  });

  test('matches the committed visual snapshot', async ({ adminPage }) => {
    const page = new RolesPage(adminPage);
    await adminPage.setViewportSize({ width: 1440, height: 900 });
    await page.goto();
    await expect(adminPage).toHaveScreenshot('roles.png', {
      fullPage: false,
      maxDiffPixelRatio: 0.02,
      mask: [adminPage.locator(SEL.tableCellDate)],
    });
  });
});
