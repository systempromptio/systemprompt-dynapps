import { BASE, rawFetch, rawResponse } from '../services/api.js';
import { showToast } from '../services/toast.js';
import { showConfirmDialog } from '../services/confirm.js';

const ENDPOINT = `${BASE}/api/profile/provider-credentials`;

const providerUrl = (provider) => `${ENDPOINT}/${encodeURIComponent(provider)}`;

const REFUSALS = {
  personal_key_rejected: 'The provider refused this key. Copy a fresh key from your subscription console and paste it again.',
  personal_key_unverified: 'The provider could not be reached to check the key. Wait a moment and select Save and test again.',
  not_personal_provider: 'This provider no longer takes a personal key here. Reload the page to see the current list.',
};

const refusalText = async (resp) => {
  const text = await resp.text();
  let body = {};
  try {
    body = JSON.parse(text);
  } catch {
    body = { message: text };
  }
  const code = [body.code, body.error, body.error_code].find((c) => typeof c === 'string' && c in REFUSALS);
  if (code) return REFUSALS[code];
  return body.message || body.error || `The key could not be saved (HTTP ${resp.status}).`;
};

const when = (iso) => (iso ? new Date(iso).toLocaleString() : 'never');

const metaLine = (entry) => {
  if (!entry.configured) return 'No key added. Requests for these models are refused until you add one.';
  const label = entry.label ? `${entry.label} · ` : '';
  return `${label}added ${when(entry.created_at)} · verified ${when(entry.verified_at)} · last used ${when(entry.last_used_at)}`;
};

const fillRow = (row, entry) => {
  row.querySelector('[data-field="name"]').textContent = entry.display_name || entry.credential;
  const state = row.querySelector('[data-field="state"]');
  state.textContent = entry.configured ? 'key added' : 'no key';
  state.className = `sp-badge ${entry.configured ? 'sp-badge--ok' : 'sp-badge--muted'}`;
  row.querySelector('[data-field="meta"]').textContent = metaLine(entry);
  row.querySelector('[data-action="remove"]').hidden = !entry.configured;
};

const saveKey = async (root, row, entry, form) => {
  const button = form.querySelector('button[type="submit"]');
  const data = new FormData(form);
  const label = String(data.get('label') || '').trim();
  button.disabled = true;
  const reason = row.querySelector('[data-field="reason"]');
  reason.hidden = true;
  // The store tests the key against the provider before keeping it, so a
  // successful save is also a passing test; each refusal names its own fix.
  const resp = await rawResponse(providerUrl(entry.credential), {
    method: 'PUT',
    body: JSON.stringify({ api_key: String(data.get('api_key') || '').trim(), ...(label ? { label } : {}) }),
  }).catch(() => null);
  button.disabled = false;
  if (!resp || !resp.ok) {
    reason.textContent = resp ? await refusalText(resp) : 'The console could not reach the gateway. Check your connection and try again.';
    reason.hidden = false;
    return;
  }
  showToast(`${entry.display_name || entry.credential} key saved and verified`, 'success');
  form.reset();
  await load(root);
};

const removeKey = (root, entry) => {
  showConfirmDialog(
    'Remove key',
    `Remove your ${entry.display_name || entry.credential} key? Requests for its models are refused until you add one again.`,
    'Remove key',
    async () => {
      await rawFetch(providerUrl(entry.credential), { method: 'DELETE' });
      showToast('Key removed', 'success');
      await load(root);
    },
  );
};

const renderRow = (root, template, entry) => {
  const row = template.content.firstElementChild.cloneNode(true);
  fillRow(row, entry);
  const form = row.querySelector('[data-action="save"]');
  form.addEventListener('submit', (ev) => {
    ev.preventDefault();
    saveKey(root, row, entry, form);
  });
  row.querySelector('[data-action="remove"]').addEventListener('click', () => removeKey(root, entry));
  return row;
};

const load = async (root) => {
  const status = root.querySelector('[data-subscriptions-status]');
  const list = root.querySelector('[data-subscriptions-list]');
  const template = root.querySelector('template[data-subscription-row]');
  try {
    const entries = (await rawFetch(ENDPOINT)) || [];
    list.replaceChildren(...entries.map((entry) => renderRow(root, template, entry)));
    status.textContent = entries.length
      ? ''
      : 'This installation offers no personal subscriptions.';
    status.hidden = entries.length > 0;
  } catch {
    status.hidden = false;
    status.textContent = 'Could not load your subscriptions. Reload the page to try again.';
  }
};

export const initSubscriptions = () => {
  const root = document.querySelector('[data-subscriptions]');
  if (root) load(root);
};

initSubscriptions();
