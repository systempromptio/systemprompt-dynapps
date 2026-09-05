// REQ-006 (organizational drill-down), REQ-007/008 (engineering measures), and
// REQ-009 (spend limits).
//
// The "organizational dimension" is groups and projects since migration 041;
// the organization/plan/seat spine and its `?org=` and `?department=`
// parameters are gone, and with them the plan-cap spend meters.
//
// REQ-006 and REQ-008 are Partial and REQ-007 is not feasible as specified;
// these specs pin what IS true, including the labelling that keeps a proxy from
// being read as the metric it stands in for.
import { test, expect } from '../support/fixtures';

test.describe('@REQ-006 organizational drill-down', () => {
  test('@REQ-006 group, project and user each narrow the view', async ({
    platformAdminPage: page,
  }) => {
    await page.goto('/admin/analytics?tab=overview&preset=30d');
    const unfiltered = await page.locator('tbody tr').count();

    await page.goto('/admin/analytics?tab=overview&preset=30d&group=business-users');
    const byGroup = await page.locator('tbody tr').count();
    expect(byGroup).toBeLessThanOrEqual(unfiltered);

    await page.goto('/admin/analytics?tab=overview&preset=30d&project=checkout');
    const byProject = await page.locator('tbody tr').count();
    expect(byProject).toBeLessThanOrEqual(unfiltered);

    await page.goto(
      '/admin/analytics?tab=overview&preset=30d&user_id=e2e-member-1',
    );
    const leaderboard = page.locator('.sp-table').first();
    await expect(leaderboard).toContainText('e2e-member-1');
    await expect(leaderboard).not.toContainText('e2e-member-3');
  });

  test('@REQ-006 an unknown scope in the URL narrows to nothing, it does not widen', async ({
    adminPage: page,
  }) => {
    // Scope now comes from the group/project selects. A value that names no
    // group must not fall back to the unscoped view — that fallback is how a
    // URL edit turns into a read of everything.
    const res = await page.goto('/admin/analytics?tab=overview&preset=30d&group=no-such-group');
    expect(res?.status()).toBe(200);
    await expect(page.locator('main')).not.toContainText('e2e-member-1');
  });

  test('@REQ-006 the scope dropdowns list the seeded groups and projects', async ({
    adminPage: page,
  }) => {
    await page.goto('/admin/analytics?tab=overview');
    const groups = (await page.locator('select[name="group"] option').allInnerTexts()).map((t) =>
      t.trim(),
    );
    for (const g of ['Commerce devs', 'Europe devs', 'Business users', 'Unassigned']) {
      expect(groups).toContain(g);
    }
    const projects = (
      await page.locator('select[name="project"] option').allInnerTexts()
    ).map((t) => t.trim());
    for (const pr of ['Storefront', 'Checkout', 'Payments']) {
      expect(projects).toContain(pr);
    }
  });

  // 404, not 403: a 403 would confirm the id names a real account the caller
  // may not read, which is the oracle this guards against. Cross-organization
  // staging is gone with the org spine, so what remains provable in a browser
  // is that an id naming nobody is a dead end rather than a rendered shell.
  // Per-user analytics is the usage tab of the user's own detail page since
  // a7baa198; the old /admin/analytics/users/{id} path only 308s there.
  test('@REQ-006 the per-user drill-down is not an existence oracle', async ({
    adminPage: page,
  }) => {
    const res = await page.goto('/admin/users/e2e-no-such-user?tab=usage');
    expect(res?.status()).toBe(404);
  });
});

test.describe('@REQ-007 productivity measures are labelled as proxies', () => {
  test('@REQ-007 the code tab states there is no acceptance rate to report', async ({
    platformAdminPage: page,
  }) => {
    await page.goto('/admin/analytics?tab=overview&preset=30d');
    // If this label ever disappears, a proxy has quietly become a headline
    // metric -- which is the specific failure this requirement invites.
    await expect(page.locator('.sp-code-frames')).toContainText(/no accept\/reject signal/i);
  });
});

test.describe('@REQ-008 commit activity', () => {
  test('@REQ-008 commit activity is plotted beside AI line deltas', async ({
    platformAdminPage: page,
  }) => {
    await page.goto('/admin/analytics?tab=overview&preset=30d');
    const frames = page.locator('.sp-code-frames .sp-code-frame');
    await expect(frames.first()).toBeVisible();
    await expect(page.locator('main')).toContainText(/commit/i);
  });
});

test.describe('@REQ-009 spend limits and budget monitoring', () => {
  // The plan-cap spend meters (`.spend-meter-list`, the soft/hard ticks and the
  // budget-warning table) were removed with the organization plan model: spend
  // is now reported per group and project with no configured ceiling to draw
  // against. The enforcement side (the 429 at the hard cap) is the Rust tier's;
  // there is currently no browser surface for the cap itself.
  test.fixme('@REQ-009 spend is shown against the plan cap', async () => {});

  test.fixme('@REQ-009 the burn-up names both the soft and hard threshold', async () => {});

  test('@REQ-009 the spend tab reports cost for a scoped group', async ({
    platformAdminPage: page,
  }) => {
    await page.goto('/admin/analytics?tab=overview&preset=30d&group=europe-devs');
    await expect(page.locator('.sp-kpi').first()).toBeVisible();
  });

  test('@REQ-009 a non-admin cannot read spend', async ({ userPage: page }) => {
    const res = await page.goto('/admin/analytics?tab=overview');
    // Non-admins are bounced to their profile rather than shown the dashboard.
    expect(page.url()).toContain('/admin/profile');
    expect(res?.status()).toBe(200);
  });
});
