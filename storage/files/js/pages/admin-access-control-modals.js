import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';
import { groupList } from './admin-access-control-state.js';

const dialog = (name) => document.querySelector(`dialog[data-dialog="${name}"]`);

const openDialog = (name) => {
  const el = dialog(name);
  if (el) el.showModal();
};

const closeDialogs = () => {
  for (const el of document.querySelectorAll('dialog[data-dialog]')) el.close();
};

const field = (name) => document.querySelector(`[data-ac="${name}"]`);

const showGroupError = (message) => {
  const err = field('new-group-error');
  if (!err) return;
  err.textContent = message;
  err.hidden = !message;
};

const showYaml = async () => {
  openDialog('yaml');
  const target = field('yaml-content');
  target.textContent = 'Loading…';
  try {
    const yaml = await apiFetch('/access-control/yaml-snapshot');
    target.textContent = yaml || 'No band rules are stored in the database yet.';
  } catch {
    target.textContent = 'Failed to load the YAML snapshot.';
  }
};

const copyYaml = async (button) => {
  try {
    await navigator.clipboard.writeText(field('yaml-content').textContent);
    button.textContent = 'Copied';
  } catch {
    showToast('Copy failed — select the text manually', 'error');
  }
};

const saveGroup = async () => {
  const id = field('new-group-id').value.trim();
  const name = field('new-group-name').value.trim();
  const description = field('new-group-desc').value.trim();
  showGroupError('');
  if (!id || !name) {
    showGroupError('An identifier and a name are both required.');
    return;
  }
  if (groupList().some((g) => g.id === id)) {
    showGroupError('A group with that identifier already exists.');
    return;
  }
  try {
    await apiFetch('/groups', {
      method: 'POST',
      body: JSON.stringify({ id, name, description }),
    });
    window.location.reload();
  } catch (err) {
    showGroupError(err?.message ?? 'Failed to create the group');
  }
};

const ACTIONS = {
  'show-yaml': showYaml,
  'new-group': () => openDialog('new-group'),
  'close-dialog': closeDialogs,
  'copy-yaml': (target) => copyYaml(target),
  'save-group': saveGroup,
};

const bindRoot = (root) => {
  root.addEventListener('click', (ev) => {
    const target = ev.target.closest('[data-action]');
    const handler = target && ACTIONS[target.dataset.action];
    if (!handler) return;
    handler(target);
  });
};

export const bindDialogs = () => {
  const toolbar = document.querySelector('.sp-toolbar');
  if (toolbar) bindRoot(toolbar);
  for (const el of document.querySelectorAll('dialog[data-dialog]')) bindRoot(el);
};
