// The freshness indicator beside a live page's range control. The relative
// time ticks silently in an aria-hidden span; only a change of state — paused,
// failing, signed out — reaches the live region, so a screen reader hears
// about the page's health and never about every passing second.

const TICK_MS = 5000;
const STATES = ['fresh', 'loading', 'paused', 'error', 'auth'];

const describeAge = (ms) => {
  const seconds = Math.round(ms / 1000);
  if (seconds < 5) return 'updated just now';
  if (seconds < 60) return `updated ${seconds}s ago`;
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `updated ${minutes}m ago`;
  return `updated ${Math.floor(minutes / 60)}h ago`;
};

const describeRetry = (ms) => {
  const seconds = Math.round(ms / 1000);
  return seconds < 60 ? `${seconds}s` : `${Math.round(seconds / 60)} min`;
};

export const createLiveStatus = (root, { onToggle }) => {
  if (!root) return null;
  const text = root.querySelector('[data-live-text]');
  const announce = root.querySelector('[data-live-announce]');
  const toggle = root.querySelector('[data-live-toggle]');
  let state = 'fresh';
  let updatedAt = Date.now();
  let retryMs = 0;

  const label = () => {
    if (state === 'paused') return 'Paused';
    if (state === 'auth') return 'Signed out';
    if (state === 'error') return `Retrying in ${describeRetry(retryMs)}`;
    if (state === 'loading' && !updatedAt) return 'Loading';
    return `Live · ${describeAge(Date.now() - updatedAt)}`;
  };

  const paint = () => {
    for (const name of STATES) root.classList.toggle(`sp-live--${name}`, state === name);
    if (text) text.textContent = label();
  };

  const say = (message) => {
    if (announce) announce.textContent = message;
  };

  const setState = (next, detail = {}) => {
    const changed = next !== state;
    state = next;
    if (next === 'fresh') updatedAt = detail.at ?? Date.now();
    if (next === 'error') retryMs = detail.retryMs ?? 0;
    paint();
    if (!changed) return;
    if (next === 'paused') say('Live refresh paused');
    if (next === 'error') say(`Could not refresh; retrying in ${describeRetry(retryMs)}`);
    if (next === 'auth') say('Session expired; reload to sign in again');
    if (next === 'fresh' && toggle) toggle.textContent = 'Pause';
  };

  toggle?.addEventListener('click', () => {
    const pausing = toggle.getAttribute('aria-pressed') !== 'true';
    toggle.setAttribute('aria-pressed', pausing ? 'true' : 'false');
    toggle.textContent = pausing ? 'Resume' : 'Pause';
    onToggle(pausing);
  });

  window.setInterval(() => {
    if (state === 'fresh') paint();
  }, TICK_MS);

  root.hidden = false;
  paint();
  return { setState };
};
