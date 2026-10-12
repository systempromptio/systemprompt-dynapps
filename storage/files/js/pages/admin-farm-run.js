// The run page follows a live run. Every poll asks for the events after the
// last seq on screen, appends them to the rail, morphs the KPI strip, swaps
// the timeline and the header badge, and stops once the run is terminal.
// The duration tile ticks between polls so a running clock reads as one.

import '../components/sp-tabs.js';
import { on, initDelegation } from '../services/events.js';
import { createPoller } from '../services/live-poll.js';
import { morphKpis, swapRegion } from './admin-overview-morph.js';
import { fetchRunLive } from './admin-farm-run-fetch.js';

const POLL_MS = 2500;
const TICK_MS = 1000;

const region = (scope, name) => scope.querySelector(`[data-live-region="${name}"]`);

const formatElapsed = (ms) => {
  if (ms < 1000) return `${ms} ms`;
  if (ms < 60000) return `${(ms / 1000).toFixed(2)} s`;
  if (ms < 3600000) return `${(ms / 60000).toFixed(1)} min`;
  return `${(ms / 3600000).toFixed(1)} h`;
};

const setFolds = (root, open) => {
  for (const fold of root.querySelectorAll('.sp-run-event__fold')) fold.open = open;
};

const appendEvents = (root, next) => {
  const rail = region(root, 'events');
  const incoming = [...(region(next, 'events')?.children ?? [])];
  if (!rail || incoming.length === 0) return;
  const known = new Set([...rail.children].map((li) => li.dataset.seq));
  const fresh = incoming
    .filter((li) => !known.has(li.dataset.seq))
    .map((li) => {
      const node = document.importNode(li, true);
      node.classList.add('sp-run-event--new');
      return node;
    });
  if (fresh.length === 0) return;
  rail.append(...fresh);
  root.querySelector('[data-farm-run-empty]')?.remove();
  const follow = root.querySelector('[data-farm-run-follow]');
  if (follow?.checked) fresh.at(-1).scrollIntoView({ block: 'nearest', behavior: 'smooth' });
};

const applyLive = (root, next) => {
  appendEvents(root, next);
  const summary = region(root, 'summary');
  const nextSummary = region(next, 'summary');
  if (summary && nextSummary) morphKpis(summary, nextSummary);
  const timeline = region(root, 'timeline');
  const nextTimeline = region(next, 'timeline');
  if (timeline && nextTimeline) swapRegion(timeline, nextTimeline);
  const status = region(root, 'status');
  const nextStatus = region(next, 'status');
  if (status && nextStatus) status.replaceChildren(...nextStatus.cloneNode(true).childNodes);
  root.dataset.runStatus = next.dataset.runStatus ?? root.dataset.runStatus;
  root.dataset.lastSeq = next.dataset.lastSeq ?? root.dataset.lastSeq;
  root.dataset.liveActive = next.dataset.liveActive ?? 'false';
  if (next.dataset.startedMs) root.dataset.startedMs = next.dataset.startedMs;
};

const startClock = (root) => {
  const started = Number(root.dataset.startedMs);
  if (!Number.isFinite(started) || started <= 0) return null;
  return window.setInterval(() => {
    const value = root.querySelector('[data-kpi="duration"] .sp-kpi__value');
    if (value) value.textContent = formatElapsed(Math.max(0, Date.now() - started));
  }, TICK_MS);
};

const settle = (root, poller, clock) => {
  poller.stop();
  window.clearInterval(clock);
  const cancel = root.querySelector('[data-farm-run-cancel]');
  if (cancel) cancel.disabled = true;
  const follow = root.querySelector('[data-farm-run-follow]');
  if (follow) follow.checked = false;
};

export const initFarmRun = () => {
  const root = document.querySelector('[data-farm-run]');
  if (!root) return null;
  initDelegation();
  on('click', '[data-action="farm-run-expand"]', () => setFolds(root, true));
  on('click', '[data-action="farm-run-collapse"]', () => setFolds(root, false));
  if (root.dataset.liveActive !== 'true' || !root.dataset.pollUrl) return null;

  let clock = startClock(root);
  const poller = createPoller({
    intervalMs: POLL_MS,
    maxBackoffMs: POLL_MS * 8,
    run: async (signal) => {
      const next = await fetchRunLive(root.dataset.pollUrl, root.dataset.lastSeq ?? '0', signal);
      applyLive(root, next);
      clock = clock ?? startClock(root);
      if (root.dataset.liveActive !== 'true') settle(root, poller, clock);
    },
    onState: (state) => {
      if (state === 'auth') settle(root, poller, clock);
    },
  });
  poller.start();
  return poller;
};

initFarmRun();
