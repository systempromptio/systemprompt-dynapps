// /admin/analytics — six tabs over one scope and one window.
//
// Absorbs the retired tests/analytics-dashboard.spec.ts and
// tests/requirements/req-003-005-analytics.spec.ts: REQ-003 (usage analytics),
// REQ-004 (cost and model analytics) and REQ-005 (adoption) are proven here.
//
// Everything is asserted against the deterministic seed in
// playwright/setup/seed/traffic.ts: 240 requests over a six-model wheel with a
// ten-slot outcome wheel (48 rejected, so 48 rows land in the unrouted bucket),
// 60 tool executions across three servers, and 40 skill invocations that are
// all slash commands — which is why the measured-cost share is 0% and the
// unattributed count is the whole 40.
import { AUTH, test, expect, snapshot, type Principal } from '../support/fixtures';
import { expectAccessible, expectDensity } from '../support/a11y';
import { PATHS } from '../support/paths';
import { SEL, TOKENS } from '../support/pages/selectors';
import { AnalyticsPage, ANALYTICS_TABS, type AnalyticsTab } from '../support/pages/analytics.page';

const PATH = PATHS.analytics;

const ACCESS: { principal: Principal | 'anon'; status: number }[] = [
  { principal: 'platformAdmin', status: 200 },
  { principal: 'admin', status: 200 },
  { principal: 'pm', status: 200 },
  { principal: 'developer', status: 303 },
  { principal: 'businessUser', status: 303 },
  { principal: 'unassigned', status: 303 },
  { principal: 'user', status: 303 },
  { principal: 'anon', status: 307 },
];

// The three models the europe-devs slice must name. Scoped rather than
// unscoped, because the instance-wide view collapses the tail and that is
// exactly what would hide a missing model.
const SEEDED_MODELS = ['claude-opus-5', 'claude-sonnet-5', 'claude-haiku-4-5'];

test.describe('renders', () => {
  for (const tab of ANALYTICS_TABS) {
    test(`the ${tab} tab renders from the seeded dataset`, async ({ adminPage }) => {
      const analytics = new AnalyticsPage(adminPage);
      await analytics.openTab(tab);
      await analytics.expectRendered();
      expect(await analytics.activeTabName()).toContain(tab);
    });
  }

  test('@REQ-003 the KPI strip reports a non-zero request count', async ({
    platformAdminPage: page,
  }) => {
    const analytics = new AnalyticsPage(page);
    await analytics.openTab('overview');
    // The seed writes 240 requests inside 21 days, so a 30d window must be
    // positive; a zero here means the window or the scope resolution broke.
    expect(await analytics.kpi('Requests').number()).toBeGreaterThan(0);
  });

  test('@REQ-004 cost per request is reported, not just total spend', async ({
    platformAdminPage: page,
  }) => {
    const analytics = new AnalyticsPage(page);
    await analytics.openTab('models');
    await expect(analytics.kpi('Spend').locator).toContainText('per request');
  });

  test('@REQ-004 the models tab names every seeded model', async ({
    platformAdminPage: page,
  }) => {
    const analytics = new AnalyticsPage(page);
    await analytics.openTab('models', { group: 'europe-devs' });
    const body = await page.locator('main').innerText();
    for (const model of SEEDED_MODELS) expect(body).toContain(model);
  });

  test('rejected requests appear as an unrouted bucket, never dropped', async ({
    adminPage,
  }) => {
    const analytics = new AnalyticsPage(adminPage);
    await analytics.openTab('models');
    // 48 of the 240 seeded requests are rejected before routing: they carry a
    // NULL model, and the page must show them rather than silently omit them.
    await expect(analytics.tableWithCaption(/by model/i)).toContainText('unrouted');
    expect(await analytics.kpi('Unrouted').number()).toBeGreaterThan(0);
  });

  test('@REQ-005 the overview leaderboard lists the seeded members', async ({
    platformAdminPage: page,
  }) => {
    const analytics = new AnalyticsPage(page);
    await analytics.openTab('overview', { sort: 'requests' });
    const table = analytics.tableWithCaption(/by person/i);
    for (const member of ['e2e-member-1', 'e2e-member-2', 'e2e-member-3']) {
      await expect(table).toContainText(member);
    }
  });

  test('skill cost is measured or blank, with the gap stated', async ({ adminPage }) => {
    const analytics = new AnalyticsPage(adminPage);
    await analytics.openTab('skills');
    // Every seeded invocation is a slash command, so none can be priced. The
    // page must say how many rather than estimating a number.
    expect(await analytics.kpi('Cost measured').value()).toBe('0%');
    expect(await analytics.kpi('Unattributed').number()).toBeGreaterThan(0);
    await expect(adminPage.locator(SEL.notice).first()).toContainText("blank rather than estimated");
  });

  test('the tools tab reports measured execution health per server', async ({ adminPage }) => {
    const analytics = new AnalyticsPage(adminPage);
    await analytics.openTab('tools');
    const servers = analytics.tableWithCaption(/by MCP server/i);
    for (const server of ['systemprompt', 'knowledge-bank', 'atlassian']) {
      await expect(servers).toContainText(server);
    }
  });

  test('the sessions tab labels its figures as client-reported', async ({ adminPage }) => {
    const analytics = new AnalyticsPage(adminPage);
    await analytics.openTab('sessions');
    await expect(adminPage.locator('main')).toContainText('Client-reported');
  });
});

