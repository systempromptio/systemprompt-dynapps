import { watchPortfolio } from '../services/portfolio-live.js';

// The server renders the table; live updates only refresh the status line and
// flag a newer retained generation, so there is exactly one renderer for the
// rows and an active window never reloads under the reader.
const root = document.querySelector('[data-portfolio]');
if (root) {
  const set = (name, text) => { const node = root.querySelector(`[data-${name}]`); if (node) node.textContent = text; };
  const since = iso => {
    const minutes = Math.round((Date.now() - Date.parse(iso)) / 60000);
    if (!Number.isFinite(minutes)) return 'never synced';
    if (minutes < 1) return 'synced just now';
    if (minutes < 60) return `synced ${minutes}m ago`;
    if (minutes < 1440) return `synced ${Math.round(minutes / 60)}h ago`;
    return `synced ${Math.round(minutes / 1440)}d ago`;
  };
  // The meta line reads the same as the server rendered it: scope, size,
  // audience, when the inventory last synced. Generation counters stay in
  // the watch key and never reach the reader; a version is its hash.
  const scope = data => data.all_view
    ? `${data.marketplace_count} marketplaces`
    : (data.selected_marketplace_name || 'No marketplace');
  const render = data => {
    const generation = `${data.health.generation}.${data.inventory.generation}.${data.installations.generation}`;
    const newer = generation !== root.dataset.generation && !data.pending_jobs;
    set('live-status', data.pending_jobs
      ? 'Preparing the requested range…'
      : newer
        ? 'Newer snapshot available — reload to refresh.'
        : `${scope(data)} · ${data.kpis.skills} skills across ${data.plugin_count} plugins · ${data.audience} people entitled · ${data.synced_at ? since(data.synced_at) : 'never synced'}${data.snapshot_state === 'pending' ? ' · snapshot pending' : ''}`);
    set('freshness', `${data.generated_at} · pending facts ${data.health.pending_changes} · producer changes ${data.health.pending_producer_changes} ${data.health.last_error || ''} ${data.inventory.last_error || ''}`);
    set('unknown-usage', data.unknown_usage === null ? 'Unknown resource attribution coverage is unavailable.' : `${data.unknown_usage} invocations have unknown resource attribution.`);
  };
  const dispose = watchPortfolio({ url: '/admin/analysis/skills.json' + location.search, generation: root.dataset.generation, onData: render, onStatus: text => { if (text !== 'Live snapshots') set('live-status', text); } });
  const stop = () => { dispose(); observer.disconnect(); window.removeEventListener('pagehide', stop); };
  const observer = new MutationObserver(() => { if (!root.isConnected) stop(); });
  observer.observe(document.body, { childList: true, subtree: true });
  window.addEventListener('pagehide', stop, { once: true });
}
