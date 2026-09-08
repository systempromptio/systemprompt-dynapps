// The roster (/admin/users) and the user detail page (/admin/users/{id}).
//
// This spec absorbs the retired tests/users-roster.spec.ts,
// tests/user-membership.spec.ts and tests/requirements/req-001-user-management.spec.ts:
// the roster stopped bucketing by group, membership moved onto a tab, and the
// per-user analytics page folded into the Usage tab, so all three were driving
// markup that no longer exists. Every @REQ-001 title is kept alive here.
import { AUTH, snapshot, test, expect, apiAs, uniqueEmail, type Principal } from '../support/fixtures';
import { expectAccessible, expectDensity } from '../support/a11y';
import { PATHS } from '../support/paths';
import { SEL, TOKENS } from '../support/pages/selectors';
import { UsersPage, UserDetailPage } from '../support/pages/users.page';

const PATH = PATHS.users;

// Every principal the suite mints, and the status this page returns to each.
// 200 = renders, 303 = redirected to a page they may see, 307 = sent to sign in.
// These are the literal values `tests/contract/admin/baseline.txt` pins.
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
  test('lists the seeded people with their membership and spend', async ({ adminPage }) => {
    const page = new UsersPage(adminPage);
    await page.goto();
    expect(await page.table().rowCount()).toBeGreaterThan(0);
    const ids = await page.userIds();
    expect(ids).toContain('e2e-member-1');
    // The chip shows the group's name and links by its id, so the id is
    // asserted on the href rather than on the text.
    await expect(
      adminPage.locator('tr[data-user-id="e2e-member-1"] a[href*="group=europe-devs"]'),
    ).toBeVisible();
  });

  test('states the decision-relevant totals above the fold', async ({ adminPage }) => {
    const page = new UsersPage(adminPage);
    await page.goto();
    for (const label of ['Accounts', 'Active', 'Unassigned', 'No role', 'Idle 30d', '30d cost']) {
      await expect(page.kpi(label).locator, `KPI "${label}"`).toBeVisible();
    }
    expect(await page.kpi('Accounts').number()).toBeGreaterThan(0);
  });

  test('pages at fifty rows and says so', async ({ adminPage }) => {
    const page = new UsersPage(adminPage);
    await page.goto();
    expect(await page.table().rowCount()).toBeLessThanOrEqual(50);
    await expect(adminPage.locator(SEL.paginationInfo)).toContainText(/of \d+ users/);
  });

  test('@REQ-001 the roster is reachable from the sidebar, not just a breadcrumb', async ({
    adminPage: page,
  }) => {
    await page.goto(PATHS.analytics);
    const link = page.locator(`nav[data-nav] a[href="${PATHS.users}"]`).first();
    await expect(link).toBeVisible();
    await link.click();
    await expect(page).toHaveURL(new RegExp(`${PATHS.users}$`));
  });
});

