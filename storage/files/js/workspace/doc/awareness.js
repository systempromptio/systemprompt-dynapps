// Awareness: posts this user's caret and selection as Loro cursors (so they
// survive concurrent edits) and draws everyone else's as decorations. The
// server broadcasts at most one state per 100 ms, so a trailing post follows
// the last move and the final position always reaches the others.
import { Plugin, PluginKey } from 'prosemirror-state';
import { Decoration, DecorationSet } from 'prosemirror-view';
import { rawResponse } from '../../services/api.js';
import { blockList, textOf, encodeCursor, cursorOffset } from './loro.js';
import { person, toneOf } from '../people.js';

// An agent acting for a person announces with that person's user id and an
// `agent` name; keyed apart from the person so both carets show, and the
// person's own id still never draws a caret over their own selection.
const peerKey = (event) => (event.state?.agent ? `${event.user_id}|${event.state.agent}` : event.user_id);

const THROTTLE_MS = 100;
const TRAILING_MS = 180;
const STALE_MS = 60000;

const key = new PluginKey('sp-awareness');

const findBlock = (doc, id) => {
  const list = blockList(doc);
  for (let i = 0; i < list.length; i += 1) if (list.get(i).get('id') === id) return list.get(i);
  return null;
};

const blockStart = (pmDoc, id) => {
  let found = null;
  pmDoc.forEach((node, offset) => {
    if (found === null && node.attrs.id === id) found = offset + 1;
  });
  return found;
};

const caret = (name, tone) => {
  const el = document.createElement('span');
  el.className = `sp-doc-caret sp-doc-caret--tone-${tone}`;
  const label = document.createElement('span');
  label.className = 'sp-doc-caret__label';
  label.textContent = name;
  el.append(label);
  return el;
};

export const createAwareness = ({ api, session, selfId }) => {
  const peers = new Map();
  let view = null;
  let last = 0;
  let trailing = 0;

  const cursorAt = (pmPos) => {
    const $pos = view.state.doc.resolve(pmPos);
    const block = $pos.depth >= 1 ? $pos.node(1) : null;
    const map = block && findBlock(session.state.doc, block.attrs.id);
    const text = map && textOf(map);
    if (!text) return null;
    return { block: block.attrs.id, at: encodeCursor(text, Math.min($pos.parentOffset, text.length)) };
  };

  const send = () => {
    last = Date.now();
    const { from, to, empty } = view.state.selection;
    const a = cursorAt(from);
    const b = empty ? a : cursorAt(to);
    const mode = { edit: 'editing', comment: 'commenting' }[session.state.mode] ?? 'viewing';
    const body = { session_id: session.state.id, color: `tone-${toneOf(selfId)}`, mode };
    if (a?.at) body.cursor = { block: a.block, anchor: a.at, head: b?.block === a.block ? b.at : a.at };
    rawResponse(api, { method: 'POST', body: JSON.stringify(body) }).catch(() => undefined);
  };

  const moved = () => {
    clearTimeout(trailing);
    if (Date.now() - last >= THROTTLE_MS) send();
    trailing = setTimeout(send, TRAILING_MS);
  };

  const decorations = (state) => {
    const decos = [];
    const now = Date.now();
    for (const [key, peer] of peers) {
      const userId = key.split('|')[0];
      if (key === selfId || now - peer.at > STALE_MS || !peer.state.cursor) continue;
      const { block, anchor, head } = peer.state.cursor;
      const start = blockStart(state.doc, block);
      const a = cursorOffset(session.state.doc, anchor);
      const h = cursorOffset(session.state.doc, head);
      if (start === null || a === null || h === null) continue;
      const tone = peer.state.agent ? 'agent' : toneOf(userId);
      const max = state.doc.content.size;
      const from = Math.min(start + Math.min(a, h), max);
      const to = Math.min(start + Math.max(a, h), max);
      if (to > from) decos.push(Decoration.inline(from, to, { class: `sp-doc-select sp-doc-select--tone-${tone}` }));
      decos.push(Decoration.widget(Math.min(start + h, max), () => caret(peer.name, tone), { key: `${userId}:${peer.at}`, side: 1 }));
    }
    return DecorationSet.create(state.doc, decos);
  };

  const plugin = new Plugin({
    key,
    props: { decorations },
    view: (editorView) => {
      view = editorView;
      return {
        update: (v, prev) => {
          if (!prev.selection.eq(v.state.selection) || !prev.doc.eq(v.state.doc)) moved();
        },
      };
    },
  });

  return {
    plugin,
    receive: (event) => {
      peers.set(peerKey(event), { state: event.state, name: event.display_name || person(event.user_id), at: Date.now() });
      view?.dispatch(view.state.tr.setMeta(key, 'peers'));
    },
    hello: () => view && send(),
  };
};
