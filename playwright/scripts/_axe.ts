import { chromium } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
const BASE = process.env.GATEWAY_URL ?? 'http://localhost:8081';
const PAGES = (process.env.AXE_PATHS ?? '/admin/groups,/admin/governance').split(',');
(async () => {
  const b = await chromium.launch();
  const ctx = await b.newContext({
    storageState: '/var/www/html/systemprompt-astound/playwright/screenshots/design-a3/state.json',
    viewport: { width: 1440, height: 900 },
    colorScheme: 'light',
  });
  let bad = 0;
  for (const path of PAGES) {
    const p = await ctx.newPage();
    await p.goto(BASE + path, { waitUntil: 'networkidle' });
    const r = await new AxeBuilder({ page: p }).withTags(['wcag2aa', 'wcag2a']).analyze();
    console.log(`\n${path}  (${p.url()})  violations: ${r.violations.length}`);
    for (const v of r.violations) {
      console.log(`  ${v.id} (${v.impact}) x${v.nodes.length}`);
      for (const n of v.nodes.slice(0, 8)) {
        console.log(`    ${n.target.join(' ')}`);
        console.log(`      ${(n.failureSummary ?? '').split('\n').filter(Boolean).slice(1, 3).join(' | ')}`);
      }
      if (v.impact === 'serious' || v.impact === 'critical') bad += 1;
    }
    await p.close();
  }
  await b.close();
  console.log('\nTOTAL serious/critical:', bad);
})();
