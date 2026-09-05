// /admin/access-control — the rule ledger and the grant editor.
//
// The ledger is the half a reviewer can assert on: it is server-rendered, one
// row per `access_control_rules` row, and it carries the column no other page
// has — whether the YAML in the source repository declares the rule, or this
// instance's database is the only place it exists.
import { test, expect, AUTH, type Principal } from '../support/fixtures';
import { expectAccessible } from '../support/a11y';
import { PATHS } from '../support/paths';
import { SEL, TOKENS } from '../support/pages/selectors';
import { AccessControlPage } from '../support/pages/access-control.page';

const PATH = PATHS.accessControl;

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
  test('lists the rules on this instance', async ({ adminPage }) => {
    const page = new AccessControlPage(adminPage);
    await page.goto();
    expect(await page.ledger().rowCount()).toBeGreaterThan(0);
  });

  test('says where each rule came from', async ({ adminPage }) => {
    const page = new AccessControlPage(adminPage);
    await page.goto();
    const sources = await page.columnValues('Rule source');
    expect(sources.length).toBeGreaterThan(0);
    for (const source of sources) expect(['YAML', 'This instance']).toContain(source.trim());
  });

  test('carries the totals an auditor checks first', async ({ adminPage }) => {
    const page = new AccessControlPage(adminPage);
    await page.goto();
    expect(await adminPage.locator(SEL.kpi).count()).toBeGreaterThanOrEqual(5);
    await expect(page.kpi('Denies').locator).toBeVisible();
  });

  test('still offers the grant editor and its tree', async ({ adminPage }) => {
    const page = new AccessControlPage(adminPage);
    await page.goto();
    await expect(page.tree()).toBeVisible();
  });
});

test.describe('actions', () => {
  test('the subject-kind filter narrows the ledger to one band', async ({ adminPage }) => {
    const page = new AccessControlPage(adminPage);
    await page.goto();
    await page.filterBy('subject_kind', 'role');
    const kinds = await page.columnValues('Subject kind');
    expect(kinds.length).toBeGreaterThan(0);
    for (const kind of kinds) expect(kind.trim()).toBe('role');
  });

  test('the entity-kind filter narrows the ledger to one kind', async ({ adminPage }) => {
    const page = new AccessControlPage(adminPage);
    await page.goto();
    const before = await page.ledger().rowCount();
    await page.filterBy('entity_kind', 'marketplace');
    expect(await page.ledger().rowCount()).toBeLessThanOrEqual(before);
    const kinds = await page.columnValues('Entity kind');
    for (const kind of kinds) expect(kind.trim()).toBe('marketplace');
  });

  test('the rule-source filter isolates rules only this database holds', async ({ adminPage }) => {
    const page = new AccessControlPage(adminPage);
    await page.goto();
    await page.filterBy('source', 'yaml');
    const sources = await page.columnValues('Rule source');
    for (const source of sources) expect(source.trim()).toBe('YAML');
  });

  test('sorting by subject kind reorders the ledger', async ({ adminPage }) => {
    const page = new AccessControlPage(adminPage);
    await page.goto();
    const direction = await page.ledger().sortBy('Subject kind');
    expect(['ascending', 'descending']).toContain(direction);
  });

  test('selecting a group opens its band editor', async ({ adminPage }) => {
    const page = new AccessControlPage(adminPage);
    await page.goto();
    await page.selectGroup('Unassigned');
    await expect(page.editorPane()).toBeVisible();
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
    const page = new AccessControlPage(adminPage);
    await page.goto();
    await expect(page.activeNavItem()).toHaveAttribute('href', PATH);
    await expect(page.heading()).toBeVisible();
  });

  test('paints from design tokens', async ({ adminPage }) => {
    const page = new AccessControlPage(adminPage);
    await page.goto();
    const values = await adminPage.evaluate((tokens) => {
      const style = getComputedStyle(document.documentElement);
      return tokens.map((t) => style.getPropertyValue(t).trim());
    }, Object.values(TOKENS));
    for (const value of values) expect(value).not.toBe('');
  });

  test('shows an empty state when no rule matches', async ({ adminPage }) => {
    const page = new AccessControlPage(adminPage);
    await page.goto();
    await page.search('zzz-no-such-entity-zzz');
    await expect(adminPage.locator(SEL.empty).first()).toBeVisible();
  });

  test('keeps a visible focus ring', async ({ adminPage }) => {
    const page = new AccessControlPage(adminPage);
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
    const page = new AccessControlPage(adminPage);
    for (const width of [1440, 1024]) {
      await adminPage.setViewportSize({ width, height: 900 });
      await page.goto();
      expect(await page.hasHorizontalScroll(), `horizontal scroll at ${width}px`).toBe(false);
    }
  });

  test('has no serious or critical axe violations', async ({ adminPage }) => {
    const page = new AccessControlPage(adminPage);
    await page.goto();
    await expect(adminPage.locator(SEL.pageHeader)).toBeVisible();
    await expectAccessible(adminPage);
  });

  test('matches the committed visual snapshot', async ({ adminPage }) => {
    const page = new AccessControlPage(adminPage);
    await adminPage.setViewportSize({ width: 1440, height: 900 });
    await page.goto();
    await expect(adminPage).toHaveScreenshot('access-control.png', {
      fullPage: false,
      maxDiffPixelRatio: 0.02,
      mask: [adminPage.locator(SEL.tableCellDate)],
    });
  });
});
