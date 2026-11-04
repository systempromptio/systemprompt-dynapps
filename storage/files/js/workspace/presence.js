// The presence avatars in the top bar. The page says it is viewing on a
// heartbeat (core expires a silent viewer) and follows `document.presence`
// events for arrivals and state changes. Core sends no event when a viewer
// expires, so the list is re-read on a slow cycle to drop who has left.
// While a live session holds this page in edit or comment mode, the heartbeat
// repeats that state: a beat saying `viewing` would demote the session's
// presence row, and core refuses the next update post from a viewer.
import { getPresence, touchPresence } from '../documents/documents-api.js';
import { initialsOf, person, resolvePeople, toneOf } from './people.js';
import { registerLiveHandler } from './registry.js';

const HEARTBEAT_MS = 20000;
const REREAD_MS = 60000;

// A presence row from the bridge or an MCP agent is an assistant working for
// that person, so it shows as "AI" in the person's tone instead of a second
// copy of their initials.
const avatar = (editor) => {
  const template = document.querySelector('template[data-template="ws-avatar"]');
  const item = template.content.firstElementChild.cloneNode(true);
  const badge = item.querySelector('[data-field="initials"]');
  const name = person(editor.user_id);
  const assistant = editor.source && editor.source !== 'web';
  badge.textContent = assistant ? 'AI' : initialsOf(name);
  badge.title = assistant ? `Claude Code for ${name} (${editor.state})` : `${name} (${editor.state})`;
  badge.classList.add(`sp-avatar--tone-${toneOf(editor.user_id)}`);
  if (assistant) item.classList.add('sp-ws__person--assistant');
  return item;
};

const keyOf = (editor) => `${editor.user_id}:${editor.source ?? 'web'}`;

export const initPresence = async (ctx) => {
  const { root, documentId } = ctx;
  const slot = root.querySelector('[data-slot="ws-presence"]');
  const editors = new Map();
  const draw = async () => {
    await resolvePeople(ctx.peopleApi, [...editors.values()].map((e) => e.user_id));
    slot.replaceChildren(...[...editors.values()].map(avatar));
  };
  registerLiveHandler('document.presence', (name, value) => {
    const editor = { user_id: value.user_id, state: value.state, source: value.source_kind };
    editors.set(keyOf(editor), editor);
    draw();
  });
  const reread = async () => {
    const body = await getPresence(documentId);
    editors.clear();
    for (const editor of body?.editors ?? []) editors.set(keyOf(editor), editor);
    await draw();
  };
  let held = 'viewing';
  document.addEventListener('sp-ws-collab-mode', (event) => {
    held = event.detail?.state ?? 'viewing';
  });
  const beat = () => touchPresence(documentId, held).catch(() => undefined);
  window.setInterval(beat, HEARTBEAT_MS);
  window.setInterval(() => reread().catch(() => undefined), REREAD_MS);
  try {
    await beat();
    await reread();
  } catch {
    slot.replaceChildren();
  }
};
