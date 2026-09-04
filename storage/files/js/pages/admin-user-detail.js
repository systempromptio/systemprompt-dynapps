import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';

const BASELINE_ROLES = ['user', 'project_manager', 'admin'];

const buildRoleCheckboxes = async () => {
  const group = document.getElementById('user-roles-group');
  if (!group) return;
  const held = (group.dataset.roles || '')
    .split(',')
    .map((r) => r.trim())
    .filter(Boolean);
  let known = [];
  try {
    const roles = await apiFetch('/users/roles');
    known = Array.isArray(roles) ? roles.map((r) => (typeof r === 'string' ? r : r.role)) : [];
  } catch {
    known = [];
  }
  const all = [...new Set([...BASELINE_ROLES, ...known.filter(Boolean), ...held])].sort();
  group.innerHTML = all
    .map((role) => {
      const checked = held.includes(role) ? ' checked' : '';
      return `<label class="checkbox-label"><input type="checkbox" name="roles" value="${role}"${checked}> ${role}</label>`;
    })
    .join('');
};

const selectedRoles = () =>
  Array.from(document.querySelectorAll('#user-roles-group input[name="roles"]:checked')).map(
    (cb) => cb.value,
  );

const form = document.getElementById('user-edit-form');
if (form) {
  const status = document.getElementById('user-edit-status');
  buildRoleCheckboxes();
  form.addEventListener('submit', async (event) => {
    event.preventDefault();
    const userId = form.dataset.userId;
    if (!userId) return;
    const data = new FormData(form);
    const roles = selectedRoles();
    const body = {
      display_name: (data.get('display_name') || '').toString(),
      email: (data.get('email') || '').toString(),
      roles,
      is_active: form.elements.namedItem('is_active').checked,
    };
    if (status) status.textContent = 'Saving…';
    try {
      await apiFetch('/users/' + encodeURIComponent(userId), {
        method: 'PUT',
        body: JSON.stringify(body),
      });
      if (status) status.textContent = 'Saved';
      showToast('User updated', 'success');
      setTimeout(() => window.location.reload(), 600);
    } catch (err) {
      const msg = err && err.message ? err.message : 'Failed to update user';
      if (status) status.textContent = '';
      showToast(msg, 'error');
    }
  });
}
