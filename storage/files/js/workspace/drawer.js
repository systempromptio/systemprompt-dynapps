// The details drawer. Its tabs and panes are built from the registered drawer
// tabs the viewer may see. A pane is filled by its tab's `load` when first
// shown (at once for an `eager` tab, so a feature that is off hides its tab
// before anyone opens it), again when one of the tab's live events arrives,
// and is removed for good when its route says the feature is off.
import { drawerTab, drawerTabs, registerLiveHandler } from './registry.js';

const state = { ctx: null, loaded: new Set() };

const drawerOf = (root) => root.querySelector('[data-slot="ws-drawer"]');

const paneOf = (root, id) => drawerOf(root).querySelector(`[data-pane="${CSS.escape(id)}"]`);

const isShown = (root, id) => {
  const drawer = drawerOf(root);
  return !drawer.hidden && paneOf(root, id)?.hidden === false;
};

const load = (root, id) => {
  const pane = paneOf(root, id);
  const tab = drawerTab(id);
  if (!pane || !tab?.load || !state.ctx) return;
  state.loaded.add(id);
  Promise.resolve()
    .then(() => tab.load(pane, state.ctx))
    .catch(() => {
      const save = root.querySelector('[data-field="save"]');
      if (save) save.textContent = 'Could not load';
    });
};

export const reloadTab = (root, id) => {
  if (isShown(root, id) || state.loaded.has(id)) load(root, id);
};

export const showTab = (root, id) => {
  const drawer = drawerOf(root);
  if (!paneOf(root, id)) return;
  drawer.querySelectorAll('[role="tab"]').forEach((b) => b.setAttribute('aria-selected', String(b.dataset.tab === id)));
  drawer.querySelectorAll('[data-pane]').forEach((p) => { p.hidden = p.dataset.pane !== id; });
  load(root, id);
};

export const toggleDrawer = (root, tab, force) => {
  const drawer = drawerOf(root);
  const open = force ?? drawer.hidden;
  drawer.hidden = !open;
  root.querySelector('[data-command="details"]')?.setAttribute('aria-pressed', String(open));
  if (open) showTab(root, tab ?? 'details');
};

const hideTab = (root, id) => {
  const drawer = drawerOf(root);
  drawer.querySelector(`[role="tab"][data-tab="${CSS.escape(id)}"]`)?.remove();
  paneOf(root, id)?.remove();
  root.querySelectorAll(`[data-command="${CSS.escape(id)}"]`).forEach((b) => { b.hidden = true; });
};

const tabButton = (tab) => {
  const template = document.querySelector('template[data-template="ws-tab"]');
  const button = template.content.firstElementChild.cloneNode(true);
  button.dataset.tab = tab.id;
  button.id = `ws-tab-${tab.id}`;
  button.setAttribute('aria-controls', `ws-pane-${tab.id}`);
  button.textContent = tab.label;
  return button;
};

const paneFor = (tab) => {
  const pane = document.createElement('section');
  pane.className = 'sp-ws__pane';
  pane.id = `ws-pane-${tab.id}`;
  pane.dataset.pane = tab.id;
  pane.hidden = true;
  pane.setAttribute('role', 'tabpanel');
  pane.setAttribute('aria-labelledby', `ws-tab-${tab.id}`);
  const template = document.querySelector(`template[data-template="ws-pane-${tab.id}"]`);
  if (template) pane.append(template.content.cloneNode(true));
  return pane;
};

export const initDrawer = (ctx) => {
  const { root } = ctx;
  state.ctx = ctx;
  const drawer = drawerOf(root);
  const tablist = drawer.querySelector('[data-slot="ws-tabs"]');
  const tabs = drawerTabs(ctx);
  tablist.replaceChildren(...tabs.map(tabButton));
  drawer.append(...tabs.map(paneFor));
  for (const tab of tabs) {
    for (const prefix of tab.live) registerLiveHandler(prefix, () => reloadTab(root, tab.id));
  }
  tabs.filter((tab) => tab.eager).forEach((tab) => load(root, tab.id));
  tablist.addEventListener('click', (event) => {
    const tab = event.target.closest('[data-tab]')?.dataset.tab;
    if (tab) showTab(root, tab);
  });
  drawer.addEventListener('ws:pane-unavailable', (event) => {
    const id = event.target.closest('[data-pane]')?.dataset.pane;
    if (id) hideTab(root, id);
  });
};
