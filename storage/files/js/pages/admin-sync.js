// The Code ↔ Instance page's actions. Everything is server-rendered; this
// file confirms a write in words that name what will move, posts it, and
// reloads so the tables are re-derived by the server rather than patched here.

import { apiFetch, rawResponse, API_BASE, errorMessage } from '../services/api.js';
import { showToast } from '../services/toast.js';

const dialog = (name) => document.querySelector(`dialog[data-dialog="${name}"]`);
const field = (name) => document.querySelector(`[data-sync="${name}"]`);

const closeDialogs = () => {
  for (const el of document.querySelectorAll('dialog[data-dialog]')) el.close();
};

let pending = null;

const CONFIRM = {
  insert_only: (button) => ({
    title: 'Insert only',
    body: `Add ${button.dataset.count} row(s) the file declares and this database lacks. Nothing existing is changed or removed.`,
  }),
  overwrite: (button) => ({
    title: 'Overwrite from code',
    body:
      `Make the database equal the file for every entity the file names. ` +
      `${button.dataset.count} row(s) only this database holds will be deleted — ` +
      `${button.dataset.dashboard} of them were written from the console. ` +
      `Per-person overrides and entities the file does not mention are untouched.`,
  }),
};

const askSync = (button) => {
  const words = CONFIRM[button.dataset.mode]?.(button);
  if (!words) return;
  pending = { plane: button.dataset.plane, mode: button.dataset.mode };
  field('confirm-title').textContent = words.title;
  field('confirm-body').textContent = words.body;
  dialog('confirm')?.showModal();
};

const runSync = async () => {
  if (!pending) return;
  const { plane, mode } = pending;
  pending = null;
  closeDialogs();
  try {
    const result = await apiFetch(`/sync/planes/${plane}/apply`, {
      method: 'POST',
      body: JSON.stringify({ mode }),
    });
    const o = result?.outcome ?? {};
    showToast(`Synced: +${o.inserted ?? 0} ~${o.updated ?? 0} −${o.deleted ?? 0}`, 'success');
    window.location.reload();
  } catch (err) {
    showToast(err?.message ?? 'Sync failed', 'error');
  }
};

// Why: the export is the plane's file (YAML), not JSON, so it is read as
// text through the raw response rather than the JSON helper.
const showExport = async (button) => {
  const url = `${API_BASE}/sync/planes/${button.dataset.plane}/export`;
  const link = field('export-download');
  link.href = url;
  link.setAttribute('download', button.dataset.file?.split('/').pop() ?? 'export.yaml');
  field('export-title').textContent = `This database as ${button.dataset.file}`;
  dialog('export')?.showModal();
  const target = field('export-content');
  target.textContent = 'Loading…';
  try {
    const resp = await rawResponse(url);
    if (!resp.ok) throw new Error(await errorMessage(resp));
    target.textContent = await resp.text();
  } catch (err) {
    target.textContent = err?.message ?? 'Failed to render the export.';
  }
};

const copyExport = async (button) => {
  try {
    await navigator.clipboard.writeText(field('export-content').textContent);
    button.textContent = 'Copied';
  } catch {
    showToast('Copy failed — select the text manually', 'error');
  }
};

const askRefresh = () => {
  const out = field('refresh-result');
  out.hidden = true;
  out.textContent = '';
  dialog('refresh')?.showModal();
};

// Why: the reply is shown verbatim — `reconciled` says the new composition is
// live, and `restart_recommended` names the one case (a kit shipping hooks)
// an in-process import cannot fully serve.
const runRefresh = async (button) => {
  button.disabled = true;
  const out = field('refresh-result');
  out.hidden = false;
  out.textContent = 'Refreshing…';
  try {
    const result = await apiFetch('/sync/sources/refresh', { method: 'POST' });
    out.textContent = JSON.stringify(result, null, 2);
    let summary = 'Composition unchanged';
    if (result?.changed) {
      summary = result?.restart_recommended
        ? 'Imported and reconciled — a restart would also load the kit hooks'
        : 'Imported and reconciled in place';
    }
    showToast(summary, 'success');
    if (!result?.restarting) window.setTimeout(() => window.location.reload(), 1200);
  } catch (err) {
    out.textContent = err?.message ?? 'Refresh failed';
    showToast(err?.message ?? 'Refresh failed', 'error');
  } finally {
    button.disabled = false;
  }
};

const ACTIONS = {
  sync: askSync,
  'confirm-sync': runSync,
  export: showExport,
  'copy-export': copyExport,
  'refresh-sources': askRefresh,
  'confirm-refresh': runRefresh,
  'close-dialog': closeDialogs,
};

const init = () => {
  document.addEventListener('click', (ev) => {
    const target = ev.target.closest('[data-action]');
    const handler = target && ACTIONS[target.dataset.action];
    if (handler) handler(target);
  });
};

init();
