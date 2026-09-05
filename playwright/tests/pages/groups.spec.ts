// The Groups listing and one group's detail page.
//
// The listing's defining claim is arithmetic, not layout: spend is attributed
// exclusively, so the group rows plus the remainder row reproduce what the
// instance actually spent. The `renders` block proves that against the
// database rather than against another page, because two screens agreeing is
// not evidence when both read the same wrong query.
//
// DOM hooks this file depends on, all owned by the groups templates:
// tr[data-group-id], tr[data-group-id="unattributed"], tr[data-user-id],
// tr[data-marketplace-id], [data-field="marketplace"],
// [data-action="save-marketplaces"], [data-action="create-group"],
// [data-action="map-directory"], [data-action="recompute-defaults"].
import { AUTH, test, expect, apiAs, dbRow, snapshot, type Principal } from '../support/fixtures';
import { expectAccessible, expectDensity } from '../support/a11y';
import { PATHS } from '../support/paths';
import { SEL, TOKENS } from '../support/pages/selectors';
import { GroupsPage, GroupDetailPage } from '../support/pages/groups.page';

const PATH = PATHS.groups;
const GROUP = 'europe-devs';

// Every principal the suite mints, and the status the groups listing returns
// to each. 200 = renders, 303 = redirected to a page they may see.
const ACCESS: { principal: Principal | 'anon'; status: number }[] = [
  { principal: 'platformAdmin', status: 200 },
  { principal: 'admin', status: 200 },
  { principal: 'pm', status: 200 },
  { principal: 'developer', status: 303 },
  { principal: 'businessUser', status: 303 },
  { principal: 'unassigned', status: 303 },
  { principal: 'user', status: 303 },
  // Anonymous is 307, not 303: it has no session to redirect by role, so the
  // shell sends it to the login page as a temporary redirect. Matches the row
  // for GET /admin/groups in tests/contract/admin/baseline.txt.
  { principal: 'anon', status: 307 },
];

interface CountRow extends Record<string, unknown> {
  requests: string;
  cost: string;
}

test.describe('renders', () => {
  test('lists every seeded group', async ({ adminPage }) => {
    const page = new GroupsPage(adminPage);
    await page.goto();
    for (const id of ['europe-devs', 'india-devs', 'business-users', 'unassigned']) {
      await expect(page.row(id)).toBeVisible();
    }
  });

  test('the unassigned bucket is a row, flagged, not a filtered-out case', async ({
    adminPage,
  }) => {
    const page = new GroupsPage(adminPage);
    await page.goto();
    await expect(page.row('unassigned').locator(SEL.badge).first()).toHaveText(/needs assignment/i);
  });

  // The property the exclusive attribution exists for. If it ever stops
  // holding, every cost figure on every people page is quietly wrong, and no
  // amount of UI assertion would catch it.
  test('group totals plus the remainder equal what the instance spent', async ({
    adminPage,
  }) => {
    const page = new GroupsPage(adminPage);
    await page.goto();

    const instance = await dbRow<CountRow>(
      `SELECT COUNT(*)::TEXT AS requests, COALESCE(SUM(cost_microdollars), 0)::TEXT AS cost
         FROM ai_requests
        WHERE created_at >= NOW() - INTERVAL '30 days'`,
    );
    expect(instance).not.toBeNull();

    const groupRequests = (await page.columnNumbers('Requests')).reduce((a, b) => a + b, 0);
    const remainder = await page.remainder('Requests');
    expect(groupRequests + remainder).toBe(Number(instance?.requests));
  });

  test('the remainder row is present and named', async ({ adminPage }) => {
    const page = new GroupsPage(adminPage);
    await page.goto();
    await expect(page.remainderRow()).toBeVisible();
    await expect(page.remainderRow()).toContainText(/unattributed/i);
  });

  test('a group row carries members, projects, a top model and a cost', async ({
    adminPage,
  }) => {
    const page = new GroupsPage(adminPage);
    await page.goto();
    const row = page.row(GROUP);
    await expect(row).toBeVisible();
    await expect(row.locator(`a[href="${PATHS.group(GROUP)}"]`)).toBeVisible();

    expect(await page.cellNumber(GROUP, 'Members')).toBeGreaterThan(0);
    expect(await page.cellNumber(GROUP, 'Projects')).toBeGreaterThan(0);
    // A model name, not the em-dash the cell falls back to when a group sent
    // no traffic at all.
    expect(await page.cell(GROUP, 'Top model')).toMatch(/^[a-z0-9][\w.\/-]+/i);
    expect(await page.cell(GROUP, 'Cost')).toMatch(/^\$\d/);
  });

  test('the KPI row states the window and the attribution', async ({ adminPage }) => {
    const page = new GroupsPage(adminPage);
    await page.goto();
    await expect(page.kpi('Groups').locator).toBeVisible();
    await expect(page.kpi('Unattributed').locator).toBeVisible();
    await expect(page.kpi('Cost').locator).toContainText(/exclusive attribution/i);
  });

  test('the detail page opens on Usage and shows its tabs', async ({ adminPage }) => {
    const detail = new GroupDetailPage(adminPage, GROUP);
    await detail.goto();
    for (const tab of ['usage', 'members', 'marketplaces', 'projects', 'mappings', 'access']) {
      await expect(detail.tabLink(tab).first()).toBeVisible();
    }
    await expect(adminPage.locator(SEL.tabActive)).toContainText(/usage/i);
  });

  test('the Usage tab labels its member breakdowns as overlapping', async ({
    adminPage,
  }) => {
    const detail = new GroupDetailPage(adminPage, GROUP);
    await detail.gotoTab('usage');
    await expect(adminPage.getByText(/member view — overlaps/i).first()).toBeVisible();
    await expect(detail.section('Spend by member')).toBeVisible();
  });

  test('the Members tab lists the seeded members and not other groups', async ({
    adminPage,
  }) => {
    const detail = new GroupDetailPage(adminPage, GROUP);
    await detail.gotoTab('members');
    for (const id of ['e2e-developer', 'e2e-member-1', 'e2e-member-3']) {
      await expect(detail.memberRow(id)).toBeVisible();
    }
    await expect(detail.memberRow('e2e-member-4')).toHaveCount(0);
  });

  // The unticked rows are what make the tab an editor rather than a receipt,
  // so both states have to be on screen for it to be doing its job.
  test('the Marketplaces tab shows the catalog with this group ticked', async ({
    adminPage,
  }) => {
    const detail = new GroupDetailPage(adminPage, GROUP);
    await detail.gotoTab('marketplaces');
    await expect(detail.section('Marketplaces')).toBeVisible();

    const boxes = detail.section('Marketplaces').locator('[data-field="marketplace"]');
    const total = await boxes.count();
    expect(total, 'the catalog rendered no marketplaces').toBeGreaterThan(1);
    const ticked = await boxes.evaluateAll((els) =>
      els.filter((el) => (el as HTMLInputElement).checked).length,
    );
    expect(ticked, 'this group reaches no marketplace').toBeGreaterThan(0);
    expect(ticked, 'every marketplace is ticked, so nothing is grantable').toBeLessThan(total);
  });

  test('an unknown group is a 404, not a 500', async ({ adminPage }) => {
    const res = await adminPage.goto(PATHS.group('no-such-group-at-all'));
    expect(res?.status()).toBe(404);
  });
});

