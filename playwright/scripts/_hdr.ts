import { chromium } from '@playwright/test';
import { mintToken } from '../setup/mint-token';
const BASE = process.env.GATEWAY_URL ?? 'http://localhost:8081';
(async () => {
  const jwks = (await (await fetch(`${BASE}/.well-known/jwks.json`)).json()) as { keys: { kid: string }[] };
  const token = await mintToken({ userId: 'e2e-platform-admin', email: 'e2e-platform-admin@e2e.local',
    issuer: BASE, kid: jwks.keys[0]?.kid ?? '', pemPath: process.env.E2E_SIGNING_KEY ?? '', sessionId: 'e2e-session-platform-admin' });
  const b = await chromium.launch();
  const ctx = await b.newContext({ viewport: { width: 1440, height: 900 } });
  await ctx.addCookies([{ name: 'access_token', value: token, domain: 'localhost', path: '/' }]);
  const p = await ctx.newPage();
  await p.goto(BASE + '/admin/groups', { waitUntil: 'networkidle' });
  console.log('GROUPS HEADERS', JSON.stringify(await p.evaluate(() =>
    [...document.querySelectorAll('.sp-table__el thead th')].map((th) => {
      const r = th.getBoundingClientRect();
      const range = document.createRange(); range.selectNodeContents(th);
      const textW = Math.ceil(range.getBoundingClientRect().width);
      return { text: th.textContent?.trim().replace(/\s+/g, ' '), boxW: Math.round(r.width), boxH: Math.round(r.height),
               scrollW: th.scrollWidth, clientW: th.clientWidth, textW, lines: Math.round(r.height / 16) };
    })), null, 1));
  await p.goto(BASE + '/admin/profile', { waitUntil: 'networkidle' });
  console.log('PROFILE OVERLAPS', JSON.stringify(await p.evaluate(() => {
    const out: unknown[] = [];
    for (const table of document.querySelectorAll('.sp-table__el')) {
      const tds = [...table.querySelectorAll('tbody tr:first-child td')];
      for (let i = 1; i < tds.length; i++) {
        const a = tds[i - 1].getBoundingClientRect(), c = tds[i].getBoundingClientRect();
        if (a.right > c.left + 2) out.push({ table: table.closest('.sp-table')?.previousElementSibling?.textContent?.trim().slice(0, 40) ?? '?',
          left: tds[i-1].textContent?.trim().slice(0, 24), right: tds[i].textContent?.trim().slice(0, 24),
          leftRight: Math.round(a.right), rightLeft: Math.round(c.left), by: Math.round(a.right - c.left) });
      }
    }
    return out;
  }), null, 1));
  await b.close();
})();
