// Remote cursors over the document's awareness channel. A sheet cursor names
// the sheet id as its block and the anchor and head cells by "<rowId>:<colId>",
// so a row someone else inserts moves the box with the cell it marked. A cell
// past the replica's last row or column has no id yet and is sent by position
// ("@row,col"). Posts are throttled the way the doc editor's are.
import { rawResponse } from '../../services/api.js';
import { person, toneOf } from '../people.js';
import { cellKey } from './replica.js';

const THROTTLE_MS = 100;
const TRAILING_MS = 180;
const STALE_MS = 60000;

const peerKey = (event) => (event.state?.agent ? `${event.user_id}|${event.state.agent}` : event.user_id);

const encode = (sheet, { row, col }) => {
  const rowId = sheet?.rows[row];
  const colId = sheet?.cols[col];
  return rowId != null && colId != null ? cellKey(rowId, colId) : `@${row},${col}`;
};

const decode = (sheet, value) => {
  if (typeof value !== 'string') return null;
  if (value.startsWith('@')) {
    const [row, col] = value.slice(1).split(',').map(Number);
    return Number.isInteger(row) && Number.isInteger(col) ? { row, col } : null;
  }
  const [rowId, colId] = value.split(':');
  const row = sheet.rows.indexOf(rowId);
  const col = sheet.cols.indexOf(colId);
  return row >= 0 && col >= 0 ? { row, col } : null;
};

export const createPresence = ({ api, session, selfId, onPeers }) => {
  const peers = new Map();
  let last = 0;
  let trailing = 0;
  let pending = null;

  const send = () => {
    last = Date.now();
    if (!pending) return;
    const { sheet, selection } = pending;
    const mode = { edit: 'editing', comment: 'commenting' }[session.state.mode] ?? 'viewing';
    const body = { session_id: session.state.id, color: `tone-${toneOf(selfId)}`, mode };
    if (sheet) body.cursor = { block: sheet.id, anchor: encode(sheet, selection.anchor), head: encode(sheet, selection.head) };
    rawResponse(api, { method: 'POST', body: JSON.stringify(body) }).catch(() => undefined);
  };

  const ranges = (sheet) => {
    const now = Date.now();
    const out = [];
    for (const [key, peer] of peers) {
      const cursor = peer.state.cursor;
      if (key === selfId || now - peer.at > STALE_MS || !sheet || cursor?.block !== sheet.id) continue;
      const a = decode(sheet, cursor.anchor);
      const h = decode(sheet, cursor.head) ?? a;
      if (!a) continue;
      out.push({
        name: peer.name,
        tone: peer.state.agent ? 'agent' : toneOf(key.split('|')[0]),
        range: { top: Math.min(a.row, h.row), left: Math.min(a.col, h.col), bottom: Math.max(a.row, h.row), right: Math.max(a.col, h.col) },
      });
    }
    return out;
  };

  return {
    moved: (sheet, selection) => {
      pending = { sheet, selection };
      clearTimeout(trailing);
      if (Date.now() - last >= THROTTLE_MS) send();
      trailing = setTimeout(send, TRAILING_MS);
    },
    receive: (event) => {
      peers.set(peerKey(event), { state: event.state, name: event.display_name || person(event.user_id), at: Date.now() });
      onPeers();
    },
    ranges,
  };
};