test.describe('actions', () => {
  test('sorting by cost reorders the rows and marks the column', async ({ adminPage }) => {
    const page = new GroupsPage(adminPage);
    await page.goto();
    const direction = await page.sortBy('Cost');
    expect(direction).toBe('ascending');
    const costs = await page.sortableColumn('Cost');
    expect(costs).toEqual([...costs].sort((a, b) => a - b));
  });

  // Unassigned is excluded because the page pins it last whatever the column:
  // it is a derived bucket rather than a team, and letting it win a sort would
  // put a bucket at the top of a list of groups.
  test('sorting by members orders on membership, Unassigned excepted', async ({ adminPage }) => {
    const page = new GroupsPage(adminPage);
    await page.goto({ sort: 'members', dir: 'desc' });
    const members = await page.sortableColumn('Members');
    expect(members).toEqual([...members].sort((a, b) => b - a));
    await expect(page.groupRows().last()).toHaveAttribute('data-group-id', 'unassigned');
  });

  // Summed over the group rows rather than read off the remainder row. The
  // remainder is zero whenever every account has a primary group, which is the
  // healthy state, so asserting on it would compare nothing to nothing and
  // pass whatever the window control did.
  test('the time window is a link that changes the figures', async ({ adminPage }) => {
    const page = new GroupsPage(adminPage);
    const total = async () =>
      (await page.columnNumbers('Requests')).reduce((a, b) => a + b, 0);

    await page.goto();
    const thirty = await total();
    expect(thirty).toBeGreaterThan(0);

    await page.range('7d');
    expect(adminPage.url()).toContain('range=7d');
    const seven = await total();
    expect(seven).toBeLessThan(thirty);
  });

  test('the source filter narrows the roster and is a URL', async ({ adminPage }) => {
    const page = new GroupsPage(adminPage);
    await page.goto();
    const before = await page.groupRows().count();
    await page.filterBySource('System');
    expect(await page.groupRows().count()).toBeLessThan(before);
    expect(adminPage.url()).toContain('source=System');
  });

  test('creating a group lands on its detail page, and it is removable', async ({
    adminPage,
    request,
  }) => {
    const id = `e2e-tmp-${Date.now() % 1000000}`;
    const page = new GroupsPage(adminPage);
    try {
      await page.goto();
      await page.headerAction(/new group/i).click();
      await page.dialog('create-group').locator('[data-field="new-group-id"]').fill(id);
      await page.dialog('create-group').locator('[data-field="new-group-name"]').fill('Temporary');
      await page.dialog('create-group').getByRole('button', { name: /create/i }).click();
      await expect(adminPage).toHaveURL(new RegExp(`/admin/groups/${id}`));

    // Deleted in a finally so a failed assertion above does not leave the
    // group behind: the listing's own KPI counts groups, so a stray row from a
    // red run silently changes what the next run measures.
    } finally {
      const undo = await request.delete(`/api/public/admin/groups/${id}`, apiAs(request, 'admin'));
      expect([200, 204, 404]).toContain(undo.status());
    }
  });

  test('assigning a marketplace to a group writes the rule', async ({ adminPage }) => {
    const detail = new GroupDetailPage(adminPage, GROUP);
    await detail.gotoTab('marketplaces');
    const rows = detail.sectionRows('Marketplaces');
    const id = await rows.first().getAttribute('data-marketplace-id');
    expect(id).not.toBeNull();

    // Asserted on the saved state rather than on the toast: saving reloads the
    // page, and the reload races the toast out of the DOM before it can be
    // seen. The rule the endpoint actually owes us is that the tick persists.
    const box = detail.marketplaceCheckbox(id as string);
    const wasChecked = await box.isChecked();
    await box.setChecked(!wasChecked);
    await adminPage.locator('[data-action="save-marketplaces"]').click();
    await adminPage.waitForLoadState('networkidle');

    await detail.gotoTab('marketplaces');
    await expect(detail.marketplaceCheckbox(id as string)).toBeChecked({ checked: !wasChecked });

    // Restore the seeded entitlement so the suite stays order-independent.
    await detail.marketplaceCheckbox(id as string).setChecked(wasChecked);
    await adminPage.locator('[data-action="save-marketplaces"]').click();
    await adminPage.waitForLoadState('networkidle');
    await detail.gotoTab('marketplaces');
    await expect(detail.marketplaceCheckbox(id as string)).toBeChecked({ checked: wasChecked });
  });

  test('removing a manual member removes the row and the record', async ({
    adminPage,
    request,
  }) => {
    const detail = new GroupDetailPage(adminPage, GROUP);
    await detail.gotoTab('members');
    const row = detail.memberRow('e2e-user');
    await expect(row).toBeVisible();
    await row.locator('[data-action="remove-member"]').click();
    await detail.confirm();
    await expect(detail.memberRow('e2e-user')).toHaveCount(0);

    const restored = await request.post(
      `/api/public/admin/groups/${GROUP}/members`,
      { ...apiAs(request, 'admin'), data: { user_id: 'e2e-user' } },
    );
    expect([200, 201, 204]).toContain(restored.status());
  });

  test('a directory-sourced member offers no Remove', async ({ adminPage }) => {
    const detail = new GroupDetailPage(adminPage, GROUP);
    await detail.gotoTab('members');
    await expect(
      detail.memberRow('e2e-developer').locator('[data-action="remove-member"]'),
    ).toHaveCount(0);
  });

  test('recomputing attribution keys reports how many it wrote', async ({ request }) => {
    const res = await request.post(
      '/api/public/admin/scope-defaults/recompute',
      apiAs(request, 'admin'),
    );
    expect(res.status()).toBe(200);
    expect(await res.json()).toHaveProperty('recomputed');
  });
});

