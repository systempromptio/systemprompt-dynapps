// The Drive listing's rows: each folder and document cloned from its
// <template>, filled with text only, run through the registered row
// extensions, and given the row menu when any item is registered for its
// kind. The listing object built here is what every extension receives.
import { cloneRow, editorsLabel, formatDate, loadEditors, loadPeople, shortId } from './drive-rows.js';
import { applyRowExtensions, onRegistryChange } from './drive-listing-registry.js';
import { hasMenu, initRowMenus, openRowMenu } from './drive-listing-menu.js';

const enc = encodeURIComponent;

const fillEditors = async (row, documentsApi, id) => {
  const cell = row.querySelector('[data-field="editors"]');
  try {
    const editors = await loadEditors(`${documentsApi}/${enc(id)}/presence`);
    const names = await loadPeople(editors.map((e) => e.user_id));
    cell.textContent = editorsLabel(editors, names);
  } catch {
    cell.textContent = 'Unavailable';
  }
};

const folderRow = (folder, { baseUrl }) => {
  const row = cloneRow('drive-folder-row', { updated: formatDate(folder.updated_at) });
  const link = row.querySelector('[data-field="link"]');
  link.textContent = folder.name;
  link.href = `${baseUrl}?folder=${enc(folder.id)}`;
  const policy = row.querySelector('[data-action="drive-policy"]');
  if (policy) {
    policy.dataset.folderId = folder.id;
    policy.value = folder.agent_edit_policy ?? 'direct';
    policy.dataset.saved = policy.value;
  }
  return row;
};

const documentRow = (doc, { baseUrl, documentsApi }) => {
  const row = cloneRow('drive-document-row', {
    format: doc.format,
    head: shortId(doc.head_revision_id),
    updated: formatDate(doc.updated_at),
  });
  const link = row.querySelector('[data-field="link"]');
  link.textContent = doc.name;
  link.href = `${baseUrl}/${enc(doc.id)}`;
  fillEditors(row, documentsApi, doc.id);
  return row;
};

const BUILDERS = { folder: folderRow, document: documentRow };

// The admin-only column is in every row template; a non-admin listing drops
// it so the row lines up with a header that has no such column.
const dropAdminCells = (row, dataset) => {
  if (!('admin' in dataset)) row.querySelectorAll('[data-admin-only]').forEach((cell) => cell.remove());
  return row;
};

export const createListing = (root, { reload }) => {
  const rows = new Map();
  const { dataset } = root;
  const listing = {
    root,
    folderId: dataset.folderId ?? null,
    isAdmin: 'admin' in dataset,
    reload,
    invalidate: () => rows.forEach((entry, row) => refresh(row, entry)),
  };

  const refresh = (row, { kind, item }) => {
    applyRowExtensions(row, row.querySelector('[data-slot="ext"]'), kind, item);
    const button = row.querySelector('[data-action="drive-row-menu"]');
    if (button) button.hidden = !hasMenu(kind);
  };

  const render = (kind, item) => {
    const row = dropAdminCells(BUILDERS[kind](item, dataset), dataset);
    row.dataset.kind = kind;
    rows.set(row, { kind, item });
    refresh(row, { kind, item });
    return row;
  };

  const forget = () => rows.clear();

  const openFor = (row, point) => {
    const entry = rows.get(row);
    if (!entry || !hasMenu(entry.kind)) return false;
    const opener = row.querySelector('[data-action="drive-row-menu"]') ?? row;
    openRowMenu({ ...entry, listing, opener, point });
    return true;
  };

  initRowMenus();
  root.addEventListener('click', (event) => {
    const button = event.target.closest('[data-action="drive-row-menu"]');
    if (button) openFor(button.closest('tr'));
  });
  root.addEventListener('contextmenu', (event) => {
    const row = event.target.closest('tr[data-kind]');
    const fromKeyboard = event.clientX === 0 && event.clientY === 0;
    const point = fromKeyboard ? null : { x: event.clientX, y: event.clientY };
    if (row && openFor(row, point)) event.preventDefault();
  });
  onRegistryChange(() => listing.invalidate());

  return { listing, render, forget };
};
