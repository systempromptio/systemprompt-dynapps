import { rawFetch } from '../services/api.js';
import { showConfirmDialog } from '../services/confirm.js';
import { showToast } from '../services/toast.js';
import { on } from '../services/events.js';
import { dayAfter, validUntilFrom } from '../services/validity.js';

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
    // Why: the reload is deferred so the confirmation is actually seen. Calling
    // it straight after the toast tears the page down in the same frame, and
    // the only feedback a destructive action gives is gone before it is painted.
    window.setTimeout(() => window.location.reload(), 900);
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

// Why: the console posts to the same `/admin/devices/pats` the bridge's
// device-link flow uses, for the signed-in person. The secret is shown once
// in the dialog and the page is not reloaded until it is closed, so nothing
// paints over the only copy there will ever be.
const patDialog = () => document.querySelector('dialog[data-dialog="issue-pat"]');
const patField = (name) => document.querySelector(`[data-field="${name}"]`);

const openIssuePat = () => {
  const dialog = patDialog();
  if (!dialog) return;
  const until = patField('issue-pat-until');
  const maxDays = Number(dialog.dataset.maxPatDays) || 90;
  until.min = dayAfter(1);
  until.max = dayAfter(maxDays);
  patField('issue-pat-result').hidden = true;
  patField('issue-pat-secret').textContent = '';
  dialog.showModal();
};

const issuePat = async (button) => {
  const name = patField('issue-pat-name')?.value.trim();
  if (!name) {
    showToast('Give the token a name', 'error');
    return;
  }
  button.disabled = true;
  try {
    const issued = await rawFetch('/admin/devices/pats', {
      method: 'POST',
      body: JSON.stringify({ name, expires_at: validUntilFrom(patField('issue-pat-until')) }),
    });
    patField('issue-pat-secret').textContent = issued.secret;
    patField('issue-pat-result').hidden = false;
    showToast(`Issued ${issued.name}; expires ${new Date(issued.expires_at).toLocaleDateString()}.`, 'success');
  } catch (err) {
    showToast(err.message || 'Could not issue a token.', 'error');
  } finally {
    button.disabled = false;
  }
};

const closeIssuePat = () => {
  const issued = !patField('issue-pat-result')?.hidden;
  patDialog()?.close();
  if (issued) window.location.reload();
};

const setCertExpiry = async (button) => {
  const input = button.parentElement?.querySelector('[data-cert-expiry-input]');
  const id = button.dataset.certExpiryId;
  button.disabled = true;
  try {
    await rawFetch(`/api/public/admin/devices/certs/${encodeURIComponent(id)}/expiry`, {
      method: 'PUT',
      body: JSON.stringify({ valid_until: validUntilFrom(input) }),
    });
    showToast(`Expiry saved for ${button.dataset.certExpiryName}.`, 'success');
    window.setTimeout(() => window.location.reload(), 900);
  } catch (err) {
    button.disabled = false;
    showToast(err.message || 'Could not save the expiry.', 'error');
  }
};

on('click', '[data-action="issue-pat"]', openIssuePat);
on('click', '[data-action="confirm-issue-pat"]', (event, button) => issuePat(button));
on('click', '[data-action="close-issue-pat"]', closeIssuePat);
on('click', '[data-cert-expiry-id]', (event, button) => setCertExpiry(button));
