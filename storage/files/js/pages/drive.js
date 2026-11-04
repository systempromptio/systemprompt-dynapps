// Drive: one folder's subfolders and documents, or the recent, shared and
// own lists, searched and sorted by the document API page by page, shown as
// a list or a grid. Files dropped on the page are imported as documents. An
// administrator also edits each folder's agent edit policy in its row.
// Features extend the listing through drive-listing-registry.js; the class
// badges and the folder Class dialog are two of them.
import { readJson } from '../services/drive-rows.js';
import { createListing } from '../services/drive-listing-view.js';
import { listingActionsFor, onRegistryChange } from '../services/drive-listing-registry.js';
import { initCreateDialog } from '../services/drive-listing-create.js';
import { initClassBadges } from '../services/drive-class-badges.js';
import { initFolderClass } from '../services/drive-folder-class.js';
import { on, initDelegation } from '../services/events.js';
import { rawFetch } from '../services/api.js';
import { showPromptDialog } from '../services/confirm.js';
import { showToast } from '../services/toast.js';
import { uploadAll } from '../workspace/upload/queue.js';
import { REOPENED, onDocumentEvent } from '../services/doc-events.js';

const enc = encodeURIComponent;

const state = { root: null, view: null, cursor: null, scope: 'folder', query: '', sort: 'updated' };

const savePolicy = async (evt, select) => {
  const { foldersApi } = state.root.dataset;
  try {
    await rawFetch(`${foldersApi}/${enc(select.dataset.folderId)}`, {
      method: 'PATCH',
      body: JSON.stringify({ agent_edit_policy: select.value }),
    });
    select.dataset.saved = select.value;
    showToast('Agent edit policy saved.', 'success');
  } catch (err) {
    select.value = select.dataset.saved;
    showToast(err.message, 'error');
  }
};

const pageUrl = () => {
  const { documentsApi, folderId } = state.root.dataset;
  const params = new URLSearchParams({ scope: state.scope, sort: state.sort });
  if (state.scope === 'folder' && folderId) params.set('folder', folderId);
  if (state.query) params.set('query', state.query);
  if (state.cursor) params.set('cursor', state.cursor);
  return `${documentsApi}?${params}`;
};

const loadPage = async () => {
  const { root, view } = state;
  const rows = root.querySelector('[data-slot="drive-rows"]');
  const more = root.querySelector('[data-action="drive-more"]');
  const listing = await readJson(pageUrl());
  root.querySelector('[data-slot="drive-loading"]')?.remove();
  rows.append(
    ...(state.cursor ? [] : listing.folders.map((f) => view.render('folder', f))),
    ...listing.documents.map((d) => view.render('document', d)),
  );
  state.cursor = listing.next_cursor;
  more.hidden = !state.cursor;
  root.querySelector('[data-slot="drive-empty"]').hidden = rows.children.length > 0;
};

const loadSafely = async () => {
  try {
    await loadPage();
  } catch {
    const empty = state.root.querySelector('[data-slot="drive-empty"]');
    empty.textContent = 'The document list could not be loaded.';
    empty.hidden = false;
    state.root.querySelector('[data-slot="drive-loading"]')?.remove();
  }
};

const reload = () => {
  state.cursor = null;
  state.view.forget();
  state.root.querySelector('[data-slot="drive-rows"]').replaceChildren();
  return loadSafely();
};

const press = (buttons, active) => buttons.forEach((b) => b.setAttribute('aria-pressed', String(b === active)));

const newFolder = () => showPromptDialog('New folder', 'Folder name', '', async (name) => {
  await rawFetch(state.root.dataset.foldersApi, {
    method: 'POST',
    body: JSON.stringify({ name: name.trim(), parent_id: state.root.dataset.folderId ?? null }),
  });
  reload();
});

const upload = async (files) => {
  const list = state.root.querySelector('[data-slot="drive-uploads"]');
  list.hidden = false;
  await uploadAll([...files], { list, folderId: state.root.dataset.folderId });
  reload();
};

const wireFilters = () => {
  const form = state.root.querySelector('[data-slot="drive-filters"]');
  const scopes = [...form.querySelectorAll('[data-scope]')];
  const views = [...form.querySelectorAll('[data-view]')];
  let pending = null;
  form.addEventListener('submit', (event) => event.preventDefault());
  form.elements.query.addEventListener('input', () => {
    window.clearTimeout(pending);
    pending = window.setTimeout(() => {
      state.query = form.elements.query.value.trim();
      reload();
    }, 250);
  });
  form.elements.sort.addEventListener('change', () => {
    state.sort = form.elements.sort.value;
    reload();
  });
  scopes.forEach((b) => b.addEventListener('click', () => {
    press(scopes, b);
    state.scope = b.dataset.scope;
    reload();
  }));
  views.forEach((b) => b.addEventListener('click', () => {
    press(views, b);
    state.root.classList.toggle('sp-p-drive--grid', b.dataset.view === 'grid');
  }));
  form.querySelector('[data-action="drive-new-folder"]').addEventListener('click', newFolder);
  form.querySelector('[data-slot="drive-upload"]').addEventListener('change', (event) => upload(event.target.files));
};

const wireDrop = () => {
  const hint = state.root.querySelector('[data-slot="drive-drop"]');
  state.root.addEventListener('dragover', (event) => {
    event.preventDefault();
    hint.hidden = false;
  });
  state.root.addEventListener('dragleave', () => { hint.hidden = true; });
  state.root.addEventListener('drop', (event) => {
    event.preventDefault();
    hint.hidden = true;
    upload(event.dataTransfer.files);
  });
};

const renderActions = (listing) => {
  const slot = document.querySelector('[data-slot="drive-listing-actions"]');
  const template = document.querySelector('template[data-template="drive-listing-action"]');
  if (!slot || !template) return;
  slot.replaceChildren(...listingActionsFor(listing.isAdmin).map((action) => {
    const button = template.content.firstElementChild.cloneNode(true);
    button.textContent = action.label;
    button.dataset.listingAction = action.id;
    button.addEventListener('click', () => action.run({ listing }));
    return button;
  }));
};

export const init = () => {
  state.root = document.querySelector('[data-drive-list]');
  if (!state.root) return;
  state.view = createListing(state.root, { reload });
  const { listing } = state.view;
  initDelegation();
  on('click', '[data-action="drive-more"]', loadSafely);
  on('change', '[data-action="drive-policy"]', savePolicy);
  wireFilters();
  wireDrop();
  const create = document.querySelector('[data-drive-create]');
  if (create) initCreateDialog(create, listing);
  initClassBadges(listing);
  const classDialog = document.querySelector('[data-drive-folder-class]');
  if (classDialog) initFolderClass(classDialog);
  renderActions(listing);
  onRegistryChange((what) => { if (what === 'actions') renderActions(listing); });
  loadSafely();
  onDocumentEvent((name) => {
    const missed = name === 'document.shared' || name === REOPENED;
    if (missed && state.scope !== 'mine') reload();
  });
};

init();
