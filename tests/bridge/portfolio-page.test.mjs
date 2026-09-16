import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import vm from 'node:vm';

async function page(present = true) {
  const nodes = new Map();
  const root = { isConnected: true, dataset: { generation: '4.5.6' }, querySelector: selector => { if (!nodes.has(selector)) nodes.set(selector, { textContent: '' }); return nodes.get(selector); } };
  const listeners = new Map(), observers = [], watches = [];
  let disposed = 0;
  class Observer {
    constructor(callback) { this.callback = callback; this.disconnected = false; observers.push(this); }
    observe(target, options) { this.target = target; this.options = options; }
    disconnect() { this.disconnected = true; }
  }
  const body = {};
  const context = vm.createContext({
    document: { body, querySelector: () => present ? root : null },
    location: { search: '?start=2026-01-01&end=2026-02-01' },
    MutationObserver: Observer,
    window: { addEventListener: (name, fn) => listeners.set(name, fn), removeEventListener: name => listeners.delete(name) },
  });
  const dependency = new vm.SyntheticModule(['watchPortfolio'], function () {
    this.setExport('watchPortfolio', options => { watches.push(options); return () => { disposed++; }; });
  }, { context });
  const module = new vm.SourceTextModule(await readFile(new URL('../../storage/files/js/pages/analysis-portfolio.js', import.meta.url), 'utf8'), { context });
  await module.link(specifier => {
    assert.equal(specifier, '../services/portfolio-live.js');
    return dependency;
  });
  await module.evaluate();
  return { nodes, root, body, listeners, observers, watches, disposed: () => disposed };
}

test('removing the portfolio DOM disposes its live transport and observer', async () => {
  const h = await page();
  assert.equal(h.watches.length, 1);
  assert.equal(h.watches[0].generation, '4.5.6');
  assert.equal(h.watches[0].url, '/admin/analysis/resources.json?start=2026-01-01&end=2026-02-01');
  assert.equal(h.observers[0].target, h.body);
  h.observers[0].callback(); assert.equal(h.disposed(), 0);
  h.root.isConnected = false; h.observers[0].callback();
  assert.equal(h.disposed(), 1);
  assert.ok(h.observers[0].disconnected);
  assert.equal(h.listeners.size, 0);
});

test('pagehide releases page observer and pages without a portfolio start no transport', async () => {
  const h = await page(); h.listeners.get('pagehide')();
  assert.equal(h.disposed(), 1); assert.ok(h.observers[0].disconnected);
  assert.equal(h.listeners.size, 0);
  const absent = await page(false);
  assert.equal(absent.watches.length, 0); assert.equal(absent.observers.length, 0); assert.equal(absent.listeners.size, 0);
});


test('retained generations update freshness without presenting raw live metrics or claiming unknown attribution is zero', async () => {
  const h = await page();
  const data = {
    health: { generation: 4, fact_generation: 3, facts_generation: 5, pending_changes: 2, pending_producer_changes: 1 },
    inventory: { generation: 5, entries: 53 }, installations: { generation: 6 },
    generated_at: '2026-09-15T10:00:00Z', pending_jobs: 0, unknown_usage: null,
    synced_at: '2026-09-15T10:00:00Z', snapshot_state: 'current',
  };
  h.watches[0].onData(data);
  assert.match(h.nodes.get('[data-live-status]').textContent, /Inventory generation 5 · synced .* · snapshot current/);
  assert.doesNotMatch(h.nodes.get('[data-live-status]').textContent, /observed events/);
  assert.match(h.nodes.get('[data-freshness]').textContent, /pending facts 2.*producer changes 1/);
  assert.match(h.nodes.get('[data-unknown-usage]').textContent, /unavailable/);
  h.watches[0].onData({ ...data, health: { ...data.health, generation: 8 } });
  assert.match(h.nodes.get('[data-live-status]').textContent, /reload to refresh/);
  h.watches[0].onData({ ...data, pending_jobs: 1 });
  assert.match(h.nodes.get('[data-live-status]').textContent, /Preparing the requested range/);
});
