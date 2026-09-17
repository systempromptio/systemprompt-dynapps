import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import vm from 'node:vm';

async function harness() {
  const timers = new Map(), documentListeners = new Map(), windowListeners = new Map();
  const sources = [], requests = [], values = [], statuses = [];
  class Source {
    constructor(url) { this.url = url; this.listeners = new Map(); this.closed = false; sources.push(this); }
    close() { this.closed = true; }
    addEventListener(name, fn) { this.listeners.set(name, fn); }
    event(name, id) { this.listeners.get(name)?.({ type: name, lastEventId: id }); }
  }
  const document = { hidden: false, addEventListener: (name, fn) => documentListeners.set(name, fn), removeEventListener: name => documentListeners.delete(name) };
  const context = vm.createContext({
    EventSource: Source, AbortController, document,
    window: { addEventListener: (name, fn) => windowListeners.set(name, fn), removeEventListener: name => windowListeners.delete(name) },
    fetch: (url, options) => new Promise((resolve, reject) => requests.push({ url, options, resolve, reject })),
    setTimeout: (fn, delay) => { const timer = {}; timers.set(timer, { fn, delay }); return timer; },
    clearTimeout: timer => timers.delete(timer),
  });
  const source = await readFile(new URL('../../storage/files/js/services/portfolio-live.js', import.meta.url), 'utf8');
  const module = new vm.SourceTextModule(source, { context });
  await module.link(() => { throw new Error('unexpected dependency'); }); await module.evaluate();
  const dispose = module.namespace.watchPortfolio({ url: '/admin/analysis/skills.json?days=30', generation: '1.2.3', onData: data => values.push(data), onStatus: value => statuses.push(value) });
  const timer = delay => { const entry = [...timers].find(([,value]) => value.delay === delay); assert.ok(entry, `timer ${delay}`); timers.delete(entry[0]); entry[1].fn(); };
  return { timers, documentListeners, windowListeners, sources, requests, values, statuses, dispose, timer,
    visibility: hidden => { document.hidden = hidden; documentListeners.get('visibilitychange')?.(); } };
}
const flush = () => new Promise(resolve => setImmediate(resolve));
const reply = (request, data, status = 200) => request.resolve({ ok: status === 200, status, json: async () => data });

test('teardown closes stream, aborts work and removes every timer and listener', async () => {
  const h = await harness(); assert.match(h.sources[0].url, /after=1.2.3/);
  h.sources[0].event('resync', '3.2.3'); assert.equal(h.requests.length, 1);
  h.dispose(); assert.ok(h.sources[0].closed); assert.ok(h.requests[0].options.signal.aborted);
  assert.equal(h.timers.size, 0); assert.equal(h.documentListeners.size, 0); assert.equal(h.windowListeners.size, 0);
  reply(h.requests[0], { stale: true }); await flush(); assert.equal(h.values.length, 0);
});

test('resync supersedes stale replies and reconnects with the durable composite cursor', async () => {
  const h = await harness(); const old = h.sources[0];
  old.event('snapshot', '2.2.3'); old.event('resync', '8.4.5'); assert.ok(h.requests[0].options.signal.aborted);
  reply(h.requests[1], { id: 'new', pending_jobs: 0 }); await flush(); reply(h.requests[0], { id: 'old' }); await flush(); assert.deepEqual(h.values.map(value => value.id), ['new']);
  old.onerror(); assert.ok(old.closed); h.timer(1000); const current = h.sources[1]; assert.match(current.url, /after=8.4.5/);
  old.onerror(); old.event('resync', '99.99.99'); assert.equal(current.closed, false); assert.equal(h.requests.length, 2); h.dispose();
});

test('hidden pages release transport and resumed pages reject old work', async () => {
  const h = await harness(); h.sources[0].event('resync', '4.2.3'); h.visibility(true);
  assert.ok(h.sources[0].closed); assert.equal(h.timers.size, 0); assert.ok(h.requests[0].options.signal.aborted);
  h.visibility(false); assert.equal(h.sources.length, 2); assert.equal(h.requests.length, 2);
  reply(h.requests[0], {}, 403); await flush(); assert.equal(h.sources[1].closed, false);
  reply(h.requests[1], { pending_jobs: 2 }); await flush(); assert.ok([...h.timers.values()].some(value => value.delay === 2000)); h.dispose();
});

