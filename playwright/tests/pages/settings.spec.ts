// /admin/settings — the account form and the danger zone.
//
// Density note: this is a form page, not a listing, so the checklist's "twelve
// table rows above the fold" does not apply. It carries no table at all: the
// page is two bound forms and one destructive action, and a one-row table of
// that action would be a table shape wrapped around a sentence.
import { test, expect, type Principal } from '../support/fixtures';
import { expectAccessible } from '../support/a11y';
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
  test('saving a display name confirms in the status region', async ({ developerPage }) => {
    const page = new SettingsPage(developerPage);
    await page.goto();
    await page.displayName().fill('E2E Developer');
    await page.saveButton().click();
    await expect(page.toast()).toBeVisible();
  });

  // Destructive and irreversible, so it sits behind the shared confirm dialog.
  // The spec opens it and cancels: an accepted delete would remove the
  // principal every other spec in the suite signs in as.
  test('deleting the account asks for confirmation first', async ({ developerPage }) => {
    const page = new SettingsPage(developerPage);
    await page.goto();
    await page.deleteAccountButton().click();
    await expect(developerPage.locator(SEL.dialog)).toBeVisible();
    await page.confirm(false);
  });
});

test.describe('authorization', () => {
  for (const { principal, status } of ACCESS) {
    test(`${principal} gets ${status}`, async ({ browser }) => {
      const context = await browser.newContext(
        principal === 'anon' ? {} : { storageState: `.auth/${principal}.json` },
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
    const ring = await developerPage.evaluate(() => {
      const el = document.activeElement;
      if (!el) return null;
      const s = getComputedStyle(el);
      return { style: s.outlineStyle, width: s.outlineWidth };
    });
    expect(ring).not.toBeNull();
    expect(ring?.style).not.toBe('none');
    expect(Number.parseFloat(ring?.width ?? '0')).toBeGreaterThan(0);
  });

  test('does not scroll horizontally at 1440 or 1024', async ({ developerPage }) => {
    const page = new SettingsPage(developerPage);
    for (const width of [1440, 1024]) {
      await developerPage.setViewportSize({ width, height: 900 });
      await page.goto();
      expect(await page.hasHorizontalScroll(), `horizontal scroll at ${width}px`).toBe(false);
    }
  });

  test('has no serious or critical axe violations', async ({ developerPage }) => {
    const page = new SettingsPage(developerPage);
    await page.goto();
    await expect(developerPage.locator(SEL.pageHeader)).toBeVisible();
    await expectAccessible(developerPage);
  });
});
