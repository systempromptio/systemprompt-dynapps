import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import vm from 'node:vm';

function fakeStrip(names, stored) {
  const store = new Map(stored ? [['sp-tabs:strip', stored]] : []);
  const panels = new Map();
  const tabs = names.map((name) => {
    const handlers = {};
    const classes = new Set();
    const attrs = new Map([['aria-controls', `panel-${name}`], ['aria-selected', name === names[0] ? 'true' : 'false']]);
    const panel = { hidden: name !== names[0] };
    panels.set(`panel-${name}`, panel);
    return {
      dataset: { tab: name }, tabIndex: 0, focused: false,
      classList: { toggle: (c, on) => (on ? classes.add(c) : classes.delete(c)), has: (c) => classes.has(c) },
      getAttribute: (k) => attrs.get(k) ?? null, setAttribute: (k, v) => attrs.set(k, v),
      addEventListener: (type, fn) => { handlers[type] = fn; },
      fire: (type, event = {}) => handlers[type]({ preventDefault() {}, ...event }),
      focus() { this.focused = true; }, panel,
    };
  });
  const root = {
    dataset: { tabs: 'strip' },
    querySelectorAll: () => tabs,
    ownerDocument: { getElementById: (id) => panels.get(id) },
  };
  return { root, tabs, store };
}

async function load(store) {
  const context = vm.createContext({
    localStorage: { getItem: (k) => store.get(k) ?? null, setItem: (k, v) => store.set(k, v) },
  });
  const source = await readFile(new URL('../../storage/files/js/components/sp-tabs.js', import.meta.url), 'utf8');
  const module = new vm.SourceTextModule(source, { context });
  await module.link(() => { throw new Error('sp-tabs imports nothing'); });
  await module.evaluate();
  return module.namespace;
}

test('click selects a tab, shows its panel and remembers the choice', async () => {
  const { root, tabs, store } = fakeStrip(['a', 'b', 'c']);
  const { initTabs } = await load(store);
  initTabs(root);
  assert.equal(tabs[0].panel.hidden, false);
  tabs[1].fire('click');
  assert.equal(tabs[1].getAttribute('aria-selected'), 'true');
  assert.equal(tabs[0].getAttribute('aria-selected'), 'false');
  assert.equal(tabs[1].panel.hidden, false);
  assert.equal(tabs[0].panel.hidden, true);
  assert.equal(tabs[1].tabIndex, 0);
  assert.equal(tabs[0].tabIndex, -1);
  assert.ok(tabs[1].classList.has('sp-tab--active'));
  assert.equal(store.get('sp-tabs:strip'), 'b');
});

test('arrow keys wrap and move focus; Home and End jump', async () => {
  const { root, tabs, store } = fakeStrip(['a', 'b', 'c']);
  const { initTabs } = await load(store);
  initTabs(root);
  tabs[0].fire('keydown', { key: 'ArrowLeft' });
  assert.equal(tabs[2].getAttribute('aria-selected'), 'true');
  assert.ok(tabs[2].focused);
  tabs[2].fire('keydown', { key: 'ArrowRight' });
  assert.equal(tabs[0].getAttribute('aria-selected'), 'true');
  tabs[0].fire('keydown', { key: 'End' });
  assert.equal(tabs[2].getAttribute('aria-selected'), 'true');
  tabs[2].fire('keydown', { key: 'Home' });
  assert.equal(tabs[0].getAttribute('aria-selected'), 'true');
});

test('a remembered tab is restored on init, and an unknown one falls back to the first', async () => {
  const remembered = fakeStrip(['a', 'b', 'c'], 'c');
  const { initTabs } = await load(remembered.store);
  initTabs(remembered.root);
  assert.equal(remembered.tabs[2].getAttribute('aria-selected'), 'true');
  assert.equal(remembered.tabs[0].panel.hidden, true);

  const stale = fakeStrip(['a', 'b'], 'gone');
  initTabs(stale.root);
  assert.equal(stale.tabs[0].getAttribute('aria-selected'), 'true');
});
