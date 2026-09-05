// The marketplace catalog: the audience matrix, the legacy redirect, and the
// visibility badge that tells an admin at a glance who can see a plugin.
//
// DOM contract asserted here: .audience-matrix with tr[data-subject] rows and
// td[data-marketplace] cells carrying is-allow / is-deny, and
// .visibility-badge on the plugin cards.
import { test, expect } from './support/fixtures';

const COMMERCE = 'astound-commerce-dev';

test.describe('marketplace catalog', () => {
  test('the singular path redirects to the plural one', async ({ adminPage: page }) => {
    const res = await page.goto('/admin/catalog/marketplace');
    expect(res?.status()).toBe(200);
    await expect(page).toHaveURL(/\/admin\/catalog\/marketplaces$/);
  });

  test('every marketplace has a card', async ({ adminPage: page }) => {
    await page.goto('/admin/catalog/marketplaces');
    for (const id of [COMMERCE, 'astound-europe-dev', 'astound-business', 'enterprise-demo']) {
      await expect(page.locator(`[data-marketplace-id="${id}"]`)).toBeVisible();
    }
  });

  test('the audience matrix has a cell per group and marketplace', async ({
    platformAdminPage: page,
  }) => {
    await page.goto('/admin/catalog/marketplaces');
    const matrix = page.locator('.audience-matrix');
    await expect(matrix).toBeVisible();

    for (const group of ['commerce-devs', 'europe-devs', 'business-users', 'unassigned']) {
      await expect(matrix.locator(`tr[data-subject="group:${group}"]`)).toBeVisible();
    }

    const allowed = matrix.locator(
      `tr[data-subject="group:commerce-devs"] td[data-marketplace="${COMMERCE}"]`,
    );
    await expect(allowed).toHaveClass(/is-allow/);
    await expect(allowed).toContainText('group');

    const denied = matrix.locator(
      `tr[data-subject="group:business-users"] td[data-marketplace="${COMMERCE}"]`,
    );
    await expect(denied).toHaveClass(/is-deny/);
  });

  test('the detail page names the group that can see it', async ({ platformAdminPage: page }) => {
    const res = await page.goto(`/admin/catalog/marketplaces/${COMMERCE}`);
    expect(res?.status()).toBe(200);
    await expect(page.locator('main')).toContainText('Commerce devs');
  });

  test('plugin cards carry a visibility badge', async ({ adminPage: page }) => {
    await page.goto('/admin/catalog/plugins');
    const badges = page.locator('.visibility-badge');
    expect(await badges.count()).toBeGreaterThan(0);
  });
});
