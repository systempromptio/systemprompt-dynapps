// LanceDB store: schema, upsert, hybrid retrieval (vector + full-text fused via
// RRF), Bedrock reranking, and stats.
//
// Hybrid search is done manually (two searches merged with reciprocal rank
// fusion) because embeddings are produced externally by Bedrock rather than by a
// LanceDB embedding function. This keeps the store model-agnostic.

import * as lancedb from '@lancedb/lancedb';
import { Index } from '@lancedb/lancedb';
import { Schema, Field, Utf8, Int32, Float64, Float32, Bool, FixedSizeList } from 'apache-arrow';

import { warn } from '../core/log.mjs';

const TABLE = 'chunks';

const RESULT_COLUMNS = [
  'chunk_id',
  'doc_id',
  'text',
  'file',
  'source_type',
  'title',
  'lines',
  'chunk_index',
  'chunk_total',
  'doc_date',
  'date_source',
];

function buildSchema(dim) {
  return new Schema([
    new Field('chunk_id', new Utf8(), true),
    new Field('doc_id', new Utf8(), true),
    new Field('text', new Utf8(), true),
    new Field('vector', new FixedSizeList(dim, new Field('item', new Float32(), true)), false),
    new Field('file', new Utf8(), true),
    new Field('source_type', new Utf8(), true),
    new Field('title', new Utf8(), true),
    new Field('lines', new Utf8(), true),
    new Field('chunk_index', new Int32(), true),
    new Field('chunk_total', new Int32(), true),
    new Field('mtime', new Float64(), true),
    new Field('content_hash', new Utf8(), true),
    new Field('indexed_at', new Utf8(), true),
    // Document dating + archive foundation (see src/ingest/dating.mjs).
    new Field('doc_date', new Utf8(), true), // effective date, YYYY-MM-DD
    new Field('date_source', new Utf8(), true), // frontmatter | llm | mtime
    new Field('archived', new Bool(), true), // archive lifecycle flag (default false)
  ]);
}

export async function connect(config) {
  return lancedb.connect(config.indexPath);
}

/**
 * Open the chunks table. With { create: true } an empty table is created if it
 * does not exist; otherwise a missing table yields null.
 */
export async function openTable(config, { create = false } = {}) {
  const db = await connect(config);
  const names = await db.tableNames();
  if (names.includes(TABLE)) return db.openTable(TABLE);
  if (!create) return null;
  return db.createEmptyTable(TABLE, buildSchema(config.bedrock.embedDim), { existOk: true });
}

export async function dropTable(config) {
  const db = await connect(config);
  const names = await db.tableNames();
  if (names.includes(TABLE)) await db.dropTable(TABLE);
}

function toRow(chunk, vector) {
  return {
    chunk_id: chunk.chunkId,
    doc_id: chunk.docId,
    text: chunk.text,
    vector,
    file: chunk.file,
    source_type: chunk.sourceType,
    title: chunk.title,
    lines: chunk.lines,
    chunk_index: chunk.chunkIndex,
    chunk_total: chunk.chunkTotal,
    mtime: chunk.mtime,
    content_hash: chunk.contentHash,
    indexed_at: chunk.indexedAt,
    doc_date: chunk.docDate ?? '',
    date_source: chunk.dateSource ?? '',
    archived: chunk.archived ?? false,
  };
}

async function deleteSafe(table, predicate) {
  try {
    await table.delete(predicate);
  } catch {
    // empty table or nothing to delete
  }
}

