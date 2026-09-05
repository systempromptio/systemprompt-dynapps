// The group and unassigned pages: what the roster shows, who may write, and
// whether assigning a user to a group actually moves them out of Unassigned.
//
// Every DOM hook used here is a contract with the page templates and is listed
// in the slice report: tr[data-group-id], .group-member-count,
// tr[data-user-id], [data-action="remove-member"],
// select[data-action="assign-group"], [data-action="assign-submit"].
import { test, expect, apiAs } from './support/fixtures';

interface GroupSummary {
  id: string;
  name: string;
  member_count: number;
}

test.describe('groups list', () => {
  test('one row per group, and the counts match the API', async ({ adminPage: page, request }) => {
    const res = await request.get('/api/public/admin/groups', apiAs(request, 'admin'));
    expect(res.status()).toBe(200);
    const groups = ((await res.json()) as { groups: GroupSummary[] }).groups;

    await page.goto('/admin/access/groups');
    for (const g of groups) {
      const row = page.locator(`tr[data-group-id="${g.id}"]`);
      await expect(row).toBeVisible();
      await expect(row.locator('.group-member-count')).toHaveText(String(g.member_count));
    }
  });

  test('the unassigned row is present and flagged', async ({ adminPage: page }) => {
    await page.goto('/admin/access/groups');
    const row = page.locator('tr[data-group-id="unassigned"]');
    await expect(row).toBeVisible();
    await expect(row.locator('.badge')).toBeVisible();
  });
});

test.describe('group detail', () => {
  test('tabs are links, so every tab is a real URL', async ({ platformAdminPage: page }) => {
    await page.goto('/admin/access/groups/commerce-devs');
    for (const tab of ['overview', 'members', 'access', 'projects', 'mappings']) {
      await expect(page.locator(`a[href*="tab=${tab}"]`).first()).toBeVisible();
    }
    await page.goto('/admin/access/groups/commerce-devs?tab=members');
    await expect(page.locator('tr[data-user-id="e2e-developer"]')).toBeVisible();
  });

  test('the seeded members are on the members tab', async ({ adminPage: page }) => {
    await page.goto('/admin/access/groups/europe-devs?tab=members');
    for (const id of ['e2e-pm', 'e2e-member-4', 'e2e-member-5']) {
      await expect(page.locator(`tr[data-user-id="${id}"]`)).toBeVisible();
    }
    await expect(page.locator('tr[data-user-id="e2e-developer"]')).toHaveCount(0);
  });

  // The project manager is a console (read) role, not a manage (write) role:
  // it sees the page and none of the controls that change it.
  test('a project manager sees the page but no write controls', async ({ pmPage: page }) => {
    const res = await page.goto('/admin/access/groups/europe-devs?tab=members');
    expect(res?.status()).toBe(200);
    await expect(page.locator('tr[data-user-id="e2e-member-4"]')).toBeVisible();
    await expect(page.locator('[data-action="remove-member"]')).toHaveCount(0);
    await expect(page.locator('[data-action="add-member"]')).toHaveCount(0);
  });

  test('a developer cannot reach the group pages at all', async ({ developerPage: page }) => {
    await page.goto('/admin/access/groups');
    await expect(page).toHaveURL(/\/admin\/profile/);
  });
});

test.describe('unassigned', () => {
  test('assigning a user to a group removes them from the unassigned list', async ({
    adminPage: page,
    request,
  }) => {
    await page.goto('/admin/access/groups/unassigned');
    const row = page.locator('tr[data-user-id="e2e-unassigned"]');
    await expect(row).toBeVisible();

    await row.locator('select[data-action="assign-group"]').selectOption('business-users');
    await row.locator('[data-action="assign-submit"]').click();

    await expect(page.locator('tr[data-user-id="e2e-unassigned"]')).toHaveCount(0);

    const members = await request.get(
      '/api/public/admin/groups/business-users/members',
      apiAs(request, 'admin'),
    );
    expect(members.status()).toBe(200);
    expect(await members.text()).toContain('e2e-unassigned');

    // Restore the seeded shape so the suite stays order-independent.
    const undo = await request.delete(
      '/api/public/admin/groups/business-users/members/e2e-unassigned',
      apiAs(request, 'admin'),
    );
    expect([200, 204]).toContain(undo.status());
  });
});
