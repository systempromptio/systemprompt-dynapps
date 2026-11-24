// The live page runtime (`storage/files/js/workspace/live/runtime-*.js`) against
// Anthropic's artifact runtime contract 0.2.99: `use()` and its alias, the
// 10 s answer when no host replies, each namespace's requests and error
// codes, and `watchTool` replay, coalescing and polling. The runtime runs
// in this realm over a fake window whose parent plays the viewer shell.
import test, { afterEach, beforeEach, mock } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

// The frame route inlines the three parts in this order, as one script.
const SOURCE = ['runtime-core.js', 'runtime-mcp.js', 'runtime-use.js']
  .map((part) => readFileSync(new URL(`../../storage/files/js/workspace/live/${part}`, import.meta.url), 'utf8'))
  .join('');
const ORIGIN = 'https://governance.test';
const PROTOCOL = 'governance-live/1';

// Every test runs on mocked timers, so the runtime's 10 s and reply-budget
// timers never hold the run open; a test moves time with `mock.timers.tick`.
beforeEach(() => mock.timers.enable({ apis: ['setTimeout', 'setInterval'] }));
afterEach(() => mock.timers.reset());

const flush = async (rounds = 8) => {
  for (let round = 0; round < rounds; round += 1) await Promise.resolve();
};

// A page frame: the runtime evaluated over a fake `window` and `document`.
// `shell` answers each request the way the test says, or leaves it pending.
const boot = ({ framed = true, served = null, answer = null, host = { origins: [ORIGIN], contract: '0.2.99' } } = {}) => {
  const listeners = [];
  const visibility = [];
  const sent = [];
  const document = {
    visibilityState: 'visible',
    addEventListener: (type, fn) => { if (type === 'visibilitychange') visibility.push(fn); },
    querySelector: () => null,
  };
  const window = { addEventListener: (type, fn) => { if (type === 'message') listeners.push(fn); } };
  const deliver = (data, source = window.parent, origin = ORIGIN) => listeners.forEach((fn) => fn({ data, source, origin }));
  const parent = {
    postMessage: (message) => {
      sent.push(message);
      if (message.type === 'hello' && served) Promise.resolve().then(() => deliver({ governance: PROTOCOL, type: 'welcome', served, budgetMs: 130000 }));
      if (message.type === 'request' && answer) {
        Promise.resolve().then(async () => {
          try {
            const result = await answer(message.method, message.params);
            if (result !== undefined) deliver({ governance: PROTOCOL, type: 'reply', id: message.id, result });
          } catch (error) {
            deliver({ governance: PROTOCOL, type: 'reply', id: message.id, error });
          }
        });
      }
    },
  };
  window.parent = framed ? parent : window;
  const source = SOURCE.replace('/*HOST*/null', JSON.stringify(host));
  new Function('window', 'document', source)(window, document);
  const requests = (method) => sent.filter((m) => m.type === 'request' && (!method || m.method === method));
  const hidden = (value) => {
    document.visibilityState = value ? 'hidden' : 'visible';
    visibility.forEach((fn) => fn());
  };
  return { window, document, sent, requests, deliver, hidden };
};

const SERVED = ['permissions', 'mcp', 'user', 'artifact'];

test('window.claude is window.governance: one frozen object that offers only use', () => {
  const page = boot();
  assert.equal(page.window.claude, page.window.governance);
  assert.ok(Object.isFrozen(page.window.claude));
  assert.deepEqual(Object.keys(page.window.claude), ['use']);
  assert.throws(() => { page.window.claude = { use: () => 'replaced' }; }, TypeError);
  assert.throws(() => { page.window.governance.use = () => null; }, TypeError);
  assert.equal(page.window.claude, page.window.governance, 'the alias cannot be replaced');
});

test('use() memoizes a capability, aliases self to artifact, and answers null for unknown names', async () => {
  const page = boot({ served: SERVED });
  const { use } = page.window.governance;
  assert.equal(use('mcp'), use('mcp'));
  assert.equal(use('self'), use('artifact'));
  assert.equal(await use('db'), null);
  assert.equal(await use('not-a-capability'), null);
  const mcp = await use('mcp');
  assert.ok(Object.isFrozen(mcp));
  assert.deepEqual(Object.keys(mcp).sort(), ['callTool', 'describeTool', 'invalidate', 'listTools', 'server', 'watchTool']);
  assert.equal(await page.window.claude.use('mcp'), mcp, 'the alias resolves the same namespace');
});

test('a capability the shell does not serve resolves null', async () => {
  const page = boot({ served: ['permissions', 'user', 'artifact'] });
  assert.equal(await page.window.governance.use('mcp'), null);
  assert.notEqual(await page.window.governance.use('user'), null);
});

