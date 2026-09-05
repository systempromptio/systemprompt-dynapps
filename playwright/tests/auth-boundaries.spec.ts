// The auth fixture's acceptance test: one case per principal proving the
// storageState minted in global-setup lands on the right side of every gate
// (anonymous redirect, non-admin bounce, admin pass, platform gate).
//
// "the profile page renders" moved to tests/pages/profile.spec.ts, where the
// full principal matrix for that page lives.
import { test, expect, apiAs } from './support/fixtures';

test.describe('anonymous', () => {
  test('analytics redirects to login carrying the target', async ({ anonPage }) => {
    await anonPage.goto('/admin/analytics');
    await expect(anonPage).toHaveURL(/\/admin\/login\?.*redirect=/);
  });

  test('users roster redirects to login', async ({ anonPage }) => {
    await anonPage.goto('/admin/access/users');
    await expect(anonPage).toHaveURL(/\/admin\/login/);
  });
});

test.describe('authenticated non-admin', () => {
  // The admin root's own bounce is asserted in tests/pages/overview.spec.ts,
  // which owns that page.
  test('analytics bounces to the profile page', async ({ userPage }) => {
    await userPage.goto('/admin/analytics');
    await expect(userPage).toHaveURL(/\/admin\/profile/);
  });

  test('the admin API refuses with 401/403', async ({ request }) => {
    const res = await request.get('/api/public/admin/users', apiAs(request, 'user'));
    expect([401, 403]).toContain(res.status());
  });
});

test.describe('org admin', () => {
  // The admin root renders the overview in place rather than redirecting; that
  // page's spec (tests/pages/overview.spec.ts) owns the assertion.

  // The enterprise/organization console was removed with the org spine
  // (migration 041). The surviving platform-only boundary is role granting:
  // an org admin may grant `admin` but not `platform_admin`.
  test('an org admin cannot grant the platform_admin role', async ({ request }) => {
    const res = await request.put('/api/public/admin/users/e2e-victim/roles', {
      ...apiAs(request, 'admin'),
      data: { roles: ['user', 'platform_admin'] },
    });
    expect(res.status()).toBe(403);
  });
});

test.describe('developer', () => {
  test('the group pages redirect to the profile page', async ({ developerPage }) => {
    await developerPage.goto('/admin/access/groups');
    await expect(developerPage).toHaveURL(/\/admin\/profile/);
  });

  test('the admin API refuses a developer', async ({ request }) => {
    const res = await request.get('/api/public/admin/groups', apiAs(request, 'developer'));
    expect([401, 403]).toContain(res.status());
  });
});

test.describe('project manager', () => {
  test('reads the groups API but cannot write to it', async ({ request }) => {
    const read = await request.get('/api/public/admin/groups', apiAs(request, 'pm'));
    expect(read.status()).toBe(200);
    const write = await request.post('/api/public/admin/groups', {
      ...apiAs(request, 'pm'),
      data: { id: 'e2e-pm-should-not-create', name: 'nope' },
    });
    expect([401, 403]).toContain(write.status());
  });
});

test.describe('platform admin', () => {
  test('a platform admin may grant and revoke platform_admin', async ({ request }) => {
    const grant = await request.put('/api/public/admin/users/e2e-victim/roles', {
      ...apiAs(request, 'platformAdmin'),
      data: { roles: ['user', 'platform_admin'] },
    });
    expect(grant.status()).toBe(200);
    const revoke = await request.put('/api/public/admin/users/e2e-victim/roles', {
      ...apiAs(request, 'platformAdmin'),
      data: { roles: ['user'] },
    });
    expect(revoke.status()).toBe(200);
  });
});
