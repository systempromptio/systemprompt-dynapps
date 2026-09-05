// Screenshot pack for the groups / projects / marketplace-access work.
//
// Unlike scripts/capture.ts (which writes the tracked requirements evidence
// pack into storage/files/images/evidence/), this pack is review material: it
// lands in the gitignored playwright/screenshots/groups/ directory, because
// scripts/check-asset-reachability.py requires every file under
// storage/files/images/ to be referenced by name from the site, and a review
// pack is referenced by nobody.
//
// Reuses the storageStates minted by global-setup and re-mints them on every
// run — an expired token is silently downgraded to anonymous, which turns the
// whole pack into photographs of the login page.
import { mkdirSync, readFileSync, statSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { chromium, type Browser } from '@playwright/test';
import globalSetup, { AUTH } from '../setup/global-setup';

const HERE = __dirname;
const BASE = process.env.GATEWAY_URL ?? 'http://localhost:8080';
const OUT_DIR = join(HERE, '..', 'screenshots', 'groups');

const MAX_BYTES_PER_IMAGE = 700 * 1024;
const MAX_TOTAL_BYTES = 11 * 1024 * 1024;

type PrincipalName = keyof typeof AUTH;

interface Shot {
  name: string;
  title: string;
  path: string;
  principal: PrincipalName;
  fullPage?: boolean;
  focus?: string;
}

const SHOTS: Shot[] = [
  {
    name: 'groups-list',
    title: 'Groups — roster with usage and cost',
    path: '/admin/access/groups',
    principal: 'platformAdmin',
  },
  {
    name: 'group-commerce-overview',
    title: 'Commerce devs — overview (model mix, daily requests, top skills)',
    path: '/admin/access/groups/commerce-devs?tab=overview',
    principal: 'platformAdmin',
  },
  {
    name: 'group-commerce-members',
    title: 'Commerce devs — members with per-member usage',
    path: '/admin/access/groups/commerce-devs?tab=members',
    principal: 'admin',
  },
  {
    name: 'group-commerce-access',
    title: 'Commerce devs — effective access and deciding layer',
    path: '/admin/access/groups/commerce-devs?tab=access',
    principal: 'platformAdmin',
  },
  {
    name: 'group-europe-overview',
    title: 'Europe devs — overview (Vertex-weighted model mix)',
    path: '/admin/access/groups/europe-devs?tab=overview',
    principal: 'admin',
  },
  {
    name: 'group-business-overview',
    title: 'Business users — overview (low-volume Haiku traffic)',
    path: '/admin/access/groups/business-users?tab=overview',
    principal: 'admin',
  },
  {
    name: 'group-business-mappings',
    title: 'Business users — directory group mappings',
    path: '/admin/access/groups/business-users?tab=mappings',
    principal: 'admin',
  },
  {
    name: 'unassigned-assign',
    title: 'Unassigned — assign a signed-in user to a group',
    path: '/admin/access/groups/unassigned',
    principal: 'admin',
    focus: 'select[data-action="assign-group"]',
  },
  {
    name: 'projects-list',
    title: 'Projects — work attribution across groups',
    path: '/admin/access/projects',
    principal: 'platformAdmin',
  },
  {
    name: 'project-storefront',
    title: 'Storefront — members, groups represented, model mix',
    path: '/admin/access/projects/storefront',
    principal: 'platformAdmin',
  },
  {
    name: 'marketplaces-audience-matrix',
    title: 'Marketplaces — who can see what, by group and role',
    path: '/admin/catalog/marketplaces',
    principal: 'platformAdmin',
  },
  {
    name: 'marketplace-commerce-detail',
    title: 'astound-commerce-dev — contents and audience',
    path: '/admin/catalog/marketplaces/astound-commerce-dev',
    principal: 'platformAdmin',
  },
  {
    name: 'catalog-plugins-visibility',
    title: 'Plugin catalog — visibility badges on every card',
    path: '/admin/catalog/plugins',
    principal: 'admin',
  },
  {
    name: 'users-roster-group-filter',
    title: 'Users — roster filtered to Commerce devs',
    path: '/admin/access/users?group=commerce-devs',
    principal: 'admin',
  },
  {
    name: 'user-detail-groups-roles',
    title: 'User detail — group, project and role editing',
    path: '/admin/access/user?id=e2e-developer',
    principal: 'platformAdmin',
  },
  {
    name: 'access-control',
    title: 'Access control — group rules and the per-user matrix',
    path: '/admin/access/control?user=e2e-developer',
    principal: 'platformAdmin',
    fullPage: false,
  },
  {
    name: 'developer-profile-marketplaces',
    title: 'Profile — the marketplaces this developer receives',
    path: '/admin/profile',
    principal: 'developer',
  },
  {
    name: 'pm-groups-readonly',
    title: 'Project manager — the same members tab, without write controls',
    path: '/admin/access/groups/europe-devs?tab=members',
    principal: 'pm',
  },
];

async function capture(browser: Browser, shot: Shot): Promise<number> {
  const context = await browser.newContext({
    baseURL: BASE,
    viewport: { width: 1440, height: 900 },
    storageState: AUTH[shot.principal],
  });
  const page = await context.newPage();
  await page.goto(shot.path, { waitUntil: 'networkidle' });

  if (new URL(page.url()).pathname === '/admin/login') {
    throw new Error(
      `${shot.name}: expected ${shot.path} as ${shot.principal} but landed on /admin/login — ` +
        'the storage state did not authenticate.',
    );
  }
  if (shot.focus) {
    const target = page.locator(shot.focus).first();
    if (await target.count()) await target.scrollIntoViewIfNeeded();
  }

  const file = join(OUT_DIR, `${shot.name}.png`);
  await page.screenshot({ path: file, fullPage: shot.fullPage !== false });
  await context.close();

  const bytes = statSync(file).size;
  if (bytes > MAX_BYTES_PER_IMAGE) {
    console.warn(
      `${shot.name}.png is ${(bytes / 1024).toFixed(0)} KB, over the ${
        MAX_BYTES_PER_IMAGE / 1024
      } KB budget — the review page embeds these as data URIs, so trim the page or crop the shot.`,
    );
  }
  console.log(`captured ${shot.name} (${(bytes / 1024).toFixed(0)} KB)`);
  return bytes;
}

async function main() {
  await globalSetup({ projects: [{ use: { baseURL: BASE } }] } as never);
  mkdirSync(OUT_DIR, { recursive: true });

  const browser = await chromium.launch();
  let total = 0;
  try {
    for (const shot of SHOTS) total += await capture(browser, shot);
  } finally {
    await browser.close();
  }

  const stamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19);
  const manifest = [
    '# Groups, projects and marketplace access — screenshot pack',
    '',
    `Captured ${stamp} against ${BASE} from the deterministic e2e seed`,
    '(`just e2e-seed --reset`). Regenerate with `just e2e-screens-groups`.',
    '',
    `Total ${(total / 1024 / 1024).toFixed(2)} MB across ${SHOTS.length} images.`,
    '',
    '| File | Page | URL | Principal |',
    '|------|------|-----|-----------|',
    ...SHOTS.map((s) => `| ${s.name}.png | ${s.title} | \`${s.path}\` | ${s.principal} |`),
  ].join('\n');
  writeFileSync(join(OUT_DIR, 'index.md'), `${manifest}\n`);

  if (total > MAX_TOTAL_BYTES) {
    throw new Error(
      `the pack is ${(total / 1024 / 1024).toFixed(2)} MB, over the ${
        MAX_TOTAL_BYTES / 1024 / 1024
      } MB budget for a single self-contained review page.`,
    );
  }
  console.log(`screenshots in ${OUT_DIR}`);
  // Read back so a truncated write fails here rather than in the review page.
  readFileSync(join(OUT_DIR, 'index.md'), 'utf8');
}

// Guarded so scripts/review-html.ts can import the manifest without capturing.
if (require.main === module) {
  main().catch((e) => {
    console.error(e);
    process.exit(1);
  });
}

export { SHOTS, OUT_DIR };
export type { Shot };
