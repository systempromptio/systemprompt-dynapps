import { watchPortfolio } from '../services/portfolio-live.js';

const root = document.querySelector('[data-portfolio]');
if (root) {
  const set = (name, text) => { const node = root.querySelector(`[data-${name}]`); if (node) node.textContent = text; };
  const cell = (row, value) => { const td = document.createElement('td'); td.textContent = value; row.append(td); return td; };
  const render = data => {
    set('freshness', `Snapshot generation ${data.health.generation}; facts ${data.health.fact_generation}/${data.health.facts_generation}; generated ${data.generated_at}. Pending facts ${data.health.pending_changes}; producer changes ${data.health.pending_producer_changes}.`);
    set('inventory-health', `Inventory generation ${data.inventory.generation}; ${data.inventory.entries} observed entries. ${data.inventory.last_error || ''}`);
    set('installation-health', `Installation coverage generation ${data.installations.generation}; observed ${data.installations.observed_at || 'not yet'}.`);
    set('unknown-usage', data.unknown_usage === null ? 'Usage attribution is not yet available.' : `${data.unknown_usage} invocations have unknown resource attribution.`);
    const tbody = root.querySelector('tbody'); tbody.replaceChildren();
    for (const item of data.rows) {
      const row = document.createElement('tr');
      const name = cell(row, item.entry.resource_key);
      if (item.href) { const link = document.createElement('a'); link.href = item.href; link.textContent = item.entry.resource_key; name.replaceChildren(link); }
      const detail = document.createElement('small'); detail.textContent = ` ${item.entry.origin} · ${item.entry.availability}${item.entry.diagnostic ? ' · ' + item.entry.diagnostic : ''}`; name.append(document.createElement('br'), detail);
      const metrics = item.snapshot?.metrics;
      cell(row, metrics ? `${metrics.invocations} / ${item.users}` : 'Usage unavailable');
      cell(row, metrics ? `${metrics.verified_invocations} / ${metrics.invocations}` : 'Unknown');
      const installs = item.installation;
      cell(row, installs ? `${installs.current_acknowledged_devices}/${installs.eligible_devices} eligible devices acknowledged; ${installs.current_verified_devices} verified; ${installs.unverifiable_installations} installations unverified` : 'Coverage pending or resource unbound');
      cell(row, metrics ? `${metrics.priced_requests}/${metrics.requests} priced; ${item.cost || 'cost unknown'}; ${metrics.failed_requests} failed` : 'Unknown');
      cell(row, metrics ? `${item.p95 ? '≤ ' + item.p95 + ' ms' : 'Unknown'}; ${metrics.latency_measured_requests}/${metrics.requests} measured` : 'Unknown');
      cell(row, metrics ? `${metrics.assessed_conversations}/${metrics.assessment_conversations} scored; ${metrics.failed_assessments} failed` : 'Unknown');
      cell(row, item.snapshot ? `${item.snapshot.suppressed_days} suppressed days` : item.usage_status);
      tbody.append(row);
    }
    if (!data.rows.length) { const row = document.createElement('tr'); cell(row, 'No observed inventory entries match this page.').colSpan = 8; tbody.append(row); }
    const next = root.querySelector('[data-next]'); next.hidden = !data.next_href; next.href = data.next_href || '#';
  };
  const dispose = watchPortfolio({ url: '/admin/analysis/resources.json' + location.search, generation: root.dataset.generation, onData: render, onStatus: text => set('live-status', text) });
  const stop = () => { dispose(); observer.disconnect(); window.removeEventListener('pagehide', stop); };
  const observer = new MutationObserver(() => { if (!root.isConnected) stop(); });
  observer.observe(document.body, { childList: true, subtree: true });
  window.addEventListener('pagehide', stop, { once: true });
}
