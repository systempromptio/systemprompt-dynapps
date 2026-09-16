import { on, onKey } from './events.js';

const STORAGE_KEY = 'sp-nav:open-groups';

const readOpen = () => {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    return new Set(raw ? JSON.parse(raw) : []);
  } catch {
    return new Set();
  }
};

const writeOpen = (open) => {
  try { localStorage.setItem(STORAGE_KEY, JSON.stringify([...open])); } catch { /* storage unavailable */ }
};

const setGroupOpen = (group, isOpen) => {
  group.classList.toggle('is-open', isOpen);
  group.querySelector('[data-action="nav-group-toggle"]')?.setAttribute('aria-expanded', String(isOpen));
};

// Why: the group holding the current page is always open and cannot be
// collapsed — a rail that hides where you are is a rail you have to search.
// The reader's choices for the other groups are honoured from storage.
const holdsCurrent = (group) => group.querySelector('a.is-active, a.is-ancestor') !== null;

const applyState = (nav, open) => {
  for (const group of nav.querySelectorAll('[data-nav-group]')) {
    setGroupOpen(group, holdsCurrent(group) || open.has(group.dataset.navGroup));
    const count = group.querySelector('[data-nav-group-count]');
    if (count) count.textContent = String(group.querySelectorAll('.sp-nav-group__inner > a').length);
  }
  nav.dataset.navReady = 'true';
};

const toggleGroup = (btn) => {
  const group = btn.closest('[data-nav-group]');
  if (!group || holdsCurrent(group)) return;
  const isOpen = !group.classList.contains('is-open');
  setGroupOpen(group, isOpen);
  const open = readOpen();
  if (isOpen) {
    open.add(group.dataset.navGroup);
  } else {
    open.delete(group.dataset.navGroup);
  }
  writeOpen(open);
};

const linkText = (link) => (link.textContent || '').replace(/\s+/g, ' ').trim().toLowerCase();

const applySearch = (nav, query) => {
  const q = query.trim().toLowerCase();
  nav.classList.toggle('is-searching', q.length > 0);
  let hits = 0;
  for (const link of nav.querySelectorAll('.sp-nav-group__inner > a')) {
    const hit = q.length > 0 && linkText(link).includes(q);
    link.classList.toggle('is-search-hit', hit);
    if (hit) hits += 1;
  }
  for (const group of nav.querySelectorAll('[data-nav-group]')) {
    const groupHit = q.length > 0 && group.querySelector('a.is-search-hit') !== null;
    group.classList.toggle('is-search-hit', groupHit);
    if (groupHit) setGroupOpen(group, true);
  }
  const empty = nav.querySelector('[data-nav-search-empty]');
  if (empty) empty.hidden = !(q.length > 0 && hits === 0);
  if (q.length === 0) applyState(nav, readOpen());
};

const isTypingTarget = (el) => el instanceof HTMLElement
  && (el.matches('input, textarea, select, [contenteditable]') || el.isContentEditable);

// Why: the first visible hit is where the reader most likely wants to go, so
// Enter in the box acts as a click on it instead of submitting nothing.
const openFirstHit = (nav) => {
  const first = nav.querySelector('a.is-search-hit');
  if (first) window.location.assign(first.href);
};

export const initNavGroups = () => {
  const nav = document.querySelector('#sp-admin-sidebar nav[data-nav]');
  if (!nav) return;
  applyState(nav, readOpen());
  // Why: groups restored from storage must land open, not animate open. A
  // single frame callback runs before that frame's style pass, so the open
  // state and the lifted gate would land together and still animate; the
  // second frame is the first one painted with the state already applied.
  requestAnimationFrame(() => {
    requestAnimationFrame(() => { nav.dataset.navMotion = 'true'; });
  });

  on('click', '[data-action="nav-group-toggle"]', (e, btn) => {
    e.preventDefault();
    toggleGroup(btn);
  });

  const search = nav.querySelector('[data-nav-search]');
  if (!(search instanceof HTMLInputElement)) return;
  on('input', '[data-nav-search]', (e, input) => applySearch(nav, input.value));
  on('keydown', '[data-nav-search]', (e, input) => {
    if (e.key === 'Enter') {
      e.preventDefault();
      openFirstHit(nav);
    } else if (e.key === 'Escape' && input.value) {
      input.value = '';
      applySearch(nav, '');
    }
  });
  onKey('/', (e) => {
    if (isTypingTarget(e.target) || e.ctrlKey || e.metaKey || e.altKey) return;
    e.preventDefault();
    search.focus();
    search.select();
  });
};
