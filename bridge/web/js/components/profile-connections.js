// Provider credentials never enter the webview. The identity enrichment carries
// only the public, server-owned connection contract used by the Account page.
import { bridge } from '/assets/js/bridge.js';
import { escapeHtml } from '/assets/js/utils/escape.js';
import { subscribeSnapshot } from '/assets/js/services/state-store.js';
const statuses = { not_configured: 'Not configured', not_connected: 'Not connected', connected: 'Connected', reconnect_required: 'Reconnect required', temporarily_unavailable: 'Temporarily unavailable' };
let currentUser = null;
let current = null;
let unavailable = false;
let checkedAt = '';
let inFlight = false;
let epoch = 0;
const watchers = new Set();
function notify() { for (const fn of watchers) fn(); }
subscribeSnapshot(snapshot => {
  const user = snapshot?.verified_identity?.user_id || null;
  if (user !== currentUser) { currentUser = user; current = null; ++epoch; notify(); }
});
async function refresh() {
  if (!currentUser || inFlight) return;
  inFlight = true; const requestedEpoch = epoch;
  try {
    const profile = await bridge.profileFetch();
    if (requestedEpoch !== epoch) return;
    const data = profile?.identity?.extra?.connections;
    if (!data || data.version !== 1) { unavailable = true; notify(); return; }
    if (data.user_id !== currentUser) { current = null; unavailable = true; notify(); return; }
    if (current && data.revision < current.revision) return;
    // Token refresh need not reinstall anything. Sync only when connector
    // usability changes, rather than whenever a verification timestamp moves.
    const availability = state => JSON.stringify(state.connections.map(c => [c.provider, c.configured && c.entitled && ['connected', 'temporarily_unavailable'].includes(c.status) && !!c.verified_at]));
    const changed = current && availability(current) !== availability(data);
    current = data; unavailable = false; checkedAt = new Date().toLocaleTimeString();
    notify();
    if (changed) await bridge.sync();
  } catch (_) { unavailable = true; notify(); }
  finally { inFlight = false; }
}
// This module remains loaded while the bridge window is running, even when the
// Account tab is hidden. Enforcement always happens server-side on each call.
setInterval(refresh, 30000);
window.addEventListener('focus', refresh);
export function watchConnections(fn) {
  watchers.add(fn); refresh();
  const timer = setInterval(() => { if (!document.hidden) refresh(); }, 15000);
  return () => { clearInterval(timer); watchers.delete(fn); };
}
export function renderConnections(profile) {
  const data = current || profile?.identity?.extra?.connections;
  if (!data || data.version !== 1) return '<article class="sp-profile-card"><h2>Connected accounts</h2><p>Update the server and bridge to manage connected accounts.</p></article>';
  const cards = data.connections.map(account => `<section>
    <h3>${escapeHtml(account.provider)}</h3>
    <p>${escapeHtml(statuses[account.status] || 'Unavailable')}</p>
    ${account.account_name ? `<p>${escapeHtml(account.account_name)}</p>` : ''}
    ${account.resource_name ? `<p>${escapeHtml(account.resource_name)}</p>` : ''}
    ${!account.entitled ? '<p>India Development marketplace access required.</p>' : ''}
    ${account.verified_at ? `<p>Last verified: ${escapeHtml(new Date(account.verified_at).toLocaleString())}</p>` : ''}
    <button class="sp-btn-ghost" type="button" data-action="manage-connection">Manage on server</button>
  </section>`).join('');
  return `<article class="sp-profile-card"><h2>Connected accounts</h2>
    ${unavailable ? `<p role="status">Last known state (${escapeHtml(checkedAt)}). Server unavailable.</p>` : ''}
    ${cards}</article>`;
}
