// Applying a fresh overview body over the one on screen without losing the
// reader's place. Tiles are matched by key and edited in place, so a focused
// tile keeps focus and a number counts to its new value; boards are swapped
// wholesale, with focus put back on the same link and changed cells flashed.

import { tweenText } from '../services/live-number.js';

const VEIL = ['sp-skeleton', 'sp-skeleton--veil'];
const VEILED = '.sp-kpi__value, .sp-kpi__sub, .sp-delta, .sp-table__el tbody td';

const textOf = (root, selector) => root.querySelector(selector)?.textContent ?? '';

const syncAttr = (from, to, name) => {
  const value = from.getAttribute(name);
  if (value === null) to.removeAttribute(name);
  else if (to.getAttribute(name) !== value) to.setAttribute(name, value);
};

const syncTone = (from, to) => {
  for (const cls of [...to.classList]) if (cls.startsWith('sp-kpi--') && cls !== 'sp-kpi--link') to.classList.remove(cls);
  for (const cls of from.classList) if (cls.startsWith('sp-kpi--') && cls !== 'sp-kpi--link') to.classList.add(cls);
};

const replay = (el, cls) => {
  el.classList.remove(cls);
  void el.offsetWidth;
  el.classList.add(cls);
};

const syncDelta = (from, to) => {
  const next = from.querySelector('.sp-delta');
  const current = to.querySelector('.sp-delta');
  if (!next && current) {
    current.remove();
    return;
  }
  if (!next) return;
  if (!current) {
    to.querySelector('.sp-kpi__body')?.append(next.cloneNode(true));
    return;
  }
  if (current.textContent === next.textContent && current.className === next.className) return;
  current.className = next.className;
  current.textContent = next.textContent;
  replay(current, 'is-flipping');
};

const syncTrend = (from, to) => {
  const next = from.querySelector('.sp-kpi__trend');
  const current = to.querySelector('.sp-kpi__trend');
  if (!next || !current || next.innerHTML === current.innerHTML) return;
  current.replaceChildren(...next.cloneNode(true).childNodes);
  replay(current, 'is-updating');
};

const syncTile = (from, to) => {
  syncAttr(from, to, 'href');
  syncAttr(from, to, 'title');
  syncTone(from, to);
  const value = to.querySelector('.sp-kpi__value');
  if (value) tweenText(value, textOf(from, '.sp-kpi__value'));
  const note = to.querySelector('.sp-kpi__sub');
  const nextNote = textOf(from, '.sp-kpi__sub');
  if (note && note.textContent !== nextNote) note.textContent = nextNote;
  syncDelta(from, to);
  syncTrend(from, to);
};

// Tiles the fresh body has and the page does not (a notice replaced them, or
// vice versa) fall back to swapping the region, so the strip never shows a
// tile whose number belongs to a window that has gone.
export const morphKpis = (current, next) => {
  const nextTiles = [...next.querySelectorAll('[data-kpi]')];
  const currentTiles = new Map([...current.querySelectorAll('[data-kpi]')].map((t) => [t.dataset.kpi, t]));
  const sameShape =
    nextTiles.length === currentTiles.size &&
    nextTiles.every((t) => currentTiles.has(t.dataset.kpi)) &&
    textOf(current, '.sp-p-overview__note') === textOf(next, '.sp-p-overview__note') &&
    Boolean(current.querySelector('.sp-notice')) === Boolean(next.querySelector('.sp-notice'));
  if (!sameShape) {
    swapRegion(current, next);
    return;
  }
  for (const tile of nextTiles) syncTile(tile, currentTiles.get(tile.dataset.kpi));
};

const focusKey = (el) => el?.getAttribute('href') ?? el?.dataset.focusKey ?? null;

const flashChanges = (current, next) => {
  const before = current.querySelectorAll('tbody td');
  const after = next.querySelectorAll('tbody td');
  if (before.length !== after.length) return;
  after.forEach((cell, i) => {
    if (cell.textContent !== before[i].textContent) cell.classList.add('sp-u-changed');
  });
};

export const swapRegion = (current, next) => {
  const active = document.activeElement;
  const key = current.contains(active) ? focusKey(active) : null;
  flashChanges(current, next);
  current.replaceChildren(...next.cloneNode(true).childNodes);
  if (!key) return;
  const target = [...current.querySelectorAll('a[href], [data-focus-key]')].find((el) => focusKey(el) === key);
  target?.focus({ preventScroll: true });
};

export const veil = (root, on) => {
  for (const el of root.querySelectorAll(VEILED)) {
    if (on) el.classList.add(...VEIL);
    else el.classList.remove(...VEIL);
  }
};
