// Design-system evidence for the admin migration: four representative pages at
// 1440x900, plus the counts the change set is judged on — visible table rows
// above the fold, and whether any card paints its shadow at a different width
// from its own content, which is what the mid-block shadow was.
import { mkdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { chromium } from '@playwright/test';
import { writeFileSync as wf } from 'node:fs';
import { mintToken } from '../setup/mint-token';


const BASE = process.env.GATEWAY_URL ?? 'http://localhost:8080';
const OUT = join(__dirname, '..', 'screenshots', 'design-a3');

const PAGES = [
  { name: 'governance', path: '/admin/governance', state: '' },
  { name: 'users', path: '/admin/users', state: '' },
  { name: 'requests', path: '/admin/requests', state: '' },
  { name: 'traces', path: '/admin/traces', state: '' },
  { name: 'user-detail', path: '/admin/user?id=d32ac864-ff79-4b41-9a84-3376cd887355', state: '' },
  { name: 'profile', path: '/admin/profile', state: '' },
];

async function main() {
  // Mints its own cookie rather than running global-setup: seeding takes a
  // database lock this clone's other agents are already holding, and these
  // captures only need to read pages the seed data already backs.
  const jwks = (await (await fetch(`${BASE}/.well-known/jwks.json`)).json()) as { keys: { kid: string }[] };
  const kid = jwks.keys[0]?.kid ?? '';
  const issuer = process.env.CAPTURE_ISSUER ?? BASE;
  const token = await mintToken({
    userId: process.env.CAPTURE_USER_ID ?? '',
    email: process.env.CAPTURE_EMAIL ?? '',
    issuer,
    kid,
    pemPath: process.env.E2E_SIGNING_KEY ?? join(__dirname, '..', '..', 'signing_key.pem'),
    sessionId: process.env.CAPTURE_SESSION_ID ?? '',
  });
  const state = join(OUT, 'state.json');
  mkdirSync(OUT, { recursive: true });
  wf(state, JSON.stringify({ cookies: [{ name: 'access_token', value: token, domain: new URL(BASE).hostname, path: '/', httpOnly: true, secure: false, sameSite: 'Lax', expires: Math.floor(Date.now() / 1000) + 7200 }], origins: [] }));
  mkdirSync(OUT, { recursive: true });
  const browser = await chromium.launch();
  const report: string[] = [];
  for (const page of PAGES) {
    const ctx = await browser.newContext({
      storageState: state,
      viewport: { width: 1440, height: 900 },
      deviceScaleFactor: 1,
      colorScheme: 'light',
      reducedMotion: 'reduce',
    });
    const p = await ctx.newPage();
    await p.goto(`${BASE}${page.path}`, { waitUntil: 'networkidle' });
    await p.screenshot({ path: join(OUT, `${page.name}.png`) });
    const stats = await p.evaluate(() => {
      const main = document.querySelector('.sp-shell__main');
      const mainBox = main?.getBoundingClientRect();
      const rows = [...document.querySelectorAll('.sp-table__el tbody tr')].filter((r) => {
        const b = r.getBoundingClientRect();
        return b.top >= 0 && b.bottom <= window.innerHeight;
      }).length;
      // A card whose own box is wider than the content it wraps is the defect.
      const mismatched = [...document.querySelectorAll('.sp-table, .sp-card, .sp-kpi')].filter((el) => {
        const box = el.getBoundingClientRect();
        const inner = el.firstElementChild?.getBoundingClientRect();
        return inner ? Math.abs(box.left - inner.left) > 24 : false;
      }).length;
      const header = document.querySelector('.sp-page-header')?.getBoundingClientRect();
      const toolbar = document.querySelector('.sp-toolbar')?.getBoundingClientRect();
      const kpi = document.querySelector('.sp-kpi-grid')?.getBoundingClientRect();
      const all = document.querySelectorAll('.sp-table__el tbody tr');
      const first = all[0]?.getBoundingClientRect();
      return {
        rowsInViewport: rows,
        rowsInDom: all.length,
        rowHeightPx: first ? Math.round(first.height) : null,
        rowsThatWouldFit: first && first.height
          ? Math.floor((window.innerHeight - first.top) / first.height)
          : null,
        mismatchedSurfaces: mismatched,
        mainPaddingLeft: main ? getComputedStyle(main).paddingLeft : null,
        mainWidth: mainBox?.width ?? null,
        headerAboveFold: header ? header.bottom <= window.innerHeight : null,
        kpiAboveFold: kpi ? kpi.bottom <= window.innerHeight : null,
        toolbarAboveFold: toolbar ? toolbar.bottom <= window.innerHeight : null,
      };
    });
    report.push(`${page.name} ${page.path}\n  ${JSON.stringify(stats)}`);
    await ctx.close();
  }
  await browser.close();
  writeFileSync(join(OUT, 'report.txt'), `${report.join('\n')}\n`);
  console.log(report.join('\n'));
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
