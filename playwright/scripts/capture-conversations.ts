// Review pack for the conversations redesign: sessions, conversations (by
// user and flat), one conversation as admin, the users roster, a user's usage
// tab, the profile, and the owner-facing history + reader.
//
// Lands in the gitignored playwright/screenshots/conversations/ directory for
// the same reason capture-groups.ts does: a review pack is referenced by
// nobody, and scripts/check-asset-reachability.py would reject it under
// storage/files/images/.
import { mkdirSync } from 'node:fs';
import { join } from 'node:path';
import { chromium, type Browser } from '@playwright/test';
import globalSetup, { AUTH } from '../setup/global-setup';

const HERE = __dirname;
const BASE = process.env.GATEWAY_URL ?? 'http://localhost:8080';
const OUT_DIR = join(HERE, '..', 'screenshots', 'conversations');
const CONTEXT = process.env.CONTEXT_ID ?? '';

type PrincipalName = keyof typeof AUTH;

interface Shot {
  name: string;
  path: string;
  principal: PrincipalName;
  fullPage?: boolean;
}

const SHOTS: Shot[] = [
  { name: 'sessions', path: '/admin/sessions', principal: 'platformAdmin' },
  { name: 'conversations-by-user', path: '/admin/contexts', principal: 'platformAdmin' },
  { name: 'conversations-all', path: '/admin/contexts?view=all', principal: 'platformAdmin' },
  { name: 'conversation-admin', path: `/admin/contexts/${CONTEXT}`, principal: 'platformAdmin' },
  { name: 'conversation-admin-requests', path: `/admin/contexts/${CONTEXT}?tab=requests`, principal: 'platformAdmin' },
  { name: 'users', path: '/admin/users', principal: 'platformAdmin', fullPage: false },
  { name: 'profile', path: '/admin/profile', principal: 'developer' },
  { name: 'history', path: '/admin/history', principal: 'developer' },
];

async function capture(browser: Browser, shot: Shot): Promise<void> {
  const context = await browser.newContext({
    baseURL: BASE,
    viewport: { width: 1440, height: Number(process.env.SHOT_HEIGHT ?? 2600) },
    storageState: AUTH[shot.principal],
  });
  const page = await context.newPage();
  const response = await page.goto(shot.path, { waitUntil: 'networkidle' });
  const status = response?.status() ?? 0;
  const landed = new URL(page.url()).pathname;
  console.log(`${shot.name}: ${status} ${landed}`);
  await page.screenshot({ path: join(OUT_DIR, `${shot.name}.png`), fullPage: shot.fullPage !== false });
  await context.close();
}

(async () => {
  await globalSetup({ projects: [{ use: { baseURL: BASE } }] } as never);
  mkdirSync(OUT_DIR, { recursive: true });
  const browser = await chromium.launch();
  for (const shot of SHOTS) {
    if (shot.path.includes('/admin/contexts/') && !CONTEXT) continue;
    await capture(browser, shot);
  }
  await browser.close();
})();
