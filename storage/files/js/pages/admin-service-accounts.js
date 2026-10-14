// Service accounts: create and delete an account, attach, rotate and remove
// its machine credentials. The pages are server-rendered; these are the
// mutations only, and every one reloads to show the server's answer.
// A secret is read from its field once, sent, and the field is cleared.
import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';
import { showConfirmDialog } from '../services/confirm.js';
import { on, initDelegation } from '../services/events.js';

const dialog = (name) => document.querySelector(`dialog[data-dialog="${name}"]`);
const field = (name) => document.querySelector(`[data-field="${name}"]`);
const value = (name) => field(name)?.value.trim() ?? '';
const enc = encodeURIComponent;

const createAccount = async () => {
  const name = value('new-sa-name');
  const email = value('new-sa-email');
  if (!name || !email) {
    showToast('A service account needs a name and an email', 'error');
    return;
  }
  try {
    const created = await apiFetch('/service-accounts', {
      method: 'POST',
      body: JSON.stringify({ name, email }),
    });
    window.location.assign(`/admin/identity/service-accounts/${enc(created.user_id)}`);
  } catch (err) {
    showToast(err.message || 'Failed to create the service account', 'error');
  }
};

const showKindFields = () => {
  const kind = value('kind');
  document.querySelectorAll('[data-kind-fields]').forEach((group) => {
    group.hidden = group.dataset.kindFields !== kind;
  });
};

const attachCredential = async (form) => {
  const provider = value('provider');
  const secret = field('secret');
  if (!provider || !secret?.value) {
    showToast('Name the provider and paste the secret', 'error');
    return;
  }
  const body = { kind: value('kind'), secret: secret.value };
  for (const name of ['app_id', 'installation_id', 'instance_url', 'consumer_key', 'email', 'url', 'db', 'login']) {
    body[name] = value(name);
  }
  secret.value = '';
  try {
    await apiFetch(`/service-accounts/${enc(form.dataset.userId)}/credentials/${enc(provider)}`, {
      method: 'POST',
      body: JSON.stringify(body),
    });
    showToast('Credential saved', 'success');
    window.location.reload();
  } catch (err) {
    showToast(err.message || 'Failed to save the credential', 'error');
  }
};

const removeCredential = (button) => {
  const { provider, userId } = button.dataset;
  showConfirmDialog(
    `Remove the ${provider} credential?`,
    'Workflow write-backs that act as this account stop reaching the provider until a new credential is attached.',
    'Remove',
    async () => {
      try {
        await apiFetch(`/service-accounts/${enc(userId)}/credentials/${enc(provider)}`, { method: 'DELETE' });
        window.location.reload();
      } catch (err) {
        showToast(err.message || 'Failed to remove the credential', 'error');
      }
    },
  );
};

const deleteAccount = (button) => {
  showConfirmDialog(
    'Delete this service account?',
    'Definitions that run as it will fail their write-backs until they name another account.',
    'Delete',
    async () => {
      try {
        await apiFetch(`/service-accounts/${enc(button.dataset.userId)}`, { method: 'DELETE' });
        window.location.assign('/admin/identity/service-accounts');
      } catch (err) {
        showToast(err.message || 'Failed to delete the service account', 'error');
      }
    },
  );
};

export const init = () => {
  initDelegation();
  on('click', '[data-action="create-service-account"]', () => dialog('create-service-account')?.showModal());
  on('click', '[data-action="close-create-service-account"]', () => dialog('create-service-account')?.close());
  on('click', '[data-action="confirm-create-service-account"]', () => {
    dialog('create-service-account')?.close();
    createAccount();
  });
  on('change', '[data-field="kind"]', showKindFields);
  on('click', '[data-action="attach-credential"]', (_event, el) => attachCredential(el.closest('form')));
  on('click', '[data-action="rotate-credential"]', (_event, el) => {
    field('provider').value = el.dataset.provider;
    field('secret')?.focus();
  });
  on('click', '[data-action="remove-credential"]', (_event, el) => removeCredential(el));
  on('click', '[data-action="delete-service-account"]', (_event, el) => deleteAccount(el));
};

init();
