// The document's live channel: collab updates, resync and close signals,
// and the awareness/comment/presence events the editor relays.
const RELAYED = ['document.awareness', 'document.comment.created', 'document.comment.resolved', 'document.presence'];

export const listen = (liveApi, { onUpdate, rejoin, onEvent, onState }) => {
  const source = new EventSource(liveApi);
  const parse = (handler) => (event) => {
    try {
      handler(JSON.parse(event.data));
    } catch {
      // A malformed frame is skipped; the next gap check re-joins.
    }
  };
  source.addEventListener('document.collab.update', parse(onUpdate));
  source.addEventListener('document.live.resync', () => rejoin('Catching up…', true, 'live channel asked for a resync'));
  source.addEventListener('document.live.closed', () => {
    source.close();
    onState('Access removed');
    document.dispatchEvent(new CustomEvent('sp-ws-access-lost'));
  });
  for (const name of RELAYED) source.addEventListener(name, parse((data) => onEvent(name, data)));
  source.onerror = () => onState(navigator.onLine ? 'Reconnecting…' : 'Offline');
  source.onopen = () => onState('Saved');
  return source;
};
