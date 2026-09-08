#!/usr/bin/env node
// Atlassian sync CLI: fetch Confluence + Jira into inbox/, then run the
// incremental indexer.
//
//   npm run sync                     both sources, then index
//   npm run sync -- --source jira    jira only
//   npm run sync -- --no-index       skip indexing
//
// Settings come from .env: ATLASSIAN_SITE, ATLASSIAN_EMAIL, ATLASSIAN_API_TOKEN,
// CONFLUENCE_SPACES, JIRA_PROJECTS, JIRA_JQL, ATLASSIAN_INCLUDE_COMMENTS.

import { readFileSync, writeFileSync, existsSync, realpathSync } from 'node:fs';
import { join } from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

import { loadEnv } from '../core/env.mjs';
import { loadConfig } from '../core/config.mjs';
import { preflight } from '../core/preflight.mjs';
import { log, error } from '../core/log.mjs';
import { settings, syncConfluence, syncJira } from '../ingest/atlassian.mjs';

// Run each source in order, persisting the sync cursor after every source (even on
// failure) so one source throwing never discards the cursor another just earned.
export async function runSources(sources, statePath, state) {
  for (const [label, fn] of sources) {
    try {
      const c = await fn();
      log(`[${label}] done: +${c.added} ~${c.updated} =${c.unchanged} -${c.deleted}`);
    } finally {
      writeFileSync(statePath, JSON.stringify(state, null, 2), 'utf8');
    }
  }
}

async function main() {
  const config = loadConfig();
  const home = config.home;
  loadEnv(config.envPath);
  preflight({ requireAtlassian: true });
  const cfg = settings();

  const args = process.argv.slice(2);
  const source = args.includes('--source') ? args[args.indexOf('--source') + 1] : 'both';
  const noIndex = args.includes('--no-index');

  const statePath = join(home, '.atlassian_sync_state.json');
  const state = existsSync(statePath) ? JSON.parse(readFileSync(statePath, 'utf8')) : {};

  const sources = [];
  if (source === 'both' || source === 'confluence') sources.push(['confluence', () => syncConfluence(cfg, home, state)]);
  if (source === 'both' || source === 'jira') sources.push(['jira', () => syncJira(cfg, home, state)]);
  await runSources(sources, statePath, state);

  if (!noIndex) {
    log('\n-- indexing --');
    const indexScript = fileURLToPath(new URL('./index.mjs', import.meta.url));
    const r = spawnSync(process.execPath, [indexScript, '--sync'], {
      stdio: 'inherit',
      env: { ...process.env, PROJECT_CONTEXT_HOME: home },
    });
    if (r.status !== 0) process.exit(r.status ?? 1);
  }
  log('\n[done] Atlassian sync complete.');
}

function isEntry() {
  try {
    return fileURLToPath(import.meta.url) === realpathSync(process.argv[1]);
  } catch {
    return false;
  }
}

if (isEntry()) {
  main().catch((err) => {
    error(err?.stack ?? String(err));
    process.exit(1);
  });
}