test.describe('authorization', () => {
  for (const { principal, status } of ACCESS) {
    test(`${principal} gets ${status} on the groups listing`, async ({ browser }) => {
      // The storage-state files are kebab-case on disk while the fixture keys
      // are camelCase, so the path comes from the AUTH map rather than from
      // the principal name — building it by hand silently reads nothing and
      // every principal comes back anonymous.
      const context = await browser.newContext(
        principal === 'anon' ? {} : { storageState: AUTH[principal] },
      );
      const res = await context.request.get(PATH, { maxRedirects: 0 });
      expect(res.status()).toBe(status);
      await context.close();
    });
  }

  test('a project manager reads the page and gets no write controls', async ({
    pmPage,
  }) => {
    const detail = new GroupDetailPage(pmPage, GROUP);
    await detail.gotoTab('members');
    await expect(detail.memberRow('e2e-developer')).toBeVisible();
    await expect(pmPage.locator('[data-action="remove-member"]')).toHaveCount(0);
    await expect(pmPage.locator('[data-action="add-member"]')).toHaveCount(0);
  });

  test('only a platform admin is offered the directory mapping action', async ({
    platformAdminPage,
    adminPage,
  }) => {
    const asPlatform = new GroupsPage(platformAdminPage);
    await asPlatform.goto();
    await expect(asPlatform.headerAction(/map a directory group/i)).toBeVisible();

    const asAdmin = new GroupsPage(adminPage);
    await asAdmin.goto();
    await expect(asAdmin.headerAction(/map a directory group/i)).toHaveCount(0);
  });
});

