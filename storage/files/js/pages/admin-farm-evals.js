// The evals pages. The evals overview upgrades its trend figures with
// sp-chart. The eval run page's case rows are polled by admin-farm-live-list,
// which swaps the rows region wholesale; the swapped-in detail rows carry the
// server's no-JS `hidden`, so they are handed back to the expand behaviour
// here, the same way table-expand adopts them on first load.

import { initAllCharts } from '../components/sp-chart.js';

const adoptDetailRows = (scope) => {
  for (const row of scope.querySelectorAll('[data-expand-row]')) {
    const btn = row.querySelector('[data-action="row-toggle"]');
    const detail = document.getElementById(btn?.getAttribute('aria-controls') || '');
    if (!detail) continue;
    detail.hidden = false;
    detail.classList.toggle('is-open', btn.getAttribute('aria-expanded') === 'true');
  }
};

const watchLiveRows = () => {
  for (const region of document.querySelectorAll('[data-live-region="rows"]')) {
    const observer = new MutationObserver(() => adoptDetailRows(region));
    observer.observe(region, { childList: true });
  }
};

export const initFarmEvals = () => {
  initAllCharts(document);
  watchLiveRows();
};

initFarmEvals();
