import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';
import { on, initDelegation } from '../services/events.js';

// The role editor sends the whole set it wants, so both actions read the
// person's current roles first and post the union or the difference. Sending
// only the changed role would clear every other role they hold.
const rolesOf = (userId) => apiFetch('/users/' + encodeURIComponent(userId) + '/roles');

const setRoles = (userId, roles) =>
  apiFetch('/users/' + encodeURIComponent(userId) + '/roles', {
    method: 'PUT',
    body: JSON.stringify({ roles }),
  });

const dialog = (name) => document.querySelector('dialog[data-dialog="' + name + '"]');
const field = (name) => document.querySelector('[data-roles="' + name + '"]');

const closeDialogs = () => {
  document.querySelectorAll('dialog[open]').forEach((d) => d.close());
};

const showError = (el, message) => {
  if (!el) return;
  el.textContent = message;
  el.hidden = !message;
};

let chosen = null;

const renderResults = (users) => {
  const list = field('grant-results');
  if (!list) return;
  list.textContent = '';
  users.slice(0, 8).forEach((user) => {
    const item = document.createElement('li');
    const button = document.createElement('button');
    button.type = 'button';
    button.className = 'sp-btn sp-btn--sm sp-btn--outline';
    button.textContent = (user.display_name || user.email || user.user_id) + ' — ' + (user.email || '');
    button.addEventListener('click', () => {
      chosen = user.user_id || user.id;
      const note = field('toast');
      if (note) {
        note.textContent = 'Selected ' + button.textContent;
        note.hidden = false;
      }
    });
    item.append(button);
    list.append(item);
  });
};

const search = async (term) => {
  if (!term) {
    renderResults([]);
    return;
  }
  try {
    const found = await apiFetch('/users/search?q=' + encodeURIComponent(term));
    renderResults(Array.isArray(found) ? found : found?.users || []);
  } catch {
    renderResults([]);
  }
};

const grant = async () => {
  const role = field('grant-role')?.value;
  if (!chosen || !role) {
    showError(field('grant-error'), 'Pick a person and a role first.');
    return;
  }
  try {
    const current = await rolesOf(chosen);
    const next = Array.from(new Set([...(current?.roles || []), role]));
    await setRoles(chosen, next);
    showToast('Granted ' + role, 'success');
    window.location.reload();
  } catch (err) {
    showError(field('grant-error'), err.message || 'Could not grant that role.');
  }
};

let pending = null;

const askRevoke = (el) => {
  pending = { userId: el.dataset.userId, role: el.dataset.role };
  const summary = field('revoke-summary');
  if (summary) {
    summary.textContent =
      'Remove the ' + pending.role + ' role from ' + (el.dataset.person || pending.userId) + '?';
  }
  showError(field('revoke-error'), '');
  dialog('revoke')?.showModal();
};

const revoke = async () => {
  if (!pending) return;
  try {
    const current = await rolesOf(pending.userId);
    const next = (current?.roles || []).filter((r) => r !== pending.role);
    await setRoles(pending.userId, next);
    showToast('Revoked ' + pending.role, 'success');
    window.location.reload();
  } catch (err) {
    showError(field('revoke-error'), err.message || 'Could not revoke that role.');
  }
};

export const init = () => {
  initDelegation();
  on('click', '[data-action="grant-role"]', () => {
    chosen = null;
    showError(field('grant-error'), '');
    dialog('grant')?.showModal();
  });
  on('click', '[data-action="grant-confirm"]', () => grant());
  on('click', '[data-action="revoke-role"]', (e, el) => askRevoke(el));
  on('click', '[data-action="revoke-confirm"]', () => revoke());
  on('click', '[data-action="dialog-close"]', () => closeDialogs());
  field('grant-search')?.addEventListener('input', (e) => search(e.target.value.trim()));
};

init();
