// Live lists on the agent farm pages. A server-rendered list that still has
// a queued or running row carries `data-farm-live-list` naming its fragment
// URL; this module polls that URL, swaps each `data-live-region` the answer
// carries into the region of the same name, and stops for good once the
// fragment reports `data-live-active="false"`, because a list with no live
// row has nothing left that can change. Without scripts the page is complete.

import { rawResponse } from '../services/api.js';
import { createPoller } from '../services/live-poll.js';
import { createLiveStatus } from '../services/live-status.js';
import { morphKpis, swapRegion } from './admin-overview-morph.js';

const DEFAULT_INTERVAL_MS = 10000;

const failure = (kind, message) => Object.assign(new Error(message), { kind });

const fragmentUrl = (root) => {
  const base = root.dataset.farmLiveList;
  if (root.dataset.liveQuery !== '1' || !window.location.search) return base;
  const joiner = base.includes('?') ? '&' : '?';
  return `${base}${joiner}${window.location.search.slice(1)}`;
};

// A redirect or a 401/403 means the console session is gone; reporting it as
// its own kind makes the poller stop instead of retrying into a login page.
const fetchFragment = async (url, signal) => {
  let resp;
  try {
    resp = await rawResponse(url, { signal, headers: { Accept: 'text/html' } });
  } catch (error) {
    if (error?.name === 'AbortError') throw error;
    throw failure('offline', error?.message || 'Network unavailable');
  }
  if (resp.redirected || resp.status === 401 || resp.status === 403) {
    throw failure('auth', `Session no longer valid (${resp.status})`);
  }
  if (!resp.ok) throw failure('http', `Refresh failed with status ${resp.status}`);
  const html = await resp.text();
  return new DOMParser().parseFromString(html, 'text/html');
};

const applyRegions = (root, doc) => {
  for (const next of doc.querySelectorAll('[data-live-region]')) {
    const name = next.dataset.liveRegion;
    const current = root.querySelector(`[data-live-region="${name}"]`);
    if (!current) continue;
    if (name === 'kpis') morphKpis(current, next);
    else swapRegion(current, next);
  }
};

const isActive = (doc) => doc.querySelector('[data-live-active]')?.dataset.liveActive !== 'false';

const startList = (root) => {
  const url = fragmentUrl(root);
  const parsed = Number.parseInt(root.dataset.liveInterval ?? '', 10);
  const intervalMs = Number.isFinite(parsed) && parsed > 0 ? parsed : DEFAULT_INTERVAL_MS;
  let status = null;
  let poller = null;

  const settle = () => {
    poller?.stop();
    root.removeAttribute('data-farm-live-list');
    const indicator = root.querySelector('[data-live-status]');
    if (indicator) indicator.hidden = true;
  };

  const refresh = async (signal) => {
    const doc = await fetchFragment(url, signal);
    applyRegions(root, doc);
    for (const marker of root.querySelectorAll('[data-live-active]')) {
      marker.dataset.liveActive = isActive(doc) ? 'true' : 'false';
    }
    if (!isActive(doc)) window.setTimeout(settle, 0);
  };

  poller = createPoller({
    intervalMs,
    run: refresh,
    onState: (state, detail) => status?.setState(state, detail),
  });
  status = createLiveStatus(root.querySelector('[data-live-status]'), {
    onToggle: (pausing) => (pausing ? poller.pause() : poller.resume()),
  });
  poller.start();
};

export const initFarmLiveList = () => {
  for (const root of document.querySelectorAll('[data-farm-live-list]')) startList(root);
};

initFarmLiveList();