test('top level, every use() resolves null and nothing is posted', async () => {
  const page = boot({ framed: false });
  assert.equal(await page.window.governance.use('mcp'), null);
  assert.equal(page.sent.length, 0);
});

test('framed by a host that never answers, use() resolves null after 10 s and not before', async () => {
  const page = boot();
  let answered = 'pending';
  page.window.governance.use('mcp').then((value) => { answered = value; });
  mock.timers.tick(9999);
  await flush();
  assert.equal(answered, 'pending');
  mock.timers.tick(1);
  await flush();
  assert.equal(answered, null);
  assert.ok(page.sent.filter((m) => m.type === 'hello').length >= 2, 'the runtime says hello until answered');
});

test('only the parent at a listed origin is heard', async () => {
  const page = boot();
  const other = { postMessage: () => {} };
  const welcome = { governance: PROTOCOL, type: 'welcome', served: SERVED };
  page.deliver(welcome, other);
  page.deliver(welcome, page.window.parent, 'https://evil.test');
  let resolved = false;
  page.window.governance.use('mcp').then(() => { resolved = true; });
  await flush();
  assert.equal(resolved, false, 'a welcome from another source or origin is ignored');
  page.deliver(welcome);
  assert.notEqual(await page.window.governance.use('mcp'), null);
});

test('callTool asks the shell and resolves the result with its payload', async () => {
  const page = boot({
    served: SERVED,
    answer: (method, params) => {
      if (params.tool === 'structured') return { content: [{ type: 'text', text: 'x' }], structuredContent: { rows: 2 } };
      if (params.tool === 'text-json') return { content: [{ type: 'text', text: '{"rows":3}' }] };
      return { content: [{ type: 'text', text: 'plain words' }] };
    },
  });
  const mcp = await page.window.governance.use('mcp');
  assert.deepEqual((await mcp.callTool('odoo', 'structured', { b: 1, a: 2 })).payload, { rows: 2 });
  assert.deepEqual((await mcp.callTool('odoo', 'text-json')).payload, { rows: 3 });
  assert.equal((await mcp.callTool('odoo', 'plain')).payload, 'plain words');
  const [first] = page.requests('mcp.callTool');
  assert.deepEqual(first.params, { server: 'odoo', tool: 'structured', input: { b: 1, a: 2 }, cache: undefined });
  assert.deepEqual(page.requests('mcp.callTool')[1].params.input, {}, 'no input is the input-less call');
});

test('a tool failure rejects tool_error with the whole result; shell errors keep their code', async () => {
  const failed = { content: [{ type: 'text', text: 'no such partner' }], isError: true };
  const page = boot({
    served: SERVED,
    answer: (method, params) => {
      if (params.tool === 'fails') return failed;
      throw { code: 'server_unavailable', message: 'down', retryable: true, server: 'odoo' };
    },
  });
  const mcp = await page.window.governance.use('mcp');
  await assert.rejects(mcp.callTool('odoo', 'fails'), (error) => {
    assert.equal(error.code, 'tool_error');
    assert.equal(error.message, 'no such partner');
    assert.deepEqual(error.result, failed);
    return true;
  });
  await assert.rejects(mcp.callTool('odoo', 'other'), (error) => error.code === 'server_unavailable' && error.retryable === true);
});

test('callTool never throws synchronously: bad arguments reject bad_request without reaching the shell', async () => {
  const page = boot({ served: SERVED, answer: () => ({ content: [] }) });
  const mcp = await page.window.governance.use('mcp');
  const before = page.requests().length;
  await assert.rejects(mcp.callTool(42, 'tool'), { code: 'bad_request' });
  await assert.rejects(mcp.callTool('odoo', 'tool', { when: new Date() }), { code: 'bad_request' });
  await assert.rejects(mcp.callTool('odoo', 'tool', { ids: new Set([1]) }), { code: 'bad_request' });
  await assert.rejects(mcp.callTool('odoo', 'tool', { n: 1n }), { code: 'bad_request' });
  await assert.rejects(mcp.callTool('odoo', 'tool', { file: { $file: { data: new ArrayBuffer(1), name: 'a.txt', type: 'text/plain' } } }), { code: 'capability_disabled' });
  assert.equal(page.requests().length, before);
});

test('an aborted call rejects cancelled at once and tells the shell', async () => {
  const page = boot({ served: SERVED, answer: () => undefined });
  const mcp = await page.window.governance.use('mcp');
  const controller = new AbortController();
  const call = mcp.callTool('odoo', 'slow', {}, { signal: controller.signal });
  await flush();
  controller.abort();
  await assert.rejects(call, { code: 'cancelled' });
  assert.equal(page.sent.filter((m) => m.type === 'cancel').length, 1);
});

