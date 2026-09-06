// /admin/marketplaces — the unit entitlement is granted on.
//
// Absorbs the old tests/marketplaces.spec.ts and tests/marketplace-catalog.spec.ts:
// the audience matrix's DOM contract, the per-marketplace detail naming its
// group, and the fact that the catalog is auth-gated rather than public.
//
// The page's whole point is that "declared" and "resolved" are two answers, so
// the assertions read both columns rather than trusting either alone.
import { AUTH, snapshot, test, expect, type Principal } from '../support/fixtures';
import { expectAccessible, expectDensity } from '../support/a11y';
import { PATHS } from '../support/paths';
import { SEL, TOKENS } from '../support/pages/selectors';
import { BasePage } from '../support/pages/base.page';

const PATH = PATHS.marketplaces;
const EUROPE = 'astound-europe-dev';
const EUROPE_GROUP = 'europe-devs';

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
  test('every seeded marketplace has a row', async ({ adminPage }) => {
    const page = new BasePage(adminPage, PATH);
    await page.goto();
    for (const id of [EUROPE, 'astound-india-dev', 'astound-cowork', 'enterprise-demo']) {
      await expect(adminPage.locator(`tr[data-marketplace-id="${id}"]`)).toBeVisible();
    }
  });

  test('the audience matrix has a cell per group and marketplace', async ({
    platformAdminPage,
  }) => {
    // The matrix is the second view rather than a second table under the
    // first: stacked, it pushed the listing's own rows off a 900px fold.
    const page = new BasePage(platformAdminPage, `${PATH}?tab=audience`);
    await page.goto();
    const matrix = platformAdminPage.locator('.sp-audience-matrix');
    await expect(matrix).toBeVisible();
    for (const group of ['europe-devs', 'india-devs', 'business-users']) {
      await expect(matrix.locator(`tr[data-subject="group:${group}"]`)).toBeVisible();
    }
    const allowed = matrix.locator(
      `tr[data-subject="group:${EUROPE_GROUP}"] td[data-marketplace="${EUROPE}"]`,
    );
    await expect(allowed).toHaveClass(/is-allow/);
    const denied = matrix.locator(
      `tr[data-subject="group:business-users"] td[data-marketplace="${EUROPE}"]`,
    );
    await expect(denied).toHaveClass(/is-deny/);
  });

  test('the detail page names the group that holds it, declared and resolved', async ({
    platformAdminPage,
  }) => {
    const res = await platformAdminPage.goto(PATHS.marketplace(EUROPE));
    expect(res?.status()).toBe(200);
    const row = platformAdminPage.locator(`tr[data-group-id="${EUROPE_GROUP}"]`);
    await expect(row).toBeVisible();
    await expect(row).toContainText(/granted/i);
    await expect(row).toContainText(/allow/i);
  });

  test('an unknown marketplace id is a 404, not a 500', async ({ adminPage }) => {
    const res = await adminPage.goto(PATHS.marketplace('no-such-marketplace'));
    expect(res?.status()).toBe(404);
  });
});

test.describe('actions', () => {
  test('filtering narrows the table', async ({ adminPage }) => {
    const page = new BasePage(adminPage, PATH);
    await page.goto();
    const before = await page.table().rowCount();
    await page.table().filter('india');
    expect(await page.table().rowCount()).toBeLessThan(before);
  });

  test('assigning and unassigning a group writes and removes the grant', async ({
    platformAdminPage,
  }) => {
    const target = 'business-users';
    await platformAdminPage.goto(PATHS.marketplace(EUROPE));
    const row = platformAdminPage.locator(`tr[data-group-id="${target}"]`);
    await expect(row).toContainText(/not granted/i);

    await row.locator('[data-action="marketplace-assign"]').click();
    await platformAdminPage.waitForLoadState('networkidle');
    await expect(
      platformAdminPage.locator(`tr[data-group-id="${target}"]`),
    ).toContainText(/granted/i);

    await platformAdminPage
      .locator(`tr[data-group-id="${target}"] [data-action="marketplace-unassign"]`)
      .click();
    const page = new BasePage(platformAdminPage, PATHS.marketplace(EUROPE));
    await page.confirm();
    await platformAdminPage.waitForLoadState('networkidle');
    await expect(
      platformAdminPage.locator(`tr[data-group-id="${target}"]`),
    ).toContainText(/not granted/i);
  });
});