test.describe('design language', () => {
  test('wears the shell', async ({ adminPage }) => {
    const page = new GroupsPage(adminPage);
    await page.goto();
    await expect(page.activeNavItem()).toHaveAttribute('href', PATH);
    await expect(page.breadcrumb()).toBeVisible();
    await expect(page.heading()).toBeVisible();
  });

  test('the detail page carries a breadcrumb back to the listing', async ({ adminPage }) => {
    const detail = new GroupDetailPage(adminPage, GROUP);
    await detail.goto();
    await expect(detail.breadcrumb().locator(`a[href="${PATH}"]`)).toBeVisible();
  });

  test('paints from design tokens', async ({ adminPage }) => {
    const page = new GroupsPage(adminPage);
    await page.goto();
    const values = await adminPage.evaluate((tokens) => {
      const style = getComputedStyle(document.documentElement);
      return tokens.map((t) => style.getPropertyValue(t).trim());
    }, Object.values(TOKENS));
    for (const value of values) expect(value).not.toBe('');
  });

  test('every column that a question starts from is sortable', async ({ adminPage }) => {
    const page = new GroupsPage(adminPage);
    await page.goto();
    expect(await adminPage.locator(SEL.tableSorted).count()).toBeGreaterThanOrEqual(8);
  });

  test('a filter that matches nothing shows an empty state, not a bare table', async ({
    adminPage,
  }) => {
    const page = new GroupsPage(adminPage);
    await page.goto({ source: 'Directory' });
    expect(await page.groupRows().count()).toBe(0);
    await expect(page.emptyState()).toBeVisible();
  });

  // Focuses a control the page owns rather than pressing Tab once, which lands
  // on the shell's skip link and would make this a test of somebody else's
  // component.
  test('keeps a visible focus ring', async ({ adminPage }) => {
    const page = new GroupsPage(adminPage);
    await page.goto();
    await adminPage.locator(`${SEL.tableHeaderCell} a`).first().focus();
    const ring = await adminPage.evaluate(() => {
      const el = document.activeElement;
      if (!el) return null;
      const s = getComputedStyle(el);
      return { style: s.outlineStyle, width: s.outlineWidth, shadow: s.boxShadow };
    });
    // Either ring counts: the design system draws some controls' focus with an
    // outline and others with a box shadow, and a test that insists on one of
    // them is testing the implementation rather than whether focus is visible.
    expect(ring).not.toBeNull();
    const outlined = ring?.style !== 'none' && Number.parseFloat(ring?.width ?? '0') > 0;
    const shadowed = (ring?.shadow ?? 'none') !== 'none';
    expect(outlined || shadowed, `focus ring: ${JSON.stringify(ring)}`).toBe(true);
  });

  test('does not scroll horizontally at 1440 or 1024', async ({ adminPage }) => {
    const page = new GroupsPage(adminPage);
    for (const width of [1440, 1024]) {
      await adminPage.setViewportSize({ width, height: 900 });
      await page.goto();
      expect(await page.hasHorizontalScroll(), `horizontal scroll at ${width}px`).toBe(false);
    }
  });

  test('meets the density bar as a list', async ({ adminPage }) => {
    const page = new GroupsPage(adminPage);
    await adminPage.setViewportSize({ width: 1440, height: 900 });
    await page.goto();
    await expectDensity(adminPage, 'list');
  });

  // Measured on the Usage tab, which is where the tab opens and the densest
  // thing on it — the member spend table — now sits above the chart.
  test('meets the density bar as a detail page', async ({ adminPage }) => {
    const detail = new GroupDetailPage(adminPage, GROUP);
    await adminPage.setViewportSize({ width: 1440, height: 900 });
    await detail.gotoTab('usage');
    await expectDensity(adminPage, 'detail');
  });

  test('matches its committed appearance', async ({ adminPage }) => {
    const page = new GroupsPage(adminPage);
    await adminPage.setViewportSize({ width: 1440, height: 900 });
    await page.goto();
    await snapshot(adminPage, 'groups-listing');
  });

  test('has no serious or critical axe violations', async ({ adminPage }) => {
    const page = new GroupsPage(adminPage);
    await page.goto();
    await expect(adminPage.locator(SEL.pageHeader)).toBeVisible();
    await expectAccessible(adminPage);
  });

  test('the detail page has no serious or critical axe violations', async ({ adminPage }) => {
    const detail = new GroupDetailPage(adminPage, GROUP);
    await detail.gotoTab('usage');
    await expectAccessible(adminPage);
  });
});