test('an unanswered request rejects upstream_error after the reply budget; progress keeps it alive', async () => {
  const page = boot();
  page.deliver({ governance: PROTOCOL, type: 'welcome', served: SERVED, budgetMs: 1000 });
  await flush();
  const mcp = await page.window.governance.use('mcp');
  let outcome = 'pending';
  mcp.callTool('odoo', 'slow').catch((error) => { outcome = error.code; });
  await flush();
  const [request] = page.requests('mcp.callTool');
  mock.timers.tick(900);
  page.deliver({ governance: PROTOCOL, type: 'progress', id: request.id });
  mock.timers.tick(900);
  await flush();
  assert.equal(outcome, 'pending', 'a progress note re-arms the budget');
  mock.timers.tick(100);
  await flush();
  assert.equal(outcome, 'upstream_error');
});

test('watchTool throws TypeError only for a missing handler and delivers no earlier than a microtask', async () => {
  const page = boot({ served: SERVED, answer: () => ({ content: [{ type: 'text', text: '1' }] }) });
  const mcp = await page.window.governance.use('mcp');
  assert.throws(() => mcp.watchTool('odoo', 'read', {}, null), TypeError);
  const events = [];
  const unsubscribe = mcp.watchTool('odoo', 'read', {}, (event) => events.push(event));
  assert.equal(typeof unsubscribe, 'function');
  assert.equal(events.length, 0, 'nothing is delivered synchronously');
  await flush();
  assert.equal(events[0].type, 'data');
  unsubscribe();
  unsubscribe();
});

test('watches of one identity share one flight, whatever the key order of their input', async () => {
  const page = boot({ served: SERVED, answer: () => ({ content: [{ type: 'text', text: '{"n":1}' }] }) });
  const mcp = await page.window.governance.use('mcp');
  const first = [];
  const second = [];
  mcp.watchTool('odoo', 'read', { a: 1, b: 2 }, (event) => first.push(event));
  mcp.watchTool('odoo', 'read', { b: 2, a: 1 }, (event) => second.push(event));
  await flush(16);
  assert.equal(page.requests('mcp.watch').length, 1, 'one flight for both watches');
  assert.equal(page.requests('mcp.watch')[0].params.mode, 'peek');
  assert.deepEqual(first.map((event) => event.result.payload), [{ n: 1 }]);
  assert.deepEqual(second.map((event) => event.result.payload), [{ n: 1 }]);
});

test('a watch replays the entry a cached call left, marked revalidating, then refreshes it', async () => {
  let count = 0;
  const page = boot({ served: SERVED, answer: () => ({ content: [{ type: 'text', text: String(++count) }] }) });
  const mcp = await page.window.governance.use('mcp');
  await mcp.callTool('odoo', 'read', { id: 7 });
  const events = [];
  mcp.watchTool('odoo', 'read', { id: 7 }, (event) => events.push(event));
  await flush(16);
  assert.equal(events[0].result.payload, 1);
  assert.equal(events[0].result.cache.revalidating, true);
  assert.equal(page.requests('mcp.watch')[0].params.mode, 'refresh');
  assert.equal(events[1].result.payload, 2, 'the refresh is delivered next');
});

test('a cached call feeds watchers of the same identity; a call that opts out of caching does not', async () => {
  let count = 0;
  const page = boot({ served: SERVED, answer: () => ({ content: [{ type: 'text', text: String(++count) }] }) });
  const mcp = await page.window.governance.use('mcp');
  const events = [];
  mcp.watchTool('odoo', 'read', null, (event) => events.push(event));
  await flush(16);
  await mcp.callTool('odoo', 'read', {}, { cache: false });
  assert.equal(events.length, 1);
  await mcp.callTool('odoo', 'read', {});
  assert.deepEqual(events.map((event) => event.result.payload), [1, 3]);
});

test('polling is clamped to 30 s, paused while the page is hidden and caught up on return', async () => {
  const page = boot({ served: SERVED, answer: () => ({ content: [{ type: 'text', text: '1' }] }) });
  await flush();
  const mcp = await page.window.governance.use('mcp');
  mcp.watchTool('odoo', 'read', {}, () => {}, { refetchInterval: 1000 });
  await flush(16);
  const polls = () => page.requests('mcp.watch').filter((m) => m.params.mode === 'refresh').length;
  mock.timers.tick(29999);
  await flush();
  assert.equal(polls(), 0, 'no poll before the 30 s floor');
  mock.timers.tick(1);
  await flush(16);
  assert.equal(polls(), 1);
  page.hidden(true);
  mock.timers.tick(30000);
  await flush(16);
  assert.equal(polls(), 1, 'no poll while hidden');
  page.hidden(false);
  await flush(16);
  assert.equal(polls(), 2, 'one catch-up poll on return');
});

