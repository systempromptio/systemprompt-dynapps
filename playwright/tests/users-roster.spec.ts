// Users roster: group bucketing, the group filter, search, the create-user
// panel, and the per-row actions popup. Users created here use unique @e2e.local emails and
// are deleted through the admin API in the same test.
import { test, expect, apiAs, uniqueEmail } from './support/fixtures';

test.describe('users roster', () => {
  test('buckets seeded users by group', async ({ adminPage: page }) => {
    await page.goto('/admin/access/users');
    const commerce = page.locator('tbody.table-group[data-group="europe-devs"]');
    await expect(commerce).toContainText('e2e-member-1');
    const business = page.locator('tbody.table-group[data-group="business-users"]');
    await expect(business).toContainText('e2e-member-6');
  });

  test('the group filter narrows the roster to one bucket', async ({ adminPage: page }) => {
    await page.goto('/admin/access/users?group=europe-devs');
    await expect(page.locator('tbody.table-group[data-group="europe-devs"]')).toBeVisible();
    await expect(page.locator('tbody.table-group[data-group="business-users"]')).toHaveCount(0);
    await expect(page.locator('tr[data-user-id="e2e-member-6"]')).toHaveCount(0);
  });

  test('the project filter narrows the roster too', async ({ adminPage: page }) => {
    await page.goto('/admin/access/users?project=payments');
    await expect(page.locator('tr[data-user-id="e2e-member-4"]')).toBeVisible();
    await expect(page.locator('tr[data-user-id="e2e-member-1"]')).toHaveCount(0);
  });

  // Why this asserts on what DISAPPEARS: the previous version filled the box
  // and checked the matching row was still visible, which passed whether or not
  // the box was wired to anything at all -- and it wasn't.
  test('search narrows the roster', async ({ adminPage: page }) => {
    await page.goto('/admin/access/users');
    const match = page.locator('tr[data-user-name*="e2e-member-1"]');
    const other = page.locator('tr[data-user-name*="e2e-member-6"]');
    await expect(match).toBeVisible();
    await expect(other).toBeVisible();

    await page.locator('#user-search').fill('e2e-member-1');
    await expect(match).toBeVisible();
    await expect(other).toBeHidden();
    // The group bucket holding only non-matching members collapses with them.
    await expect(page.locator('tbody.table-group[data-group="business-users"]')).toBeHidden();

    await page.locator('#user-search').fill('');
    await expect(other).toBeVisible();
  });

  test('a search matching nobody shows the empty state', async ({ adminPage: page }) => {
    await page.goto('/admin/access/users');
    await page.locator('#user-search').fill('zzz-no-such-user');
    await expect(page.locator('#user-search-empty')).toBeVisible();
    await expect(page.locator('tr[data-user-name]:visible')).toHaveCount(0);
  });

  test('create-user panel creates a user', async ({ adminPage: page, request }) => {
    const email = uniqueEmail('created');
    await page.goto('/admin/access/users');
    await page.locator('#btn-add-user').click();

    const panel = page.locator('#create-user-panel, .side-panel, [data-panel="create-user"]').first();
    await expect(panel).toBeVisible();
    await panel.locator('input[name="user_id"], #new-user-id').first().fill(email.split('@')[0]);
    await panel.locator('input[name="email"], #new-user-email').first().fill(email);
    await panel.locator('input[name="display_name"], #new-user-name').first().fill('E2E Created');

    // A created user is placed in no group, so they land in the derived
    // Unassigned bucket — group membership is granted afterwards, on the user
    // detail page or through the groups API. The Unassigned bucket holds
    // hundreds of rows, so creation is confirmed through the search API rather
    // than by hunting the new row in the roster.
    await panel.locator('button[type="submit"], .btn-primary').last().click();
    // The roster reloads after the create call resolves; poll the search API
    // rather than racing that navigation.
    await expect
      .poll(async () => {
        const r = await request.get(
          `/api/public/admin/users/search?q=${encodeURIComponent(email)}`,
          apiAs(request, 'admin'),
        );
        if (!r.ok()) return 0;
        return ((await r.json()) as { users: unknown[] }).users.length;
      })
      .toBeGreaterThan(0);

    const list = await request.get(
      `/api/public/admin/users/search?q=${encodeURIComponent(email)}`,
      apiAs(request, 'admin'),
    );
    expect(list.status()).toBe(200);
    const found = ((await list.json()) as { users: { id: string }[] }).users;
    expect(found.length).toBeGreaterThan(0);

    // Cleanup through the API so the roster stays deterministic.
    await request.delete(`/api/public/admin/users/${found[0].id}`, apiAs(request, 'admin'));
  });

  test('the per-row actions popup opens', async ({ adminPage: page }) => {
    await page.goto('/admin/access/users');
    await page.locator('button.btn-actions-trigger[data-user-id="e2e-member-1"]').click();
    await expect(page.locator('#user-actions-popup')).toBeVisible();
    await expect(page.locator('#user-actions-popup')).toContainText('Edit User');
  });
});
