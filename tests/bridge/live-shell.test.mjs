// The viewer shell of a live page (`storage/files/js/workspace/live/`): the router
// reads only its own frame's messages, and the bridge turns the page's
// requests into broker calls, asking the viewer for a connector and for
// each write, and reshaping the broker's answers into the runtime's codes.
// The broker is a fake transport; the prompts are scripted answers.
import test from 'node:test';
import assert from 'node:assert/strict';
import { createBridge } from '../../storage/files/js/workspace/live/bridge.js';
import { createRouter, PROTOCOL } from '../../storage/files/js/workspace/live/router.js';
import { connectorHint, summaryOf } from '../../storage/files/js/workspace/live/labels.js';

const API = '/api/v1/artifacts/doc-1';

const permissions = (overrides = {}) => ({
  capabilities: { mcp: { servers: [{ server: 'odoo', tools: ['search', 'write_partner'] }] }, artifact: {}, user: { scopes: ['profile'] } },
  contract_version: '0.2.99',
  revision: 'rev-1',
  writer: true,
  viewer: { id: 'u-1', name: 'Ana Ruiz', email: 'ana@dynapps.test', owner: true },
  states: { 'mcp:odoo': 'prompt', mcp: 'prompt', artifact: 'granted', user: 'granted' },
  ...overrides,
});

// A broker that answers by route; `calls` records each request.
const broker = (routes) => {
  const calls = [];
  const transport = async (url, options) => {
    const body = options.body ? JSON.parse(options.body) : undefined;
    calls.push({ url, method: options.method, body });
    const route = routes[`${options.method} ${url.replace(API, '')}`];
    if (!route) throw new Error(`no route for ${options.method} ${url}`);
    return route(body, calls);
  };
  return { calls, transport };
};

const prompts = (answers = {}) => {
  const asked = [];
  const consented = [];
  return {
    asked,
    consented,
    consent: async (servers, call) => { asked.push(['consent', servers]); consented.push(call); return answers.consent ?? true; },
    confirm: async (call) => { asked.push(['confirm', call]); return answers.confirm ?? true; },
    panel: async () => { asked.push(['panel']); },
    busy: () => false,
  };
};

const ok = (body) => ({ status: 200, body });
const refused = (status, key, extra = {}) => ({ status, body: { error: key, error_key: key, ...extra } });

const bridgeWith = async (routes, answers, extra = {}) => {
  const fake = broker({ 'GET /permissions': () => ok(permissions(extra.permissions)), ...routes });
  const asked = prompts(answers);
  const published = [];
  const failures = [];
  const bridge = createBridge({
    urls: { api: API, people: '/drive/people', search: '/api/v1/documents/people', document: 'doc-1' },
    transport: fake.transport,
    prompts: asked,
    onPublished: (version) => published.push(version),
    onFailure: (why) => failures.push(why),
  });
  await bridge.load();
  return { bridge, calls: fake.calls, asked: asked.asked, consented: asked.consented, published, failures };
};

const granted = () => ok(permissions({ states: { 'mcp:odoo': 'granted', mcp: 'granted' } }));

test('the router reads only its own frame, and replies to that frame alone', async () => {
  const frame = { posted: [], postMessage(message, target) { this.posted.push({ message, target }); } };
  const stranger = { postMessage: () => assert.fail('a reply went to another window') };
  const handled = [];
  const route = createRouter({
    frame: () => frame,
    bridge: { served: () => ['permissions', 'mcp'], serverOf: () => null, handle: async (method) => { handled.push(method); return { ok: true }; } },
    onSize: () => {},
    onActivity: () => {},
  });
  const request = { governance: PROTOCOL, type: 'request', id: 'r1', method: 'mcp.listTools', params: {} };
  route({ source: stranger, data: request });
  route({ source: stranger, data: { jsonrpc: '2.0', id: 1, method: 'tools/call', params: { name: 'search' } } });
  await Promise.resolve();
  assert.deepEqual(handled, [], 'messages from another source are dropped unread');
  route({ source: frame, data: { governance: PROTOCOL, type: 'hello' } });
  route({ source: frame, data: request });
  await new Promise((resolve) => setImmediate(resolve));
  assert.deepEqual(frame.posted[0].message, { governance: PROTOCOL, type: 'welcome', served: ['permissions', 'mcp'], budgetMs: 130000 });
  assert.equal(frame.posted[0].target, '*', 'an opaque-origin frame is only reachable with "*"');
  assert.deepEqual(frame.posted[1].message, { governance: PROTOCOL, type: 'reply', id: 'r1', result: { ok: true } });
});

