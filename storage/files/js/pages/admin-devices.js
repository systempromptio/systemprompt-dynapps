import { rawFetch } from '../services/api.js';
import { showConfirmDialog } from '../services/confirm.js';
import { showToast } from '../services/toast.js';
import { on } from '../services/events.js';

const LABELS = { pats: 'access token', certs: 'device certificate' };

const revoke = async (button) => {
  const kind = button.dataset.revokeKind;
  const id = button.dataset.revokeId;
  button.disabled = true;
  try {
    await rawFetch(`/api/public/admin/devices/${kind}/${encodeURIComponent(id)}`, {
      method: 'DELETE',
    });
    showToast(`Revoked ${button.dataset.revokeName}.`, 'success');
    window.location.reload();
  } catch (err) {
    button.disabled = false;
    showToast(err.message || 'Could not revoke that credential.', 'error');
  }
};

on('click', '[data-revoke-id]', (event, button) => {
  const noun = LABELS[button.dataset.revokeKind] || 'credential';
  showConfirmDialog(
    `Revoke this ${noun}?`,
    `${button.dataset.revokeName} stops working immediately and the machine holding it must enrol again.`,
    'Revoke',
    () => revoke(button),
  );
});
