// /admin/settings — the account form and the danger zone.
//
// Density note: this is a form page, not a listing, so the checklist's "twelve
// table rows above the fold" does not apply. It carries no table at all: the
// page is two bound forms and one destructive action, and a one-row table of
// that action would be a table shape wrapped around a sentence.
import { test, expect, AUTH, snapshot, type Principal } from '../support/fixtures';
import { expectAccessible, expectDensity } from '../support/a11y';
import { PATHS } from '../support/paths';
import { SEL, TOKENS } from '../support/pages/selectors';
import { SettingsPage } from '../support/pages/settings.page';

const PATH = PATHS.settings;

// Mirrors tests/contract/admin/baseline.txt for GET /admin/settings: everyone
// signed in owns a settings page; anonymous is sent to login.
const ACCESS: { principal: Principal | 'anon'; status: number }[] = [
  { principal: 'platformAdmin', status: 200 },
  { principal: 'admin', status: 200 },
  { principal: 'pm', status: 200 },
  { principal: 'developer', status: 200 },
  { principal: 'businessUser', status: 200 },
  { principal: 'unassigned', status: 200 },
  { principal: 'user', status: 200 },
  { principal: 'anon', status: 307 },
];

test.describe('renders', () => {
  test('shows the account form bound to the signed-in person', async ({ developerPage }) => {
    const page = new SettingsPage(developerPage);
    await page.goto();
    await expect(page.heading()).toHaveText('Account settings');
    await expect(page.email()).toHaveValue(/e2e-developer/);
  });

  test('marks directory-owned fields as not editable here', async ({ developerPage }) => {
    const page = new SettingsPage(developerPage);
    await page.goto();
    await expect(page.email()).toBeDisabled();
    await expect(developerPage.locator('#settings-username')).toBeDisabled();
    await expect(page.displayName()).toBeEnabled();
  });

  test('offers a populated timezone picker', async ({ developerPage }) => {
    const page = new SettingsPage(developerPage);
    await page.goto();
    expect(await page.timezone().locator('option').count()).toBeGreaterThan(1);
  });

  test('states the danger zone consequence before the button', async ({ developerPage }) => {
    const page = new SettingsPage(developerPage);
    await page.goto();
    await expect(page.deleteAccountButton()).toBeVisible();
    await expect(developerPage.locator('.sp-card--danger')).toContainText('cannot be undone');
  });
});

test.describe('actions', () => {
  // The write, end to end: the header button submits the form, the page PUTs to
  // the settings route, and the value is still there on the next load. Asserted
  // through persistence rather than through a toast, because a toast proves the
  // page said something and this proves the row changed.
  test('saving a display name persists it', async ({ developerPage }) => {
    const page = new SettingsPage(developerPage);
    await page.goto();
    const name = `E2E Developer ${Date.now()}`;
    await page.displayName().fill(name);
    const [response] = await Promise.all([
      developerPage.waitForResponse(
        (r) => r.request().method() === 'PUT' && r.url().includes('/user/settings'),
      ),
      page.saveButton().click(),
    ]);
    expect(response.status()).toBe(200);
    await page.goto();
    await expect(page.displayName()).toHaveValue(name);
  });

  // The timezone is stored and then used to render every timestamp the console
  // shows this person, so a value that cannot be a zone name is refused.
  test('a timezone that cannot be a zone name is refused', async ({ developerPage }) => {
    const res = await developerPage.request.put('/api/public/admin/user/settings', {
      data: { timezone: 'not a zone; DROP TABLE users' },
    });
    expect(res.status()).toBe(400);
  });

  // Deleting is irreversible, so the route refuses a request that does not name
  // the account. This drives that guard rather than the delete itself: an
  // accepted delete would remove the principal the rest of the suite signs in
  // as. The delete that succeeds is proven in the Rust contract suite.
  test('deleting an account without naming it is refused', async ({ developerPage }) => {
    const bare = await developerPage.request.delete('/api/public/admin/user/account');
    expect(bare.status()).toBe(400);
    const wrong = await developerPage.request.delete('/api/public/admin/user/account', {
      data: { confirm_email: 'someone-else@e2e.local' },
    });
    expect(wrong.status()).toBe(400);
  });

  test('one click on delete does not delete', async ({ developerPage }) => {
    const page = new SettingsPage(developerPage);
    await page.goto();
    let deleted = false;
    developerPage.on('request', (r) => {
      if (r.method() === 'DELETE') deleted = true;
    });
    await expect(page.deleteAccountButton()).toHaveAttribute('type', 'button');
    await page.deleteAccountButton().click();
    await developerPage.waitForTimeout(500);
    expect(deleted, 'delete fired without a confirmation').toBe(false);
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

  test('each principal sees their own account and nobody else’s', async ({
    developerPage,
    userPage,
  }) => {
    await new SettingsPage(developerPage).goto();
    await new SettingsPage(userPage).goto();
    await expect(developerPage.locator('#settings-email')).toHaveValue(/e2e-developer/);
    await expect(userPage.locator('#settings-email')).toHaveValue(/e2e-user/);
  });
});

test.describe('design language', () => {
  test('wears the shell', async ({ developerPage }) => {
    const page = new SettingsPage(developerPage);
    await page.goto();
    await expect(page.activeNavItem()).toHaveAttribute('href', PATH);
    await expect(page.breadcrumb()).toBeVisible();
    await expect(page.heading()).toBeVisible();
  });

  test('carries no trace of the old bespoke form classes', async ({ developerPage }) => {
    const page = new SettingsPage(developerPage);
    await page.goto();
    for (const legacy of ['.sp-settings-form-grid', '.sp-settings-field', '.sp-danger-zone-item']) {
      expect(await developerPage.locator(legacy).count(), legacy).toBe(0);
    }
  });

  test('paints from design tokens', async ({ developerPage }) => {
    const page = new SettingsPage(developerPage);
    await page.goto();
    const values = await developerPage.evaluate((tokens) => {
      const style = getComputedStyle(document.documentElement);
      return tokens.map((t) => style.getPropertyValue(t).trim());
    }, Object.values(TOKENS));
    for (const value of values) expect(value).not.toBe('');
  });

  test('keeps a visible focus ring', async ({ developerPage }) => {
    const page = new SettingsPage(developerPage);
    await page.goto();
    await developerPage.keyboard.press('Tab');
    // The design system draws focus with an outline on most controls and with a
    // box-shadow ring on others (the skip link, which is what the first Tab
    // lands on). Either is a visible ring; requiring an outline specifically
    // fails on a page whose first focusable element uses the shadow.
    const ring = await developerPage.evaluate(() => {
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

  test('does not scroll horizontally at 1440 or 1024', async ({ developerPage }) => {
    const page = new SettingsPage(developerPage);
    for (const width of [1440, 1024]) {
      await developerPage.setViewportSize({ width, height: 900 });
      await page.goto();
      expect(await page.hasHorizontalScroll(), `horizontal scroll at ${width}px`).toBe(false);
    }
  });

  test('meets the density bar', async ({ developerPage }) => {
    const page = new SettingsPage(developerPage);
    await page.goto();
    await expectDensity(developerPage, 'detail');
  });

  test('looks like its committed figure', async ({ developerPage }) => {
    const page = new SettingsPage(developerPage);
    await page.goto();
    await snapshot(developerPage, 'settings');
  });

  test('has no serious or critical axe violations', async ({ developerPage }) => {
    const page = new SettingsPage(developerPage);
    await page.goto();
    await expect(developerPage.locator(SEL.pageHeader)).toBeVisible();
    await expectAccessible(developerPage);
  });
});
