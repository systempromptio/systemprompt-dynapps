import { BASE, rawFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';
import { showConfirmDialog } from '../services/confirm.js';

const ENDPOINT = `${BASE}/api/profile/provider-credentials`;

const providerUrl = (provider) => `${ENDPOINT}/${encodeURIComponent(provider)}`;

const when = (iso) => (iso ? new Date(iso).toLocaleString() : 'never');

const metaLine = (entry) => {
  if (!entry.configured) return 'No key added. Requests for these models are refused until you add one.';
  const label = entry.label ? `${entry.label} · ` : '';
  return `${label}added ${when(entry.created_at)} · verified ${when(entry.verified_at)} · last used ${when(entry.last_used_at)}`;
};

const fillRow = (row, entry) => {
  row.querySelector('[data-field="name"]').textContent = entry.display_name || entry.provider;
  const state = row.querySelector('[data-field="state"]');
  state.textContent = entry.configured ? 'key added' : 'no key';
  state.className = `sp-badge ${entry.configured ? 'sp-badge--ok' : 'sp-badge--muted'}`;
  row.querySelector('[data-field="meta"]').textContent = metaLine(entry);
  row.querySelector('[data-action="remove"]').hidden = !entry.configured;
};

const saveKey = async (root, entry, form) => {
  const button = form.querySelector('button[type="submit"]');
  const data = new FormData(form);
  const label = String(data.get('label') || '').trim();
  button.disabled = true;
  try {
    // The store tests the key against the provider before keeping it, so a
    // successful save is also a passing test; a refused key answers 422.
    await rawFetch(providerUrl(entry.provider), {
      method: 'PUT',
      body: JSON.stringify({ api_key: String(data.get('api_key') || '').trim(), ...(label ? { label } : {}) }),
    });
    showToast(`${entry.display_name || entry.provider} key saved and verified`, 'success');
    form.reset();
    await load(root);
  } catch {
    button.disabled = false;
  }
};

const removeKey = (root, entry) => {
  showConfirmDialog(
    'Remove key',
    `Remove your ${entry.display_name || entry.provider} key? Requests for its models are refused until you add one again.`,
    'Remove key',
    async () => {
      await rawFetch(providerUrl(entry.provider), { method: 'DELETE' });
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
    saveKey(root, entry, form);
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
