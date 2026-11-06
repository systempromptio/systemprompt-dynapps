// One live editing session over core's collab protocol v2: join for a
// snapshot, post local Loro updates (one request in flight, coalesced),
// follow the document's live channel for everyone else's updates, and
// re-join whenever the server says the replica is out of step. A re-join
// carries this replica's unsent edits across: they are exported once the
// join resolves (so keystrokes typed while it was in flight come too), merged
// over the new snapshot in a scratch doc, and handed to `onReset` to be
// written again under the new peer.
import { rawResponse } from '../../services/api.js';
import { permits } from '../people.js';
import { listen } from './live.js';
import { retryDelay } from './contended.js';
import { newDoc, fromB64, toB64, versionB64, sentWithPeers } from './loro.js';

const MODES = ['edit', 'comment', 'view'];
const FLUSH_MS = 60;
const RETRY_MS = 3000;

// `localStorage['sp-debug']` turns on the rejoin trace: a rejoin mid-typing
// means core refused a post, and the reason is what an e2e rerun reads back
// from `window.spCollabTrace` (the standards ban console output).
const trace = (entry) => {
  try {
    if (!localStorage.getItem('sp-debug')) return;
  } catch {
    return;
  }
  window.spCollabTrace = [...(window.spCollabTrace ?? []), { at: Date.now(), ...entry }].slice(-50);
};

