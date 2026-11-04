// The Proposals tab: changes an agent or a reviewer-gated folder held back for
// review. Open proposals first, then the decided ones; each shows its message,
// who proposed it and when, its changed lines on request, and, for an editor,
// Accept or Reject with an optional note. Hidden when proposals are off.
import { decideProposal, getProposal, listProposals } from '../documents/documents-api.js';
import { person, resolvePeople } from './people.js';
import { alertLine, attempt, button, el, lineList, relativeTime } from './pane-dom.js';
import { paneUnavailable, registerDrawerTab } from './registry.js';

const panes = new WeakMap();

const changesToggle = (proposal, ctx, state, reload) => {
  const open = state.changes.has(proposal.id);
  const toggle = button(open ? 'Hide changes' : 'Show changes', 'sp-btn--ghost', () => {
    if (open) {
      state.changes.delete(proposal.id);
      reload();
      return;
    }
    attempt(state.alert, async () => {
      const { lines } = await getProposal(ctx.documentId, proposal.id);
      state.changes.set(proposal.id, lines);
    }, reload);
  });
  toggle.setAttribute('aria-expanded', String(open));
  return open ? [toggle, lineList(state.changes.get(proposal.id), 'No changed blocks')] : [toggle];
};

const decision = (proposal, ctx, state, reload) => {
  const box = el('div', 'sp-ws-pane__actions');
  const note = el('input', 'sp-field-input');
  note.placeholder = 'Note (optional)';
  note.setAttribute('aria-label', 'Note');
  const decide = (choice) => () => attempt(state.alert, () => decideProposal(
    ctx.documentId, proposal.id, choice, note.value.trim() || undefined,
  ), reload);
  box.append(note, button('Accept', 'sp-btn--primary', decide('accept')), button('Reject', 'sp-btn--outline', decide('reject')));
  return box;
};

const proposalRow = (proposal, ctx, state, reload) => {
  const item = el('li', `sp-ws__entry sp-ws-proposal sp-ws-proposal--${proposal.status}`);
  const who = proposal.actor_kind === 'agent' ? `${person(proposal.proposer)} via agent` : person(proposal.proposer);
  item.append(
    el('span', 'sp-ws__entry-title', proposal.message || '(no message)'),
    el('span', 'sp-ws__muted', `${who} · ${relativeTime(proposal.created_at)} · ${proposal.status}`),
  );
  if (proposal.note) item.append(el('span', 'sp-ws__muted', `Note: ${proposal.note}`));
  item.append(...changesToggle(proposal, ctx, state, reload));
  // Why: core refuses `self_review` when the proposer decides their own
  // proposal, so the viewer is told who can, rather than offered a button
  // that can only fail.
  const own = proposal.proposer === ctx.userId;
  if (proposal.status === 'open' && ctx.canEdit && own) item.append(el('span', 'sp-ws__muted', 'Waiting for another editor to review'));
  else if (proposal.status === 'open' && ctx.canEdit) item.append(decision(proposal, ctx, state, reload));
  return item;
};

const section = (title, rows, empty) => {
  const box = el('section', 'sp-ws__pane');
  const list = el('ul', 'sp-ws__list');
  list.append(...(rows.length ? rows : [el('li', 'sp-ws__muted', empty)]));
  box.append(el('h3', 'sp-ws-pane__heading', title), list);
  return box;
};

const load = async (pane, ctx) => {
  if (!panes.has(pane)) {
    const state = { alert: alertLine(), changes: new Map(), body: el('div', 'sp-ws__pane') };
    pane.replaceChildren(state.alert.node, state.body);
    panes.set(pane, { state, reload: () => load(pane, ctx).catch(() => undefined) });
  }
  const { state, reload } = panes.get(pane);
  let proposals;
  try {
    ({ proposals } = await listProposals(ctx.documentId));
  } catch (err) {
    if (err.status === 404) {
      paneUnavailable(pane);
      return;
    }
    state.alert.show(err);
    return;
  }
  await resolvePeople(ctx.peopleApi, proposals.flatMap((p) => [p.proposer, p.reviewer]));
  const row = (p) => proposalRow(p, ctx, state, reload);
  const open = proposals.filter((p) => p.status === 'open');
  const closed = proposals.filter((p) => p.status !== 'open');
  state.body.replaceChildren(
    section('Open', open.map(row), 'No open proposals'),
    section('Closed', closed.map(row), 'No closed proposals'),
  );
};

registerDrawerTab({
  id: 'proposals',
  label: 'Proposals',
  order: 50,
  eager: true,
  live: ['document.proposal', 'document.revision.created'],
  load,
});
