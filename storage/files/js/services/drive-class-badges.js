// Class badges on the Drive listing's rows, through the listing's row
// extension. The extension is synchronous: an uncached row renders bare,
// its id is queued, and the listing re-renders once the labels arrive.
import { registerRowExtension } from './drive-listing-registry.js';
import { loadClassLabels } from './document-class.js';

const TONES = { confidential: 'sp-badge--warn', restricted: 'sp-badge--err' };

const badge = (entry) => {
  const tone = TONES[entry?.class_id];
  if (!tone) return null;
  const span = document.createElement('span');
  span.className = `sp-badge sp-badge--sm ${tone}`;
  span.textContent = entry.label;
  return span;
};

const fetchQueued = async (cache, queue) => {
  for (const [kind, ids] of Object.entries(queue)) {
    const wanted = [...ids];
    ids.clear();
    if (!wanted.length) continue;
    const found = await loadClassLabels(kind, wanted);
    for (const id of wanted) cache.set(`${kind}:${id}`, found.get(id) ?? null);
  }
};

export const initClassBadges = (listing) => {
  const cache = new Map();
  const queue = { document: new Set(), folder: new Set() };
  let scheduled = false;
  const flush = async () => {
    scheduled = false;
    await fetchQueued(cache, queue);
    listing.invalidate();
  };
  registerRowExtension(({ kind, item }) => {
    const key = `${kind}:${item.id}`;
    if (cache.has(key)) {
      const entry = cache.get(key);
      return { attrs: { 'data-class': entry?.class_id ?? '' }, content: badge(entry) };
    }
    queue[kind]?.add(item.id);
    if (!scheduled) {
      scheduled = true;
      queueMicrotask(flush);
    }
    return null;
  });
  listing.root.addEventListener('sp-document-class-changed', () => {
    cache.clear();
    listing.invalidate();
  });
};
