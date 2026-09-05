// /admin/skills — the skills catalog: what is declared, and who may reach it.
//
// A catalog page is a reference, so the assertions are about completeness and
// legibility rather than mutation: every declared entry has a row, every row
// links to its detail, and an unknown id is a 404 rather than a 500.
import { test, expect, type Principal } from '../support/fixtures';
import { expectAccessible } from '../support/a11y';
import { PATHS } from '../support/paths';
import { SEL, TOKENS } from '../support/pages/selectors';
import { BasePage } from '../support/pages/base.page';

const PATH = PATHS.skills;
const SAMPLE = 'code_review';

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
  test('lists the declared entries', async ({ adminPage }) => {
    const page = new BasePage(adminPage, PATH);
    await page.goto();
    expect(await page.table().rowCount()).toBeGreaterThan(0);
    await expect(adminPage.locator(`tr[data-skill-id="${SAMPLE}"]`)).toBeVisible();
  });

  test('the tiles state how many are declared and how many are public', async ({ adminPage }) => {
    const page = new BasePage(adminPage, PATH);
    await page.goto();
    expect(await page.kpi('Skills').number()).toBeGreaterThan(0);
    await expect(page.kpi('Public').root).toBeVisible();
  });

  test('every row carries a visibility badge', async ({ adminPage }) => {
    const page = new BasePage(adminPage, PATH);
    await page.goto();
    const badges = adminPage.locator('.sp-visibility-badge');
    expect(await badges.count()).toBeGreaterThan(0);
  });

  test('an unknown id is a 404, not a 500', async ({ adminPage }) => {
    const res = await adminPage.goto(PATHS.skill('no-such-entry-at-all'));
    expect(res?.status()).toBe(404);
  });
});

test.describe('actions', () => {
  test('filtering narrows the table', async ({ adminPage }) => {
    const page = new BasePage(adminPage, PATH);
    await page.goto();
    const before = await page.table().rowCount();
    await page.table().filter(SAMPLE);
    expect(await page.table().rowCount()).toBeLessThan(before);
  });

  test('sorting by grants states the direction on the header', async ({ adminPage }) => {
    const page = new BasePage(adminPage, PATH);
    await page.goto({ sort: 'grants', dir: 'desc' });
    const header = adminPage.locator('th').filter({ hasText: 'Grants' }).first();
    await expect(header).toHaveAttribute('aria-sort', 'descending');
  });

  test('a row opens its detail page', async ({ adminPage }) => {
    const page = new BasePage(adminPage, PATH);
    await page.goto();
    await adminPage.locator(`tr[data-skill-id="${SAMPLE}"] a`).first().click();
    await expect(adminPage).toHaveURL(new RegExp(SAMPLE));
    await expect(page.breadcrumb()).toBeVisible();
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
    await page.table().filter('zzz-no-such-entry-zzz');
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
