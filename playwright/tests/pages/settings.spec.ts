// /admin/settings — the account form and the danger zone.
//
// Density note: this is a form page, not a listing, so the checklist's "twelve
// table rows above the fold" does not apply. It carries no table at all: the
// page is two bound forms and one destructive action, and a one-row table of
// that action would be a table shape wrapped around a sentence.
import { test, expect, AUTH, snapshot, type Principal } from '../support/fixtures';
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
  // What this can honestly assert today is that the form is bound: the header
  // button submits it and the page issues the write. It does not assert the
  // result, because the endpoint the page calls does not exist on this
  // instance — PUT /api/public/admin/user/settings answers 404, so the form
  // has never been able to save. Reported to team-lead; see the fixme below
  // for the confirmation half.
  test('saving a display name issues the write', async ({ developerPage }) => {
    const page = new SettingsPage(developerPage);
    await page.goto();
    await page.displayName().fill('E2E Developer');
    const [request] = await Promise.all([
      developerPage.waitForRequest(
        (r) => r.method() === 'PUT' && r.url().includes('/user/settings'),
      ),
      page.saveButton().click(),
    ]);
    expect(request.url()).toContain('/user/settings');
  });

  // Unskip when sp-toast is fixed. The component's template emits .sp-toast,
  // .sp-icon and .sp-message, and show() queries .toast, .icon and .message, so
  // every call throws on the first null and no toast has ever appeared on any
  // admin page. Shared component, not this page's to change.
  test.fixme('saving confirms in a toast', async ({ developerPage }) => {
    const page = new SettingsPage(developerPage);
    await page.goto();
    await page.saveButton().click();
    await expect(page.toast()).toBeVisible();
  });

  // Destructive and irreversible, so it sits behind the shared confirm dialog.
  // The button is wired to it and the element is on the page; opening it throws
  // inside sp-confirm-dialog ("Cannot read properties of null reading
  // addEventListener"), the same null-selector fault as sp-toast. Unskip when
  // that component is fixed. Cancelling rather than accepting is deliberate:
  // an accepted delete would remove the principal the rest of the suite uses.
  test.fixme('deleting the account asks for confirmation first', async ({ developerPage }) => {
    const page = new SettingsPage(developerPage);
    await page.goto();
    await page.deleteAccountButton().click();
    await expect(developerPage.locator(SEL.dialog)).toBeVisible();
    await page.confirm(false);
  });

  // The safety property, which holds today: the destructive action is a button
  // that asks first, not a link that acts. One click issues no delete. (The
  // confirm dialog element is created on demand by the shared confirm module,
  // so it is legitimately absent from the page until something opens it.)
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
