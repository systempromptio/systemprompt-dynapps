// The signed-in user's live document and notification events: one EventSource per page on
// core's AG-UI stream, fanned out to every subscriber. The page shell
// carries the stream's URL (`data-doc-stream`), so no module hardcodes it.
const listeners = new Set();
let source = null;

const dispatch = (message) => {
  let event;
  try {
    event = JSON.parse(message.data);
  } catch {
    return;
  }
  const name = String(event?.name);
  if (event?.type !== 'CUSTOM' || !(name.startsWith('document.') || name.startsWith('notification.'))) return;
  for (const listener of listeners) listener(event.name, event.value);
};

// EventSource reconnects by itself, but an event sent while it was down is
// lost; every open after the first is announced as `stream.reopened`, so a
// subscriber can reread what it shows.
export const REOPENED = 'stream.reopened';

const open = () => {
  const url = document.querySelector('[data-doc-stream]')?.dataset.docStream;
  if (!url) return null;
  const stream = new EventSource(url);
  let opened = false;
  stream.addEventListener('message', dispatch);
  stream.addEventListener('open', () => {
    if (opened) for (const listener of listeners) listener(REOPENED, null);
    opened = true;
  });
  return stream;
};

export const onDocumentEvent = (listener) => {
  listeners.add(listener);
  source ??= open();
  return () => {
    listeners.delete(listener);
    if (listeners.size || !source) return;
    source.close();
    source = null;
  };
};
