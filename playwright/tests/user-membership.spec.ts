// Group and project membership on the user-detail page.
//
// This replaces the organization-membership widget removed with migration 041.
// Membership is what decides which marketplaces an account reaches, so the
// checkbox grid is the write surface the group entitlement chain hangs off.
// Serial: both tests move the same seeded victim user.
import { test, expect, apiAs } from './support/fixtures';

test.describe.configure({ mode: 'serial' });

const VICTIM = 'e2e-victim';

test.describe('platform admin', () => {
  test('adds the victim to a group and removes them again', async ({
    platformAdminPage: page,
    request,
  }) => {
    await page.goto(`/admin/access/user?id=${VICTIM}`);
    const card = page.locator('[data-membership]');
    await expect(card).toBeVisible();

    // Seeded with no group rows, so the derived `unassigned` box is the only
    // one ticked.
    const derived = card.locator('input[data-membership-id="unassigned"]');
    await expect(derived).toBeChecked();

    const commerce = card.locator('input[data-membership-id="europe-devs"]');
    await expect(commerce).not.toBeChecked();
    await commerce.check();
    await expect(page.locator('[data-status="membership"]')).not.toBeEmpty();

    const members = await request.get(
      '/api/public/admin/groups/europe-devs/members',
      apiAs(request, 'platformAdmin'),
    );
    expect(members.status()).toBe(200);
    expect(await members.text()).toContain(VICTIM);

    await page.reload();
    await expect(page.locator('input[data-membership-id="europe-devs"]')).toBeChecked();

    // Restore the seeded shape so the suite stays order-independent.
    await page.locator('input[data-membership-id="europe-devs"]').uncheck();
    await expect(page.locator('[data-status="membership"]')).not.toBeEmpty();
    const undo = await request.delete(
      `/api/public/admin/groups/europe-devs/members/${VICTIM}`,
      apiAs(request, 'platformAdmin'),
    );
    expect([200, 204, 404]).toContain(undo.status());
  });

  test('project membership is edited from the same card', async ({
    platformAdminPage: page,
    request,
  }) => {
    await page.goto(`/admin/access/user?id=${VICTIM}`);
    const storefront = page.locator('input[data-membership-id="storefront"]');
    await expect(storefront).not.toBeChecked();
    await storefront.check();
    await expect(page.locator('[data-status="membership"]')).not.toBeEmpty();

    const members = await request.get(
      '/api/public/admin/projects/storefront/members',
      apiAs(request, 'platformAdmin'),
    );
    expect(members.status()).toBe(200);
    expect(await members.text()).toContain(VICTIM);

    await page.reload();
    await page.locator('input[data-membership-id="storefront"]').uncheck();
    await expect(page.locator('[data-status="membership"]')).not.toBeEmpty();
    const undo = await request.delete(
      `/api/public/admin/projects/storefront/members/${VICTIM}`,
      apiAs(request, 'platformAdmin'),
    );
    expect([200, 204, 404]).toContain(undo.status());
  });
});

test.describe('non-admin', () => {
  test('cannot change another user\'s group membership', async ({ request }) => {
    const res = await request.post('/api/public/admin/groups/europe-devs/members', {
      ...apiAs(request, 'user'),
      data: { user_id: VICTIM },
    });
    expect([401, 403]).toContain(res.status());
  });
});