test('MCP Apps: ui/initialize is answered, tools/call runs on the one server declaring the tool', async () => {
  const frame = { posted: [], postMessage(message) { this.posted.push(message); } };
  const calls = [];
  const route = createRouter({
    frame: () => frame,
    bridge: {
      served: () => [],
      serverOf: (tool) => (tool === 'search' ? 'odoo' : null),
      handle: async (method, params) => { calls.push([method, params]); return { content: [{ type: 'text', text: 'hit' }] }; },
    },
    onSize: () => {},
    onActivity: () => {},
  });
  route({ source: frame, data: { jsonrpc: '2.0', id: 1, method: 'ui/initialize', params: {} } });
  route({ source: frame, data: { jsonrpc: '2.0', id: 2, method: 'tools/call', params: { name: 'search', arguments: { q: 'acme' } } } });
  route({ source: frame, data: { jsonrpc: '2.0', id: 3, method: 'tools/call', params: { name: 'unknown' } } });
  route({ source: frame, data: { jsonrpc: '2.0', id: 4, method: 'ui/open-link', params: {} } });
  await new Promise((resolve) => setImmediate(resolve));
  const byId = Object.fromEntries(frame.posted.map((message) => [message.id, message]));
  assert.equal(byId[1].result.protocolVersion, '2026-01-26');
  assert.deepEqual(calls, [['mcp.callTool', { server: 'odoo', tool: 'search', input: { q: 'acme' } }]]);
  assert.deepEqual(byId[2].result, { content: [{ type: 'text', text: 'hit' }] });
  assert.equal(byId[3].error.code, -32602);
  assert.equal(byId[4].error.code, -32601);
});

test('a first call asks the viewer for the connector once, records the grant, then runs', async () => {
  const run = await bridgeWith({
    'POST /permissions': (body) => { assert.deepEqual(body, { scope: 'mcp:odoo', decision: 'granted' }); return granted(); },
    'POST /mcp/call': () => ok({ content: [{ type: 'text', text: '[]' }] }),
  });
  await run.bridge.handle('mcp.callTool', { server: 'odoo', tool: 'search', input: { q: 'a' } });
  await run.bridge.handle('mcp.callTool', { server: 'odoo', tool: 'search', input: { q: 'b' } });
  assert.deepEqual(run.asked, [['consent', ['odoo']]], 'asked once, the grant then holds');
  assert.equal(run.calls.filter((call) => call.url.endsWith('/mcp/call')).length, 2);
});

test('a declined connector is not_in_manifest for the rest of the load, with no call and no second ask', async () => {
  const run = await bridgeWith({ 'POST /mcp/call': () => assert.fail('a declined connector was called') }, { consent: false });
  await assert.rejects(run.bridge.handle('mcp.callTool', { server: 'odoo', tool: 'search', input: {} }), { code: 'not_in_manifest', server: 'odoo' });
  await assert.rejects(run.bridge.handle('mcp.watch', { server: 'odoo', tool: 'search', input: {}, mode: 'peek' }), { code: 'not_in_manifest' });
  assert.equal(run.asked.length, 1);
  assert.equal(await run.bridge.handle('permissions.state', { name: 'mcp:odoo' }), 'denied');
});