test.describe('actions', () => {
  test('@REQ-003 the bucket toggle changes the plotted series', async ({
    platformAdminPage: page,
  }) => {
    const analytics = new AnalyticsPage(page);
    await analytics.openTab('overview', { bucket: 'day' });
    const daily = await page.locator('figure.sp-svgchart').first().innerHTML();
    await analytics.openTab('overview', { bucket: 'week' });
    const weekly = await page.locator('figure.sp-svgchart').first().innerHTML();
    expect(daily).not.toEqual(weekly);
  });

  test('@REQ-005 the group scope narrows the leaderboard', async ({ adminPage }) => {
    const analytics = new AnalyticsPage(adminPage);
    await analytics.openTab('overview', { group: 'europe-devs' });
    const main = adminPage.locator('main');
    await expect(main).toContainText('e2e-member-1');
    await expect(main).not.toContainText('e2e-member-4');
  });

  test('the project scope narrows the leaderboard the other way', async ({ adminPage }) => {
    const analytics = new AnalyticsPage(adminPage);
    await analytics.openTab('overview', { project: 'payments' });
    const main = adminPage.locator('main');
    await expect(main).toContainText('e2e-member-4');
    await expect(main).not.toContainText('e2e-member-1');
  });

  test('member attribution says on screen that its rows overlap', async ({ adminPage }) => {
    const analytics = new AnalyticsPage(adminPage);
    await analytics.openTab('overview');
    await expect(analytics.overlapNotice()).toHaveCount(0);
    await analytics.setAttribution('Member view');
    await expect(analytics.overlapNotice()).toBeVisible();
  });

  test('every model row drills into the request log carrying the scope', async ({
    adminPage,
  }) => {
    const analytics = new AnalyticsPage(adminPage);
    await analytics.openTab('models', { group: 'europe-devs' });
    const hrefs = await analytics.drillHrefs();
    expect(hrefs.length).toBeGreaterThan(0);
    for (const href of hrefs) {
      expect(href).toContain('model=');
      expect(href).toContain('group=europe-devs');
      expect(href).toContain('preset=30d');
    }
  });

  test('@REQ-004 the cost tab exports a CSV whose customer view has no cost column', async ({
    adminPage,
  }) => {
    const analytics = new AnalyticsPage(adminPage);
    await analytics.openTab('cost', { audience: 'customer' });
    const res = await adminPage.request.get(
      '/admin/analytics/cost.csv?audience=customer&preset=30d',
    );
    expect(res.status()).toBe(200);
    const body = await res.text();
    expect(body.split('\n')[0].trim()).toBe(
      'container,users,sessions,requests,input_tokens,output_tokens',
    );
    expect(body.toLowerCase()).not.toContain('cost');
  });

  test('@REQ-004 the internal cost export carries the supplier figure', async ({ adminPage }) => {
    const res = await adminPage.request.get(
      '/admin/analytics/cost.csv?audience=internal&preset=30d',
    );
    expect(res.status()).toBe(200);
    expect((await res.text()).split('\n')[0].trim()).toBe('provider,requests,tokens,cost_usd');
  });

  test('the cost tab switches its container axis between group and project', async ({
    adminPage,
  }) => {
    const analytics = new AnalyticsPage(adminPage);
    await analytics.openTab('cost', { axis: 'project' });
    await expect(analytics.tableWithCaption(/each container consumed/i)).toBeVisible();
    await expect(adminPage.locator('main')).toContainText('Project');
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

  test('the cost export refuses the same principals the page does', async ({ browser }) => {
    const context = await browser.newContext({ storageState: AUTH.developer });
    const res = await context.request.get('/admin/analytics/cost.csv', { maxRedirects: 0 });
    expect(res.status()).toBe(303);
    await context.close();
  });
});

test.describe('design language', () => {
  test('wears the shell', async ({ adminPage }) => {
    const analytics = new AnalyticsPage(adminPage);
    await analytics.openTab('models');
    await expect(analytics.activeNavItem()).toHaveAttribute('href', /\/admin\/analytics/);
    await expect(analytics.breadcrumb()).toBeVisible();
    await expect(analytics.heading()).toBeVisible();
    await expect(adminPage.locator(SEL.breadcrumbCurrent)).toHaveText('Models');
  });

  test('paints from design tokens', async ({ adminPage }) => {
    const analytics = new AnalyticsPage(adminPage);
    await analytics.openTab('overview');
    const values = await adminPage.evaluate((tokens) => {
      const style = getComputedStyle(document.documentElement);
      return tokens.map((t) => style.getPropertyValue(t).trim());
    }, Object.values(TOKENS));
    for (const value of values) expect(value).not.toBe('');
  });

  test('@REQ-003 an empty window renders empty states on every tab, never a 500', async ({
    adminPage,
  }) => {
    for (const tab of ANALYTICS_TABS) {
      const res = await adminPage.goto(
        `${PATH}?tab=${tab}&from=2000-01-01T00:00:00Z&to=2000-01-08T00:00:00Z`,
      );
      expect(res?.status(), `tab=${tab}`).toBe(200);
      await expect(adminPage.locator(SEL.empty).first()).toBeVisible();
    }
  });

  test('paging past the seeded rows renders cleanly', async ({ adminPage }) => {
    const res = await adminPage.goto(`${PATH}?tab=tools&page=99`);
    expect(res?.status()).toBe(200);
  });

  // The first tab stop is the shell's skip link, which draws its ring with
  // box-shadow rather than outline. Both are visible indicators, so this asks
  // for one of them rather than for the specific property the other pages
  // happen to use — demanding `outline` here would fail a control that is
  // perfectly visible.
  test('keeps a visible focus ring', async ({ adminPage }) => {
    const analytics = new AnalyticsPage(adminPage);
    await analytics.openTab('overview');
    await adminPage.keyboard.press('Tab');
    const ring = await adminPage.evaluate(() => {
      const el = document.activeElement;
      if (!el) return null;
      const s = getComputedStyle(el);
      return { outline: s.outlineStyle, width: s.outlineWidth, shadow: s.boxShadow };
    });
    expect(ring).not.toBeNull();
    const outlined = ring?.outline !== 'none' && Number.parseFloat(ring?.width ?? '0') > 0;
    const shadowed = (ring?.shadow ?? 'none') !== 'none';
    expect(outlined || shadowed, `focused element draws no visible ring: ${JSON.stringify(ring)}`).toBe(true);
  });

  // The four table-first tabs are measured against the list bar. Overview and
  // Cost lead with a chart on purpose — the Overview's job is the shape of the
  // instance and the Cost tab's is what the spend was made of, and a chart
  // answers both before a table can — so they are held to the same bar for
  // everything except rows above the fold, and to a minimum table length
  // instead. Excluding them from the row count is a stated exception, not a
  // silent one.
  const TABLE_FIRST: AnalyticsTab[] = ['models', 'skills', 'tools', 'sessions'];

  test('meets the density bar on every table-first tab', async ({ adminPage }) => {
    const analytics = new AnalyticsPage(adminPage);
    await adminPage.setViewportSize({ width: 1440, height: 900 });
    for (const tab of TABLE_FIRST) {
      await analytics.openTab(tab);
      await expectDensity(adminPage, 'list');
    }
  });

  test('the chart-first tabs still carry a table worth scrolling to', async ({ adminPage }) => {
    const analytics = new AnalyticsPage(adminPage);
    await adminPage.setViewportSize({ width: 1440, height: 900 });
    for (const tab of ['overview', 'cost'] as AnalyticsTab[]) {
      await analytics.openTab(tab);
      const rows = await adminPage.locator(`${SEL.tableEl} tbody tr`).count();
      expect(rows, `${tab} leads with a chart, so its table must be worth the scroll`)
        .toBeGreaterThanOrEqual(8);
      // Everything the bar checks other than rows-above-the-fold still applies.
      expect(await analytics.hasHorizontalScroll(), `${tab} scrolls sideways`).toBe(false);
    }
  });

  test('does not scroll horizontally at 1440 or 1024', async ({ adminPage }) => {
    const analytics = new AnalyticsPage(adminPage);
    for (const width of [1440, 1024]) {
      await adminPage.setViewportSize({ width, height: 900 });
      await analytics.openTab('models');
      expect(await analytics.hasHorizontalScroll(), `horizontal scroll at ${width}px`).toBe(false);
    }
  });

  test('looks the same as its committed baseline on every tab', async ({ adminPage }) => {
    const analytics = new AnalyticsPage(adminPage);
    for (const tab of ANALYTICS_TABS) {
      await analytics.openTab(tab);
      await snapshot(adminPage, `analytics-${tab}`);
    }
  });

  test('has no serious or critical axe violations on any tab', async ({ adminPage }) => {
    const analytics = new AnalyticsPage(adminPage);
    for (const tab of ANALYTICS_TABS) {
      await analytics.openTab(tab);
      await expect(adminPage.locator(SEL.pageHeader)).toBeVisible();
      await expectAccessible(adminPage);
    }
  });
});
