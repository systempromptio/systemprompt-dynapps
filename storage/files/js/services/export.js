// The export dialog. The server renders the dialog (`components/export-dialog`)
// with every dataset's columns and the page's filter query; this module owns
// its state: which dataset, window, format and columns are picked, the URL
// they resolve to, and the live count fetched from `/preview` so the reader
// knows how many rows and cells a download will hold before asking for it.

import { on } from './events.js';
import { rawResponse, errorMessage } from './api.js';
import { selection, buildUrl } from './export-url.js';

const PREVIEW_DEBOUNCE_MS = 250;

const dialogEl = () => document.querySelector('[data-export-dialog]');

const showWindow = (dialog) => {
  const option = dialog.querySelector('[data-export-dataset]').selectedOptions[0];
  const kind = option.dataset.window;
  for (const block of dialog.querySelectorAll('[data-export-window]')) {
    block.hidden = block.dataset.exportWindow !== kind;
  }
  for (const block of dialog.querySelectorAll('[data-export-columns]')) {
    block.hidden = block.dataset.exportColumns !== option.value;
  }
  for (const block of dialog.querySelectorAll('[data-export-window]')) {
    const checked = block.querySelector('input[type="radio"]:checked');
    const custom = block.querySelector('[data-export-custom]');
    if (custom) custom.hidden = checked?.value !== 'custom';
  }
};

const renderPreview = (el, preview, columns) => {
  el.replaceChildren();
  const rows = document.createElement('strong');
  rows.textContent = preview.rows.toLocaleString();
  const cells = document.createElement('strong');
  cells.textContent = (preview.rows * columns).toLocaleString();
  el.append(rows, ` rows × ${columns} columns = `, cells, ' cells');
  if (preview.capped) {
    el.dataset.tone = 'warn';
    el.append(` — capped at ${preview.cap.toLocaleString()} rows; narrow the window for the rest.`);
  } else {
    delete el.dataset.tone;
  }
};

let previewTimer = null;
let previewSeq = 0;

const refresh = (dialog) => {
  const picked = selection(dialog);
  dialog.querySelector('[data-export-download]').href = buildUrl(dialog, picked, false);
  const el = dialog.querySelector('[data-export-preview]');
  el.textContent = 'Counting…';
  delete el.dataset.tone;
  clearTimeout(previewTimer);
  const seq = ++previewSeq;
  previewTimer = setTimeout(async () => {
    try {
      const resp = await rawResponse(buildUrl(dialog, picked, true));
      if (seq !== previewSeq) return;
      if (!resp.ok) throw new Error(await errorMessage(resp));
      renderPreview(el, await resp.json(), picked.columns.length);
    } catch (err) {
      if (seq !== previewSeq) return;
      el.dataset.tone = 'error';
      el.textContent = err.message || 'Could not count rows';
    }
  }, PREVIEW_DEBOUNCE_MS);
};

const setColumns = (dialog, predicate) => {
  const { dataset } = selection(dialog);
  for (const input of dialog.querySelectorAll(`[data-export-columns="${dataset}"] input`)) {
    input.checked = predicate(input);
  }
  refresh(dialog);
};

let opener = null;

const open = (trigger) => {
  const dialog = dialogEl();
  if (!dialog) return false;
  opener = trigger;
  showWindow(dialog);
  dialog.showModal();
  refresh(dialog);
  return true;
};

const close = () => {
  const dialog = dialogEl();
  if (dialog?.open) dialog.close();
};

export const initExport = () => {
  const dialog = dialogEl();
  if (!dialog) return;
  on('click', '[data-export-open]', (e, trigger) => {
    if (open(trigger)) e.preventDefault();
  });
  on('click', '[data-action="export-close"]', close);
  on('click', '[data-action="export-columns-all"]', () => setColumns(dialog, () => true));
  on('click', '[data-action="export-columns-default"]', () =>
    setColumns(dialog, (input) => input.dataset.default === 'true'));
  on('click', '[data-export-download]', () => setTimeout(close, 0));
  dialog.addEventListener('change', () => {
    showWindow(dialog);
    refresh(dialog);
  });
  dialog.addEventListener('close', () => opener?.focus());
};