test('a write is confirmed with its server, tool and input, approved, and called once with the approval', async () => {
  let callCount = 0;
  const run = await bridgeWith({
    'POST /mcp/call': (body) => {
      callCount += 1;
      if (!body.approval) return refused(428, 'approval_required');
      assert.equal(body.approval, 'appr-1');
      return ok({ content: [{ type: 'text', text: '{"id":9}' }] });
    },
    'POST /mcp/approve': (body) => { assert.deepEqual(body, { server: 'odoo', tool: 'write_partner', input: { name: 'Acme' } }); return ok({ approval: 'appr-1' }); },
  }, {}, { permissions: { states: { 'mcp:odoo': 'granted', mcp: 'granted' } } });
  const result = await run.bridge.handle('mcp.callTool', { server: 'odoo', tool: 'write_partner', input: { name: 'Acme' } });
  assert.deepEqual(result.content, [{ type: 'text', text: '{"id":9}' }]);
  assert.deepEqual(run.asked, [['confirm', { server: 'odoo', tool: 'write_partner', summary: [['name', 'Acme']] }]]);
  assert.equal(callCount, 2);
});

test('a first write asks once: the one Allow opens the connector and approves this call', async () => {
  const run = await bridgeWith({
    'POST /permissions': () => granted(),
    'POST /mcp/call': (body) => (body.approval === 'appr-7' ? ok({ content: [] }) : refused(428, 'approval_required')),
    'POST /mcp/approve': () => ok({ approval: 'appr-7' }),
  });
  await run.bridge.handle('mcp.callTool', { server: 'odoo', tool: 'write_partner', input: { name: 'Acme' } });
  assert.deepEqual(run.asked, [['consent', ['odoo']]], 'one prompt, no second confirmation');
  assert.deepEqual(run.consented, [{ server: 'odoo', tool: 'write_partner', summary: [['name', 'Acme']] }], 'the prompt names the call it approves');
  assert.equal(run.calls.filter((call) => call.url.endsWith('/mcp/approve')).length, 1);
  await run.bridge.handle('mcp.callTool', { server: 'odoo', tool: 'write_partner', input: { name: 'Beta' } });
  assert.equal(run.asked.at(-1)[0], 'confirm', 'a later write is confirmed on its own');
});

test('a declined write is approval_required and never approved; a watch never asks to approve one', async () => {
  const run = await bridgeWith({
    'POST /mcp/call': () => refused(428, 'approval_required'),
    'POST /mcp/approve': () => assert.fail('a declined write was approved'),
  }, { confirm: false }, { permissions: { states: { 'mcp:odoo': 'granted', mcp: 'granted' } } });
  await assert.rejects(run.bridge.handle('mcp.callTool', { server: 'odoo', tool: 'write_partner', input: {} }), { code: 'approval_required' });
  const asked = run.asked.length;
  await assert.rejects(run.bridge.handle('mcp.watch', { server: 'odoo', tool: 'write_partner', input: {}, mode: 'peek' }), { code: 'approval_required' });
  assert.equal(run.asked.length, asked, 'the watch asked nothing');
});

test('a watch peeks at the stored entry of any age, a refresh always runs the tool', async () => {
  const run = await bridgeWith({ 'POST /mcp/call': () => ok({ content: [] }) }, {}, { permissions: { states: { 'mcp:odoo': 'granted', mcp: 'granted' } } });
  await run.bridge.handle('mcp.watch', { server: 'odoo', tool: 'search', input: {}, mode: 'peek', cache: { gcTime: 60000 } });
  await run.bridge.handle('mcp.watch', { server: 'odoo', tool: 'search', input: {}, mode: 'refresh', cache: {} });
  const bodies = run.calls.filter((call) => call.url.endsWith('/mcp/call')).map((call) => call.body.cache);
  assert.deepEqual(bodies, [{ staleTime: 60000, gcTime: 60000 }, { gcTime: 300000, refresh: true }]);
});

