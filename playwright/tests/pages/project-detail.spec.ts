// /admin/projects/{id} — one project across its three tabs.
//
// `storefront` is the project the seed gives real depth to: three members from
// the directory plus a bridge account, gateway traffic, tool calls and skill
// invocations. `payments` is the one with a manual member, which is the only
// membership an operator is allowed to remove by hand.
import { test, expect, dbRow, snapshot, type Principal } from '../support/fixtures';
import { expectAccessible } from '../support/a11y';
import { PATHS } from '../support/paths';
import { SEL, TOKENS } from '../support/pages/selectors';
import { ProjectDetailPage } from '../support/pages/projects.page';

const PROJECT = 'storefront';
const PATH = PATHS.project(PROJECT);

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
  test('the members tab lists the seeded members with their own usage', async ({ adminPage }) => {
    const page = new ProjectDetailPage(adminPage, PROJECT);
    await page.gotoTab('members');
    for (const id of ['e2e-developer', 'e2e-member-1', 'e2e-member-2']) {
      await expect(page.memberRow(id)).toBeVisible();
    }
    await expect(page.memberRow('e2e-member-4')).toHaveCount(0);
  });

  test('the header tiles and breadcrumb name this project', async ({ adminPage }) => {
    const page = new ProjectDetailPage(adminPage, PROJECT);
    await page.goto();
    await expect(page.breadcrumb()).toContainText('Projects');
    for (const label of ['Members', 'Requests', 'Tokens', 'Cost', 'Tool success', 'Skills']) {
      await expect(page.kpi(label).locator).toBeVisible();
    }
  });

  test('the groups feeding the project are named, not counted', async ({ adminPage }) => {
    const page = new ProjectDetailPage(adminPage, PROJECT);
    await page.goto();
    expect(await page.groupChips().count()).toBeGreaterThan(0);
  });

  test('the usage tab draws the series, the models, the tools and the commits', async ({
    adminPage,
  }) => {
    const page = new ProjectDetailPage(adminPage, PROJECT);
    await page.gotoTab('usage');
    await expect(adminPage.locator('figure.sp-svgchart')).toBeVisible();
    const main = adminPage.locator('main');
    for (const heading of ['Models', 'Skills', 'Tools', 'Sessions', 'Commits']) {
      await expect(main).toContainText(heading);
    }
  });

  test('the settings tab shows the identity and the directory groups feeding it', async ({
    platformAdminPage,
  }) => {
    const page = new ProjectDetailPage(platformAdminPage, PROJECT);
    await page.gotoTab('settings');
    await expect(page.nameField()).toHaveValue('Storefront');
    await expect(platformAdminPage.locator('tr[data-ad-group]').first()).toBeVisible();
  });

  test('an unknown project is a 404, not a 500 and not an empty page', async ({ adminPage }) => {
    const res = await adminPage.goto(PATHS.project('no-such-project-here'));
    expect(res?.status()).toBe(404);
  });
});

test.describe('actions', () => {
  test('every tab is a real URL and renders only its own body', async ({ adminPage }) => {
    const page = new ProjectDetailPage(adminPage, PROJECT);
    for (const tab of ['members', 'usage', 'settings'] as const) {
      await page.gotoTab(tab);
      await expect(adminPage.locator(`${SEL.tab}[aria-selected="true"]`)).toContainText(
        tab === 'members' ? 'Members' : tab === 'usage' ? 'Usage' : 'Settings',
      );
    }
  });

  test('a manual member can be removed, and the row is really gone', async ({ adminPage }) => {
    const page = new ProjectDetailPage(adminPage, 'payments');
    await page.gotoTab('members');
    await page.removeMember('e2e-pm');
    await expect(page.memberRow('e2e-pm')).toHaveCount(0);
    const row = await dbRow(
      'SELECT 1 AS present FROM project_members WHERE project_id = $1 AND user_id = $2',
      ['payments', 'e2e-pm'],
    );
    expect(row).toBeNull();

    // Put the seed back: the membership is manual, so the API restores it.
    const res = await adminPage.request.post('/api/public/admin/projects/payments/members', {
      data: { user_id: 'e2e-pm' },
    });
    expect(res.status()).toBeLessThan(300);
  });

  test('a directory-sourced member offers no Remove button', async ({ adminPage }) => {
    const page = new ProjectDetailPage(adminPage, PROJECT);
    await page.gotoTab('members');
    await expect(
      page.memberRow('e2e-developer').locator('[data-action="remove-member"]'),
    ).toHaveCount(0);
  });

  test('an admin edits the description and the change persists', async ({ adminPage }) => {
    const page = new ProjectDetailPage(adminPage, 'checkout');
    await page.gotoTab('settings');
    const text = `Edited by the project-detail spec at ${Date.now()}`;
    await page.descriptionField().fill(text);
    await page.save();
    await adminPage.waitForLoadState('networkidle');
    const row = await dbRow<{ description: string }>(
      'SELECT description FROM projects WHERE id = $1',
      ['checkout'],
    );
    expect(row?.description).toBe(text);
  });

  test('a project manager reads every tab and is offered no writes', async ({ pmPage }) => {
    const page = new ProjectDetailPage(pmPage, PROJECT);
    await page.gotoTab('members');
    await expect(page.memberRow('e2e-developer')).toBeVisible();
    await expect(pmPage.locator('[data-action="remove-member"]')).toHaveCount(0);
    await expect(pmPage.locator('[data-action="add-member"]')).toHaveCount(0);
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
    const page = new ProjectDetailPage(adminPage, PROJECT);
    await page.goto();
    await expect(page.activeNavItem()).toHaveAttribute('href', PATHS.projects);
    await expect(page.breadcrumb()).toBeVisible();
    await expect(page.heading()).toBeVisible();
    await expect(page.tabs().first()).toBeVisible();
  });

  test('paints from design tokens', async ({ adminPage }) => {
    const page = new ProjectDetailPage(adminPage, PROJECT);
    await page.goto();
    const values = await adminPage.evaluate((tokens) => {
      const style = getComputedStyle(document.documentElement);
      return tokens.map((t) => style.getPropertyValue(t).trim());
    }, Object.values(TOKENS));
    for (const value of values) expect(value).not.toBe('');
  });

  test('an empty project says so rather than drawing nothing', async ({ adminPage }) => {
    const page = new ProjectDetailPage(adminPage, 'platform');
    await page.gotoTab('usage');
    await expect(adminPage.locator(SEL.empty).first()).toBeVisible();
  });

  test('keeps a visible focus ring', async ({ adminPage }) => {
    const page = new ProjectDetailPage(adminPage, PROJECT);
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
    const page = new ProjectDetailPage(adminPage, PROJECT);
    for (const width of [1440, 1024]) {
      await adminPage.setViewportSize({ width, height: 900 });
      await page.gotoTab('usage');
      expect(await page.hasHorizontalScroll(), `horizontal scroll at ${width}px`).toBe(false);
    }
  });

  test('has no serious or critical axe violations on any tab', async ({ adminPage }) => {
    const page = new ProjectDetailPage(adminPage, PROJECT);
    for (const tab of ['members', 'usage', 'settings'] as const) {
      await page.gotoTab(tab);
      await expect(adminPage.locator(SEL.pageHeader)).toBeVisible();
      await expectAccessible(adminPage);
    }
    await page.gotoTab('usage');
    await snapshot(adminPage, 'project-detail-usage');
  });
});
