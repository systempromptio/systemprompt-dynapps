import { chromium } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import { mintToken } from '../setup/mint-token';
const BASE = process.env.GATEWAY_URL ?? 'http://localhost:8081';
(async () => {
  const jwks = (await (await fetch(`${BASE}/.well-known/jwks.json`)).json()) as { keys: { kid: string }[] };
  const token = await mintToken({
    userId: 'e2e-platform-admin', email: 'e2e-platform-admin@e2e.local',
    issuer: BASE, kid: jwks.keys[0]?.kid ?? '',
    pemPath: process.env.E2E_SIGNING_KEY ?? '', sessionId: 'e2e-session-platform-admin',
  });
  const b = await chromium.launch();
  const ctx = await b.newContext({ viewport: { width: 1440, height: 900 }, colorScheme: 'light' });
  await ctx.addCookies([{ name: 'access_token', value: token, domain: 'localhost', path: '/' }]);
  let bad = 0;
  for (const path of (process.env.AXE_PATHS ?? '/admin/users').split(',')) {
    const p = await ctx.newPage();
    const r0 = await p.goto(BASE + path, { waitUntil: 'networkidle' });
    const r = await new AxeBuilder({ page: p }).withTags(['wcag2aa', 'wcag2a']).analyze();
    const overlap = await p.evaluate(() => {
      const d = document.querySelector('.sp-install-dropdown') as HTMLElement | null;
      if (!d) return 'no dropdown';
      const box = d.getBoundingClientRect();
      return `${Math.round(box.width)}x${Math.round(box.height)} display=${getComputedStyle(d).display}`;
    });
    console.log(`${r0?.status()} ${path}  violations: ${r.violations.length}  install-dropdown: ${overlap}`);
    for (const v of r.violations) {
      console.log(`    ${v.id} (${v.impact}) x${v.nodes.length} ${v.nodes[0]?.target.join(' ')}`);
      console.log(`      ${(v.nodes[0]?.failureSummary ?? '').split('\n').filter(Boolean).slice(1, 2).join('')}`);
      if (v.impact === 'serious' || v.impact === 'critical') bad += 1;
    }
    await p.close();
  }
  await b.close();
  console.log('TOTAL serious/critical:', bad);
})();
