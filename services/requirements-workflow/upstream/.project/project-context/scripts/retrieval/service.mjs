// Retrieval service: lazy singletons shared by the MCP tools.
//
// Heavy resources (LanceDB table, Bedrock client) are created on the first tool
// call so the server starts in well under a second. The index `.version` stamp
// is re-checked on each call so a rebuild is picked up without a restart.

import { readFileSync, existsSync } from 'node:fs';
import { join, relative, resolve, isAbsolute } from 'node:path';

import { loadConfig } from '../core/config.mjs';
import { warn } from '../core/log.mjs';

let state = null;
let indexVersion = null;

function versionStamp(config) {
  const f = join(config.indexPath, '.version');
  return existsSync(f) ? readFileSync(f, 'utf8').trim() : '';
}

async function init() {
  const config = loadConfig();
  const current = versionStamp(config);
  if (state && current === indexVersion) return state;

  const { openTable } = await import('./store.mjs');
  const { createBedrock } = await import('../core/bedrock.mjs');

  const bedrock = createBedrock(config);
  const table = await openTable(config, { create: false });
  if (!table) {
    throw new Error(
      'Index is empty. Add files to inbox/ and run `npm run index -- --rebuild`.',
    );
  }

  state = { config, table, bedrock };
  indexVersion = current;
  return state;
}

/**
 * Hybrid search with reranking. Returns an array of result records.
 * `sourceType` is an array of categories (the MCP contract) but a single string
 * or `null` is also accepted for internal/legacy callers; `store.hybridSearch`
 * normalizes it into a pre-ranking `source_type IN (...)` filter.
 */
export async function search(query, { sourceType = null, topK = null } = {}) {
  const { config, table, bedrock } = await init();
  const { hybridSearch } = await import('./store.mjs');

  const k = Math.max(1, Math.min(topK ?? config.retrieval.topK, 20));
  const candidateK = k * config.retrieval.candidateMultiplier;
  const [queryVector] = await bedrock.embed([query]);

  const results = await hybridSearch({
    table,
    bedrock,
    queryText: query,
    queryVector,
    sourceType,
    topK: k,
    candidateK,
    rerank: config.retrieval.rerankEnabled,
  });

  // Prefix inbox-relative paths (legacy index) with the workspace-relative
  // path to project-context so agents can open files directly.
  // config.home = <workspace>/.cursor/.project/project-context, so workspace
  // root is 3 levels up and homePrefix = .cursor/.project/project-context.
  const workspaceRoot = resolve(config.home, '../../..');
  const homePrefix = relative(workspaceRoot, config.home).split('\\').join('/');

  function resolveFile(f) {
    if (!f || isAbsolute(f)) return f;
    // Already workspace-relative (stored by updated indexer or previous fix).
    if (f.startsWith(homePrefix + '/') || f.startsWith('.cursor/') || f.startsWith('.project/')) return f;
    return `${homePrefix}/${f}`;
  }

  return results.map((r) => ({
    text: r.text ?? '',
    file: resolveFile(r.file ?? ''),
    lines: r.lines ?? '',
    title: r.title ?? '',
    source_type: r.source_type ?? '',
    date: r.doc_date ?? '',
    date_source: r.date_source ?? '',
    chunk_index: Number(r.chunk_index ?? 0),
    chunk_total: Number(r.chunk_total ?? 0),
    score: Number(r.score ?? 0),
  }));
}

export async function listSources() {
  const { table } = await init();
  const { getStats } = await import('./store.mjs');
  const stats = await getStats(table);
  return { sources: stats.sourceCounts, last_sync: stats.lastSync };
}

export async function indexStats() {
  const { table } = await init();
  const { getStats } = await import('./store.mjs');
  const stats = await getStats(table);
  return {
    total_chunks: stats.totalChunks,
    source_counts: stats.sourceCounts,
    last_sync: stats.lastSync,
  };
}

/** Reset cached singletons (used by tests). */
export function reset() {
  state = null;
  indexVersion = null;
  warn('service state reset');
}