test('expired authorization stops retries and prevents later cached data updates', async () => {
  const h = await harness(); h.sources[0].event('resync', '4.2.3'); reply(h.requests[0], {}, 401); await flush();
  assert.ok(h.sources[0].closed); assert.equal(h.timers.size, 0); assert.equal(h.documentListeners.size, 0); assert.equal(h.values.length, 0); assert.match(h.statuses.at(-1), /Sign in/);
});


test('fallback polling completes custom ranges without an SSE notification', async () => {
  const h = await harness();
  h.timer(2000);
  assert.equal(h.requests.length, 1);
  assert.equal(h.requests[0].options.credentials, 'same-origin');
  assert.equal(h.requests[0].options.cache, 'no-store');
  reply(h.requests[0], { pending_jobs: 1 }); await flush();
  assert.match(h.statuses.at(-1), /Preparing/);
  h.timer(2000);
  reply(h.requests[1], { pending_jobs: 0, result: 'ready' }); await flush();
  assert.equal(h.values.at(-1).result, 'ready');
  assert.equal([...h.timers.values()].filter(value => value.delay === 2000).length, 0);
  h.timer(30000); assert.equal(h.requests.length, 3);
  h.dispose();
});

test('failed refreshes retain results and recover from HTTP, transport and malformed JSON failures', async () => {
  const h = await harness();
  h.timer(2000); reply(h.requests[0], { pending_jobs: 0, result: 'retained' }); await flush();
  h.sources[0].event('snapshot', '2.2.3'); reply(h.requests[1], {}, 503); await flush();
  assert.deepEqual(h.values.map(value => value.result), ['retained']);
  assert.match(h.statuses.at(-1), /Retained results remain visible/);
  h.timer(5000); h.requests[2].reject(new Error('offline')); await flush();
  h.timer(5000); h.requests[3].resolve({ ok: true, status: 200, json: async () => { throw new SyntaxError('invalid JSON'); } }); await flush();
  h.timer(5000); reply(h.requests[4], { pending_jobs: 0, result: 'recovered' }); await flush();
  assert.deepEqual(h.values.map(value => value.result), ['retained', 'recovered']);
  assert.equal([...h.timers.values()].filter(value => value.delay === 5000).length, 0);
  h.dispose();
});

test('reconnect backoff is capped and a successful open resets it', async () => {
  const h = await harness();
  for (const delay of [1000, 2000, 4000, 8000, 16000, 30000, 30000]) {
    const active = h.sources.at(-1);
    active.onerror(); assert.ok(active.closed);
    // The independent fallback poll can also have a two-second delay.
    const timers = [...h.timers].filter(([, value]) => value.delay === delay);
    const [id, timer] = timers.at(-1); assert.ok(timer, `reconnect ${delay}`);
    h.timers.delete(id); timer.fn();
    assert.notEqual(h.sources.at(-1), active);
  }
  h.sources.at(-1).onopen(); h.sources.at(-1).onerror();
  h.timer(1000); assert.equal(h.sources.length, 9);
  h.dispose(); assert.equal(h.timers.size, 0);
});

test('invalid and duplicate wakeups do not fetch while explicit resync refreshes unchanged generations', async () => {
  const h = await harness();
  for (const id of ['', '1.2', '-1.2.3', '1.2.3.extra', 'x.2.3', '9'.repeat(97) + '.2.3']) h.sources[0].event('snapshot', id);
  h.sources[0].event('snapshot', '1.2.3');
  assert.equal(h.requests.length, 0);
  h.sources[0].event('resync', '1.2.3'); assert.equal(h.requests.length, 1);
  h.sources[0].onerror(); h.timer(1000);
  assert.match(h.sources[1].url, /after=1.2.3$/);
  h.windowListeners.get('pagehide')();
  assert.ok(h.sources[1].closed); assert.ok(h.requests[0].options.signal.aborted);
  assert.equal(h.timers.size, 0);
});
