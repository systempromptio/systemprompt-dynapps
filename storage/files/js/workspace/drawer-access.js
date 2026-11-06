// Who has access to a document, as core explains it: the owner, each direct
// grant with its expiry (an expired grant stays, greyed, until the sweep
// removes it), each grant inherited from a folder, the viewer's own level,
// and, on request, the history of every grant change.
import { person } from './people.js';
import { button, el, heading, relativeTime } from './pane-dom.js';

const LEVEL_LABELS = { view: 'Can view', comment: 'Can comment', edit: 'Can edit', share: 'Can share', own: 'Owner' };

const levelText = (level) => LEVEL_LABELS[level] ?? level;

const subjectText = (kind, value, name) => (kind === 'user' ? name || person(value) : `${kind}: ${name || value}`);

const expiry = (grant) => {
  if (!grant.valid_until) return '';
  return ` · ${grant.expired ? 'expired' : 'until'} ${relativeTime(grant.valid_until)}`;
};

const row = (who, what, expired) => {
  const item = el('li', 'sp-ws__entry sp-ws__entry--row sp-ws-grant');
  if (expired) item.dataset.expired = 'true';
  item.append(el('span', 'sp-ws__entry-title', who), el('span', 'sp-ws__muted', what));
  return item;
};

export const accessUserIds = (access) => [
  access.owner,
  ...(access.grants ?? []).filter((g) => g.subject_kind === 'user').map((g) => g.subject_value),
  ...(access.inherited ?? []).filter((g) => g.subject_kind === 'user').map((g) => g.subject_value),
  ...(access.requests ?? []).map((r) => r.requester_user_id),
  ...(access.history ?? []).map((h) => h.actor_user_id),
];

export const whoHasAccess = (access, { canRevoke, onRevoke }) => {
  const list = el('ul', 'sp-ws__list');
  list.append(row(person(access.owner), 'Owner'));
  for (const grant of access.grants ?? []) {
    const item = row(subjectText(grant.subject_kind, grant.subject_value, grant.display_name), `${levelText(grant.level)}${expiry(grant)}`, grant.expired);
    if (canRevoke) item.append(button('Remove', 'sp-btn--ghost', () => onRevoke(grant.grant_id)));
    list.append(item);
  }
  for (const grant of access.inherited ?? []) {
    const from = grant.inherited_from?.name ?? grant.folder_id ?? 'a folder';
    list.append(row(subjectText(grant.subject_kind, grant.subject_value, grant.display_name), `${levelText(grant.level)} · from folder ${from}${expiry(grant)}`, grant.expired));
  }
  const box = el('section', 'sp-ws__pane');
  box.append(heading('Who has access'), list, el('p', 'sp-ws__muted', `You hold ${levelText(access.my_access)}.`));
  return box;
};

const historyRow = (event) => {
  const levels = [event.old_level, event.new_level].filter(Boolean).map(levelText).join(' → ');
  const actor = event.actor_user_id ? ` by ${person(event.actor_user_id)}` : '';
  return row(`${event.event} ${subjectText(event.subject_kind, event.subject_value)}`, `${levels}${actor} · ${relativeTime(event.at)}`);
};

export const accessHistory = (access, { isOpen, onToggle }) => {
  const box = el('section', 'sp-ws__pane');
  const toggle = button(isOpen ? 'Hide access history' : 'Access history', 'sp-btn--ghost', onToggle);
  toggle.setAttribute('aria-expanded', String(isOpen));
  box.append(toggle);
  if (!isOpen) return box;
  const list = el('ul', 'sp-ws__list');
  list.setAttribute('aria-label', 'Access history');
  const events = access.history ?? [];
  list.append(...(events.length ? events.map(historyRow) : [el('li', 'sp-ws__muted', 'No changes recorded.')]));
  box.append(list);
  return box;
};