test.describe('authorization', () => {
  for (const { principal, status } of ACCESS) {
    test(`${principal} gets ${status}`, async ({ browser }) => {
      // AUTH, not a path built from the principal name: the files are
      // kebab-case, and a template literal silently yields an anonymous
      // context for every multi-word principal — which reads as the page
      // refusing them, exactly the pass this block exists to disprove.
      const context = await browser.newContext(
        principal === 'anon' ? {} : { storageState: AUTH[principal] },
      );
      const res = await context.request.get(PATH, { maxRedirects: 0 });
      expect(res.status()).toBe(status);
      await context.close();
    });
  }

  test('the catalog is not readable without a session', async ({ anonPage }) => {
    await anonPage.goto(PATHS.plugins);
    await expect(anonPage).toHaveURL(/login/);
    await expect(anonPage.getByText('astound-dev')).toHaveCount(0);
  });

  test('bridge plugin files still require a token', async ({ request }) => {
    const response = await request.get(
      '/v1/bridge/plugins/astound-dev/.claude-plugin/plugin.json',
      { failOnStatusCode: false },
    );
    expect([401, 403]).toContain(response.status());
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

  test('shows an empty state when nothing matches', async ({ adminPage }) => {
    const page = new BasePage(adminPage, PATH);
    await page.goto();
    await page.table().filter('zzz-no-such-marketplace-zzz');
    await page.table().expectEmpty();
  });

  test('keeps a visible focus ring', async ({ adminPage }) => {
    const page = new BasePage(adminPage, PATH);
    await page.goto();
    await adminPage.keyboard.press('Tab');
    if (await adminPage.locator('.sp-skip-link:focus').count()) {
      await adminPage.keyboard.press('Tab');
    }
    const ring = await adminPage.evaluate(() => {
      const el = document.activeElement;
      if (!el || el === document.body) return null;
      const s = getComputedStyle(el);
      return { outline: `${s.outlineStyle} ${s.outlineWidth}`, shadow: s.boxShadow, tag: el.tagName };
    });
    expect(ring, 'nothing took focus on Tab').not.toBeNull();
    const hasOutline =
      !ring?.outline.startsWith('none') && Number.parseFloat(ring?.outline.split(' ')[1] ?? '0') > 0;
    const hasShadow = !!ring?.shadow && ring.shadow !== 'none';
    expect(hasOutline || hasShadow, `no focus ring on <${ring?.tag}>: ${JSON.stringify(ring)}`).toBe(
      true,
    );
  });

  test('does not scroll horizontally at 1440 or 1024', async ({ adminPage }) => {
    const page = new BasePage(adminPage, PATH);
    for (const width of [1440, 1024]) {
      await adminPage.setViewportSize({ width, height: 900 });
      await page.goto();
      expect(await page.hasHorizontalScroll(), `horizontal scroll at ${width}px`).toBe(false);
    }
  });

  test('meets the density bar above the fold', async ({ adminPage }) => {
    const page = new BasePage(adminPage, PATH);
    await adminPage.setViewportSize({ width: 1440, height: 900 });
    await page.goto();
    await expectDensity(adminPage, 'list');
  });


  test('the listing looks like itself', async ({ adminPage }) => {
    const page = new BasePage(adminPage, PATH);
    await page.goto();
    await snapshot(adminPage, 'marketplaces');
  });

  test('one marketplace looks like itself', async ({ adminPage }) => {
    const page = new BasePage(adminPage, PATHS.marketplace('astound-europe-dev'));
    await page.goto();
    await snapshot(adminPage, 'marketplace-detail');
  });

  test('has no serious or critical axe violations', async ({ adminPage }) => {
    const page = new BasePage(adminPage, PATH);
    await page.goto();
    await expect(adminPage.locator(SEL.pageHeader)).toBeVisible();
    await expectAccessible(adminPage);
  });
});
