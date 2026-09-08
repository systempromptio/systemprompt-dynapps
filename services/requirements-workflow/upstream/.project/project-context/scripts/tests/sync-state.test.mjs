// Tests that runSources persists the sync cursor after every source, even when a
// later source throws, so a Jira failure never discards the cursor Confluence
// just earned (and the failing source's cursor is not advanced).

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

import { runSources } from '../cli/sync.mjs';

function tmpStatePath() {
  const dir = mkdtempSync(join(tmpdir(), 'pc-sync-state-'));
  return { dir, statePath: join(dir, '.atlassian_sync_state.json') };
}

const okCounts = { added: 1, updated: 0, unchanged: 2, deleted: 0 };

test('a later source throwing keeps the earlier source cursor and skips the failed one', async () => {
  const { dir, statePath } = tmpStatePath();
  try {
    const state = {};
    const sources = [
      ['confluence', async () => {
        state.confluence = { last_sync: 'C' };
        return okCounts;
      }],
      ['jira', async () => {
        // Real syncJira sets state.jira only after all writes succeed.
        throw new Error('boom');
      }],
    ];

    await assert.rejects(runSources(sources, statePath, state), /boom/);

    const persisted = JSON.parse(readFileSync(statePath, 'utf8'));
    assert.deepEqual(persisted.confluence, { last_sync: 'C' });
    assert.equal(persisted.jira, undefined);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test('all sources succeeding persists every cursor', async () => {
  const { dir, statePath } = tmpStatePath();
  try {
    const state = {};
    const sources = [
      ['confluence', async () => {
        state.confluence = { last_sync: 'C' };
        return okCounts;
      }],
      ['jira', async () => {
        state.jira = { last_sync: 'J' };
        return okCounts;
      }],
    ];

    await runSources(sources, statePath, state);

    const persisted = JSON.parse(readFileSync(statePath, 'utf8'));
    assert.deepEqual(persisted.confluence, { last_sync: 'C' });
    assert.deepEqual(persisted.jira, { last_sync: 'J' });
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});
