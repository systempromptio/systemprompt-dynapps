// The export dialog. The server renders the dialog (`components/export-dialog`)
// with every dataset's grouped columns and the page's filter query; this
// module owns its state: which dataset, window, format and columns are
// picked, whether a row selection or a breakdown bucket replaces the window,
// the URL they resolve to, and the live count fetched from `/preview` so the
// reader knows how many rows and cells a download will hold before asking.

import { on } from './events.js';
import { rawResponse, errorMessage } from './api.js';
import { selection, buildUrl } from './export-url.js';
import { selectedRowIds } from '../components/sp-table-select.js';

const PREVIEW_DEBOUNCE_MS = 250;
const STORAGE_PREFIX = 'sp-export:';

const dialogEl = () => document.querySelector('[data-export-dialog]');

const setHidden = (el, hidden) => {
  el.hidden = hidden;
  el.classList.toggle('is-hidden', hidden);
};

const remember = (dataset, value) => {
  try { localStorage.setItem(STORAGE_PREFIX + dataset, JSON.stringify(value)); } catch { /* storage unavailable */ }
};

const recall = (dataset) => {
  try { return JSON.parse(localStorage.getItem(STORAGE_PREFIX + dataset) ?? 'null'); } catch { return null; }
};

const datasetOption = (dialog) => dialog.querySelector('[data-export-dataset]').selectedOptions[0];

const columnInputs = (dialog, dataset) =>
  [...dialog.querySelectorAll(`[data-export-columns="${dataset}"] input`)];

// Only the picked dataset's window kind and column list are visible; a
// selection disables the window entirely because the ids decide the rows.
const showWindow = (dialog) => {
  const option = datasetOption(dialog);
  const kind = option.dataset.window;
  const selected = Boolean(dialog.dataset.exportIds);
  for (const block of dialog.querySelectorAll('[data-export-window]')) {
    setHidden(block, block.dataset.exportWindow !== kind);
    block.disabled = selected;
  }
  for (const block of dialog.querySelectorAll('[data-export-columns]')) {
    setHidden(block, block.dataset.exportColumns !== option.value);
  }
  for (const block of dialog.querySelectorAll('[data-export-window]')) {
    const checked = block.querySelector('input[type="radio"]:checked');
    const custom = block.querySelector('[data-export-custom]');
    if (custom) setHidden(custom, checked?.value !== 'custom');
  }
  const description = dialog.querySelector('[data-export-description]');
  if (description) description.textContent = option.dataset.description ?? '';
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

const updateColumnCount = (dialog, picked) => {
  const count = dialog.querySelector('[data-export-columns-count]');
  if (count) count.textContent = `${picked.columns.length} of ${columnInputs(dialog, picked.dataset).length}`;
};

let previewTimer = null;
let previewSeq = 0;

const refresh = (dialog) => {
  const picked = selection(dialog);
  updateColumnCount(dialog, picked);
  remember(picked.dataset, { format: picked.format, columns: picked.columns });
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
  for (const input of columnInputs(dialog, dataset)) input.checked = predicate(input);
  refresh(dialog);
};

const applyRemembered = (dialog) => {
  const dataset = datasetOption(dialog).value;
  const saved = recall(dataset);
  if (!saved) return;
  const format = dialog.querySelector(`input[name="format"][value="${saved.format}"]`);
  if (format) format.checked = true;
  if (Array.isArray(saved.columns) && saved.columns.length) {
    for (const input of columnInputs(dialog, dataset)) input.checked = saved.columns.includes(input.value);
  }
};

const filterColumns = (dialog, query) => {
  const needle = query.trim().toLowerCase();
  for (const group of dialog.querySelectorAll('[data-export-group]')) {
    let visible = 0;
    for (const column of group.querySelectorAll('[data-export-column]')) {
      const match = needle === '' || (column.dataset.label ?? '').toLowerCase().includes(needle);
      setHidden(column, !match);
      if (match) visible += 1;
    }
    setHidden(group, visible === 0);
  }
};

// The dialog opens on the window the page is showing, so "Export" with no
// further clicks is the table on screen.
const syncFromPage = (dialog) => {
  const params = new URLSearchParams(dialog.dataset.exportQuery);
  const check = (name, value) => {
    const input = dialog.querySelector(`input[name="${name}"][value="${value}"]`);
    if (input) input.checked = true;
  };
  const preset = params.get('preset');
  if (preset) check('live_preset', preset);
  if (params.get('from') && params.get('to')) {
    check('live_preset', 'custom');
    for (const edge of ['from', 'to']) {
      dialog.querySelector(`input[name="live_${edge}"]`).value = params.get(edge).slice(0, 16);
    }
  }
  const days = params.get('days');
  if (days) {
    check('retained_days', days);
    check('days', days);
  }
  if (params.get('start') || params.get('end')) check('retained_days', 'custom');
};

// A trigger may narrow what the dialog exports: `data-export-selected`
// names the dataset the ticked rows belong to, `data-export-group-label`
// names a breakdown bucket (its `data-export-query` carrying that bucket's
// filters), and either shows in the scope strip in place of the window.
const applyScope = (dialog, trigger) => {
  const scope = dialog.querySelector('[data-export-scope]');
  const select = dialog.querySelector('[data-export-dataset]');
  const selected = trigger?.dataset.exportSelected;
  const ids = selected ? selectedRowIds() : [];
  dialog.dataset.exportIds = ids.join(',');
  if (trigger?.dataset.exportQuery !== undefined) dialog.dataset.exportQuery = trigger.dataset.exportQuery;
  if (selected && [...select.options].some((o) => o.value === selected)) select.value = selected;
  const group = trigger?.dataset.exportGroupLabel;
  if (ids.length) {
    scope.textContent = `Exporting ${ids.length.toLocaleString()} selected ${trigger.dataset.exportNoun ?? 'rows'}`;
  } else if (group) {
    scope.textContent = `Exporting the bucket “${group}”`;
  }
  setHidden(scope, !ids.length && !group);
};

let opener = null;
let pageQuery = null;

const open = (trigger) => {
  const dialog = dialogEl();
  if (!dialog) return false;
  opener = trigger;
  pageQuery = dialog.dataset.exportQuery;
  applyScope(dialog, trigger);
  syncFromPage(dialog);
  applyRemembered(dialog);
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
  on('click', '[data-action="export-columns-none"]', () => setColumns(dialog, () => false));
  on('click', '[data-export-download]', () => setTimeout(close, 0));
  on('input', '[data-export-column-filter]', (e, input) => filterColumns(dialog, input.value));
  dialog.addEventListener('change', (e) => {
    if (e.target.matches('[data-export-dataset]')) applyRemembered(dialog);
    showWindow(dialog);
    refresh(dialog);
  });
  dialog.addEventListener('close', () => {
    if (pageQuery !== null) dialog.dataset.exportQuery = pageQuery;
    delete dialog.dataset.exportIds;
    opener?.focus();
  });
};
