// Drive listing: one folder's subfolders and documents, read from the
// document API page by page, with each document's live editors filled in
// after its row is on screen; "New document" creates one from a template.
import { cloneRow, editorsLabel, formatDate, loadEditors, readJson, shortId } from '../services/drive-rows.js';
import { on, initDelegation } from '../services/events.js';
import { rawFetch } from '../services/api.js';

const enc = encodeURIComponent;

const state = { root: null, cursor: null };

const folderRow = (folder, baseUrl) => {
  const row = cloneRow('drive-folder-row', { updated: formatDate(folder.updated_at) });
  const link = row.querySelector('[data-field="link"]');
  link.textContent = folder.name;
  link.href = `${baseUrl}?folder=${enc(folder.id)}`;
  return row;
};

const fillEditors = async (row, documentsApi, id) => {
  const cell = row.querySelector('[data-field="editors"]');
  try {
    cell.textContent = editorsLabel(await loadEditors(`${documentsApi}/${enc(id)}/presence`));
  } catch {
    cell.textContent = 'Unavailable';
  }
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

const pageUrl = (listApi, cursor) => {
  if (!cursor) return listApi;
  const joiner = listApi.includes('?') ? '&' : '?';
  return `${listApi}${joiner}cursor=${enc(cursor)}`;
};

const loadPage = async () => {
  const { root } = state;
  const { driveList, documentsApi, baseUrl } = root.dataset;
  const rows = root.querySelector('[data-slot="drive-rows"]');
  const more = root.querySelector('[data-action="drive-more"]');
  const listing = await readJson(pageUrl(driveList, state.cursor));
  root.querySelector('[data-slot="drive-loading"]')?.remove();
  rows.append(
    ...(state.cursor ? [] : listing.folders.map((f) => folderRow(f, baseUrl))),
    ...listing.documents.map((d) => documentRow(d, { baseUrl, documentsApi })),
  );
  state.cursor = listing.next_cursor;
  more.hidden = !state.cursor;
  root.querySelector('[data-slot="drive-empty"]').hidden = rows.children.length > 0;
};

const loadSafely = async () => {
  try {
    await loadPage();
  } catch {
    const loading = state.root.querySelector('[data-slot="drive-loading"] td');
    if (loading) loading.textContent = 'The document list could not be loaded.';
  }
};

const createDocument = async (event) => {
  event.preventDefault();
  const form = event.currentTarget;
  const { driveCreate, baseUrl, folderId } = form.dataset;
  const name = form.elements.name.value.trim();
  if (!name) return;
  const button = form.querySelector('button[type="submit"]');
  button.disabled = true;
  try {
    const created = await rawFetch(driveCreate, {
      method: 'POST',
      body: JSON.stringify({
        template_id: form.elements.template_id.value,
        name,
        folder_id: folderId ?? null,
        params: { values: { title: name } },
      }),
    });
    window.location.assign(`${baseUrl}/${enc(created.document.id)}`);
  } catch {
    button.disabled = false;
  }
};

export const init = () => {
  document.querySelector('[data-drive-create]')?.addEventListener('submit', createDocument);
  state.root = document.querySelector('[data-drive-list]');
  if (!state.root) return;
  initDelegation();
  on('click', '[data-action="drive-more"]', loadSafely);
  loadSafely();
};

init();
