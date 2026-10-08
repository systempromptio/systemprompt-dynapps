// The overview as a live instrument. The server renders the page complete, and
// this module fetches the same body again — on a range click without a page
// load, and every half minute while the tab is visible — and morphs it over
// what is on screen: tiles in place, charts through their live handles,
// boards wholesale. Without scripts the range links are ordinary navigation.

import { chartOf, initAllCharts, initChart } from '../components/sp-chart.js';
import { updateChart } from '../components/sp-chart-update.js';
import { createPoller } from '../services/live-poll.js';
import { createLiveStatus } from '../services/live-status.js';
import { initScope } from '../services/scope.js';
import { fetchFragment } from './admin-overview-fetch.js';
import { morphKpis, swapRegion, veil } from './admin-overview-morph.js';

const root = document.querySelector('[data-overview-live]');
const ranges = document.querySelector('[data-overview-ranges]');

const region = (scope, name) => scope.querySelector(`[data-live-region="${name}"]`);

const applyCharts = (current, next) => {
  for (const slot of next.querySelectorAll('[data-chart-slot]')) {
    const target = current.querySelector(`[data-chart-slot="${slot.dataset.chartSlot}"]`);
    const figure = target?.querySelector('[data-chart]');
    const fresh = slot.querySelector('[data-chart]');
    if (!target || !fresh) continue;
    if (figure && updateChart(chartOf(figure), fresh)) continue;
    const clone = fresh.cloneNode(true);
    if (figure) figure.replaceWith(clone);
    else target.append(clone);
    initChart(clone);
  }
};

const applyHeader = (next) => {
  const subtitle = document.querySelector('.sp-page-header__subtitle');
  if (subtitle && next.dataset.subtitle) subtitle.textContent = next.dataset.subtitle;
  for (const link of ranges?.querySelectorAll('[data-overview-range]') ?? []) {
    const active = link.dataset.overviewRange === next.dataset.preset;
    link.classList.toggle('sp-time-range__btn--active', active);
    if (active) link.setAttribute('aria-current', 'page');
    else link.removeAttribute('aria-current');
  }
};

const applyFragment = (next) => {
  morphKpis(region(root, 'kpis'), region(next, 'kpis'));
  applyCharts(region(root, 'charts'), region(next, 'charts'));
  swapRegion(region(root, 'scopes'), region(next, 'scopes'));
  swapRegion(region(root, 'models'), region(next, 'models'));
  root.dataset.preset = next.dataset.preset;
  root.dataset.subtitle = next.dataset.subtitle;
  applyHeader(next);
};

const refresh = async (signal) => applyFragment(await fetchFragment(root.dataset.preset, signal));

const start = () => {
  let status = null;
  const poller = createPoller({
    run: refresh,
    onState: (state, detail) => status?.setState(state, detail),
  });
  status = createLiveStatus(document.querySelector('[data-live-status]'), {
    onToggle: (pausing) => (pausing ? poller.pause() : poller.resume()),
  });

  const switchRange = async (preset, push) => {
    if (push) window.history.pushState({ preset }, '', `/admin?preset=${preset}`);
    ranges?.setAttribute('aria-busy', 'true');
    veil(root, true);
    root.dataset.preset = preset;
    try {
      await refresh();
      status?.setState('fresh');
      initScope();
    } catch (error) {
      window.location.assign(`/admin?preset=${preset}`);
      throw error;
    } finally {
      veil(root, false);
      ranges?.removeAttribute('aria-busy');
    }
  };

  ranges?.addEventListener('click', (event) => {
    const link = event.target.closest('[data-overview-range]');
    if (!link || event.metaKey || event.ctrlKey || event.shiftKey || event.button !== 0) return;
    event.preventDefault();
    if (link.dataset.overviewRange !== root.dataset.preset) switchRange(link.dataset.overviewRange, true);
  });

  window.addEventListener('popstate', () => {
    const preset = new URLSearchParams(window.location.search).get('preset') || '24h';
    if (preset !== root.dataset.preset) switchRange(preset, false);
  });

  poller.start();
};

if (root) {
  initAllCharts(root);
  start();
}
