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

export const editorsLabel = (editors) => {
  if (!editors.length) return 'Nobody';
  const names = editors.map(({ user_id: userId, state }) => `${userId} (${state})`);
  return names.join(', ');
};

export const loadEditors = async (presenceApi) => {
  const body = await readJson(presenceApi);
  return body?.editors ?? [];
};
