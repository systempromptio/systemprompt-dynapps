// /admin/history — the viewer's own conversations, searchable.
//
// Absorbs the former tests/requirements/req-044-045-context-history.spec.ts:
// REQ-044 (context-aware access), REQ-045 and REQ-051 (searchable AI
// interaction history) are proven here, on the page that carries them.
//
// Honesty note kept from that file: the full negative access-control matrix is
// proven in the Rust tier (req_044_access_matrix), and scope resolution
// (self / org-owner / admin) in the history_scope unit test. What a browser
// adds is that the surfaces exist and are identity-gated.
import { test, expect, type Principal } from '../support/fixtures';
import { expectAccessible } from '../support/a11y';
import { PATHS } from '../support/paths';
import { SEL, TOKENS } from '../support/pages/selectors';
import { HistoryPage } from '../support/pages/history.page';

const PATH = PATHS.history;

// Mirrors tests/contract/admin/baseline.txt for GET /admin/history: every
// signed-in principal sees their own conversations; anonymous is sent to login.
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
  test('@REQ-045 lists the seeded conversations', async ({ adminPage }) => {
    const page = new HistoryPage(adminPage);
    await page.goto();
    expect(await page.table().rowCount()).toBeGreaterThan(0);
  });

  test('@REQ-045 offers the search surface to a signed-in user', async ({ adminPage }) => {
    const page = new HistoryPage(adminPage);
    await page.goto();
    await expect(page.searchInput()).toBeVisible();
  });

  test('every row carries its model, turns, tokens and cost', async ({ adminPage }) => {
    const page = new HistoryPage(adminPage);
    await page.goto();
    for (const header of ['When', 'Conversation', 'Source', 'Model', 'Turns', 'Tokens', 'Cost']) {
      expect((await page.table().headers().allInnerTexts()).map((t) => t.trim())).toContain(header);
    }
  });

  test('the page is paginated and says how many rows it holds', async ({ adminPage }) => {
    const page = new HistoryPage(adminPage);
    await page.goto();
    await expect(page.pagination()).toBeVisible();
    await expect(adminPage.locator(SEL.toolbarCount)).toContainText('conversations');
  });

  test('says whose conversations it is showing', async ({ developerPage }) => {
    const page = new HistoryPage(developerPage);
    await page.goto();
    await expect(developerPage.locator(SEL.pageHeader)).toContainText('your own conversations');
  });
});

test.describe('actions', () => {
  test('@REQ-045 searching narrows the table', async ({ adminPage }) => {
    const page = new HistoryPage(adminPage);
    await page.goto();
    const before = await page.table().rowCount();
    await page.search('zzz-no-such-conversation-zzz');
    expect(await page.table().rowCount()).toBeLessThan(before);
  });

  test('@REQ-045 clearing the search restores the full list', async ({ adminPage }) => {
    const page = new HistoryPage(adminPage);
    await page.goto();
    const before = await page.table().rowCount();
    await page.search('zzz-no-such-conversation-zzz');
    await adminPage.getByRole('link', { name: 'Clear' }).click();
    await adminPage.waitForLoadState('networkidle');
    expect(await page.table().rowCount()).toBe(before);
  });

  // The full flow, no seams skipped: a real gateway call answered by the mock
  // inference upstream, then the same transcript-capture webhook the Claude
  // Code hook posts, then history search finding the conversation.
  test('@REQ-045 @REQ-044 a governed mock-model call lands in searchable history', async ({
    platformAdminPage: page,
  }) => {
    const marker = 'orchestrating the Astound governance rollout';

    const issued = await page.request.post('/admin/devices/pats', {
      data: { name: `e2e-flow-${Date.now()}` },
    });
    expect(issued.status()).toBe(200);
    const pat = (await issued.json()) as { id: string; secret: string };

    const inference = await page.request.post('/v1/messages', {
      headers: {
        authorization: `Bearer ${pat.secret}`,
        'x-session-id': 'e2e-session-platform-admin',
        'anthropic-version': '2023-06-01',
      },
      data: {
        model: 'e2e-mock-sonnet',
        max_tokens: 256,
        messages: [{ role: 'user', content: marker }],
      },
    });
    expect(inference.status()).toBe(200);
    const reply = (await inference.json()) as { content: { type: string; text: string }[] };
    expect(reply.content[0].text).toContain(marker);

    const cookies = await page.context().cookies();
    const jwt = cookies.find((c) => c.name === 'access_token')?.value ?? '';
    const capture = await page.request.post('/api/public/hooks/transcript', {
      headers: { authorization: `Bearer ${jwt}` },
      data: {
        session_id: 'e2e-session-platform-admin',
        transcript: [
          { role: 'user', content: marker },
          { role: 'assistant', content: [{ type: 'text', text: reply.content[0].text }] },
        ],
      },
    });
    expect(capture.status()).toBe(204);

    const found = await page.request.get(
      `/admin/api/history/search?q=${encodeURIComponent('governance rollout')}`,
    );
    expect(found.status()).toBe(200);
    expect(JSON.stringify(await found.json())).toContain('e2e-session-platform-admin');

    await page.request.delete(`/admin/devices/pats/${pat.id}`);
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

  test('@REQ-044 the surface is refused signed-out and served signed-in', async ({
    anonPage,
    adminPage,
  }) => {
    await anonPage.goto(PATH);
    expect(new URL(anonPage.url()).pathname).toBe(PATHS.login);
    await adminPage.goto(PATH);
    expect(new URL(adminPage.url()).pathname).toBe(PATH);
  });

  test('@REQ-051 the search API answers in scope', async ({ adminPage }) => {
    const res = await adminPage.request.get('/admin/api/history/search?q=e2e');
    expect(res.status()).toBe(200);
  });

  test('@REQ-051 the search API refuses an out-of-scope user id', async ({ developerPage }) => {
    const res = await developerPage.request.get('/admin/api/history/search?user_id=e2e-victim');
    expect([401, 403]).toContain(res.status());
  });
});

test.describe('design language', () => {
  test('wears the shell', async ({ adminPage }) => {
    const page = new HistoryPage(adminPage);
    await page.goto();
    await expect(page.activeNavItem()).toHaveAttribute('href', PATH);
    await expect(page.breadcrumb()).toBeVisible();
    await expect(page.heading()).toBeVisible();
  });

  test('paints from design tokens', async ({ adminPage }) => {
    const page = new HistoryPage(adminPage);
    await page.goto();
    const values = await adminPage.evaluate((tokens) => {
      const style = getComputedStyle(document.documentElement);
      return tokens.map((t) => style.getPropertyValue(t).trim());
    }, Object.values(TOKENS));
    for (const value of values) expect(value).not.toBe('');
  });

  test('shows an empty state when nothing matches', async ({ adminPage }) => {
    const page = new HistoryPage(adminPage);
    await page.goto();
    await page.search('zzz-no-such-conversation-zzz');
    await page.table().expectEmpty();
  });

  test('keeps a visible focus ring', async ({ adminPage }) => {
    const page = new HistoryPage(adminPage);
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
    const page = new HistoryPage(adminPage);
    for (const width of [1440, 1024]) {
      await adminPage.setViewportSize({ width, height: 900 });
      await page.goto();
      expect(await page.hasHorizontalScroll(), `horizontal scroll at ${width}px`).toBe(false);
    }
  });

  test('has no serious or critical axe violations', async ({ adminPage }) => {
    const page = new HistoryPage(adminPage);
    await page.goto();
    await expect(adminPage.locator(SEL.pageHeader)).toBeVisible();
    await expectAccessible(adminPage);
  });
});
