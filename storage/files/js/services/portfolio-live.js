// Retained snapshots are authoritative; SSE generations only request a refresh.
export function watchPortfolio({ url, generation = '', onData, onStatus = () => {} }) {
  let stream = null;
  let reconnect = null;
  let poll = null;
  let request = null;
  let epoch = 0;
  let failures = 0;
  let disposed = false;
  let hidden = document.hidden;
  let cursor = generation;

  const refresh = async () => {
    if (disposed || hidden) return;
    request?.abort();
    request = new AbortController();
    const version = ++epoch;
    try {
      const response = await fetch(url, { credentials: 'same-origin', cache: 'no-store', signal: request.signal });
      if (disposed || hidden || version !== epoch) return;
      if (!response.ok) {
        if (response.status === 401 || response.status === 403) {
          onStatus('Access expired. Sign in again.');
          dispose();
          return;
        }
        throw new Error('Snapshot unavailable');
      }
      const data = await response.json();
      if (disposed || hidden || version !== epoch) return;
      onData(data);
      onStatus(data.pending_jobs ? 'Preparing the requested range…' : 'Live snapshots');
      if (poll) clearTimeout(poll);
      poll = setTimeout(refresh, data.pending_jobs ? 2000 : 30000);
    } catch (error) {
      if (disposed || hidden || version !== epoch || error.name === 'AbortError') return;
      onStatus('Snapshot refresh delayed. Retained results remain visible.');
      if (poll) clearTimeout(poll);
      poll = setTimeout(refresh, 5000);
    }
  };
  const disconnect = () => {
    stream?.close(); stream = null;
    if (reconnect) clearTimeout(reconnect);
    if (poll) clearTimeout(poll);
    reconnect = null; poll = null;
    ++epoch; request?.abort(); request = null;
  };
  const connect = () => {
    if (disposed || hidden || stream) return;
    const active = new EventSource('/api/v1/analytics/live' + (cursor ? '?after=' + encodeURIComponent(cursor) : ''));
    stream = active;
    stream.onopen = () => { if (disposed || hidden || stream !== active) return; failures = 0; onStatus('Live snapshots'); };
    const update = event => {
      if (disposed || hidden || stream !== active) return;
      if (!/^\d+\.\d+\.\d+$/.test(event.lastEventId) || event.lastEventId.length > 96) return;
      const changed = cursor !== event.lastEventId;
      cursor = event.lastEventId;
      if (changed || event.type === 'resync') refresh();
    };
    stream.addEventListener('snapshot', update);
    stream.addEventListener('resync', update);
    stream.onerror = () => {
      if (stream !== active) return;
      active.close(); stream = null;
      if (disposed || hidden) return;
      onStatus('Reconnecting. Retained snapshots remain visible.');
      reconnect = setTimeout(connect, Math.min(30000, 1000 * 2 ** Math.min(failures++, 5)));
    };
    if (poll) clearTimeout(poll);
    poll = setTimeout(refresh, 2000);
  };
  const visibility = () => {
    hidden = document.hidden;
    disconnect();
    if (!hidden) { connect(); refresh(); }
  };
  const dispose = () => {
    if (disposed) return;
    disposed = true;
    disconnect();
    document.removeEventListener('visibilitychange', visibility);
    window.removeEventListener('pagehide', dispose);
  };
  document.addEventListener('visibilitychange', visibility);
  window.addEventListener('pagehide', dispose);
  connect();
  return dispose;
}
