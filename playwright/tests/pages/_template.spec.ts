// The shape every page spec follows. This file is not run (the chromium
// project ignores `_*.spec.ts`); copy it when adding a page.
//
// Four describe blocks, always these four names, in this order.
// scripts/check-spec-shape.sh fails any tests/pages/*.spec.ts missing one.
//
//   renders          the page draws its own content from the seeded dataset
//   actions          every mutation the page offers, and its visible result
//   authorization    all seven principals plus anonymous, with literal statuses
//   design language  the shell, the tokens, the empty state, focus, reflow, axe
//
// The authorization block is a table, not seven hand-written tests: a literal
// expected status per principal is the only form in which a reviewer can see
// at a glance that the matrix is complete and that nothing was quietly relaxed.
import { test, expect, type Principal } from '../support/fixtures';
import { expectAccessible } from '../support/a11y';
import { PATHS } from '../support/paths';
import { SEL, TOKEN_PREFIX } from '../support/pages/selectors';
import { BasePage } from '../support/pages/base.page';

const PATH = PATHS.users;

// Every principal the suite mints, and the status this page returns to each.
// 200 = renders, 303 = redirected to a page they may see, 401/403 = refused.
const ACCESS: { principal: Principal | 'anon'; status: number }[] = [
  { principal: 'platformAdmin', status: 200 },
  { principal: 'admin', status: 200 },
  { principal: 'pm', status: 200 },
  { principal: 'developer', status: 303 },
  { principal: 'businessUser', status: 303 },
  { principal: 'unassigned', status: 303 },
  { principal: 'user', status: 303 },
  { principal: 'anon', status: 303 },
];

test.describe('renders', () => {
  test('lists the seeded people', async ({ adminPage }) => {
    const page = new BasePage(adminPage, PATH);
    await page.goto();
    expect(await page.table().rowCount()).toBeGreaterThan(0);
  });
});

test.describe('actions', () => {
  test('filtering narrows the table', async ({ adminPage }) => {
    const page = new BasePage(adminPage, PATH);
    await page.goto();
    const before = await page.table().rowCount();
    await page.table().filter('e2e-developer');
    expect(await page.table().rowCount()).toBeLessThan(before);
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
    const declared = await adminPage.evaluate((prefix) => {
      const style = getComputedStyle(document.documentElement);
      return Array.from(style).filter((p) => p.startsWith(prefix)).length;
    }, TOKEN_PREFIX);
    expect(declared).toBeGreaterThan(0);
  });

  test('shows an empty state when nothing matches', async ({ adminPage }) => {
    const page = new BasePage(adminPage, PATH);
    await page.goto();
    await page.table().filter('zzz-no-such-row-zzz');
    await page.table().expectEmpty();
  });

  test('keeps a visible focus ring', async ({ adminPage }) => {
    const page = new BasePage(adminPage, PATH);
    await page.goto();
    await adminPage.keyboard.press('Tab');
    const outline = await adminPage.evaluate(() => {
      const el = document.activeElement;
      if (!el) return '';
      const s = getComputedStyle(el);
      return `${s.outlineStyle} ${s.outlineWidth} ${s.boxShadow}`;
    });
    expect(outline).not.toMatch(/^none 0px none$/);
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
    await expect(adminPage.locator(SEL.shell)).toBeVisible();
    await expectAccessible(adminPage);
  });
});
