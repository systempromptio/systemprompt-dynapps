import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';
import {
  cloneTemplate,
  groupList,
  isWritable,
  setText,
  showPane,
  slot,
} from './admin-access-control-state.js';
import { buildMatrixSection } from './admin-access-control-rows.js';

const buildGroupOptions = (container, held) => {
  for (const group of groupList()) {
    const option = cloneTemplate('matrix-group-option');
    const input = option.querySelector('input');
    input.value = group.id;
    input.checked = held.includes(group.id);
    input.disabled = !isWritable();
    option.querySelector('[data-slot="label"]').textContent = group.name;
    container.append(option);
  }
};

const saveOverride = (userId, btn) => {
  const { entityType, entityId } = btn.closest('.sp-ac-matrix-row').dataset;
  if (btn.dataset.override === 'inherit') {
    return apiFetch('/access-control/bulk-template', {
      method: 'POST',
      body: JSON.stringify({
        entity_type: entityType,
        subject_type: 'user',
        subject_value: userId,
        action: 'clear',
      }),
    });
  }
  const path = `/access-control/entity/${encodeURIComponent(entityType)}/${encodeURIComponent(entityId)}/rules`;
  return apiFetch(path, {
    method: 'POST',
    body: JSON.stringify({ rule_type: 'user', rule_value: userId, access: btn.dataset.override }),
  });
};

const saveMembership = (userId, input) =>
  input.checked
    ? apiFetch(`/groups/${encodeURIComponent(input.sp-value)}/members`, {
        method: 'POST',
        body: JSON.stringify({ user_id: userId }),
      })
    : apiFetch(
        `/groups/${encodeURIComponent(input.sp-value)}/members/${encodeURIComponent(userId)}`,
        { method: 'DELETE' },
      );

const bindMatrix = (matrix, userId, displayName) => {
  matrix.addEventListener('click', async (ev) => {
    const btn = ev.target.closest('.sp-ac-override__btn');
    if (!btn) return;
    try {
      await saveOverride(userId, btn);
      renderUserMatrix(userId, displayName);
    } catch {
      showToast('Could not save the override', 'error');
    }
  });
  matrix.addEventListener('change', async (ev) => {
    const input = ev.target.closest('input[data-action="assign-group"]');
    if (!input) return;
    try {
      await saveMembership(userId, input);
      renderUserMatrix(userId, displayName);
    } catch {
      input.checked = !input.checked;
      showToast('Could not change the group membership', 'error');
    }
  });
};

export const renderUserMatrix = async (userId, displayName) => {
  const matrix = document.querySelector('[data-pane="matrix"]');
  const loading = cloneTemplate('matrix-loading');
  setText(loading, 'name', displayName);
  matrix.replaceChildren(loading);
  showPane('matrix');

  let data;
  try {
    data = await apiFetch(`/access-control/users/${encodeURIComponent(userId)}/matrix`);
  } catch {
    matrix.replaceChildren(cloneTemplate('matrix-error'));
    return;
  }

  const content = cloneTemplate('matrix');
  const { user } = data;
  setText(content, 'name', user.display_name || user.email || user.id);
  setText(content, 'email', user.email ?? '');
  setText(content, 'roles', `roles: ${(user.roles ?? []).join(', ') || 'none'}`);
  buildGroupOptions(slot(content, 'groups'), user.group_ids ?? []);
  const sections = slot(content, 'sections');
  for (const section of data.sections ?? []) sections.append(buildMatrixSection(section));
  matrix.replaceChildren(content);
  bindMatrix(matrix, userId, displayName);
};
