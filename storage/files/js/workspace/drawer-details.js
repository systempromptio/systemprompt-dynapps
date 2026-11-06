// The Details tab: format, owner, folder, head revision, dates, the viewer's
// access and why they hold it, and the document class picker.
import { formatDate, readJson, shortId } from '../services/drive-rows.js';
import { showClass } from './drawer-class.js';
import { person, resolvePeople } from './people.js';
import { registerDrawerTab } from './registry.js';

const setText = (root, field, value) => {
  const el = root.querySelector(`[data-field="${field}"]`);
  if (el) el.textContent = value ?? '—';
};

export const showDetails = async (root, view) => {
  const { document: doc, head, folder_path: folderPath, my_access: access } = view;
  await resolvePeople(root.dataset.peopleApi, [doc.owner_user_id]);
  setText(root, 'd-format', doc.format);
  setText(root, 'd-owner', person(doc.owner_user_id));
  setText(root, 'd-folder', folderPath ?? 'Top level');
  setText(root, 'd-head', shortId(head?.id));
  setText(root, 'd-created', formatDate(doc.created_at));
  setText(root, 'd-updated', formatDate(doc.updated_at));
  setText(root, 'd-access', access);
  try {
    const explained = await readJson(root.dataset.accessApi);
    const direct = explained.grants?.length ?? 0;
    const inherited = explained.inherited?.length ?? 0;
    setText(root, 'd-access-why', `${direct} grant(s) on this document and ${inherited} inherited from its folders.`);
  } catch {
    setText(root, 'd-access-why', '');
  }
  try {
    await showClass(root, doc.id);
  } catch (err) {
    setText(root, 'd-agents', err.message);
  }
};

registerDrawerTab({
  id: 'details',
  label: 'Details',
  order: 10,
  live: ['document.shared', 'document.ownership_transferred'],
  load: (pane, ctx) => ctx.refresh(),
});
