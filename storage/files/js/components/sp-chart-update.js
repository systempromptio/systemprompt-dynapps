// Pushing a fresh server figure into a chart that is already live. The live
// handle owns hidden-series flags, the table toggle, keyboard focus and a
// ResizeObserver, all of which a node swap would throw away, so the new
// figure is read for its data only and the existing model is redrawn.

import { readModel } from './sp-chart.js';
import { buildTable } from './sp-chart-tooltip.js';

const SERVER_PARTS = '.sp-timeseries__plot, .sp-timeseries__x-axis, .sp-svgchart__legend, .sp-svgchart__refs, .sp-empty';

const copyText = (from, to, selector) => {
  const source = from.querySelector(selector);
  const target = to.querySelector(selector);
  if (source && target && target.textContent !== source.textContent) target.textContent = source.textContent;
};

// The server's own plot stays in the figure, hidden, so a future re-init reads
// current values rather than the ones the page first loaded with.
const replaceServerParts = (figure, next) => {
  for (const stale of figure.querySelectorAll(SERVER_PARTS)) stale.remove();
  const head = figure.querySelector('.sp-timeseries__head');
  const anchor = head?.nextSibling ?? null;
  for (const part of next.querySelectorAll(SERVER_PARTS)) {
    part.hidden = true;
    figure.insertBefore(part, anchor);
  }
  figure.dataset.labels = next.dataset.labels ?? '[]';
  figure.dataset.unit = next.dataset.unit ?? '';
  figure.dataset.kind = next.dataset.kind ?? 'line';
};

const refreshLegend = (handle) => {
  if (!handle.legend) return;
  const items = handle.legend.querySelectorAll('.sp-chart__legend-item');
  handle.model.series.forEach((s, i) => {
    const value = items[i]?.querySelector('.sp-chart__legend-value');
    if (value) value.textContent = s.total;
  });
};

// Returns false when the figure has no live handle (it was empty at load and
// never initialised), so the caller swaps the node and initialises it instead.
export const updateChart = (handle, next) => {
  if (!handle) return false;
  const fresh = readModel(next);
  if (!fresh) return false;
  const { model } = handle;
  const hiddenByLabel = new Map(model.series.map((s) => [s.label, s.hidden]));
  for (const s of fresh.series) s.hidden = hiddenByLabel.get(s.label) ?? false;
  model.series = fresh.series;
  model.labels = fresh.labels;
  model.unit = fresh.unit;
  model.kind = fresh.kind;
  const figure = handle.host.closest('[data-chart]');
  if (figure) {
    copyText(next, figure, '.sp-timeseries__title');
    copyText(next, figure, '.sp-timeseries__subtitle');
    replaceServerParts(figure, next);
  }
  refreshLegend(handle);
  handle.setTable(buildTable(model));
  if (!handle.host.hidden) handle.draw();
  return true;
};
