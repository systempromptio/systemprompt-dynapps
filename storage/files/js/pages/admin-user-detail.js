import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';

const statusFor = (name) => document.querySelector(`[data-status="${name}"]`);

const setStatus = (name, text) => {
  const el = statusFor(name);
  if (el) el.textContent = text;
};

const reloadSoon = () => {
  window.setTimeout(() => window.location.reload(), 600);
};

const saveProfile = async (form) => {
  const { userId } = form.dataset;
  if (!userId) return;
  const data = new FormData(form);
  setStatus('user-profile', 'Saving…');
  try {
    await apiFetch(`/users/${encodeURIComponent(userId)}`, {
      method: 'PUT',
      body: JSON.stringify({
        display_name: String(data.get('display_name') ?? ''),
        email: String(data.get('email') ?? ''),
        is_active: form.elements.namedItem('is_active').checked,
      }),
    });
    setStatus('user-profile', 'Saved');
    showToast('User updated', 'is-success');
    reloadSoon();
  } catch (err) {
    setStatus('user-profile', '');
    showToast(err?.message ?? 'Failed to update the user', 'error');
  }
};

const saveRoles = async (form) => {
  const { userId } = form.dataset;
  if (!userId) return;
  const roles = Array.from(
    form.querySelectorAll('input[name="roles"]:checked'),
    (cb) => cb.value,
  );
  setStatus('user-roles', 'Saving…');
  try {
    await apiFetch(`/users/${encodeURIComponent(userId)}/roles`, {
      method: 'PUT',
      body: JSON.stringify({ roles }),
    });
    setStatus('user-roles', 'Saved');
    showToast('Roles updated', 'is-success');
    reloadSoon();
  } catch (err) {
    setStatus('user-roles', '');
    showToast(err?.message ?? 'Failed to update the roles', 'error');
  }
};

const membershipPath = (kind, id, userId, adding) => {
  const collection = kind === 'group' ? 'groups' : 'projects';
  const base = `/${collection}/${encodeURIComponent(id)}/members`;
  return adding ? base : `${base}/${encodeURIComponent(userId)}`;
};

const toggleMembership = async (card, input) => {
  const { userId } = card.dataset;
  const kind = input.dataset.membershipKind;
  const id = input.dataset.membershipId;
  const adding = input.checked;
  setStatus('membership', 'Saving…');
  try {
    await apiFetch(membershipPath(kind, id, userId, adding), {
      method: adding ? 'POST' : 'DELETE',
      body: adding ? JSON.stringify({ user_id: userId }) : undefined,
    });
    setStatus('membership', 'Saved');
    reloadSoon();
  } catch (err) {
    input.checked = !adding;
    setStatus('membership', '');
    showToast(err?.message ?? 'Failed to change the membership', 'error');
  }
};

const bindForms = () => {
  for (const form of document.querySelectorAll('form[data-form]')) {
    form.addEventListener('submit', (ev) => {
      ev.preventDefault();
      if (form.dataset.form === 'user-roles') {
        saveRoles(form);
        return;
      }
      saveProfile(form);
    });
  }
};

const bindMembership = () => {
  const card = document.querySelector('[data-membership]');
  if (!card) return;
  card.addEventListener('change', (ev) => {
    const input = ev.target.closest('input[data-membership-kind]');
    if (!input) return;
    toggleMembership(card, input);
  });
};

const init = () => {
  bindForms();
  bindMembership();
};

init();
