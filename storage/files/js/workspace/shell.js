// The document workspace: builds the page context from the server-rendered
// access, loads the document, then builds the drawer, the name menu and the
// palette from what the feature modules registered, mounts the format editor
// and feeds every live document event to the registered handlers through
// one subscription to the document event stream.
import { subscribeDocument } from '../documents/documents-events.js';
import { readJson } from '../services/drive-rows.js';
import { showDetails } from './drawer-details.js';
import { initDrawer, toggleDrawer } from './drawer.js';
import { mountEditor } from './editors.js';
import { initMenu } from './menu.js';
import { initPalette } from './palette.js';
import { documentIdOf, permits } from './people.js';
import { initPresence } from './presence.js';
import { showRail } from './rail.js';
import { initNotificationBell } from '../services/notification-bell.js';
import { dispatchLive, registerLiveHandler } from './registry.js';
import './commands.js';
import './drawer-versions.js';
import './drawer-sharing.js';
import './drawer-comments.js';
import './drawer-proposals.js';
import './doc/editor.js';
import './sheet/editor.js';
import './slides/editor.js';

const contextOf = (root) => {
  const access = root.dataset.myAccess || 'view';
  return {
    root,
    doc: null,
    view: null,
    access,
    canEdit: root.dataset.canEdit === 'true',
    canShare: root.dataset.canShare === 'true',
    isOwner: access === 'own',
    canComment: permits(access, 'comment'),
    userId: root.dataset.userId,
    documentId: documentIdOf(root),
    peopleApi: root.dataset.peopleApi,
    openPane: (id) => toggleDrawer(root, id, true),
    refresh: () => undefined,
  };
};

const SUITES = { xlsx: 'sheet', ods: 'sheet', pptx: 'slides', odp: 'slides' };

const showDocument = async (ctx) => {
  const { root } = ctx;
  const view = await readJson(root.dataset.documentApi);
  ctx.view = view;
  ctx.doc = view.document;
  root.querySelector('[data-field="name"]').textContent = view.document.name;
  root.querySelector('[data-field="format"]').textContent = view.document.format.toUpperCase();
  root.classList.add(`sp-ws--${SUITES[view.document.format] ?? 'doc'}`);
  document.title = `${view.document.name} - Drive`;
  await showDetails(root, view);
  return view.document;
};

const onAccessLost = (root) => {
  const notice = root.querySelector('[data-slot="ws-revoked"]');
  const body = root.querySelector('[data-slot="ws-body"]');
  if (notice) notice.hidden = false;
  if (body) body.inert = true;
};

export const init = async () => {
  const root = document.querySelector('[data-ws-root]');
  if (!root) return;
  const ctx = contextOf(root);
  ctx.refresh = () => showDocument(ctx).catch(() => undefined);
  initNotificationBell();
  document.addEventListener('sp-ws-access-lost', () => onAccessLost(root));
  let doc;
  try {
    initDrawer(ctx);
    doc = await showDocument(ctx);
  } catch {
    root.querySelector('[data-field="name"]').textContent = 'This document could not be loaded.';
    return;
  }
  initMenu(ctx, initPalette(ctx));
  registerLiveHandler('document.ownership_transferred', () => window.location.reload());
  subscribeDocument(ctx.documentId, dispatchLive);
  await Promise.allSettled([mountEditor(ctx), showRail(root, doc.format), initPresence(ctx)]);
};

init();
