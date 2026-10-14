// The person's own Odoo link on the connectors page: link (Odoo verifies the
// login and API key server-side first) and unlink. The key is read once,
// sent, and the field cleared; every mutation reloads to show the answer.
import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';
import { showConfirmDialog } from '../services/confirm.js';
import { on, initDelegation } from '../services/events.js';

const field = (name) => document.querySelector(`[data-field="${name}"]`);

const link = async () => {
  const login = field('odoo-login')?.value.trim() ?? '';
  const key = field('odoo-api-key');
  if (!login || !key?.value) {
    showToast('Enter your Odoo login and an API key', 'error');
    return;
  }
  const body = JSON.stringify({ login, api_key: key.value });
  key.value = '';
  try {
    await apiFetch('/user/odoo-link', { method: 'POST', body });
    showToast('Odoo account linked', 'success');
    window.location.reload();
  } catch (err) {
    showToast(err.message || 'Odoo did not accept that login and key', 'error');
  }
};

const unlink = () => {
  showConfirmDialog(
    'Unlink your Odoo account?',
    'Odoo tools stop acting as you until you link again.',
    'Unlink',
    async () => {
      try {
        await apiFetch('/user/odoo-link', { method: 'DELETE' });
        window.location.reload();
      } catch (err) {
        showToast(err.message || 'Failed to unlink Odoo', 'error');
      }
    },
  );
};

export const init = () => {
  if (!document.querySelector('[data-odoo-link]')) return;
  initDelegation();
  on('click', '[data-action="link-odoo"]', link);
  on('click', '[data-action="unlink-odoo"]', unlink);
};

init();
