// One drive document: its metadata, revision history, grants and live
// editors, each read from the document API as the signed-in administrator.
import { cloneRow, editorsLabel, formatDate, loadEditors, readJson } from '../services/drive-rows.js';

const setField = (root, name, value) => {
  const el = root.querySelector(`dl [data-field="${name}"]`);
  if (el) el.textContent = value ?? '—';
};

const showDocument = async (root) => {
  const { document: doc, head, folder_path: folderPath } = await readJson(root.dataset.driveDocument);
  setField(root, 'name', doc.name);
  setField(root, 'format', doc.format);
  setField(root, 'folder', folderPath ?? 'Top level');
  setField(root, 'owner', doc.owner_user_id);
  setField(root, 'head', head?.id);
  setField(root, 'updated', formatDate(doc.updated_at));
};

const showRevisions = async (root) => {
  const { revisions } = await readJson(root.dataset.revisionsApi);
  root.querySelector('[data-slot="revision-rows"]').append(
    ...revisions.map((r) => cloneRow('drive-revision-row', {
      id: r.id,
      author: r.author_user_id,
      actor: r.actor_kind,
      origin: r.origin_kind,
      message: r.message,
      created: formatDate(r.created_at),
    })),
  );
};

const showGrants = async (root) => {
  const { grants } = await readJson(root.dataset.grantsApi);
  root.querySelector('[data-slot="grant-rows"]').append(
    ...grants.map((g) => cloneRow('drive-grant-row', {
      subject: `${g.subject_kind}: ${g.subject_value}`,
      level: g.access_level,
      granted_by: g.granted_by,
      valid_until: g.valid_until ? formatDate(g.valid_until) : 'No expiry',
      created: formatDate(g.created_at),
    })),
  );
};

const showEditors = async (root) => {
  setField(root, 'editors', editorsLabel(await loadEditors(root.dataset.presenceApi)));
};

export const init = async () => {
  const root = document.querySelector('[data-drive-document]');
  if (!root) return;
  const results = await Promise.allSettled([
    showDocument(root),
    showRevisions(root),
    showGrants(root),
    showEditors(root),
  ]);
  for (const result of results) {
    if (result.status === 'rejected') console.error(result.reason);
  }
};

init();
