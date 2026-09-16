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
  const render = data => {
    const generation = `${data.health.generation}.${data.inventory.generation}.${data.installations.generation}`;
    const newer = generation !== root.dataset.generation && !data.pending_jobs;
    set('live-status', data.pending_jobs
      ? 'Preparing the requested range…'
      : newer
        ? `Newer snapshot available (generation ${data.health.generation}) — reload to refresh.`
        : `Inventory generation ${data.inventory.generation} · ${data.synced_at ? since(data.synced_at) : 'never synced'} · snapshot ${data.snapshot_state}`);
    set('freshness', `${data.generated_at} · pending facts ${data.health.pending_changes} · producer changes ${data.health.pending_producer_changes} ${data.health.last_error || ''} ${data.inventory.last_error || ''}`);
    set('unknown-usage', data.unknown_usage === null ? 'Unknown resource attribution coverage is unavailable.' : `${data.unknown_usage} invocations have unknown resource attribution.`);
  };
  const dispose = watchPortfolio({ url: '/admin/analysis/resources.json' + location.search, generation: root.dataset.generation, onData: render, onStatus: text => { if (text !== 'Live snapshots') set('live-status', text); } });
  const stop = () => { dispose(); observer.disconnect(); window.removeEventListener('pagehide', stop); };
  const observer = new MutationObserver(() => { if (!root.isConnected) stop(); });
  observer.observe(document.body, { childList: true, subtree: true });
  window.addEventListener('pagehide', stop, { once: true });
}
