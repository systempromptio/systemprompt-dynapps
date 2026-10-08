// The sidebar's to-do badges — calls held for approval, access-control drift
// — are counts that change while a page is open. Every minute this asks
// /admin/governance/attention through the shared poller (paused while the
// tab is hidden, backing off on failure) and updates every `[data-attention]`
// badge in place; a badge is hidden at zero, and nothing runs when the
// sidebar carries no badges (a participant).

import { rawResponse } from './api.js';
import { createPoller } from './live-poll.js';

const URL = '/admin/governance/attention';
const EVERY_MS = 60000;

const paint = (badge, count) => {
  const value = Number(count) || 0;
  const counter = badge.querySelector('[data-attention-count]');
  if (counter) counter.textContent = String(value);
  badge.hidden = value === 0;
  badge.title = `${value} ${badge.dataset.attentionTitle || ''}`.trim();
};

const refresh = async (badges, signal) => {
  const resp = await rawResponse(URL, { signal });
  if (resp.status === 401 || resp.status === 403) {
    throw Object.assign(new Error('signed out'), { kind: 'auth' });
  }
  if (!resp.ok) throw new Error(`attention counts: status ${resp.status}`);
  const counts = await resp.json();
  for (const badge of badges) {
    paint(badge, counts[badge.dataset.attention]);
  }
};

export const initAttention = () => {
  const badges = [...document.querySelectorAll('[data-attention]')];
  if (badges.length === 0) return;
  createPoller({ intervalMs: EVERY_MS, run: (signal) => refresh(badges, signal) }).refreshNow();
};