test('broker refusals become the runtime codes; a lapsed session tells the shell', async () => {
  const answers = [refused(503, 'server_unavailable'), refused(404, 'not_found'), refused(401, 'reauth'), refused(409, 'server_not_connected'), refused(409, 'needs_reauth')];
  const run = await bridgeWith({ 'POST /mcp/call': () => answers.shift() }, {}, { permissions: { states: { 'mcp:odoo': 'granted', mcp: 'granted' } } });
  const call = () => run.bridge.handle('mcp.callTool', { server: 'odoo', tool: 'search', input: {} });
  await assert.rejects(call(), (error) => error.code === 'server_unavailable' && error.retryable === true && error.server === 'odoo');
  await assert.rejects(call(), { code: 'not_granted' });
  await assert.rejects(call(), { code: 'upstream_error' });
  assert.deepEqual(run.failures, ['reauth']);
  await assert.rejects(call(), { code: 'server_not_connected' });
  await assert.rejects(call(), (error) => error.code === 'needs_reauth' && error.server === 'odoo' && error.retryable === undefined);
});

test('the status line offers to connect a connector, or to reconnect one that must be signed in to again', () => {
  assert.deepEqual(connectorHint('needs_reauth', 'odoo'), { text: 'Your connection to "odoo" needs you to sign in again.', label: 'Reconnect' });
  assert.deepEqual(connectorHint('server_not_connected', 'odoo'), { text: 'Connect "odoo" in your connectors to see its data.', label: 'Connectors' });
  assert.equal(connectorHint('server_unavailable', 'odoo'), null, 'an outage is not a connection to fix');
});

test('publish needs the declaration, sends the loaded revision, and reloads on success or conflict', async () => {
  const undeclared = await bridgeWith({}, {}, { permissions: { capabilities: null } });
  await assert.rejects(undeclared.bridge.handle('artifact.publish', { html: '<!doctype html>' }), { code: 'not_granted' });
  const answers = [ok({ version: 'rev-2' }), refused(409, 'conflict', { live: 'rev-3' })];
  const run = await bridgeWith({ 'POST /artifact/publish': (body) => { assert.equal(body.base_revision, 'rev-1'); return answers.shift(); } });
  assert.deepEqual(await run.bridge.handle('artifact.publish', { html: '<!doctype html><p>1' }), { version: 'rev-2' });
  await assert.rejects(run.bridge.handle('artifact.publish', { html: '<!doctype html><p>2' }), { code: 'conflict', live: 'rev-3' });
  assert.deepEqual(run.published, ['rev-2', 'rev-3']);
});

test('user hands the page only what its declared scopes allow', async () => {
  const none = await bridgeWith({}, {}, { permissions: { capabilities: { artifact: {} } } });
  const bare = await none.bridge.handle('user.me', {});
  assert.equal(bare.id, null);
  assert.equal(bare.name, '');
  assert.equal(bare.email, null);
  assert.equal(bare.isOwner, true);
  assert.match(bare.avatarUrl, /^data:image\/svg\+xml,/);
  const profile = await bridgeWith({}, {});
  const named = await profile.bridge.handle('user.me', {});
  assert.equal(named.id, 'u-1');
  assert.equal(named.name, 'Ana Ruiz');
  assert.equal(named.email, null, 'email needs its own scope');
  assert.equal(await profile.bridge.handle('user.can', { capability: 'files.write' }), true);
  assert.equal(await profile.bridge.handle('user.can', { capability: 'people.add' }), false);
});

test('permissions: state with the declines of this load, one batched request, a limited panel', async () => {
  const run = await bridgeWith({ 'POST /permissions': () => granted() });
  assert.equal(await run.bridge.handle('permissions.state', { name: 'db' }), 'unavailable');
  assert.deepEqual(await run.bridge.handle('permissions.request', { names: ['mcp', 'db'] }), { mcp: 'granted', db: 'unavailable' });
  assert.deepEqual(run.asked, [['consent', ['odoo']]]);
  for (let index = 0; index < 5; index += 1) await run.bridge.handle('permissions.manage', {});
  await assert.rejects(run.bridge.handle('permissions.manage', {}), { code: 'unavailable' });
});

test('the write summary names each top-level input and clips long values', () => {
  const summary = summaryOf({ name: 'Acme', tags: ['a', 'b'], note: 'x'.repeat(200), nested: { a: 1 } });
  assert.deepEqual(summary.slice(0, 2), [['name', 'Acme'], ['tags', '2 items: ["a","b"]']]);
  assert.equal(summary[2][1].length, 80);
  assert.deepEqual(summaryOf(null), []);
});
