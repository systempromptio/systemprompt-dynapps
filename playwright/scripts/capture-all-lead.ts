//! Capture every admin section for the redesign artifact (team-lead, throwaway).
import { mkdirSync, writeFileSync as wf } from 'node:fs';
import { join } from 'node:path';
import { chromium } from '@playwright/test';
import { mintToken } from '../setup/mint-token';
const BASE = process.env.GATEWAY_URL ?? 'http://localhost:8081';
const OUT = process.env.OUT ?? '/tmp/shots';
const PAGES: [string, string][] = [
  ['overview', '/admin'], ['users', '/admin/users'], ['user-detail', '/admin/users/e2e-developer'],
  ['groups', '/admin/groups'], ['group-detail', '/admin/groups/europe-devs'], ['projects', '/admin/projects'],
  ['project-detail', '/admin/projects/storefront'], ['roles', '/admin/roles'], ['access-control', '/admin/access-control'],
  ['devices', '/admin/devices'], ['analytics-overview', '/admin/analytics?tab=overview'], ['analytics-models', '/admin/analytics?tab=models'],
  ['analytics-skills', '/admin/analytics?tab=skills'], ['analytics-tools', '/admin/analytics?tab=tools'],
  ['analytics-sessions', '/admin/analytics?tab=sessions'], ['analytics-cost', '/admin/analytics?tab=cost'],
  ['requests', '/admin/requests'], ['sessions', '/admin/sessions'], ['traces', '/admin/traces'], ['contexts', '/admin/contexts'],
  ['governance', '/admin/governance'], ['governance-safety', '/admin/governance?tab=safety'], ['approvals', '/admin/governance/approvals'],
  ['secrets-audit', '/admin/governance/secrets'], ['mcp', '/admin/mcp'], ['marketplaces', '/admin/marketplaces'],
  ['plugins', '/admin/plugins'], ['skills', '/admin/skills'], ['gateway', '/admin/gateway'], ['profile', '/admin/profile'],
];
async function main() {
  const jwks = (await (await fetch(`${BASE}/.well-known/jwks.json`)).json()) as { keys: { kid: string }[] };
  const token = await mintToken({ userId: 'e2e-platform-admin', email: 'e2e-platform-admin@e2e.local', issuer: BASE,
    kid: jwks.keys[0]?.kid ?? '', pemPath: process.env.E2E_SIGNING_KEY ?? '', sessionId: 'e2e-session-platform-admin' });
  mkdirSync(OUT, { recursive: true });
  const state = join(OUT, 'state.json');
  wf(state, JSON.stringify({ cookies: [{ name: 'access_token', value: token, domain: new URL(BASE).hostname, path: '/', httpOnly: true, secure: false, sameSite: 'Lax', expires: Math.floor(Date.now() / 1000) + 7200 }], origins: [] }));
  const browser = await chromium.launch();
  const report: string[] = [];
  for (const [name, path] of PAGES) {
    const ctx = await browser.newContext({ storageState: state, viewport: { width: 1440, height: 900 }, colorScheme: 'light', reducedMotion: 'reduce' });
    const p = await ctx.newPage();
    const resp = await p.goto(`${BASE}${path}`, { waitUntil: 'networkidle' });
    await p.screenshot({ path: join(OUT, `${name}.png`) });
    const m = await p.evaluate(`(() => {
      const H = window.innerHeight;
      const vis = (el) => { const b = el.getBoundingClientRect(); return b.top >= 0 && b.bottom <= H; };
      const rows = [...document.querySelectorAll('.sp-table__el tbody tr')];
      const rowsVisible = rows.filter(vis).length;
      const rowH = rows[0] ? Math.round(rows[0].getBoundingClientRect().height) : null;
      const kpi = document.querySelector('.sp-kpi-grid');
      const kpiH = kpi ? Math.round(kpi.getBoundingClientRect().height) : null;
      const overflowCells = [...document.querySelectorAll('.sp-table__el td')].filter((td) => td.scrollWidth > td.clientWidth + 2 || td.scrollHeight > td.clientHeight + 2).length;
      const overlap = (() => { let n = 0; const tds = [...document.querySelectorAll('.sp-table__el tbody tr:first-child td')]; for (let i = 1; i < tds.length; i++) { const a = tds[i-1].getBoundingClientRect(), b = tds[i].getBoundingClientRect(); if (a.right > b.left + 2) n++; } return n; })();
      const hscroll = [...document.querySelectorAll('.sp-table__scroll, .sp-shell__main')].filter((el) => el.scrollWidth > el.clientWidth + 2).length;
      const bodyScroll = document.documentElement.scrollWidth > document.documentElement.clientWidth;
      const tabs = [...document.querySelectorAll('[role=tab]')].map((t) => t.textContent?.trim());
      const dupTabs = new Set(tabs).size < tabs.length;
      const zeroSubs = [...document.querySelectorAll('.sp-kpi__note, .sp-kpi__delta, .sp-section__count')].filter((e) => e.textContent?.trim() === '0').length;
      const filterRows = document.querySelectorAll('.sp-toolbar, .sp-scope-filter, .sp-filter-ribbon, form.sp-filters').length;
      const crumbs = !!document.querySelector('.sp-breadcrumbs, nav[aria-label=Breadcrumb]');
      const emptyH1 = !document.querySelector('.sp-page-header');
      return { rowsVisible, rowH, kpiH, overflowCells, overlap, hscroll, bodyScroll, tabs: tabs.length, dupTabs, zeroSubs, filterRows, crumbs, noHeader: emptyH1 };
    })()`);
    report.push(`${name} ${resp?.status()} ${JSON.stringify(m)}`);
    await ctx.close();
  }
  await browser.close();
  wf(join(OUT, 'report.txt'), report.join('\n'));
  console.log(report.join('\n'));
}
main().catch((e) => { console.error(e); process.exit(1); });
