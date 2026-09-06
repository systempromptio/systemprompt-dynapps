// Per-principal fixtures.
//
// Playwright resolves `storageState` per FILE, not per test — so a spec that
// drives two principals cannot get them by overriding that option (one
// declaration silently wins for the whole file and the other principal runs
// anonymous). Each principal is therefore its own browser context, exposed as
// a page fixture, which composes freely inside one file.
import { test as base, type APIRequestContext, type Browser, type Page } from '@playwright/test';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { Client, type QueryResultRow } from 'pg';
import { databaseUrl } from '../../setup/seed';

const AUTH_DIR = join(__dirname, '..', '..', '.auth');

export const AUTH = {
  admin: join(AUTH_DIR, 'admin.json'),
  platformAdmin: join(AUTH_DIR, 'platform-admin.json'),
  developer: join(AUTH_DIR, 'developer.json'),
  businessUser: join(AUTH_DIR, 'business-user.json'),
  unassigned: join(AUTH_DIR, 'unassigned.json'),
  pm: join(AUTH_DIR, 'pm.json'),
  user: join(AUTH_DIR, 'user.json'),
  bridgeEurope: join(AUTH_DIR, 'bridge-europe.json'),
  bridgeIndia: join(AUTH_DIR, 'bridge-india.json'),
  bridgeKnowledgeWorker: join(AUTH_DIR, 'bridge-knowledge-worker.json'),
  bridgeUnassigned: join(AUTH_DIR, 'bridge-unassigned.json'),
  bridgeAdmin: join(AUTH_DIR, 'bridge-admin.json'),
};

export type Principal = keyof typeof AUTH;

async function pageFor(browser: Browser, principal: Principal, use: (p: Page) => Promise<void>) {
  const context = await browser.newContext({ storageState: AUTH[principal] });
  const page = await context.newPage();
  await use(page);
  await context.close();
}

interface PrincipalPages {
  adminPage: Page;
  platformAdminPage: Page;
  developerPage: Page;
  businessPage: Page;
  unassignedPage: Page;
  pmPage: Page;
  userPage: Page;
  anonPage: Page;
}

export const test = base.extend<PrincipalPages>({
  adminPage: async ({ browser }, use) => {
    await pageFor(browser, 'admin', use);
  },
  platformAdminPage: async ({ browser }, use) => {
    await pageFor(browser, 'platformAdmin', use);
  },
  developerPage: async ({ browser }, use) => {
    await pageFor(browser, 'developer', use);
  },
  businessPage: async ({ browser }, use) => {
    await pageFor(browser, 'businessUser', use);
  },
  unassignedPage: async ({ browser }, use) => {
    await pageFor(browser, 'unassigned', use);
  },
  pmPage: async ({ browser }, use) => {
    await pageFor(browser, 'pm', use);
  },
  userPage: async ({ browser }, use) => {
    await pageFor(browser, 'user', use);
  },
  anonPage: async ({ browser }, use) => {
    const context = await browser.newContext();
    const page = await context.newPage();
    await use(page);
    await context.close();
  },
});

export const expect = base.expect;

/** Cookie header for API-level calls as a given principal. */
export function cookieFor(principal: Principal): { cookie: string } {
  const state = JSON.parse(readFileSync(AUTH[principal], 'utf8'));
  const c = state.cookies[0];
  return { cookie: `${c.name}=${c.value}` };
}

/** API helper: request options carrying a principal's cookie. */
export function apiAs(
  _request: APIRequestContext,
  principal: Principal,
): { headers: { cookie: string } } {
  return { headers: cookieFor(principal) };
}

let runId = '';
/** Unique, per-run e2e email that the seed's --reset backstop will clean up. */
export function uniqueEmail(prefix: string): string {
  if (!runId) runId = Math.random().toString(36).slice(2, 8);
  return `e2e-${prefix}-${runId}-${Date.now() % 100000}@e2e.local`;
}

/** Visual comparison for a page or region.
 *
 *  Baselines are per project and platform (see snapshotPathTemplate), so a
 *  developer on macOS never rewrites the Linux baselines CI compares against;
 *  a run on any other platform asserts nothing rather than failing on font
 *  rendering it cannot control.
 */
export async function snapshot(page: Page, name: string): Promise<void> {
  if (process.platform !== 'linux') return;
  await page.waitForLoadState('networkidle');
  await expect(page).toHaveScreenshot(`${name}.png`, { fullPage: true });
}

/** Read one row straight from the database the seed wrote.
 *
 *  For assertions the UI cannot make honestly — that a delete actually removed
 *  the row rather than only the table cell, that a warn-mode call was audited.
 */
export async function dbRow<T extends QueryResultRow>(
  sql: string,
  params: unknown[] = [],
): Promise<T | null> {
  const db = new Client({ connectionString: databaseUrl() });
  await db.connect();
  try {
    const { rows } = await db.query<T>(sql, params);
    return rows[0] ?? null;
  } finally {
    await db.end();
  }
}

/** Every row of a query, for count and ordering assertions. */
export async function dbRows<T extends QueryResultRow>(
  sql: string,
  params: unknown[] = [],
): Promise<T[]> {
  const db = new Client({ connectionString: databaseUrl() });
  await db.connect();
  try {
    const { rows } = await db.query<T>(sql, params);
    return rows;
  } finally {
    await db.end();
  }
}

/** What global setup found about preconditions the suite cannot satisfy itself. */
export interface Preconditions {
  marketplaceAccess: { satisfied: boolean; neverIngested: boolean; detail: string };
}

export function preconditions(): Preconditions {
  try {
    return JSON.parse(readFileSync(join(AUTH_DIR, 'preconditions.json'), 'utf8')) as Preconditions;
  } catch {
    return { marketplaceAccess: { satisfied: true, neverIngested: false, detail: '' } };
  }
}

/** Skip a spec that cannot mean anything while marketplace entitlement is absent.
 *
 *  Call at the top of a describe or test. A skip carrying the reason stays visible
 *  in every run summary; the alternative was global setup throwing, which stopped
 *  all twelve agents over a fault none of them caused and none could fix.
 */
export function skipWithoutMarketplaceAccess(): void {
  const { marketplaceAccess } = preconditions();
  base.skip(!marketplaceAccess.satisfied, `marketplace entitlement unavailable: ${marketplaceAccess.detail}`);
}
