// /admin/projects — the work-attribution listing.
//
// The seeded instance carries four projects (storefront, checkout, payments,
// platform), so the row assertions below name them rather than counting: a
// count that drifts with the seed proves nothing, a named row proves the page
// read the database it claims to.
import { test, expect, apiAs, snapshot, type Principal } from '../support/fixtures';
import { expectAccessible } from '../support/a11y';
import { PATHS } from '../support/paths';
import { SEL, TOKENS } from '../support/pages/selectors';
import { ProjectsPage } from '../support/pages/projects.page';

const PATH = PATHS.projects;
const SEEDED = ['storefront', 'checkout', 'payments', 'platform'];

// Literal status per principal, matching tests/contract/admin/baseline.txt.
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
  test('one row per seeded project, each linking to its detail page', async ({ adminPage }) => {
    const page = new ProjectsPage(adminPage);
    await page.goto();
    for (const id of SEEDED) {
      const row = page.row(id);
      await expect(row).toBeVisible();
      await expect(row.locator('a').first()).toHaveAttribute('href', PATHS.project(id));
    }
  });

  test('the header tiles carry the decisions the page is read for', async ({ adminPage }) => {
    const page = new ProjectsPage(adminPage);
    await page.goto();
    for (const label of ['Projects', 'Members', 'Requests', 'Cost', 'Tool success', 'Unattributed']) {
      await expect(page.kpi(label).locator).toBeVisible();
    }
    expect(await page.kpi('Projects').number()).toBeGreaterThanOrEqual(SEEDED.length);
  });

  test('the row counts agree with the projects API', async ({ adminPage, request }) => {
    const res = await request.get('/api/public/admin/projects', apiAs(request, 'admin'));
    expect(res.status()).toBe(200);
    const body = (await res.json()) as { projects: { id: string }[] };
    const page = new ProjectsPage(adminPage);
    await page.goto();
    for (const p of body.projects) await expect(page.row(p.id)).toBeVisible();
  });

  test('the listing is paginated and says what it is showing', async ({ adminPage }) => {
    const page = new ProjectsPage(adminPage);
    await page.goto();
    await expect(adminPage.locator(SEL.pagination)).toBeVisible();
    await expect(adminPage.locator(SEL.paginationInfo)).toContainText('projects');
  });
});

test.describe('actions', () => {
  test('every column sorts, and says which way in aria-sort', async ({ adminPage }) => {
    const page = new ProjectsPage(adminPage);
    await page.goto();
    for (const column of ['Members', 'Requests', 'Cost', 'Skills']) {
      const direction = await page.table().sortBy(column);
      expect(direction).toBe('descending');
    }
  });

  test('sorting by cost puts the most expensive project first', async ({ adminPage }) => {
    const page = new ProjectsPage(adminPage);
    await page.goto({ sort: 'cost', dir: 'desc' });
    const costs = (await page.table().column('Cost')).map((c) => Number(c.replace(/[^0-9.]/g, '')));
    const sorted = [...costs].sort((a, b) => b - a);
    expect(costs).toEqual(sorted);
  });

  test('searching narrows the listing to the matching project', async ({ adminPage }) => {
    const page = new ProjectsPage(adminPage);
    await page.goto();
    await page.searchFor('storefront');
    await expect(page.row('storefront')).toBeVisible();
    await expect(page.row('payments')).toHaveCount(0);
  });

  test('an admin creates a project and lands on it', async ({ adminPage }) => {
    const id = `e2e-proj-${Date.now() % 100000}`;
    const page = new ProjectsPage(adminPage);
    await page.goto();
    await page.create(id, 'E2E scratch project', 'Created by the projects page spec.');
    await adminPage.waitForURL(PATHS.project(id));
    await expect(adminPage.locator('h1')).toContainText('E2E scratch project');

    // Clean up through the same API the page writes with, so a re-run starts
    // from the seeded four.
    const res = await adminPage.request.delete(`/api/public/admin/projects/${id}`);
    expect(res.status()).toBeLessThan(300);
  });

  test('a project manager reads the page and is offered no writes', async ({ pmPage }) => {
    const page = new ProjectsPage(pmPage);
    await page.goto();
    await expect(page.row('storefront')).toBeVisible();
    await expect(pmPage.locator('[data-action="new-project"]')).toHaveCount(0);
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
    const page = new ProjectsPage(adminPage);
    await page.goto();
    await expect(page.activeNavItem()).toHaveAttribute('href', PATH);
    await expect(page.heading()).toBeVisible();
    await expect(adminPage.locator(SEL.toolbar)).toBeVisible();
  });

  test('paints from design tokens', async ({ adminPage }) => {
    const page = new ProjectsPage(adminPage);
    await page.goto();
    const values = await adminPage.evaluate((tokens) => {
      const style = getComputedStyle(document.documentElement);
      return tokens.map((t) => style.getPropertyValue(t).trim());
    }, Object.values(TOKENS));
    for (const value of values) expect(value).not.toBe('');
  });

  test('shows an empty state when nothing matches', async ({ adminPage }) => {
    const page = new ProjectsPage(adminPage);
    await page.goto();
    await page.searchFor('zzz-no-such-project-zzz');
    await page.table().expectEmpty();
  });

  test('keeps a visible focus ring', async ({ adminPage }) => {
    const page = new ProjectsPage(adminPage);
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
    const page = new ProjectsPage(adminPage);
    for (const width of [1440, 1024]) {
      await adminPage.setViewportSize({ width, height: 900 });
      await page.goto();
      expect(await page.hasHorizontalScroll(), `horizontal scroll at ${width}px`).toBe(false);
    }
  });

  test('has no serious or critical axe violations', async ({ adminPage }) => {
    const page = new ProjectsPage(adminPage);
    await page.goto();
    await expect(adminPage.locator(SEL.pageHeader)).toBeVisible();
    await expectAccessible(adminPage);
    await snapshot(adminPage, 'projects');
  });
});