export const createSession = ({ mode: wanted = 'edit', collabApi, liveApi, onReset, onRemote, onEvent, onState }) => {
  const s = { doc: null, id: null, seq: 0, level: 'view', mode: 'view', sent: null, timer: 0, busy: false, held: false, source: null, closed: false };

  // `carry` names the old replica and the version its unsent edits start
  // from; they are read only after the join answers.
  const join = async (carry = null) => {
    let joined = null;
    // A viewer below edit access joins in comment mode, falling back to view.
    for (const mode of MODES.slice(Math.max(0, MODES.indexOf(wanted)))) {
      const resp = await rawResponse(`${collabApi}/join`, { method: 'POST', body: JSON.stringify({ mode }) });
      if (resp.ok) {
        joined = await resp.json();
        s.mode = mode;
        break;
      }
      if (resp.status !== 403) break;
    }
    if (!joined) throw new Error('Live editing is not available for this document.');
    const unsent = carry && carry.doc.oplogVersion().compare(carry.from) !== 0
      ? carry.doc.export({ mode: 'update', from: carry.from }) : null;
    s.id = joined.session_id;
    s.seq = joined.seq ?? 0;
    s.level = joined.level;
    s.doc = newDoc(fromB64(joined.snapshot_b64));
    // Collab protocol 3: core refuses any update carrying operations of a
    // peer other than the one this join assigned.
    s.doc.setPeerId(joined.peer_id);
    s.sent = s.doc.oplogVersion();
    s.doc.subscribeLocalUpdates(() => schedule());
    const { doc } = s;
    s.doc.subscribe((batch) => {
      if (batch.by === 'import' && s.doc === doc) onRemote(doc, batch);
    });
    // Access can change while offline: the carried edits are written again
    // only when this join still grants edit, or core would refuse them.
    const mayCarry = s.mode === 'edit' && permits(s.level, 'edit');
    onReset(s, unsent && mayCarry ? mergeUnsent(joined.snapshot_b64, unsent) : null);
    const state = { edit: 'editing', comment: 'commenting' }[s.mode] ?? 'viewing';
    document.dispatchEvent(new CustomEvent('sp-ws-collab-mode', { detail: { state } }));
    onState(unsent && !mayCarry ? 'Your access changed; unsent edits were not kept' : 'Saved');
  };

  const mergeUnsent = (snapshot, unsent) => {
    const scratch = newDoc(fromB64(snapshot));
    scratch.import(unsent);
    return scratch;
  };

  // A refused change (`keep: false`) is dropped, or it would be refused again,
  // but anything typed after the refusal still carries. Posting holds until
  // the join resolves, so the old replica keeps collecting input meanwhile.
  const rejoin = async (why, keep = true, cause = why) => {
    trace({ event: 'rejoin', cause, keep, session: s.id, seq: s.seq });
    onState(why);
    const carry = s.doc ? { doc: s.doc, from: keep ? s.sent : s.doc.oplogVersion() } : null;
    clearTimeout(s.timer);
    s.held = true;
    try {
      await join(carry);
    } catch {
      onState('Offline');
    } finally {
      s.held = false;
    }
    if (pending()) schedule();
  };

  const post = async () => {
    if (s.busy || s.held || s.closed || s.mode === 'view' || !pending()) return;
    const bytes = s.doc.export({ mode: 'update', from: s.sent });
    s.busy = true;
    const version = s.doc.oplogVersion();
    onState('Saving…');
    try {
      const resp = await rawResponse(`${collabApi}/updates`, {
        method: 'POST',
        body: JSON.stringify({ session_id: s.id, vv: versionB64(s.doc), update_b64: toB64(bytes) }),
      });
      if (resp.status === 409 || resp.status === 400 || resp.status === 403) {
        s.busy = false;
        const code = await resp.clone().json().then((b) => b?.code ?? b?.error).catch(() => null);
        await rejoin(
          resp.status === 400 ? 'Change refused — reloaded' : 'Reconnecting…',
          resp.status !== 400,
          `post refused: HTTP ${resp.status}${code ? ` ${code}` : ''}`,
        );
        return;
      }
      if (resp.status === 429) {
        const wait = await retryDelay(resp, RETRY_MS);
        clearTimeout(s.timer);
        s.held = true;
        s.timer = setTimeout(() => { s.held = false; post(); }, wait);
        return;
      }
      if (!resp.ok) throw new Error(String(resp.status));
      const ack = await resp.json();
      s.sent = sentWithPeers(s.doc, version);
      s.seq = Math.max(s.seq, ack.seq ?? 0);
      if (ack.missing_b64) {
        s.doc.import(fromB64(ack.missing_b64));
        s.sent = sentWithPeers(s.doc, s.sent);
      }
      onState('Saved');
    } catch {
      onState('Unsaved changes');
      s.busy = false;
      s.timer = setTimeout(() => post(), RETRY_MS);
      return;
    } finally {
      s.busy = false;
    }
    if (pending()) schedule();
  };

  // A Loro update export always carries a header, so compare versions.
  const pending = () => s.doc && s.sent && s.doc.oplogVersion().compare(s.sent) !== 0;

  function schedule() {
    if (s.held) return;
    clearTimeout(s.timer);
    s.timer = setTimeout(() => post(), FLUSH_MS);
  }

  const onUpdate = (data) => {
    if (data.edit_session_id !== s.id) return;
    if (data.seq > s.seq + 1) {
      rejoin('Catching up…', true, `live update seq ${data.seq} skipped past ${s.seq}`);
      return;
    }
    s.seq = Math.max(s.seq, data.seq);
    s.doc.import(fromB64(data.update_b64));
    s.sent = sentWithPeers(s.doc, s.sent);
  };

  const leave = () => {
    s.closed = true;
    s.source?.close();
    if (!s.id) return;
    rawResponse(`${collabApi}/leave`, { method: 'POST', keepalive: true, body: JSON.stringify({ session_id: s.id }) })
      .catch(() => undefined);
  };

  return {
    state: s,
    // A discrete commit (a sheet cell on Enter) posts now: the coalescing
    // delay exists for keystroke runs, and on one commit it is pure latency.
    flush: () => {
      if (s.held) return;
      clearTimeout(s.timer);
      post();
    },
    start: async () => {
      await join();
      s.source = listen(liveApi, { onUpdate, rejoin, onEvent, onState });
      window.addEventListener('pagehide', leave);
    },
    flush: () => post(),
    leave,
  };
};
