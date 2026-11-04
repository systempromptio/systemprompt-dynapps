// Drive pages: the document API reads and the row filling both pages share.
// The server renders each row's markup as a <template>; these helpers clone
// it and set text only, so no document name or user id ever reaches HTML.
import { rawFetch } from './api.js';

const dateFormat = new Intl.DateTimeFormat(undefined, {
  dateStyle: 'medium',
  timeStyle: 'short',
});

export const formatDate = (value) => (value ? dateFormat.format(new Date(value)) : '—');

export const shortId = (value) => (value ? value.slice(0, 12) : '—');

export const readJson = (url) => rawFetch(url);

export const cloneRow = (name, fields) => {
  const template = document.querySelector(`template[data-template="${name}"]`);
  const row = template.content.firstElementChild.cloneNode(true);
  for (const [key, value] of Object.entries(fields)) {
    const cell = row.querySelector(`[data-field="${key}"]`);
    if (cell) cell.textContent = value ?? '—';
  }
  return row;
};

export const editorsLabel = (editors, names = new Map()) => {
  if (!editors.length) return 'Nobody';
  const labels = editors.map(({ user_id: userId, state }) => `${names.get(userId) ?? userId} (${state})`);
  return labels.join(', ');
};

const PEOPLE_API = '/drive/people';
const people = new Map();

// Display names for user ids, read once per id from the Drive people route,
// an id the route cannot name keeps showing as itself.
export const loadPeople = async (ids) => {
  const wanted = [...new Set(ids)].filter((id) => id && !people.has(id));
  if (wanted.length) {
    const body = await readJson(`${PEOPLE_API}?ids=${encodeURIComponent(wanted.join(','))}`);
    for (const person of body?.people ?? []) {
      people.set(person.user_id, person.display_name ?? person.email ?? person.user_id);
    }
    for (const id of wanted) if (!people.has(id)) people.set(id, id);
  }
  return people;
};

export const loadEditors = async (presenceApi) => {
  const body = await readJson(presenceApi);
  return body?.editors ?? [];
};
