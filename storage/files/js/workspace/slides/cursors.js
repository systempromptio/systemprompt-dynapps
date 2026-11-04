// Remote presence on the deck: each peer's caret is posted as a Loro cursor
// on the paragraph it sits in (the same awareness body the doc editor
// sends), and drawn here as a toned outline and name on the shape that
// holds that paragraph. Throttled to the server's 100 ms broadcast rate.
import { rawResponse } from '../../services/api.js';
import { encodeCursor, textOf } from '../doc/loro.js';
import { person, toneOf } from '../people.js';
import { itemsOf, shapesOf, slideList } from './deck.js';

const THROTTLE_MS = 100;
const STALE_MS = 60000;

const shapeOfBlock = (doc, blockId) => {
  for (const slide of itemsOf(slideList(doc))) {
    for (const shape of shapesOf(slide)) {
      if (itemsOf(shape.get('paragraphs')).some((p) => p.get('id') === blockId)) return shape.get('id');
    }
  }
  return null;
};

export const createCursors = ({ api, session, selfId, shapeEl }) => {
  const peers = new Map();
  let timer = 0;

  const post = (shape, view) => {
    const $pos = view.state.selection.$head;
    const block = $pos.depth >= 1 ? $pos.node(1) : null;
    const map = block && itemsOf(shape.get('paragraphs')).find((p) => p.get('id') === block.attrs.id);
    const text = map && textOf(map);
    const mode = { edit: 'editing', comment: 'commenting' }[session.state.mode] ?? 'viewing';
    const body = { session_id: session.state.id, color: `tone-${toneOf(selfId)}`, mode };
    const at = text && encodeCursor(text, Math.min($pos.parentOffset, text.length));
    if (at) body.cursor = { block: block.attrs.id, anchor: at, head: at };
    rawResponse(api, { method: 'POST', body: JSON.stringify(body) }).catch(() => undefined);
  };

  const draw = () => {
    for (const el of document.querySelectorAll('.sp-deck-peer')) el.remove();
    const now = Date.now();
    for (const [userId, peer] of peers) {
      if (userId === selfId || now - peer.at > STALE_MS || !peer.block) continue;
      const host = shapeEl(shapeOfBlock(session.state.doc, peer.block));
      if (!host) continue;
      const tag = document.createElement('span');
      tag.className = `sp-deck-peer sp-doc-caret--tone-${peer.agent ? 'agent' : toneOf(userId)}`;
      tag.textContent = peer.name;
      host.append(tag);
    }
  };

  return {
    moved: (shape, view) => {
      clearTimeout(timer);
      timer = setTimeout(() => post(shape, view), THROTTLE_MS);
    },
    receive: (event) => {
      const key = event.state?.agent ? `${event.user_id}|${event.state.agent}` : event.user_id;
      peers.set(key, {
        block: event.state?.cursor?.block ?? null,
        agent: event.state?.agent,
        name: event.display_name || person(event.user_id),
        at: Date.now(),
      });
      draw();
    },
    draw,
  };
};
