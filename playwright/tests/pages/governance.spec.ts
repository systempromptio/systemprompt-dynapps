// /admin/governance — the two enforcement planes over one window.
//
// The page's whole claim is that the governance chain and the safety scanners
// are configured apart and have to be read together, so the KPI strip spans
// both on every tab. That is what the renders block asserts: not that a table
// drew, but that both planes' numbers are on screen at once.
import { test, expect, snapshot, type Principal } from '../support/fixtures';
import { expectAccessible } from '../support/a11y';
import { PATHS } from '../support/paths';
import { SEL, TOKENS } from '../support/pages/selectors';
import { GovernancePage } from '../support/pages/governance.page';

const PATH = PATHS.governance;

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
  test('lists the seeded governance decisions', async ({ adminPage }) => {
    const page = new GovernancePage(adminPage);
    await page.goto({ preset: '30d' });
    expect(await page.table().rowCount()).toBeGreaterThan(0);
    const outcomes = await page.table().column('Outcome');
    expect(outcomes.some((o) => /deny/i.test(o))).toBe(true);
  });

  test('shows both planes in the KPI strip at once', async ({ adminPage }) => {
    const page = new GovernancePage(adminPage);
    await page.goto({ preset: '30d' });
    expect(await page.kpi('Evaluated').number()).toBeGreaterThan(0);
    expect(await page.kpi('Denied').number()).toBeGreaterThan(0);
    expect(await page.kpi('Safety findings').number()).toBeGreaterThan(0);
  });

  test('names all four chain stages as their own tiles', async ({ adminPage }) => {
    const page = new GovernancePage(adminPage);
    await page.goto({ preset: '30d' });
    for (const label of ['Scope denies', 'Secret denies', 'Blocklist denies', 'Rate denies']) {
      expect(await page.kpi(label).number()).toBeGreaterThan(0);
    }
  });

  test('the safety tab lists findings with blocked and audited apart', async ({ adminPage }) => {
    const page = new GovernancePage(adminPage);
    await page.openTab('safety');
    expect(await page.table().rowCount()).toBeGreaterThan(0);
    const outcomes = await page.table().column('Outcome');
    expect(new Set(outcomes.map((o) => o.trim()))).toContain('blocked');
  });

  test('the hooks tab lists hook events and both rankings', async ({ adminPage }) => {
    const page = new GovernancePage(adminPage);
    await page.openTab('hooks');
    expect(await page.table().rowCount()).toBeGreaterThan(0);
    expect(await page.rankLists().count()).toBe(2);
  });

  test('truncates the reason column rather than widening the table', async ({ adminPage }) => {
    const page = new GovernancePage(adminPage);
    await page.goto({ preset: '30d' });
    const cell = page.reasonCells().first();
    await expect(cell).toBeVisible();
    // The full text stays reachable on the title attribute; nothing is lost.
    expect((await cell.getAttribute('title'))?.length ?? 0).toBeGreaterThan(0);
  });
});

test.describe('actions', () => {
  test('filtering by policy narrows the log', async ({ adminPage }) => {
    const page = new GovernancePage(adminPage);
    await page.goto({ preset: '30d' });
    const before = await page.table().rowCount();
    await page.filterBy('policy', 'secret_scan');
    const policies = await page.table().column('Policy');
    expect(policies.length).toBeGreaterThan(0);
    expect(policies.length).toBeLessThanOrEqual(before);
    for (const policy of policies) expect(policy.trim()).toBe('secret_scan');
  });

  test('sorting by policy sets aria-sort and reorders the rows', async ({ adminPage }) => {
    const page = new GovernancePage(adminPage);
    await page.goto({ preset: '30d' });
    const direction = await page.table().sortBy('Policy');
    expect(direction).toBe('ascending');
    const policies = (await page.table().column('Policy')).map((p) => p.trim());
    expect([...policies].sort()).toEqual(policies);
  });

  test('switching tabs re-renders the body and keeps the window', async ({ adminPage }) => {
    const page = new GovernancePage(adminPage);
    await page.goto({ preset: '30d' });
    await page.openTab('safety');
    await expect(page.table().headers().first()).toHaveText(/when/i);
    const headers = await page.table().headers().allInnerTexts();
    expect(headers.map((h) => h.trim())).toContain('Scanner');
  });

  test('paging forward advances the log', async ({ adminPage }) => {
    const page = new GovernancePage(adminPage);
    await page.goto({ preset: '30d' });
    const first = await page.table().column('When');
    await page.table().nextPage();
    const second = await page.table().column('When');
    expect(second).not.toEqual(first);
  });

  test('the CSV export carries both planes', async ({ adminPage }) => {
    const page = new GovernancePage(adminPage);
    await page.goto({ preset: '30d' });
    const [download] = await Promise.all([
      adminPage.waitForEvent('download'),
      page.csvLink().click(),
    ]);
    const stream = await download.createReadStream();
    const chunks: Buffer[] = [];
    for await (const chunk of stream) chunks.push(Buffer.from(chunk));
    const csv = Buffer.concat(chunks).toString('utf8');
    expect(csv).toContain('plane,at,outcome');
    expect(csv).toContain('chain,');
    expect(csv).toContain('safety,');
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
    const page = new GovernancePage(adminPage);
    await page.goto();
    await expect(page.activeNavItem()).toHaveAttribute('href', PATH);
    await expect(page.breadcrumb()).toBeVisible();
    await expect(page.heading()).toBeVisible();
  });

  test('paints from design tokens', async ({ adminPage }) => {
    const page = new GovernancePage(adminPage);
    await page.goto();
    const values = await adminPage.evaluate((tokens) => {
      const style = getComputedStyle(document.documentElement);
      return tokens.map((t) => style.getPropertyValue(t).trim());
    }, Object.values(TOKENS));
    for (const value of values) expect(value).not.toBe('');
  });

  test('shows an empty state when nothing matches', async ({ adminPage }) => {
    const page = new GovernancePage(adminPage);
    await page.goto({ preset: '30d' });
    await page.table().filter('zzz-no-such-row-zzz');
    await page.table().expectEmpty();
  });

  test('keeps a visible focus ring', async ({ adminPage }) => {
    const page = new GovernancePage(adminPage);
    await page.goto();
    await adminPage.keyboard.press('Tab');
    const ring = await adminPage.evaluate(() => {
      const el = document.activeElement;
      if (!el) return null;
      const s = getComputedStyle(el);
      return { style: s.outlineStyle, width: s.outlineWidth };
    });
    expect(ring).not.toBeNull();
    expect(ring?.style).not.toBe('none');
    expect(Number.parseFloat(ring?.width ?? '0')).toBeGreaterThan(0);
  });

  test('does not scroll horizontally at 1440 or 1024', async ({ adminPage }) => {
    const page = new GovernancePage(adminPage);
    for (const width of [1440, 1024]) {
      await adminPage.setViewportSize({ width, height: 900 });
      await page.goto({ preset: '30d' });
      expect(await page.hasHorizontalScroll(), `horizontal scroll at ${width}px`).toBe(false);
    }
  });

  test('has no serious or critical axe violations', async ({ adminPage }) => {
    const page = new GovernancePage(adminPage);
    await page.goto({ preset: '30d' });
    await expect(adminPage.locator(SEL.pageHeader)).toBeVisible();
    await expectAccessible(adminPage);
  });

  test('matches the committed screenshot', async ({ adminPage }) => {
    const page = new GovernancePage(adminPage);
    await adminPage.setViewportSize({ width: 1440, height: 900 });
    await page.goto({ preset: '30d' });
    await snapshot(adminPage, 'governance');
  });
});