const esc = (s) => String(s).replace(/'/g, "''");

/** Replace all chunks for a document with a fresh batch. */
export async function upsertChunks(table, chunks, vectors) {
  if (chunks.length === 0) return;
  await deleteSafe(table, `doc_id = '${esc(chunks[0].docId)}'`);
  await table.add(chunks.map((c, i) => toRow(c, vectors[i])));
}

export async function deleteDoc(table, docId) {
  await deleteSafe(table, `doc_id = '${esc(docId)}'`);
}

// A doc is up to date when its bytes are unchanged; mtime alone never invalidates
// it (a sync rewrite bumps mtime without changing content).
export function isUpToDate(prev, contentHash) {
  return Boolean(prev) && prev.contentHash === contentHash;
}

/** Map of doc_id -> { mtime, contentHash } for incremental sync. */
export async function getIndexedDocs(table) {
  const seen = new Map();
  // An empty table is the only silent case; a failing query must surface (a
  // swallowed error here masquerades as a first run and forces a full rebuild).
  if ((await table.countRows()) === 0) return seen;
  const rows = await table.query().select(['doc_id', 'mtime', 'content_hash']).toArray();
  for (const r of rows) {
    if (!seen.has(r.doc_id)) {
      seen.set(r.doc_id, { mtime: Number(r.mtime), contentHash: r.content_hash == null ? '' : String(r.content_hash) });
    }
  }
  return seen;
}

/** (Re)build the full-text index over the text column. */
export async function buildFtsIndex(table) {
  try {
    await table.createIndex('text', { config: Index.fts(), replace: true });
  } catch (err) {
    warn(`FTS index build failed: ${err?.message ?? err}`);
  }
}

function rrfMerge(vec, fts, k = 60) {
  const scores = new Map();
  const rows = new Map();
  const bump = (list) => {
    list.forEach((row, rank) => {
      const id = row.chunk_id;
      scores.set(id, (scores.get(id) ?? 0) + 1 / (k + rank + 1));
      if (!rows.has(id)) rows.set(id, row);
    });
  };
  bump(vec);
  bump(fts);
  return [...scores.keys()]
    .sort((a, b) => scores.get(b) - scores.get(a))
    .map((id) => ({ ...rows.get(id), _rrf: scores.get(id) }));
}

/**
 * Vector + full-text hybrid search fused with RRF, then optionally reranked via
 * Bedrock. Returns up to topK records with a `score` field.
 */
export async function hybridSearch({
  table,
  bedrock,
  queryText,
  queryVector,
  sourceType = null,
  topK,
  candidateK,
  rerank = true,
}) {
  // `sourceType` may be an array of categories (the MCP `search` contract) or a
  // single string / null (internal or legacy callers). Filtering runs PRE-ranking
  // on both the vector and FTS candidate queries, so narrowing the categories
  // shrinks the pool before the expensive rerank.
  const types = (Array.isArray(sourceType) ? sourceType : sourceType ? [sourceType] : [])
    .map((t) => String(t).trim())
    .filter(Boolean);
  const where = types.length
    ? `source_type IN (${types.map((t) => `'${esc(t)}'`).join(', ')})`
    : null;

  let vq = table.search(queryVector, 'vector').select(RESULT_COLUMNS).limit(candidateK);
  if (where) vq = vq.where(where);
  const vec = await vq.toArray();

  let fts = [];
  try {
    let fq = table.search(queryText, 'fts').select(RESULT_COLUMNS).limit(candidateK);
    if (where) fq = fq.where(where);
    fts = await fq.toArray();
  } catch (err) {
    warn(`FTS search failed (index not ready?): ${err?.message ?? err}`);
  }

  const merged = rrfMerge(vec, fts);

  if (rerank && bedrock && merged.length > 0) {
    const ranked = await bedrock.rerank(queryText, merged.map((r) => r.text), { topN: topK });
    return ranked.map(({ index, score }) => ({ ...merged[index], score: round(score) }));
  }

  return merged.slice(0, topK).map((r) => ({ ...r, score: round(r._rrf) }));
}

function round(x) {
  return Math.round(Number(x) * 1e4) / 1e4;
}

export async function getStats(table) {
  try {
    const total = await table.countRows();
    const rows = await table.query().select(['source_type', 'doc_id', 'indexed_at']).toArray();
    const perSource = new Map();
    let lastSync = '';
    for (const r of rows) {
      if (!perSource.has(r.source_type)) perSource.set(r.source_type, new Set());
      perSource.get(r.source_type).add(r.doc_id);
      const ts = r.indexed_at ?? '';
      if (ts > lastSync) lastSync = ts;
    }
    const sourceCounts = {};
    for (const [src, docs] of perSource) sourceCounts[src] = docs.size;
    return { totalChunks: Number(total), sourceCounts, lastSync: lastSync || 'never' };
  } catch {
    return { totalChunks: 0, sourceCounts: {}, lastSync: 'never' };
  }
}
