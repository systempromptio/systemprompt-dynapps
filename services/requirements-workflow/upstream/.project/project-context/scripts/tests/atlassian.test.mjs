// Tests for the Atlassian ingest: byte-identical document builders, the
// write-if-changed guard (no mtime bump on unchanged bytes), the JQL builder,
// and the retry/backoff policy driven with an injected fetch + sleep.

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, statSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

import {
  buildConfluenceDoc,
  buildJiraDoc,
  buildJiraJql,
  writeIfChanged,
  requestWithRetry,
} from '../ingest/atlassian.mjs';

function tmpDir() {
  return mkdtempSync(join(tmpdir(), 'pc-atlassian-'));
}

// A minimal fetch Response stand-in.
function res(status, { body = '', retryAfter = null, json = null } = {}) {
  return {
    ok: status >= 200 && status < 300,
    status,
    headers: { get: (name) => (name.toLowerCase() === 'retry-after' ? retryAfter : null) },
    text: async () => body,
    json: async () => json ?? {},
  };
}

test('buildConfluenceDoc produces the exact expected markdown', () => {
  const page = { title: 'Hello', id: '123', spaceId: 'DEV', version: { createdAt: '2026-01-02T03:04:05Z' } };
  const out = buildConfluenceDoc(page, '<p>Body text.</p>', '**bob** (2026-01-01):\nNice.', {
    site: 'example.atlassian.net',
  });
  const expected =
    '---\n' +
    'title: "Hello"\n' +
    'source_type: confluence\n' +
    'page_id: 123\n' +
    'space: DEV\n' +
    'updated: 2026-01-02\n' +
    'url: https://example.atlassian.net/wiki/spaces/DEV/pages/123\n' +
    '---\n' +
    '\n' +
    '# Hello\n' +
    '\n' +
    'Body text.\n' +
    '\n' +
    '## Comments\n' +
    '\n' +
    '**bob** (2026-01-01):\nNice.\n';
  assert.equal(out, expected);
});

test('buildJiraDoc produces the exact expected markdown', () => {
  const issue = {
    key: 'ABC-1',
    fields: {
      summary: 'Do it',
      status: { name: 'Open' },
      issuetype: { name: 'Task' },
      priority: { name: 'High' },
      assignee: { displayName: 'Ann' },
      reporter: { displayName: 'Bob' },
      created: '2026-01-01T10:00:00.000+0000',
      updated: '2026-01-02T11:00:00.000+0000',
      description: 'Just do it.',
      labels: ['a', 'b'],
      components: [{ name: 'ui' }],
      fixVersions: [{ name: 'v1' }],
    },
  };
  const out = buildJiraDoc(issue, { site: 'ex.atlassian.net' });
  const expected =
    '---\n' +
    'title: "[ABC-1] Do it"\n' +
    'source_type: jira\n' +
    'key: ABC-1\n' +
    'status: Open\n' +
    'type: Task\n' +
    'priority: High\n' +
    'assignee: Ann\n' +
    'reporter: Bob\n' +
    'created: 2026-01-01\n' +
    'updated: 2026-01-02\n' +
    'labels: a, b\n' +
    'components: ui\n' +
    'fix_versions: v1\n' +
    'url: https://ex.atlassian.net/browse/ABC-1\n' +
    '---\n' +
    '\n' +
    '# [ABC-1] Do it\n' +
    '\n' +
    '**Status:** Open  |  **Type:** Task  |  **Priority:** High\n' +
    '**Assignee:** Ann  |  **Reporter:** Bob\n' +
    '**Created:** 2026-01-01  |  **Updated:** 2026-01-02\n' +
    '\n' +
    '## Description\n' +
    '\n' +
    'Just do it.\n' +
    '\n';
  assert.equal(out, expected);
});

test('writeIfChanged: added, then unchanged (no mtime bump), then updated', () => {
  const dir = tmpDir();
  try {
    const path = join(dir, 'doc.md');

    assert.equal(writeIfChanged(path, 'v1'), 'added');
    const mtime1 = statSync(path).mtimeMs;

    assert.equal(writeIfChanged(path, 'v1'), 'unchanged');
    assert.equal(statSync(path).mtimeMs, mtime1, 'unchanged content must not touch the file');

    assert.equal(writeIfChanged(path, 'v2'), 'updated');
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test('buildJiraJql', () => {
  assert.equal(buildJiraJql({ projects: ['A', 'B'], jql: '' }), 'project IN (A, B)');
  assert.equal(
    buildJiraJql({ projects: ['A'], jql: 'status = Open ORDER BY updated DESC' }),
    'project IN (A) AND (status = Open) ORDER BY updated DESC',
  );
  assert.equal(buildJiraJql({ projects: [], jql: 'assignee = me' }), 'assignee = me');
});

test('requestWithRetry: 429 with Retry-After retries once then succeeds', async () => {
  let calls = 0;
  const sleeps = [];
  const fetchImpl = async () => {
    calls++;
    return calls === 1 ? res(429, { retryAfter: '0' }) : res(200);
  };
  const out = await requestWithRetry('u', {}, { fetchImpl, sleep: async (ms) => sleeps.push(ms) });
  assert.equal(out.status, 200);
  assert.equal(calls, 2);
  assert.deepEqual(sleeps, [0]);
});

test('requestWithRetry: 503 backs off exponentially then succeeds', async () => {
  let calls = 0;
  const sleeps = [];
  const fetchImpl = async () => {
    calls++;
    return calls <= 2 ? res(503) : res(200);
  };
  const out = await requestWithRetry('u', {}, { fetchImpl, sleep: async (ms) => sleeps.push(ms) });
  assert.equal(out.status, 200);
  assert.equal(calls, 3);
  assert.deepEqual(sleeps, [500, 1000]);
});

test('requestWithRetry: 401 throws immediately without retry', async () => {
  let calls = 0;
  const fetchImpl = async () => {
    calls++;
    return res(401, { body: 'nope' });
  };
  await assert.rejects(
    requestWithRetry('u', {}, { fetchImpl, sleep: async () => {} }),
    /401.*nope/,
  );
  assert.equal(calls, 1);
});

test('requestWithRetry: retryable status is exhausted after MAX_ATTEMPTS', async () => {
  let calls = 0;
  const sleeps = [];
  const fetchImpl = async () => {
    calls++;
    return res(503, { body: 'busy' });
  };
  await assert.rejects(
    requestWithRetry('u', {}, { fetchImpl, sleep: async (ms) => sleeps.push(ms) }),
    /503/,
  );
  assert.equal(calls, 5);
  assert.equal(sleeps.length, 4);
});

test('requestWithRetry: network errors are retried then surface after exhaustion', async () => {
  let calls = 0;
  const fetchImpl = async () => {
    calls++;
    throw new Error('ECONNRESET');
  };
  await assert.rejects(
    requestWithRetry('u', {}, { fetchImpl, sleep: async () => {} }),
    /failed after 5 attempts.*ECONNRESET/,
  );
  assert.equal(calls, 5);
});
