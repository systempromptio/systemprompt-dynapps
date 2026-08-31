// REQ-002 — Controlled User Registration.
//
// Acceptance criteria: "An unapproved user cannot create an account or connect
// a Bridge. Approved/provisioned users can access the platform based on
// assigned entitlements."
//
// The negative cases are the requirement. Enrolment is now closed by
// construction rather than by a config flag: the self-registration endpoint no
// longer exists, and SSO is the only door. An account is created only by a
// sign-in carrying a mapped `Systemprompt-*` AD group (`auto_provision`), so
// the AD group is the approval.
import { test, expect, uniqueEmail } from '../support/fixtures';

test.describe('@REQ-002 controlled registration', () => {
  test('@REQ-002 the self-registration endpoint is gone, not merely refusing', async ({
    request,
  }) => {
    // It used to answer 403 from a config flag. A flag can be flipped back; a
    // route that does not exist cannot. Both an out-of-domain and an
    // allow-listed address must find nothing here — the domain list says an
    // address *could* belong to someone who should have access, never that
    // anyone approved them.
    for (const email of [
      'stranger@not-astound.example',
      uniqueEmail('selfreg').replace('@e2e.local', '@astounddigital.com'),
    ]) {
      const resp = await request.post('/admin/auth/passkey/register', {
        data: { email, display_name: 'Stranger' },
        maxRedirects: 0,
      });
      expect(resp.status()).toBe(404);
    }
  });

  test('@REQ-002 the login page offers no way to create an account', async ({
    anonPage: page,
  }) => {
    await page.goto('/admin/login');
    await expect(page.locator('#register-form')).toHaveCount(0);
    await expect(page.locator('#show-register')).toHaveCount(0);
    await expect(page.locator('main input')).toHaveCount(0);
    await expect(page.locator('main')).not.toContainText(/create an account/i);
  });

  test('@REQ-002 the invite door is gone on both halves', async ({ request }) => {
    // Redeeming an invite in the browser provisioned an account and enrolled a
    // credential outside SSO. The admin half that minted those links is gone
    // too: a link nobody can redeem is a 404 handed to a new starter.
    for (const path of ['/admin/auth/invite/accept', '/api/public/admin/invites']) {
      const resp = await request.post(path, { data: { token: 'anything' }, maxRedirects: 0 });
      expect(resp.status(), `${path} must not exist`).toBe(404);
    }

    const page = await request.get('/admin/invite/some-token', { maxRedirects: 0 });
    expect(page.status()).toBe(404);

    const listing = await request.get('/api/public/admin/invites', { maxRedirects: 0 });
    expect(listing.status()).toBe(404);
  });

  test('@REQ-002 the login page offers exactly one door, and it is SSO', async ({
    anonPage: page,
  }) => {
    await page.goto('/admin/login');
    // One control, pointing at the IdP. Anything else on this page is a second
    // way in, which is the whole thing REQ-002 forbids.
    await expect(page.locator('main a[href^="/admin/auth/adfs/start"]')).toHaveCount(1);
    await expect(page.locator('main button, main a.btn')).toHaveCount(1);
  });

  test('@REQ-002 the retired passkey scripts are no longer served', async ({ request }) => {
    // They are dead code, but a served credential-enrolment script is an
    // invitation to find the endpoint behind it.
    for (const file of ['webauthn-register.js', 'webauthn-login.js', 'webauthn-session.js']) {
      const resp = await request.get(`/js/services/${file}`, { maxRedirects: 0 });
      expect(resp.status(), `${file} must not be served`).toBe(404);
    }
  });

  test('@REQ-002 an anonymous caller cannot mint a bridge connect code', async ({ request }) => {
    // maxRedirects: 0 or the client follows the bounce to /admin/login and
    // reports the login page's 200, which would pass while proving nothing.
    const resp = await request.post('/admin/api/profile/bridge-code', { maxRedirects: 0 });
    expect([401, 303, 307]).toContain(resp.status());
  });

  test('@REQ-002 the profile page mints no code until asked', async ({ userPage: page }) => {
    await page.goto('/admin/profile');
    // A connect code is a bearer credential; it must not be in the HTML of a
    // page people leave open and reload.
    await expect(page.locator('#connect-code-output')).toBeHidden();
    await expect(page.locator('#issue-connect-code')).toBeVisible();

    await page.locator('#issue-connect-code').click();
    await expect(page.locator('#connect-code-output')).toBeVisible();
    await expect(page.locator('[data-connect-field="code"]')).not.toBeEmpty();
  });
});
