import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import vm from 'node:vm';

async function harness() {
  const timers = new Set();
  const listeners = new Set();
  const pending = [];
  let receive;
  let syncs = 0;
  const context = vm.createContext({
    document: { hidden: false }, URL, Date,
    window: { addEventListener: (_, fn) => listeners.add(fn), removeEventListener: (_, fn) => listeners.delete(fn) },
    setInterval: fn => { timers.add(fn); return fn; },
    clearInterval: fn => timers.delete(fn),
  });
  const dependencies = {
    '/assets/js/bridge.js': { bridge: { profileFetch: () => new Promise(resolve => pending.push(resolve)), sync: async () => { ++syncs; } } },
    '/assets/js/utils/escape.js': { escapeHtml: String },
    '/assets/js/services/state-store.js': { subscribeSnapshot: fn => { receive = fn; return () => { receive = null; }; } },
  };
  const source = await readFile(new URL('../../bridge/web/js/components/profile-connections.js', import.meta.url), 'utf8');
  const module = new vm.SourceTextModule(source, { context });
  await module.link(specifier => {
    const exports = dependencies[specifier];
    assert.ok(exports, `unexpected import: ${specifier}`);
    return new vm.SyntheticModule(Object.keys(exports), function() {
      for (const [key, value] of Object.entries(exports)) { this.setExport(key, value); }
    }, { context });
  });
  await module.evaluate();
  return {
    api: module.namespace, timers, listeners, pending,
    get subscribed() { return !!receive; }, get syncs() { return syncs; },
    snapshot: gateway => receive({ gateway_url: gateway, verified_identity: { user_id: 'user' } }),
  };
}
const flush = () => new Promise(resolve => setImmediate(resolve));
const profile = status => ({ identity: { extra: { connections: { version: 1, user_id: 'user', revision: 1,
  connections: [{ provider: 'provider', status, configured: true, entitled: true, verified_at: '2026-09-08', account_name: status }],
} } } });

test('import is idle; last unmount tears down every owned resource', async () => {
  const h = await harness();
  assert.equal(h.timers.size, 0);
  assert.equal(h.listeners.size, 0);
  assert.equal(h.subscribed, false);
  const first = h.api.watchConnections(() => {});
  const second = h.api.watchConnections(() => {});
  assert.equal(h.timers.size, 1);
  first();
  assert.equal(h.timers.size, 1);
  second();
  assert.equal(h.timers.size, 0);
  assert.equal(h.listeners.size, 0);
  assert.equal(h.subscribed, false);
});

test('replies from an old gateway or unmounted pane cannot mutate or trigger sync', async () => {
  const h = await harness();
  const dispose = h.api.watchConnections(() => {});
  h.snapshot('https://old.example');
  const oldReply = h.pending.shift();
  h.snapshot('https://new.example');
  const newReply = h.pending.shift();
  oldReply(profile('old-account'));
  await flush();
  assert.ok(!h.api.renderConnections(null).includes('old-account'));
  newReply(profile('connected'));
  await flush();
  for (const tick of h.timers) { tick(); }
  const detachedReply = h.pending.shift();
  dispose();
  detachedReply(profile('reconnect_required'));
  await flush();
  assert.equal(h.syncs, 0);
  assert.ok(!h.api.renderConnections(null).includes('reconnect_required'));
});

test('renders one compact row per provider with a display name, a status dot and a count pill', async () => {
  const h = await harness();
  const unsubscribe = h.api.watchConnections(() => {});
  h.snapshot('https://gw');
  await flush();
  h.pending.pop()({ identity: { extra: { connections: { version: 1, user_id: 'user', revision: 1, connections: [
    { provider: 'atlassian', status: 'connected', configured: true, entitled: true, verified_at: '2026-09-08T10:34:00Z', resource_name: 'https://astounddigital.atlassian.net' },
    { provider: 'github', status: 'not_connected', configured: true, entitled: true },
    { provider: 'salesforce', status: 'verification_required', configured: true, entitled: false },
  ] } } } });
  await flush();
  const html = h.api.renderConnections(null);
  assert.ok(html.includes('<header><h2 data-l10n-id="profile-section-connections">'));
  assert.ok(html.includes('1 / 3 connected'));
  assert.equal((html.match(/<li class="sp-profile-connection"/g) || []).length, 3);
  assert.ok(html.includes('sp-dot--ok') && html.includes('sp-dot--warn') && html.includes('sp-dot--unknown'));
  assert.ok(html.includes('>Atlassian<') && html.includes('>GitHub<') && !html.includes('>atlassian<'));
  assert.ok(html.includes('astounddigital.atlassian.net') && !html.includes('https://astounddigital'.replace('https://', '>https://')));
  assert.ok(html.includes('Verification required'));
  assert.ok(html.includes('>Manage<') && html.includes('>Connect<'));
  assert.equal((html.match(/data-action="manage-connection"/g) || []).length, 2);
  unsubscribe();
});

test('configured unfamiliar providers and servers without personal auth render safely', async () => {
  const h = await harness();
  const unsubscribe = h.api.watchConnections(() => {});
  h.snapshot('https://gw');
  await flush();
  h.pending.pop()({ identity: { extra: { connections: { version: 1, user_id: 'user', revision: 1, connections: [
    { provider: 'fourth-mcp', display_name: 'Fourth MCP', status: 'not_connected', configured: true, entitled: true, requires_auth: true },
    { provider: 'internal-tools', status: 'no_auth_required', configured: true, entitled: true, requires_auth: false },
    { provider: 'removed-tools', status: 'not_configured', configured: false, entitled: true, requires_auth: true },
  ] } } } });
  await flush();
  const html = h.api.renderConnections(null);
  assert.ok(html.includes('Fourth MCP') && html.includes('No authorization required'));
  assert.ok(html.includes('0 / 1 connected'));
  assert.equal((html.match(/data-action="manage-connection"/g) || []).length, 1);
  unsubscribe();
});
