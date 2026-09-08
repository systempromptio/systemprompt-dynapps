#!/usr/bin/env node
// Incremental indexer CLI.
//
//   npm run index            normalize + incremental sync (default)
//   npm run index -- --rebuild        drop the index and rebuild from scratch
//   npm run index -- --normalize-only only convert binaries to _normalized.txt
//
// Flow: normalize binaries -> walk indexable files -> chunk -> embed (Bedrock)
// -> upsert into LanceDB. Unchanged files (same md5) are skipped.

import { readFile, stat, writeFile } from 'node:fs/promises';
import { realpathSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { join, relative, resolve, extname, basename } from 'node:path';
import { fileURLToPath } from 'node:url';

import { loadEnv } from '../core/env.mjs';
import { loadConfig } from '../core/config.mjs';
import { createBedrock } from '../core/bedrock.mjs';
import { mapLimit, createMutex } from '../core/concurrency.mjs';
import { walk } from '../core/walk.mjs';
import { materialize, NORMALIZED_SUFFIX } from '../ingest/normalizer.mjs';
import { chunkDocument } from '../ingest/chunker.mjs';
import { loadManifest, saveManifest, resolveWithManifest } from '../ingest/dating.mjs';
import { openTable, dropTable, getIndexedDocs, isUpToDate, upsertChunks, deleteDoc, buildFtsIndex, getStats } from '../retrieval/store.mjs';
import { log } from '../core/log.mjs';

const NATIVE_TEXT = new Set(['.txt', '.md']);

/** A file is indexable if it is a normalized sidecar or a native text file. */
function isIndexable(path) {
  const name = basename(path);
  if (name.endsWith(NORMALIZED_SUFFIX)) return true;
  return NATIVE_TEXT.has(extname(name).toLowerCase());
}

async function md5(path) {
  return createHash('md5').update(await readFile(path)).digest('hex');
}

/** Manifest key for a doc: its path relative to the source folder. */
function manifestKey(docId, sourceType) {
  return docId.slice(sourceType.length + 1);
}

async function main() {
  const args = new Set(process.argv.slice(2));
  const rebuild = args.has('--rebuild');
  const normalizeOnly = args.has('--normalize-only');

  const config = loadConfig();
  loadEnv(config.envPath);

  log('[step 1/2] Materializing source files into inbox...');
  const norm = await materialize(config);
  log(`[info] normalize: created=${norm.created} skipped=${norm.skipped} failed=${norm.failed}`);
  if (normalizeOnly) {
    log('[done] normalize-only complete.');
    return;
  }

  const bedrock = createBedrock(config);
  const llm = (system, user) => bedrock.converse(system, user);

  if (rebuild) {
    log('[step 2/2] Rebuilding index from scratch...');
    await dropTable(config);
  } else {
    log('[step 2/2] Incremental sync...');
  }

  const table = await openTable(config, { create: true });
  const indexed = rebuild ? new Map() : await getIndexedDocs(table);

  // Per-source date manifests (inbox/<type>/.dates.json). Loaded once, updated as
  // documents are dated, and written back at the end (pruned to current files).
  const manifests = new Map(); // sourceType -> { dir, data }
  for (const sourceType of Object.keys(config.sources)) {
    const dir = join(config.inboxPath, sourceType);
    manifests.set(sourceType, { dir, data: await loadManifest(dir) });
  }

  // Pass 1: enumerate files and decide what to (re)index.
  const currentDocIds = new Set();
  const currentKeys = new Map(); // sourceType -> Set(manifest keys)
  const toProcess = [];
  let skipped = 0;
  let failed = 0;

  for await (const filePath of walk(config.inboxPath)) {
    if (!isIndexable(filePath)) continue;
    const rel = relative(config.inboxPath, filePath).split('\\').join('/');
    const sourceType = rel.split('/')[0];
    // Only index files that live under a configured source type.
    if (!rel.includes('/') || !config.sources[sourceType]) continue;
    const docId = rel;
    currentDocIds.add(docId);
    if (!currentKeys.has(sourceType)) currentKeys.set(sourceType, new Set());
    currentKeys.get(sourceType).add(manifestKey(docId, sourceType));

    let mtime;
    let contentHash;
    try {
      mtime = (await stat(filePath)).mtimeMs / 1000;
      contentHash = await md5(filePath);
    } catch (err) {
      log(`[warn] cannot read ${filePath}: ${err?.message ?? err}`);
      failed++;
      continue;
    }

    const prev = indexed.get(docId);
    if (isUpToDate(prev, contentHash)) {
      skipped++;
      continue;
    }
    // filePath (absolute) is read from disk; fileRel is the openable pointer
    // stored in the index as a workspace-relative path so agents can open it
    // directly (e.g. .cursor/.project/project-context/inbox/<type>/<relpath>).
    // config.home is always <workspace>/.cursor/.project/project-context, so
    // the workspace root sits exactly three levels up.
    const workspaceRoot = resolve(config.home, '../../..');
    const fileRel = relative(workspaceRoot, filePath).split('\\').join('/');
    toProcess.push({ filePath, fileRel, docId, sourceType, mtime, contentHash, isNew: !prev });
  }

  // Pass 2: chunk + embed documents concurrently; serialize only the writes.
  const concurrency = Math.max(1, config.indexing.docConcurrency);
  log(`[info] processing ${toProcess.length} document(s) with concurrency=${concurrency}`);
  let added = 0;
  let changed = 0;
  let done = 0;
  const writeLock = createMutex();

  await mapLimit(toProcess, concurrency, async (doc) => {
    const content = await readFile(doc.filePath, 'utf8');
    if (!content.trim()) {
      log(`[warn] empty file, skipping ${doc.docId}`);
      failed++;
      return;
    }
    try {
      // Resolve the document's effective date (frontmatter -> LLM -> mtime),
      // cached by md5 in the source manifest so unchanged files never re-run the LLM.
      const man = manifests.get(doc.sourceType);
      const { date: docDate, source: dateSource } = await resolveWithManifest(
        man.data,
        manifestKey(doc.docId, doc.sourceType),
        { content, mtime: doc.mtime, md5: doc.contentHash, llm, useLLM: config.dates.extractViaLLM },
      );

      const chunks = await chunkDocument({
        filePath: doc.fileRel,
        sourceType: doc.sourceType,
        docId: doc.docId,
        content,
        mtime: doc.mtime,
        contentHash: doc.contentHash,
        docDate,
        dateSource,
        config,
        llm,
      });
      if (chunks.length === 0) {
        log(`[warn] no chunks produced for ${doc.docId}`);
        return;
      }
      const vectors = await bedrock.embed(chunks.map((c) => c.text));
      await writeLock(() => upsertChunks(table, chunks, vectors));
      if (doc.isNew) added++;
      else changed++;
      log(`[index] (${++done}/${toProcess.length}) ${doc.docId} -> ${chunks.length} chunk(s)`);
    } catch (err) {
      log(`[error] ${doc.docId}: ${err?.message ?? err}`);
      failed++;
    }
  });

  const allIndexed = new Set((await getIndexedDocs(table)).keys());
  let pruned = 0;
  for (const docId of allIndexed) {
    if (!currentDocIds.has(docId)) {
      log(`[prune] ${docId}`);
      await deleteDoc(table, docId);
      pruned++;
    }
  }

  // Persist date manifests: prune entries for files no longer present, then write
  // (only non-empty manifests, sorted keys for stable git diffs).
  for (const [sourceType, man] of manifests) {
    const keep = currentKeys.get(sourceType) ?? new Set();
    for (const key of Object.keys(man.data)) {
      if (!keep.has(key)) delete man.data[key];
    }
    if (Object.keys(man.data).length > 0) await saveManifest(man.dir, man.data);
  }

  // The FTS index is a full rebuild over the whole table, so only run it when the
  // row set actually changed; an untouched table keeps a valid index from last run.
  if (added + changed + pruned > 0) {
    log('[info] refreshing full-text index...');
    await buildFtsIndex(table);
  } else {
    log('[info] no document changes — skipping full-text index rebuild.');
  }

  const stats = await getStats(table);
  log(`\n[done] added=${added} changed=${changed} skipped=${skipped} failed=${failed} pruned=${pruned}`);
  log(`[done] total chunks: ${stats.totalChunks}`);
  for (const [src, count] of Object.entries(stats.sourceCounts).sort()) {
    log(`       ${src}: ${count} doc(s)`);
  }

  await writeFile(join(config.indexPath, '.version'), stats.lastSync, 'utf8');
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
    log(`[fatal] ${err?.stack ?? err}`);
    process.exit(1);
  });
}
