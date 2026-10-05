import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import vm from 'node:vm';

test('sidebar groups expand and collapse through the shared event dispatcher', async () => {
  const listeners = new Map();
  const classes = new Set();
  const attributes = new Map();
  const storage = new Map();
  let group;
  const button = {
    setAttribute: (name, value) => attributes.set(name, value),
    closest: (selector) => selector === '[data-nav-group]' ? group
      : selector === '[data-action="nav-group-toggle"]' ? button : null,
  };
  group = {
    dataset: { navGroup: 'people' },
    classList: {
      contains: name => classes.has(name),
      toggle: (name, on) => on ? classes.add(name) : classes.delete(name),
    },
    querySelector: selector => selector === '[data-action="nav-group-toggle"]' ? button : null,
  };
  const nav = { dataset: {}, querySelectorAll: () => [group] };
  const context = vm.createContext({
    document: {
      querySelector: () => nav,
      addEventListener: (name, callback) => {
        const callbacks = listeners.get(name) || [];
        callbacks.push(callback);
        listeners.set(name, callbacks);
      },
    },
    localStorage: { getItem: key => storage.get(key), setItem: (key, value) => storage.set(key, value) },
    requestAnimationFrame: callback => callback(),
  });
  const events = new vm.SourceTextModule(await readFile(new URL('../../storage/files/js/services/events.js', import.meta.url), 'utf8'), { context });
  const groups = new vm.SourceTextModule(await readFile(new URL('../../storage/files/js/services/nav-groups.js', import.meta.url), 'utf8'), { context });
  await groups.link(() => events);
  await groups.evaluate();
  events.namespace.initDelegation();
  groups.namespace.initNavGroups();
  assert.equal(attributes.get('aria-expanded'), 'false');
  for (const callback of listeners.get('click')) callback({ target: button, preventDefault() {} });
  assert.equal(attributes.get('aria-expanded'), 'true');
  assert.ok(classes.has('is-open'));
  assert.equal(storage.get('sp-nav:open-groups'), '["people"]');
  for (const callback of listeners.get('click')) callback({ target: button, preventDefault() {} });
  assert.equal(attributes.get('aria-expanded'), 'false');
  assert.equal(storage.get('sp-nav:open-groups'), '[]');
});
