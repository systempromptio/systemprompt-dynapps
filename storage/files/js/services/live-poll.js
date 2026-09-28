// A polling loop for a page that refreshes itself: one run at a time, paused
// while the tab is hidden and caught up the moment it is shown again, backing
// off when a run fails and settling back to the base interval on success.
// The sequence guard drops a slow response that a newer run has overtaken.

const BASE_INTERVAL_MS = 30000;
const MAX_BACKOFF_MS = 300000;

export const createPoller = ({ intervalMs = BASE_INTERVAL_MS, maxBackoffMs = MAX_BACKOFF_MS, run, onState }) => {
  let timer = null;
  let delay = intervalMs;
  let paused = false;
  let stopped = false;
  let sequence = 0;
  let controller = null;
  let lastSuccess = 0;

  const schedule = (ms) => {
    window.clearTimeout(timer);
    if (stopped || paused || document.hidden) return;
    timer = window.setTimeout(tick, ms);
  };

  const tick = async () => {
    if (stopped || paused) return;
    controller?.abort();
    controller = new AbortController();
    const mine = ++sequence;
    onState?.('loading');
    try {
      await run(controller.signal);
      if (mine !== sequence) return;
      delay = intervalMs;
      lastSuccess = Date.now();
      onState?.('fresh', { at: lastSuccess });
    } catch (error) {
      if (mine !== sequence || error?.name === 'AbortError') return;
      if (error?.kind === 'auth') {
        stopped = true;
        onState?.('auth', { error });
        return;
      }
      delay = Math.min(delay * 2, maxBackoffMs);
      onState?.('error', { error, retryMs: delay });
    }
    schedule(delay);
  };

  const onVisibility = () => {
    if (document.hidden) {
      window.clearTimeout(timer);
      controller?.abort();
      return;
    }
    if (paused || stopped) return;
    const stale = Date.now() - lastSuccess >= intervalMs;
    schedule(stale ? 0 : delay);
  };
  document.addEventListener('visibilitychange', onVisibility);

  return {
    start: () => schedule(intervalMs),
    refreshNow: () => schedule(0),
    pause: () => {
      paused = true;
      window.clearTimeout(timer);
      controller?.abort();
      onState?.('paused');
    },
    resume: () => {
      paused = false;
      schedule(0);
    },
    stop: () => {
      stopped = true;
      window.clearTimeout(timer);
      controller?.abort();
      document.removeEventListener('visibilitychange', onVisibility);
    },
    get paused() {
      return paused;
    },
  };
};
