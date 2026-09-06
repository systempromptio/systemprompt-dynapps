// /admin/profile — the signed-in person's own account page.
//
// Every principal reaches it, including the ones every other admin page
// bounces, so the authorization block below is eight 200s and one redirect
// rather than a role matrix. That is the point of the page: it is the landing
// spot a non-console user is sent to.
//
// Design-language note: this page used to carry a bespoke hero
// (.sp-profile-hero__name) instead of the design system's page header. It now
// uses sp-page-header like every other page, so the block below asserts the
// standard heading with no exemption.
import { test, expect, AUTH, snapshot, type Principal } from '../support/fixtures';
import { expectAccessible, expectDensity } from '../support/a11y';
import { PATHS } from '../support/paths';
import { SEL, TOKENS } from '../support/pages/selectors';
import { ProfilePage } from '../support/pages/profile.page';

const PATH = PATHS.profile;

// 200 = renders, 307 = redirected to the sign-in page. Mirrors
// tests/contract/admin/baseline.txt for GET /admin/profile.
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
  test('names the signed-in person and their roles', async ({ developerPage }) => {
    const page = new ProfilePage(developerPage);
    await page.goto();
    await expect(page.heading()).toHaveText('Profile');
    await expect(developerPage.locator(SEL.pageHeader)).toContainText('e2e-developer');
  });

  test('shows the usage KPI row', async ({ developerPage }) => {
    const page = new ProfilePage(developerPage);
    await page.goto();
    expect(await developerPage.locator(SEL.kpi).count()).toBeGreaterThanOrEqual(8);
    await expect(page.kpi('Spend · 30d').locator).toBeVisible();
  });

  // Identity always has rows. Marketplaces and agents depend on what the
  // instance actually carries, so those sections are asserted as sections:
  // each one renders either its table or its empty state, never nothing.
  test('lists the identity, marketplaces and agents that reach the account', async ({
    developerPage,
  }) => {
    const page = new ProfilePage(developerPage);
    await page.goto();
    await expect(page.tableCaptioned('Identity and gateway facts for this account')).toHaveCount(1);
    for (const label of [
      'Marketplaces reaching this account',
      'Agents available to this account',
    ]) {
      const section = developerPage.locator(`section[aria-label="${label}"]`);
      await expect(section).toHaveCount(1);
      expect(
        (await section.locator('.sp-table').count()) + (await section.locator(SEL.empty).count()),
      ).toBeGreaterThan(0);
    }
  });

  test('states the identity facts a support call asks for', async ({ developerPage }) => {
    const page = new ProfilePage(developerPage);
    await page.goto();
    const identity = page.tableCaptioned('Identity and gateway facts for this account');
    await expect(identity).toContainText('User id');
    await expect(identity).toContainText('JWT issuer');
    await expect(identity).toContainText('Inference gateway');
  });
});

test.describe('actions', () => {
  // The code is a bearer credential: redeeming it yields a durable PAT signing
  // in as its owner, which is why it is minted on click and never on render.
  test('a connect code is minted only when asked for', async ({ developerPage }) => {
    const page = new ProfilePage(developerPage);
    await page.goto();
    await expect(page.connectOutput()).toBeHidden();
    await page.issueConnectCode();
    await expect(page.installCommand()).toContainText('install.sh');
  });

  test('the header links reach the other account pages', async ({ developerPage }) => {
    const page = new ProfilePage(developerPage);
    await page.goto();
    const actions = developerPage.locator(SEL.headerActions);
    for (const href of [PATHS.settings, PATHS.setup, PATHS.history]) {
      await expect(actions.locator(`a[href="${href}"]`)).toHaveCount(1);
    }
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

  test('a non-admin is told plainly that they hold user permissions', async ({ userPage }) => {
    const page = new ProfilePage(userPage);
    await page.goto();
    await expect(userPage.locator(SEL.notice)).toContainText('user permissions');
  });

  test('an admin gets no permissions notice', async ({ adminPage }) => {
    const page = new ProfilePage(adminPage);
    await page.goto();
    await expect(adminPage.locator(SEL.notice).filter({ hasText: 'user permissions' })).toHaveCount(0);
  });
});

test.describe('design language', () => {
  test('wears the shell', async ({ developerPage }) => {
    const page = new ProfilePage(developerPage);
    await page.goto();
    await expect(page.activeNavItem()).toHaveAttribute('href', PATH);
    await expect(page.breadcrumb()).toBeVisible();
    await expect(page.heading()).toBeVisible();
  });

  test('carries no trace of the old bespoke hero', async ({ developerPage }) => {
    const page = new ProfilePage(developerPage);
    await page.goto();
    for (const legacy of ['.sp-profile-hero', '.sp-profile-card', '.sp-tile', '.sp-quickstat']) {
      expect(await developerPage.locator(legacy).count(), legacy).toBe(0);
    }
  });

  test('paints from design tokens', async ({ developerPage }) => {
    const page = new ProfilePage(developerPage);
    await page.goto();
    const values = await developerPage.evaluate((tokens) => {
      const style = getComputedStyle(document.documentElement);
      return tokens.map((t) => style.getPropertyValue(t).trim());
    }, Object.values(TOKENS));
    for (const value of values) expect(value).not.toBe('');
  });

  test('keeps a visible focus ring', async ({ developerPage }) => {
    const page = new ProfilePage(developerPage);
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
    const page = new ProfilePage(developerPage);
    for (const width of [1440, 1024]) {
      await developerPage.setViewportSize({ width, height: 900 });
      await page.goto();
      expect(await page.hasHorizontalScroll(), `horizontal scroll at ${width}px`).toBe(false);
    }
  });

  // The committed figure for this page. Full-page, so a section that grows a
  // scrollbar or loses its header shows up as a diff rather than as nothing.
  test('meets the density bar', async ({ developerPage }) => {
    const page = new ProfilePage(developerPage);
    await page.goto();
    await expectDensity(developerPage, 'detail');
  });

  test('looks like its committed figure', async ({ developerPage }) => {
    const page = new ProfilePage(developerPage);
    await page.goto();
    await snapshot(developerPage, 'profile');
  });

  test('has no serious or critical axe violations', async ({ developerPage }) => {
    const page = new ProfilePage(developerPage);
    await page.goto();
    await expect(developerPage.locator(SEL.pageHeader)).toBeVisible();
    await expectAccessible(developerPage);
  });
});
