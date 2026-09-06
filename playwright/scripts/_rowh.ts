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
  for (const path of (process.env.PATHS ?? '/admin/mcp').split(',')) {
    const p = await ctx.newPage();
    await p.goto(BASE + path, { waitUntil: 'networkidle' });
    console.log(path, JSON.stringify(await p.evaluate(() => {
      const rows = [...document.querySelectorAll('.sp-table__el tbody tr')];
      if (!rows.length) return 'no rows';
      const tallest = rows.reduce((a, r) => (r.getBoundingClientRect().height > a.getBoundingClientRect().height ? r : a));
      const rh = Math.round(tallest.getBoundingClientRect().height);
      const cells = [...tallest.querySelectorAll('td')].map((td) => {
        const cs = getComputedStyle(td);
        const kids = [...td.querySelectorAll('*')].map((e) => {
          const k = getComputedStyle(e);
          return { tag: e.tagName.toLowerCase(), cls: (e.className || '').toString().slice(0, 34),
                   h: Math.round(e.getBoundingClientRect().height), lh: k.lineHeight, pad: `${k.paddingTop}/${k.paddingBottom}`, fs: k.fontSize };
        });
        const tallestKid = kids.sort((a, x) => x.h - a.h)[0];
        return { h: Math.round(td.getBoundingClientRect().height), pad: `${cs.paddingTop}/${cs.paddingBottom}`,
                 lh: cs.lineHeight, tallestKid };
      }).sort((a, x) => x.h - a.h).slice(0, 3);
      return { rowH: rh, cells };
    }), null, 1));
    await p.close();
  }
  await b.close();
})();
