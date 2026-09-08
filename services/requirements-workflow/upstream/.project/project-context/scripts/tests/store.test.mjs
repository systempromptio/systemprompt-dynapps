// Tests for the incremental-sync predicates: isUpToDate (md5 is authoritative,
// mtime never invalidates) and getIndexedDocs (empty table is silent, but a
// failing query must reject rather than masquerade as a first run).

import { test } from 'node:test';
import assert from 'node:assert/strict';

import { isUpToDate, getIndexedDocs } from '../retrieval/store.mjs';

test('isUpToDate: hash is authoritative, mtime is irrelevant', () => {
  assert.equal(isUpToDate(undefined, 'x'), false);
  assert.equal(isUpToDate(null, 'x'), false);
  assert.equal(isUpToDate({ contentHash: 'x', mtime: 1 }, 'x'), true);
  assert.equal(isUpToDate({ contentHash: 'x', mtime: 999999 }, 'x'), true);
  assert.equal(isUpToDate({ contentHash: 'x', mtime: 1 }, 'y'), false);
});

test('getIndexedDocs: empty table returns an empty map without querying', async () => {
  const table = {
    countRows: async () => 0,
    query() {
      throw new Error('query must not run on an empty table');
    },
  };
  const map = await getIndexedDocs(table);
  assert.equal(map.size, 0);
});

test('getIndexedDocs: builds the map, dedupes, and null hash becomes empty string', async () => {
  const rows = [
    { doc_id: 'a', mtime: 10, content_hash: 'h1' },
    { doc_id: 'a', mtime: 11, content_hash: 'h1-dup' },
    { doc_id: 'b', mtime: 20, content_hash: null },
  ];
  const table = {
    countRows: async () => rows.length,
    query: () => ({ select: () => ({ toArray: async () => rows }) }),
  };
  const map = await getIndexedDocs(table);
  assert.equal(map.size, 2);
  assert.deepEqual(map.get('a'), { mtime: 10, contentHash: 'h1' });
  assert.deepEqual(map.get('b'), { mtime: 20, contentHash: '' });
});

test('getIndexedDocs: a failing query rejects instead of returning empty', async () => {
  const table = {
    countRows: async () => 3,
    query() {
      throw new Error('boom');
    },
  };
  await assert.rejects(getIndexedDocs(table), /boom/);
});
