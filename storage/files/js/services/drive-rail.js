// The document page's side rail and header people: details, the revision
// feed and who is here now, with every user id shown as a name. Sharing,
// comments and proposals are the workspace's own panes, mounted in the rail.
import { cloneRow, formatDate, readJson } from './drive-rows.js';

const people = new Map();

export const resolvePeople = async (peopleApi, ids) => {
  const missing = [...new Set(ids.filter((id) => id && !people.has(id)))];
  if (!missing.length) return;
  try {
    const body = await readJson(`${peopleApi}?ids=${missing.map(encodeURIComponent).join(',')}`);
    for (const p of body?.people ?? []) people.set(p.user_id, p.display_name || p.email || p.user_id);
  } catch {
    missing.forEach((id) => people.set(id, id));
  }
};

export const personName = (id) => (id ? people.get(id) ?? id : '—');

export const initials = (name) => name
  .split(/[\s@.]+/)
  .filter(Boolean)
  .slice(0, 2)
  .map((part) => part[0].toUpperCase())
  .join('');

const relative = (value) => {
  const seconds = Math.round((Date.now() - new Date(value).getTime()) / 1000);
  if (seconds < 60) return 'just now';
  if (seconds < 3600) return `${Math.floor(seconds / 60)} min ago`;
  if (seconds < 86400) return `${Math.floor(seconds / 3600)} h ago`;
  return formatDate(value);
};

export const describeRevision = (r) => {
  if (r.message) return r.message;
  return r.parent_revision_id ? 'Edited' : 'Created';
};

export const showDetails = (root, doc, folderPath) => {
  const set = (name, value) => {
    const el = root.querySelector(`.sp-p-doc__facts [data-field="${name}"]`);
    if (el) el.textContent = value ?? '—';
  };
  set('owner', personName(doc.owner_user_id));
  set('folder', folderPath ?? 'Drive');
  set('created', formatDate(doc.created_at));
  set('updated', formatDate(doc.updated_at));
  set('template', doc.template_id ?? 'Uploaded');
};

export const showActivity = (list, revisions, onView) => {
  list.replaceChildren(...revisions.map((r, index) => {
    const who = personName(r.author_user_id);
    const row = cloneRow('doc-activity-item', {
      initials: initials(who),
      who: r.actor_kind === 'agent' ? `${who} via agent` : who,
      what: describeRevision(r),
      when: relative(r.created_at),
    });
    const view = row.querySelector('[data-action="doc-revision"]');
    if (index === 0) view.textContent = 'Current';
    view.disabled = index === 0;
    view.addEventListener('click', () => onView(r));
    return row;
  }));
};

export const showEditors = (box, editors) => {
  box.replaceChildren(...editors.map((e) => {
    const name = personName(e.user_id);
    const chip = document.createElement('span');
    chip.className = `sp-p-doc__avatar sp-p-doc__avatar--${e.state}`;
    chip.textContent = initials(name);
    chip.title = `${name} is ${e.state}`;
    return chip;
  }));
};
