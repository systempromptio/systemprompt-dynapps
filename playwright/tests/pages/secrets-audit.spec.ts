// /admin/governance/secrets — every recorded touch of a stored credential.
//
// The page exists to answer one question the table cannot answer at a glance:
// did anyone act on a credential they do not own. That is the third-party
// tile and the third-party column, and both are asserted here.
import { test, expect, snapshot, type Principal } from '../support/fixtures';
import { expectAccessible } from '../support/a11y';
import { PATHS } from '../support/paths';
import { SEL, TOKENS } from '../support/pages/selectors';
import { SecretsAuditPage } from '../support/pages/secrets-audit.page';

const PATH = PATHS.secretsAudit;

const ACCESS: { principal: Principal | 'anon'; status: number }[] = [
  { principal: 'platformAdmin', status: 200 },
  { principal: 'admin', status: 200 },
  { principal: 'pm', status: 200 },
  { principal: 'developer', status: 303 },
  { principal: 'businessUser', status: 303 },
  { principal: 'unassigned', status: 303 },
  { principal: 'user', status: 303 },
  { principal: 'anon', status: 303 },
];

test.describe('renders', () => {
  test('lists the seeded secret activity', async ({ adminPage }) => {
    const page = new SecretsAuditPage(adminPage);
    await page.goto({ preset: '30d' });
    expect(await page.table().rowCount()).toBeGreaterThan(0);
    const variables = await page.table().column('Variable');
    expect(variables.join(' ')).toContain('JIRA_TOKEN');
  });

  test('separates the owner from the actor', async ({ adminPage }) => {
    const page = new SecretsAuditPage(adminPage);
    await page.goto({ preset: '30d' });
    const headers = (await page.table().headers().allInnerTexts()).map((h) => h.trim());
    expect(headers).toContain('Owner');
    expect(headers).toContain('Actor');
  });

  test('counts reads and rotations as their own tiles', async ({ adminPage }) => {
    const page = new SecretsAuditPage(adminPage);
    await page.goto({ preset: '30d' });
    expect(await page.kpi('Entries').number()).toBeGreaterThan(0);
    expect(await page.kpi('Reads').value()).not.toBe('');
    expect(await page.kpi('Third party').value()).not.toBe('');
  });
});

test.describe('actions', () => {
  test('filtering by action narrows the trail', async ({ adminPage }) => {
    const page = new SecretsAuditPage(adminPage);
    await page.goto({ preset: '30d' });
    await page.filterByAction('accessed');
    const actions = await page.table().column('Action');
    expect(actions.length).toBeGreaterThan(0);
    for (const action of actions) expect(action.trim()).toBe('accessed');
  });

  test('searching narrows to one variable', async ({ adminPage }) => {
    const page = new SecretsAuditPage(adminPage);
    await page.goto({ preset: '30d' });
    await page.table().filter('JIRA_TOKEN');
    expect(await page.table().rowCount()).toBeGreaterThan(0);
  });

  test('the CSV export carries the third-party column', async ({ adminPage }) => {
    const page = new SecretsAuditPage(adminPage);
    await page.goto({ preset: '30d' });
    const [download] = await Promise.all([
      adminPage.waitForEvent('download'),
      page.csvLink().click(),
    ]);
    const stream = await download.createReadStream();
    const chunks: Buffer[] = [];
    for await (const chunk of stream) chunks.push(Buffer.from(chunk));
    expect(Buffer.concat(chunks).toString('utf8')).toContain('third_party');
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
});

test.describe('design language', () => {
  test('wears the shell', async ({ adminPage }) => {
    const page = new SecretsAuditPage(adminPage);
    await page.goto();
    await expect(page.activeNavItem()).toHaveAttribute('href', PATH);
    await expect(page.breadcrumb()).toBeVisible();
    await expect(page.heading()).toBeVisible();
  });

  test('paints from design tokens', async ({ adminPage }) => {
    const page = new SecretsAuditPage(adminPage);
    await page.goto();
    const values = await adminPage.evaluate((tokens) => {
      const style = getComputedStyle(document.documentElement);
      return tokens.map((t) => style.getPropertyValue(t).trim());
    }, Object.values(TOKENS));
    for (const value of values) expect(value).not.toBe('');
  });

  test('shows an empty state when nothing matches', async ({ adminPage }) => {
    const page = new SecretsAuditPage(adminPage);
    await page.goto({ preset: '30d' });
    await page.table().filter('zzz-no-such-secret-zzz');
    await page.table().expectEmpty();
  });

  test('keeps a visible focus ring', async ({ adminPage }) => {
    const page = new SecretsAuditPage(adminPage);
    await page.goto();
    await adminPage.keyboard.press('Tab');
    const ring = await adminPage.evaluate(() => {
      const el = document.activeElement;
      if (!el) return null;
      const s = getComputedStyle(el);
      return { style: s.outlineStyle, width: s.outlineWidth };
    });
    expect(ring?.style).not.toBe('none');
    expect(Number.parseFloat(ring?.width ?? '0')).toBeGreaterThan(0);
  });

  test('does not scroll horizontally at 1440 or 1024', async ({ adminPage }) => {
    const page = new SecretsAuditPage(adminPage);
    for (const width of [1440, 1024]) {
      await adminPage.setViewportSize({ width, height: 900 });
      await page.goto({ preset: '30d' });
      expect(await page.hasHorizontalScroll(), `horizontal scroll at ${width}px`).toBe(false);
    }
  });

  test('has no serious or critical axe violations', async ({ adminPage }) => {
    const page = new SecretsAuditPage(adminPage);
    await page.goto({ preset: '30d' });
    await expect(adminPage.locator(SEL.pageHeader)).toBeVisible();
    await expectAccessible(adminPage);
  });

  test('matches the committed screenshot', async ({ adminPage }) => {
    const page = new SecretsAuditPage(adminPage);
    await adminPage.setViewportSize({ width: 1440, height: 900 });
    await page.goto({ preset: '30d' });
    await snapshot(adminPage, 'secrets-audit');
  });
});
