// The Sharing tab. Everyone sees who has access and their own level; someone
// who can share also gets the grant form, revoke per grant, open access
// requests and the access history; the owner also gets transfer and revoke
// all; someone who can only view gets a request form instead. The forms and the owner actions are built once so a live reload never
// clears what is being typed; the lists below them redraw from the server.
import {
  decideAccessRequest, getAccess, listSubjectKinds, requestAccess, revokeAllGrants, revokeGrant,
  shareDocument, transferOwnership,
} from '../documents/documents-api.js';
import { resolvePeople } from './people.js';
import { accessHistory, accessUserIds, whoHasAccess } from './drawer-access.js';
import { accessRequests } from './drawer-access-requests.js';
import { ownerActions } from './drawer-owner.js';
import { requestAccessForm } from './drawer-request-access.js';
import { shareForm } from './drawer-share.js';
import { alertLine, attempt, el } from './pane-dom.js';
import { registerDrawerTab } from './registry.js';

const panes = new WeakMap();

const build = async (pane, ctx) => {
  const state = { alert: alertLine(), history: false, body: el('div', 'sp-ws__pane'), owner: null };
  const reload = () => load(pane, ctx).catch(() => undefined);
  const act = (work) => attempt(state.alert, work, reload);
  const top = [];
  if (ctx.canShare) {
    const kinds = await listSubjectKinds().then((b) => b.subject_kinds).catch(() => ['user']);
    top.push(shareForm(ctx, kinds.length ? kinds : ['user'], (grant) => act(() => shareDocument(ctx.documentId, grant))));
  }
  if (ctx.isOwner) {
    state.owner = ownerActions(ctx, {
      onTransfer: (userId) => act(async () => {
        await transferOwnership(ctx.documentId, userId);
        window.location.reload();
      }),
      onRevokeAll: () => act(() => revokeAllGrants(ctx.documentId)),
    });
  }
  const below = state.owner ? [state.owner.node] : [];
  if (ctx.access === 'view') {
    below.push(requestAccessForm(async (ask) => {
      let sent = false;
      await act(async () => {
        await requestAccess(ctx.documentId, ask);
        sent = true;
      });
      return sent;
    }));
  }
  pane.replaceChildren(...top, state.alert.node, state.body, ...below);
  return { state, reload, act };
};

async function load(pane, ctx) {
  if (!panes.has(pane)) panes.set(pane, build(pane, ctx));
  const { state, reload, act } = await panes.get(pane);
  const access = await getAccess(ctx.documentId, { history: ctx.canShare && state.history ? true : undefined });
  await resolvePeople(ctx.peopleApi, accessUserIds(access));
  const parts = [whoHasAccess(access, {
    canRevoke: ctx.canShare,
    onRevoke: (grantId) => act(() => revokeGrant(ctx.documentId, grantId)),
  })];
  if (ctx.canShare) {
    const requests = accessRequests(access.requests, (id, decision) => act(() => decideAccessRequest(ctx.documentId, id, decision)));
    if (requests) parts.push(requests);
    parts.push(accessHistory(access, {
      isOpen: state.history,
      onToggle: () => {
        state.history = !state.history;
        reload();
      },
    }));
  }
  state.body.replaceChildren(...parts);
  state.owner?.update(access);
}

registerDrawerTab({
  id: 'sharing',
  label: 'Sharing',
  order: 30,
  live: ['document.shared', 'document.access_requested', 'document.ownership_transferred'],
  load,
});
