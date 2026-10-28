// One drive document: the page view, the block editor, its metadata,
// revision history, grants and live editors, each read from the document API
// as the signed-in administrator, with user ids shown as people.
import { cloneRow, formatDate, loadEditors, readJson, shortId } from '../services/drive-rows.js';
import { createEditor } from '../services/drive-editor.js';
import { showPaper } from '../services/drive-paper.js';

const people = new Map();

const resolvePeople = async (root, ids) => {
  const missing = [...new Set(ids.filter((id) => id && !people.has(id)))];
  if (!missing.length) return;
  try {
    const body = await readJson(`${root.dataset.peopleApi}?ids=${missing.map(encodeURIComponent).join(',')}`);
    for (const p of body?.people ?? []) people.set(p.user_id, p.display_name || p.email || p.user_id);
  } catch {
    // Names are a convenience; rows fall back to the id.
  }
};

const person = (id) => (id ? people.get(id) ?? id : '—');

const setField = (root, name, value) => {
  const el = root.querySelector(`dl [data-field="${name}"]`);
  if (el) el.textContent = value ?? '—';
};

const showDocument = async (root) => {
  const { document: doc, head, folder_path: folderPath } = await readJson(root.dataset.driveDocument);
  setField(root, 'name', doc.name);
  setField(root, 'format', doc.format);
  setField(root, 'folder', folderPath ?? 'Top level');
  await resolvePeople(root, [doc.owner_user_id]);
  setField(root, 'owner', person(doc.owner_user_id));
  setField(root, 'head', shortId(head?.id));
  setField(root, 'updated', formatDate(doc.updated_at));
};

const showRevisions = async (root) => {
  const { revisions } = await readJson(root.dataset.revisionsApi);
  await resolvePeople(root, revisions.map((r) => r.author_user_id));
  root.querySelector('[data-slot="revision-rows"]').replaceChildren(
    ...revisions.map((r) => cloneRow('drive-revision-row', {
      id: shortId(r.id),
      author: person(r.author_user_id),
      actor: r.actor_kind,
      origin: r.origin_kind,
      message: r.message,
      created: formatDate(r.created_at),
    })),
  );
};

const showGrants = async (root) => {
  const { grants } = await readJson(root.dataset.grantsApi);
  await resolvePeople(root, grants.flatMap((g) => [g.granted_by, g.subject_kind === 'user' ? g.subject_value : null]));
  root.querySelector('[data-slot="grant-rows"]').replaceChildren(
    ...grants.map((g) => cloneRow('drive-grant-row', {
      subject: g.subject_kind === 'user' ? person(g.subject_value) : `${g.subject_kind}: ${g.subject_value}`,
      level: g.access_level,
      granted_by: person(g.granted_by),
      valid_until: g.valid_until ? formatDate(g.valid_until) : 'No expiry',
      created: formatDate(g.created_at),
    })),
  );
};

const showEditors = async (root) => {
  const editors = await loadEditors(root.dataset.presenceApi);
  await resolvePeople(root, editors.map((e) => e.user_id));
  setField(root, 'editors', editors.length
    ? editors.map((e) => `${person(e.user_id)} (${e.state})`).join(', ')
    : 'Nobody');
};

const showPage = (root) => showPaper(root.querySelector('[data-slot="drive-paper"]'), root.dataset.previewApi);

const setMode = (root, editing) => {
  root.querySelector('[data-slot="drive-view"]').hidden = editing;
  root.querySelector('[data-slot="drive-editor"]').hidden = !editing;
  root.querySelector('[data-action="drive-view"]').setAttribute('aria-pressed', String(!editing));
  root.querySelector('[data-action="drive-edit"]').setAttribute('aria-pressed', String(editing));
};

const wireModes = (root) => {
  const form = root.querySelector('[data-slot="drive-editor"]');
  const editor = createEditor({
    form,
    projectionApi: root.dataset.projectionApi,
    editsApi: root.dataset.editsApi,
    onSaved: () => Promise.allSettled([showPage(root), showDocument(root), showRevisions(root)]),
  });
  root.querySelector('[data-action="drive-view"]').addEventListener('click', () => setMode(root, false));
  root.querySelector('[data-action="drive-edit"]').addEventListener('click', async () => {
    setMode(root, true);
    await editor.load();
  });
  form.querySelector('[data-action="drive-discard"]').addEventListener('click', async () => {
    await editor.load();
    setMode(root, false);
  });
};

const setSlotMessage = (root, slot, message) => {
  const body = root.querySelector(`[data-slot="${slot}"]`);
  const row = document.createElement('tr');
  const cell = document.createElement('td');
  cell.colSpan = 5;
  cell.textContent = message;
  row.append(cell);
  body.replaceChildren(row);
};

const showPageFallback = (root) => {
  root.querySelector('[data-slot="drive-paper"]').srcdoc = '<p style="font:14px system-ui;padding:24px">This document could not be rendered.</p>';
};

const fallbacks = [
  showPageFallback,
  (root) => setField(root, 'name', 'Unavailable'),
  (root) => setSlotMessage(root, 'revision-rows', 'Revisions could not be loaded.'),
  (root) => setSlotMessage(root, 'grant-rows', 'Grants could not be loaded.'),
  (root) => setField(root, 'editors', 'Unavailable'),
];

export const init = async () => {
  const root = document.querySelector('[data-drive-document]');
  if (!root) return;
  wireModes(root);
  const results = await Promise.allSettled([
    showPage(root),
    showDocument(root),
    showRevisions(root),
    showGrants(root),
    showEditors(root),
  ]);
  results.forEach((result, index) => {
    if (result.status === 'rejected') fallbacks[index](root);
  });
};

init();
