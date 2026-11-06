// The people picker: a directory search listbox bound to a text input as an
// ARIA combobox. The caller decides what the search term is (the whole field
// in a share form, the word after `@` in a comment), so the picker only
// searches, shows matches and reports the one picked. With `kind` (a function
// naming the current subject kind) it searches that kind instead of people,
// and reports through `onSearchable` when the kind cannot be listed.
import { initialsOf, searchPeople, searchSubjectsOfKind, toneOf } from './people.js';

const DEBOUNCE_MS = 150;
const ACCESS_LABELS = { view: 'Can view', comment: 'Can comment', edit: 'Can edit', share: 'Can share', own: 'Owner' };
let serial = 0;

const node = (tag, className, text) => {
  const el = document.createElement(tag);
  el.className = className;
  if (text != null) el.textContent = text;
  return el;
};

const optionFor = (p, id, withAccess) => {
  const name = p.display_name || p.email || p.user_id;
  const li = node('li', 'sp-people-picker__option');
  li.id = id;
  li.setAttribute('role', 'option');
  li.setAttribute('aria-selected', 'false');
  const avatar = node('span', `sp-avatar sp-avatar--sm sp-avatar--tone-${toneOf(p.user_id)}`, initialsOf(name));
  avatar.setAttribute('aria-hidden', 'true');
  const text = node('span', 'sp-people-picker__text');
  text.append(node('span', 'sp-people-picker__name', name), node('span', 'sp-people-picker__email', p.email ?? p.user_id));
  li.append(avatar, text);
  if (!withAccess) return li;
  const badge = p.access_level
    ? node('span', 'sp-badge sp-badge--sm sp-badge--ok', ACCESS_LABELS[p.access_level] ?? p.access_level)
    : node('span', 'sp-badge sp-badge--sm sp-badge--muted', 'No access');
  li.append(badge);
  return li;
};

const findMatches = async ({ subjectsApi, documentId, kind, onSearchable }, term) => {
  const current = kind?.() ?? 'user';
  if (current === 'user') {
    onSearchable?.(true);
    return { people: await searchPeople(subjectsApi, term, documentId), withAccess: Boolean(documentId) };
  }
  const { searchable, matches } = await searchSubjectsOfKind(subjectsApi, current, term);
  onSearchable?.(searchable);
  return { people: searchable ? matches : null, withAccess: false };
};

export const createPeoplePicker = (options) => {
  const { input, onPick } = options;
  const listId = `sp-people-picker-${(serial += 1)}`;
  const list = node('ul', 'sp-people-picker');
  list.id = listId;
  list.setAttribute('role', 'listbox');
  list.hidden = true;
  input.after(list);
  input.setAttribute('role', 'combobox');
  input.setAttribute('aria-autocomplete', 'list');
  input.setAttribute('aria-controls', listId);
  input.setAttribute('aria-expanded', 'false');
  let matches = [];
  let active = -1;
  let timer = 0;
  let asked = '';

  const mark = (index) => {
    active = index;
    list.querySelectorAll('[role="option"]').forEach((li, i) => li.setAttribute('aria-selected', String(i === index)));
    const current = list.children[index];
    if (current?.id) input.setAttribute('aria-activedescendant', current.id);
    else input.removeAttribute('aria-activedescendant');
    current?.scrollIntoView({ block: 'nearest' });
  };

  const close = () => {
    clearTimeout(timer);
    asked = '';
    list.hidden = true;
    input.setAttribute('aria-expanded', 'false');
    input.removeAttribute('aria-activedescendant');
  };

  const show = (term, { people, withAccess }) => {
    if (term !== asked) return;
    if (!people) {
      close();
      return;
    }
    matches = people;
    list.replaceChildren(...(people.length
      ? people.map((p, i) => optionFor(p, `${listId}-${i}`, withAccess))
      : [node('li', 'sp-people-picker__empty', 'No matches')]));
    list.hidden = false;
    input.setAttribute('aria-expanded', 'true');
    mark(people.length ? 0 : -1);
  };

  const search = (term) => {
    clearTimeout(timer);
    asked = term;
    if (!term) {
      close();
      return;
    }
    timer = setTimeout(async () => {
      try {
        show(term, await findMatches(options, term));
      } catch {
        close();
      }
    }, DEBOUNCE_MS);
  };

  const pick = (index) => {
    const chosen = matches[index];
    if (!chosen) return false;
    close();
    onPick(chosen);
    return true;
  };

  const handleKey = (evt) => {
    if (list.hidden) return false;
    const count = matches.length;
    if (evt.key === 'ArrowDown' && count) mark((active + 1) % count);
    else if (evt.key === 'ArrowUp' && count) mark((active - 1 + count) % count);
    else if (evt.key === 'Enter' || evt.key === 'Tab') {
      if (!pick(active)) return false;
    } else if (evt.key === 'Escape') close();
    else return false;
    evt.preventDefault();
    evt.stopPropagation();
    return true;
  };

  list.addEventListener('mousedown', (evt) => evt.preventDefault());
  list.addEventListener('click', (evt) => {
    const li = evt.target.closest('[role="option"]');
    if (li) pick([...list.children].indexOf(li));
  });
  input.addEventListener('blur', close);

  return { search, close, handleKey, isOpen: () => !list.hidden };
};
