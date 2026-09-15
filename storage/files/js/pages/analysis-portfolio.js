import { watchPortfolio } from '../services/portfolio-live.js';

// The server renders the grouped table; live updates only refresh the status
// line and flag a newer retained generation, so there is exactly one renderer
// for the rows and an active window never reloads under the reader.
const root = document.querySelector('[data-portfolio]');
if (root) {
  const set = (name, text) => { const node = root.querySelector(`[data-${name}]`); if (node) node.textContent = text; };
  const render = data => {
    const generation = `${data.health.generation}.${data.inventory.generation}.${data.installations.generation}`;
    const newer = generation !== root.dataset.generation && !data.pending_jobs;
    set('live-status', data.pending_jobs
      ? 'Preparing the requested range…'
      : newer
        ? `Newer retained snapshot available (generation ${data.health.generation}); reload to refresh retained metrics.`
        : `Retained snapshots. Retained snapshot generation ${data.health.generation} (facts ${data.health.fact_generation}/${data.health.facts_generation}), inventory generation ${data.inventory.generation} with ${data.inventory.entries} entries, coverage generation ${data.installations.generation}.`);
    set('freshness', `${data.generated_at} · pending facts ${data.health.pending_changes} · producer changes ${data.health.pending_producer_changes} ${data.health.last_error || ''} ${data.inventory.last_error || ''}`);
    set('unknown-usage', data.unknown_usage === null ? 'Unknown resource attribution coverage is unavailable.' : `${data.unknown_usage} invocations have unknown resource attribution.`);
  };
  const dispose = watchPortfolio({ url: '/admin/analysis/resources.json' + location.search, generation: root.dataset.generation, onData: render, onStatus: text => { if (text !== 'Live snapshots') set('live-status', text); } });
  const stop = () => { dispose(); observer.disconnect(); window.removeEventListener('pagehide', stop); };
  const observer = new MutationObserver(() => { if (!root.isConnected) stop(); });
  observer.observe(document.body, { childList: true, subtree: true });
  window.addEventListener('pagehide', stop, { once: true });
}