test.describe('actions', () => {
  test('@REQ-001 search filters the roster', async ({ adminPage }) => {
    const page = new UsersPage(adminPage);
    await page.goto();
    await page.table().filter('e2e-member-1');
    const ids = await page.userIds();
    expect(ids).toContain('e2e-member-1');
    expect(ids).not.toContain('e2e-member-3');
  });

  // The retired Unassigned page redirects here, so this filter has to work or
  // that redirect lands on a page answering a different question.
  test('the unassigned chip is what the old Unassigned page redirects to', async ({
    adminPage,
  }) => {
    const page = new UsersPage(adminPage);
    await adminPage.goto(PATHS.unassigned);
    expect(await page.activeChip()).toBe('Unassigned');
    const ids = await page.userIds();
    expect(ids).toContain('e2e-unassigned');
    expect(ids).not.toContain('e2e-member-1');
  });

  test('every column sorts, and sorting keeps the filter', async ({ adminPage }) => {
    const page = new UsersPage(adminPage);
    await page.goto();
    await page.applyChip('Unassigned');
    const direction = await page.sortBy('Email');
    expect(direction).toBe('descending');
    expect(await page.activeChip()).toBe('Unassigned');
  });

  test('the role select narrows the roster', async ({ adminPage }) => {
    const page = new UsersPage(adminPage);
    await page.goto();
    await page.filterByRole('admin');
    const ids = await page.userIds();
    expect(ids.length).toBeGreaterThan(0);
    expect(ids).toContain('e2e-admin');
    expect(ids).not.toContain('e2e-member-1');
  });

  test('creates a user from the page header', async ({ adminPage, request }) => {
    const email = uniqueEmail('created');
    const page = new UsersPage(adminPage);
    await page.goto();
    await page.createButton().click();

    const panel = adminPage.locator('#create-user-panel');
    await expect(panel).toBeVisible();
    await adminPage.locator('#new-user-id').fill(email.split('@')[0]);
    await adminPage.locator('#new-user-email').fill(email);
    await adminPage.locator('#new-user-name').fill('E2E Created');
    await panel.locator('[data-action="save"]').click();

    await expect
      .poll(async () => {
        const r = await request.get(
          `/api/public/admin/users/search?q=${encodeURIComponent(email)}`,
          apiAs(request, 'admin'),
        );
        return r.ok() ? ((await r.json()) as { users: unknown[] }).users.length : 0;
      })
      .toBeGreaterThan(0);

    const list = await request.get(
      `/api/public/admin/users/search?q=${encodeURIComponent(email)}`,
      apiAs(request, 'admin'),
    );
    const found = ((await list.json()) as { users: { id: string }[] }).users;
    await request.delete(`/api/public/admin/users/${found[0].id}`, apiAs(request, 'admin'));
  });

  test('selecting rows arms the bulk role editor', async ({ adminPage }) => {
    const page = new UsersPage(adminPage);
    await page.goto();
    await expect(page.bulkRolesButton()).toBeDisabled();
    const ids = await page.userIds();
    await page.rowCheckbox(ids[0]).check();
    await expect(page.bulkRolesButton()).toBeEnabled();
    await expect(page.bulkRolesButton()).toHaveText(/\(1\)/);
  });

  test('a row opens the person', async ({ adminPage }) => {
    const page = new UsersPage(adminPage);
    await page.goto();
    await page.applyChip('All');
    await page.table().filter('e2e-member-1');
    const detail = await page.openUser('e2e-member-1');
    await expect(adminPage).toHaveURL(/\/admin\/users\/e2e-member-1/);
    await expect(detail.activeTab()).toHaveText(/identity/i);
  });

  test('@REQ-001 a session can be listed and revoked from the UI', async ({
    platformAdminPage,
    request,
  }) => {
    const target = 'e2e-member-1';
    const detail = new UserDetailPage(platformAdminPage, target);
    await detail.openTab('sessions');
    const rows = platformAdminPage.locator('tr[data-session-id]');
    await expect(rows.first()).toBeVisible();

    await platformAdminPage.locator('[data-revoke-session]').first().click();
    await detail.confirm();
    await expect(platformAdminPage.locator(SEL.badge).filter({ hasText: 'Revoked' }).first())
      .toBeVisible({ timeout: 10_000 });

    const resp = await request.get(
      `/api/public/admin/users/${target}/sessions`,
      apiAs(request, 'platformAdmin'),
    );
    const body = (await resp.json()) as { sessions: { revoked_at: string | null }[] };
    expect(body.sessions.some((s) => s.revoked_at !== null)).toBeTruthy();
  });

  test('@REQ-001 "sign out everywhere" leaves no live session', async ({ request }) => {
    const target = 'e2e-member-2';
    const resp = await request.delete(
      `/api/public/admin/users/${target}/sessions`,
      apiAs(request, 'platformAdmin'),
    );
    expect(resp.ok()).toBeTruthy();
    const after = await request.get(
      `/api/public/admin/users/${target}/sessions`,
      apiAs(request, 'platformAdmin'),
    );
    const body = (await after.json()) as { sessions: { revoked_at: string | null }[] };
    expect(body.sessions.every((s) => s.revoked_at !== null)).toBeTruthy();
  });

  // State-agnostic on purpose: the suite shares one database with every other
  // spec, and this page's saves reload the page themselves, so the test reads
  // the membership it finds, flips it, and puts back exactly what was there.
  test('the membership tab moves a group and puts it back', async ({ platformAdminPage }) => {
    const detail = new UserDetailPage(platformAdminPage, 'e2e-victim');
    await detail.openTab('membership');
    const box = platformAdminPage.locator('input[data-membership-id="europe-devs"]');
    await expect(box).toBeVisible();
    const before = await box.isChecked();

    await box.setChecked(!before);
    await expect(detail.status('membership')).toHaveText(/saved/i, { timeout: 10_000 });
    await expect(platformAdminPage.locator('input[data-membership-id="europe-devs"]'))
      .toBeChecked({ checked: !before, timeout: 15_000 });

    await platformAdminPage.locator('input[data-membership-id="europe-devs"]').setChecked(before);
    await expect(platformAdminPage.locator('input[data-membership-id="europe-devs"]'))
      .toBeChecked({ checked: before, timeout: 15_000 });
  });

  // The primary group is the key exclusive attribution counts by, so this is
  // the one control that can move a person's spend between group pages.
  test('the primary group editor pins attribution', async ({ platformAdminPage }) => {
    const detail = new UserDetailPage(platformAdminPage, 'e2e-member-1');
    await detail.openTab('membership');
    const select = detail.primarySelect('group');
    await expect(select).toBeVisible();

    // Only a container the person belongs to may be their primary one, so the
    // options are their memberships plus the explicit "no primary group".
    const options = await select.locator('option').evaluateAll((els) =>
      els.map((el) => (el as HTMLOptionElement).value),
    );
    const target = options.find((value) => value !== '');
    expect(target, 'e2e-member-1 belongs to no group, so nothing can be primary').toBeTruthy();

    await select.selectOption(target as string);
    await detail.saveAttribution();
    await expect(platformAdminPage.locator('select[name="primary_group_id"]'))
      .toHaveValue(target as string, { timeout: 15_000 });
    // Saving pins the choice against the automatic recomputation.
    await expect(platformAdminPage.locator('[data-form="scope-defaults"]')).toContainText('manual');
  });

  test('@REQ-001 an org admin cannot escalate a user to platform_admin', async ({ request }) => {
    const resp = await request.put('/api/public/admin/users/e2e-member-1/roles', {
      ...apiAs(request, 'admin'),
      data: { roles: ['user', 'platform_admin'] },
    });
    expect(resp.status()).toBe(403);
    const after = await request.get(
      '/api/public/admin/users/e2e-member-1/roles',
      apiAs(request, 'platformAdmin'),
    );
    expect(await after.text()).not.toContain('platform_admin');
  });

  test('the access tab resolves one table per entity kind', async ({ adminPage }) => {
    const detail = new UserDetailPage(adminPage, 'e2e-developer');
    await detail.openTab('access');
    await expect(detail.activeTab()).toHaveText(/access/i);
    const pane = adminPage.locator('[data-access]');
    await expect(pane).toBeVisible();
    expect(await pane.locator('.sp-table').count()).toBeGreaterThan(0);
    // Every row carries a resolved decision and names the band that made it.
    const first = pane.locator('tbody tr').first();
    await expect(first.locator('.sp-badge')).toHaveText(/allow|deny|warn|pending/);
    await expect(first.locator('.sp-access-layer')).not.toBeEmpty();
  });

  test('a project manager reads the access tab without a write control', async ({ pmPage }) => {
    const detail = new UserDetailPage(pmPage, 'e2e-developer');
    await detail.openTab('access');
    const selects = pmPage.locator('[data-action="set-user-rule"]');
    expect(await selects.count()).toBeGreaterThan(0);
    for (const select of await selects.all()) await expect(select).toBeDisabled();
  });

  // Allow then Inherit: the second write must remove only this one rule, not
  // every override of the kind, which is what the old editor's bulk clear did.
  test('an override set and then inherited leaves the row on inherit', async ({ platformAdminPage }) => {
    const detail = new UserDetailPage(platformAdminPage, 'e2e-member-1');
    await detail.openTab('access');
    const row = platformAdminPage.locator('[data-access] tbody tr').first();
    const entity = await row.getAttribute('data-entity-id');
    const select = row.locator('[data-action="set-user-rule"]');
    await select.selectOption('allow');
    await platformAdminPage.waitForLoadState('networkidle');
    const after = platformAdminPage.locator(`[data-access] tr[data-entity-id="${entity}"]`);
    await expect(after).toHaveAttribute('data-rule-id', /.+/, { timeout: 15_000 });
    await after.locator('[data-action="set-user-rule"]').selectOption('inherit');
    await platformAdminPage.waitForLoadState('networkidle');
    await expect(platformAdminPage.locator(`[data-access] tr[data-entity-id="${entity}"]`))
      .toHaveAttribute('data-rule-id', '', { timeout: 15_000 });
  });

  test('the usage tab is where the per-user analytics page went', async ({ adminPage }) => {
    const detail = new UserDetailPage(adminPage, 'e2e-member-1');
    await detail.openTab('usage');
    await expect(detail.activeTab()).toHaveText(/usage/i);
    expect(await detail.kpi('Requests').number()).toBeGreaterThan(0);
    await expect(adminPage.getByRole('link', { name: /request log/i }).first()).toBeVisible();
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

  test('a plain user may open their own page and nobody else\'s', async ({ browser }) => {
    const context = await browser.newContext({ storageState: AUTH.user });
    const own = await context.request.get(PATHS.user('e2e-user'), { maxRedirects: 0 });
    expect([200, 303]).toContain(own.status());
    const other = await context.request.get(PATHS.user('e2e-member-1'), { maxRedirects: 0 });
    expect([303, 403]).toContain(other.status());
    await context.close();
  });

  test('@REQ-001 a non-admin reaches none of it', async ({ request }) => {
    const resp = await request.get(
      '/api/public/admin/users/e2e-member-1/sessions',
      apiAs(request, 'user'),
    );
    expect(resp.status()).toBe(403);
  });

  test('an unknown user is a 404, not a 500', async ({ browser }) => {
    const context = await browser.newContext({ storageState: AUTH.admin });
    const res = await context.request.get(PATHS.user('no-such-account'), { maxRedirects: 0 });
    expect(res.status()).toBe(404);
    await context.close();
  });
});

test.describe('design language', () => {
  test('wears the shell', async ({ adminPage }) => {
    const page = new UsersPage(adminPage);
    await page.goto();
    await expect(page.activeNavItem()).toHaveAttribute('href', PATH);
    await expect(page.heading()).toBeVisible();
  });

  test('the detail page carries breadcrumbs and tabs', async ({ adminPage }) => {
    const detail = new UserDetailPage(adminPage, 'e2e-member-1');
    await detail.openTab('identity');
    await expect(detail.breadcrumb()).toBeVisible();
    await expect(detail.tabs()).toHaveCount(5);
  });

  test('paints from design tokens', async ({ adminPage }) => {
    const page = new UsersPage(adminPage);
    await page.goto();
    const values = await adminPage.evaluate((tokens) => {
      const style = getComputedStyle(document.documentElement);
      return tokens.map((t) => style.getPropertyValue(t).trim());
    }, Object.values(TOKENS));
    for (const value of values) expect(value).not.toBe('');
  });

  test('shows an empty state when nothing matches', async ({ adminPage }) => {
    const page = new UsersPage(adminPage);
    await page.goto();
    await page.table().filter('zzz-no-such-row-zzz');
    await expect(adminPage.locator(SEL.empty)).toBeVisible();
  });

  test('keeps a visible focus ring', async ({ adminPage }) => {
    const page = new UsersPage(adminPage);
    await page.goto();
    await adminPage.keyboard.press('Tab');
    // The shipped design system draws the ring with box-shadow, and keeps a
    // non-zero outline-width for forced-colors mode. Either mechanism is a
    // visible ring; asserting on outline-style alone reads as absent.
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
    const page = new UsersPage(adminPage);
    for (const width of [1440, 1024]) {
      await adminPage.setViewportSize({ width, height: 900 });
      await page.goto();
      expect(await page.hasHorizontalScroll(), `horizontal scroll at ${width}px`).toBe(false);
    }
  });

  test('has no serious or critical axe violations', async ({ adminPage }) => {
    const page = new UsersPage(adminPage);
    await page.goto();
    await expect(adminPage.locator(SEL.pageHeader)).toBeVisible();
    await expectAccessible(adminPage);
  });

  test('holds the density bar', async ({ adminPage }) => {
    const page = new UsersPage(adminPage);
    await page.goto();
    await expectDensity(adminPage, 'list');
  });

  // Measured on Sessions rather than Identity: the bar counts rows above the
  // fold, and Identity's body is the profile and role editors with a short
  // reference table under them. Sessions is where this page is a list.
  test('the detail page holds the density bar', async ({ adminPage }) => {
    const detail = new UserDetailPage(adminPage, 'e2e-member-1');
    await detail.openTab('sessions');
    await expectDensity(adminPage, 'detail');
  });

  test('the roster looks like itself', async ({ adminPage }) => {
    const page = new UsersPage(adminPage);
    await page.goto();
    await snapshot(adminPage, 'users');
  });

  // One baseline per tab: each renders a different body from a different
  // query, so a single identity shot would leave five of the six unwatched.
  for (const tab of ['identity', 'membership', 'access', 'devices', 'sessions', 'usage']) {
    test(`the ${tab} tab looks like itself`, async ({ adminPage }) => {
      const detail = new UserDetailPage(adminPage, 'e2e-member-1');
      await detail.openTab(tab);
      await snapshot(adminPage, `user-detail-${tab}`);
    });
  }

  test('the detail page has no serious or critical axe violations', async ({ adminPage }) => {
    const detail = new UserDetailPage(adminPage, 'e2e-member-1');
    for (const tab of ['identity', 'membership', 'devices', 'sessions', 'usage']) {
      await detail.openTab(tab);
      await expectAccessible(adminPage);
    }
  });
});
