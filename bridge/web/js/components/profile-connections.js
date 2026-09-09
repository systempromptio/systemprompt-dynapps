// Provider credentials never enter the webview. The identity enrichment carries
// only the public, server-owned connection contract used by the Account page.
import { bridge } from '/assets/js/bridge.js';
import { escapeHtml } from '/assets/js/utils/escape.js';
import { subscribeSnapshot } from '/assets/js/services/state-store.js';
const statuses = { verification_required: 'Verification required', no_auth_required: 'No authorization required', not_configured: 'Not configured', not_connected: 'Not connected', connected: 'Connected', reconnect_required: 'Reconnect required', temporarily_unavailable: 'Temporarily unavailable' };
const dots = { connected: 'ok', reconnect_required: 'warn', verification_required: 'warn', temporarily_unavailable: 'warn' };
const names = { atlassian: 'Atlassian', github: 'GitHub', salesforce: 'Salesforce' };
let currentUser = null;
let current = null;
let unavailable = false;
let checkedAt = '';
let inFlight = null;
let currentGateway = null;
let dispose = null;
let epoch = 0;
const watchers = new Set();
function notify() { for (const fn of watchers) fn(); }
function receiveSnapshot(snapshot) {
  const user = snapshot?.verified_identity?.user_id || null;
  const gateway = snapshot?.gateway_url || null;
  if (user !== currentUser || gateway !== currentGateway) {
    currentUser = user; currentGateway = gateway; current = null;
    checkedAt = ''; unavailable = false; ++epoch; inFlight = null; notify();
    refresh();
  }
}
async function refresh() {
  if (!currentUser || watchers.size === 0 || inFlight !== null || document.hidden) return;
  const requestedEpoch = epoch; inFlight = requestedEpoch;
  try {
    const profile = await bridge.profileFetch();
    if (requestedEpoch !== epoch || watchers.size === 0) return;
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
    if (changed && requestedEpoch === epoch && watchers.size > 0) await bridge.sync();
  } catch (_) { if (requestedEpoch === epoch && watchers.size > 0) { unavailable = true; notify(); } }
  finally { if (inFlight === requestedEpoch) inFlight = null; }
}
export function watchConnections(fn) {
  watchers.add(fn);
  if (watchers.size === 1) {
    const onFocus = () => refresh();
    window.addEventListener('focus', onFocus);
    const timer = setInterval(onFocus, 30000);
    const unsubscribe = subscribeSnapshot(receiveSnapshot);
    dispose = () => {
      clearInterval(timer); unsubscribe(); window.removeEventListener('focus', onFocus);
      ++epoch; inFlight = null;
    };
  }
  refresh();
  return () => {
    watchers.delete(fn);
    if (watchers.size === 0 && dispose) { dispose(); dispose = null; }
  };
}
function host(value) { try { return new URL(value).host; } catch (_) { return value; } }
function metaFor(account) {
  if (!account.entitled) return 'An active account is required to connect providers';
  const parts = [];
  if (account.resource_name) parts.push(host(account.resource_name));
  else if (account.account_name) parts.push(account.account_name);
  if (account.verified_at) parts.push(`verified ${new Date(account.verified_at).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}`);
  return parts.join(' \u00b7 ');
}
function rowFor(account) {
  const status = statuses[account.status] || 'Unavailable';
  const action = account.status === 'connected' ? 'Manage' : 'Connect';
  const button = account.entitled && account.configured && account.requires_auth !== false ? `<button class="sp-btn-ghost sp-profile-connection__action" type="button" data-action="manage-connection">${action}</button>` : '';
  return `<li class="sp-profile-connection" data-status="${escapeHtml(account.status)}">
    <span class="sp-dot sp-dot--${dots[account.status] || 'unknown'}" aria-hidden="true"></span>
    <span class="sp-profile-connection__name">${escapeHtml(account.display_name || names[account.provider] || account.provider)}</span>
    <span class="sp-profile-connection__meta" title="${escapeHtml(account.resource_name || account.account_name || '')}">${escapeHtml(metaFor(account))}</span>
    <span class="sp-profile-connection__status">${escapeHtml(status)}</span>
    ${button}
  </li>`;
}
export function renderConnections(profile) {
  const data = current || profile?.identity?.extra?.connections;
  const header = '<header><h2 data-l10n-id="profile-section-connections">Connected accounts</h2>';
  if (!data || data.version !== 1) return `<article class="sp-profile-card sp-profile-card--connections">${header}</header><p class="sp-profile-connections__note">Update the server and bridge to manage connected accounts.</p></article>`;
  const accounts = data.connections.filter(account => account.configured && account.requires_auth !== false);
  const connected = accounts.filter(account => account.status === 'connected').length;
  const note = unavailable ? `<p class="sp-profile-connections__note" role="status">Last known state (${escapeHtml(checkedAt)}). Server unavailable.</p>` : '';
  return `<article class="sp-profile-card sp-profile-card--connections">${header}<span class="sp-profile-card__count">${connected} / ${accounts.length} connected</span></header>
    <ul class="sp-profile-connections">${data.connections.map(rowFor).join('')}</ul>${note}</article>`;
}