test('the 65th watch of a view gets bad_request on its handler', async () => {
  const page = boot({ served: SERVED, answer: () => undefined });
  const mcp = await page.window.governance.use('mcp');
  for (let index = 0; index < 64; index += 1) mcp.watchTool('odoo', 'read', { index }, () => {});
  const events = [];
  mcp.watchTool('odoo', 'read', { index: 64 }, (event) => events.push(event));
  await flush();
  assert.equal(events[0].type, 'error');
  assert.equal(events[0].error.code, 'bad_request');
});

test('invalidate narrows by its arguments and re-runs the watched identities it covers', async () => {
  let count = 0;
  const page = boot({ served: SERVED, answer: (method) => (method === 'mcp.invalidate' ? null : { content: [{ type: 'text', text: String(++count) }] }) });
  const mcp = await page.window.governance.use('mcp');
  await assert.rejects(mcp.invalidate(undefined, undefined, {}), { code: 'bad_request' });
  const events = [];
  mcp.watchTool('odoo', 'read', {}, (event) => events.push(event));
  mcp.watchTool('m365', 'read', {}, () => {});
  await flush(16);
  await mcp.invalidate('odoo', 'read', null);
  await flush(16);
  assert.deepEqual(page.requests('mcp.invalidate')[0].params, { server: 'odoo', tool: 'read', input: {} });
  const refreshed = page.requests('mcp.watch').filter((m) => m.params.mode === 'refresh').map((m) => m.params.server);
  assert.deepEqual(refreshed, ['odoo']);
  assert.equal(events.at(-1).result.payload, 3);
});

test('server() hands a frozen handle whose methods are the listed tools and resolve payloads', async () => {
  const page = boot({
    served: SERVED,
    answer: (method, params) => {
      if (method === 'mcp.listTools') {
        if (params.server === 'empty') return { servers: [{ server: 'empty', authStatus: 'connected', tools: [] }] };
        return { servers: [{ server: 'odoo', authStatus: 'connected', tools: [{ name: 'search' }, { name: 'then' }, { name: 'toString' }, { name: 'get-doc' }] }] };
      }
      return { content: [{ type: 'text', text: '{"ok":true}' }] };
    },
  });
  const mcp = await page.window.governance.use('mcp');
  const odoo = await mcp.server('odoo');
  assert.ok(Object.isFrozen(odoo));
  assert.deepEqual(Object.keys(odoo).sort(), ['get-doc', 'search']);
  assert.deepEqual(await odoo.search({ q: 1 }), { ok: true });
  await assert.rejects(mcp.server('empty'), { code: 'server_not_connected' });
});

test('permissions, user and artifact talk to the shell with the contract shapes', async () => {
  const page = boot({
    served: SERVED,
    answer: (method, params) => {
      if (method === 'permissions.state') return params.name ? 'prompt' : { 'mcp:odoo': 'prompt' };
      if (method === 'permissions.request') return { 'mcp:odoo': 'granted' };
      if (method === 'user.me') return { id: 'u1', name: 'Ana', avatarUrl: 'data:x', color: '#000', email: null, isOwner: true, canEdit: false };
      if (method === 'artifact.publish') return { version: 'v2' };
      throw { code: 'unavailable', message: 'no panel' };
    },
  });
  const permissions = await page.window.governance.use('permissions');
  assert.equal(await permissions.state('mcp:odoo'), 'prompt');
  assert.deepEqual(await permissions.request(['mcp:odoo']), { 'mcp:odoo': 'granted' });
  await assert.rejects(permissions.manage(), { code: 'unavailable' });
  const user = await page.window.governance.use('user');
  assert.equal(await user.isOwner(), true);
  assert.equal(await user.canEdit(), false);
  assert.equal(await user.id(), 'u1');
  const artifact = await page.window.governance.use('artifact');
  assert.deepEqual(await artifact.publish('<!doctype html><p>x'), { version: 'v2' });
  await assert.rejects(artifact.publish({ 'index.html': 'x' }), { code: 'capability_disabled' });
  await assert.rejects(artifact.publish(7), { code: 'transform_error' });
  assert.equal('metadata' in artifact, false, 'metadata is not offered on this runtime');
});

test('user reads never reject: a failing shell answers the all-absent defaults', async () => {
  const page = boot({ served: SERVED, answer: () => { throw { code: 'upstream_error', message: 'x' }; } });
  const user = await page.window.governance.use('user');
  assert.equal(await user.isOwner(), false);
  assert.equal(await user.name(), '');
  assert.equal(await user.email(), null);
  assert.deepEqual(await user.search('ana'), []);
  assert.equal((await user.me()).id, null);
});

