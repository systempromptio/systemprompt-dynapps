import { errorMessage, rawResponse } from '/js/services/api.js';
import { showConfirmDialog } from '/js/services/confirm.js';
// Account state comes only from the server. Credentials are submitted once and
// never kept in browser storage, URL parameters, or rendered response payloads.
const root = document.querySelector('[data-connectors]');
const labels = { verification_required: 'Verification required', no_auth_required: 'Built in', not_configured: 'Not configured', not_connected: 'Not connected', not_provisioned: 'Not provisioned', connected: 'Connected', reconnect_required: 'Reconnect required', temporarily_unavailable: 'Temporarily unavailable' };
const tones = { connected: 'ok', reconnect_required: 'warn', verification_required: 'warn', temporarily_unavailable: 'err' };
const names = { atlassian: 'Atlassian', salesforce: 'Salesforce' };
const actionLabels = { connect: 'Connect', reconnect: 'Reconnect', disconnect: 'Disconnect', test: 'Test connection', manual_token: 'Use personal token' };
const notes = { sandbox_refreshed: 'This org was refreshed by an administrator — reconnect to continue', verification_failed: 'Authorization saved, but the last verification failed', grant_rejected: 'The provider rejected the saved grant', configuration_changed: 'The connector configuration changed — reconnect to authorize again', provider_unavailable: 'The provider did not answer the last check', provider_permission_denied: 'The provider refused the permissions this connector needs' };
const attention = ['reconnect_required', 'verification_required', 'temporarily_unavailable'];
const groups = [
    { key: 'attention', title: 'Needs your attention', note: 'Broken or unverified — fix these first.' },
    { key: 'ready', title: 'Ready to connect', note: 'Authorize once; every client you connect uses the same account.' },
    { key: 'connected', title: 'Connected', note: 'Verified and available to every client.' },
    { key: 'quiet', title: 'Nothing to do', note: 'Built in, not provisioned yet, or not open to your account.' }
];
const steps = [
    { id: 'token', title: 'Credential', pending: 'Checking the saved grant' },
    { id: 'initialize', title: 'MCP session', pending: 'Opening a session with the provider' },
    { id: 'tools', title: 'Tools', pending: 'Listing the tools this account can call' },
    { id: 'identity', title: 'Identity', pending: 'Confirming who the provider says you are' }
];
let revision = -1;
let sequence = 0;
let pending = false;
let lastChecked = '';
function element(tag, text, className) { const node = document.createElement(tag); if (text) node.textContent = text; if (className) node.className = className; return node; }
function message(text, tone = '') {
    const node = root.querySelector('[data-connection-message]');
    node.textContent = text; node.hidden = !text; node.className = `sp-notice sp-p-connectors__message${tone ? ` sp-notice--${tone}` : ''}`;
}
function relative(iso) {
    const minutes = Math.round((Date.now() - new Date(iso).getTime()) / 60000);
    if (minutes < 1) return 'just now';
    if (minutes < 60) return `${minutes} min ago`;
    if (minutes < 60 * 24) return `${Math.round(minutes / 60)} h ago`;
    return new Date(iso).toLocaleDateString();
}
function host(value) { try { return new URL(value).host; } catch (_) { return value; } }
function displayName(account) { return account.display_name || names[account.provider] || account.provider; }
function groupOf(account) {
    if (attention.includes(account.status) && account.entitled) return 'attention';
    if (account.status === 'not_connected' && account.entitled) return 'ready';
    if (account.status === 'connected') return 'connected';
    return 'quiet';
}
function glyph(provider) {
    if (provider === 'atlassian') return 'atlassian';
    if (provider === 'google_drive') return 'google';
    if (provider === 'systemprompt') return 'systemprompt';
    return provider.startsWith('salesforce') ? 'salesforce' : 'generic';
}
function monogram(family, name) { return { atlassian: 'At', google: 'G', systemprompt: 'Sp', salesforce: 'Sf' }[family] || name.slice(0, 2); }
function kind(family) { return { atlassian: 'Atlassian Cloud', google: 'Google Workspace', systemprompt: 'Control plane', salesforce: 'Salesforce org' }[family] || 'MCP server'; }
function fact(list, label, value, mono) {
    const row = element('div', '', 'sp-connector__fact'); const dd = element('dd', value, mono ? 'sp-u-mono' : ''); dd.title = value;
    row.append(element('dt', label), dd); list.append(row);
}
function facts(account) {
    const list = element('dl', '', 'sp-connector__facts');
    if (account.account_name) fact(list, 'Account', account.account_name, true);
    if (account.resource_name) fact(list, 'Site', host(account.resource_name), false);
    if (account.verified_at) fact(list, 'Verified', relative(account.verified_at), false);
    list.hidden = !list.childElementCount;
    return list;
}
function note(account) {
    if (!account.entitled) return account.provider.startsWith('salesforce') ? 'Your account is not entitled to this Salesforce org' : 'An active account is required to connect providers';
    if (account.status === 'not_provisioned') return 'Waiting for a Salesforce administrator to provision this org';
    return notes[account.error_code] || '';
}
function variant(action, status) {
    if (action === 'connect') return 'sp-btn--primary';
    if (action === 'reconnect') return status === 'connected' ? 'sp-btn--outline' : 'sp-btn--primary';
    if (action === 'disconnect') return 'sp-btn--outline-danger';
    if (action === 'test') return status === 'connected' ? 'sp-btn--primary' : 'sp-btn--outline';
    return 'sp-btn--ghost';
}
function actionsFor(account) {
    const actions = element('div', '', 'sp-connector__actions');
    const order = ['test', 'connect', 'reconnect', 'disconnect', 'manual_token'];
    for (const action of order.filter(name => account.actions.includes(name))) {
        const button = element('button', actionLabels[action], `sp-btn sp-btn--sm ${variant(action, account.status)}`); button.type = 'button'; button.dataset.action = action;
        actions.append(button);
    }
    return actions;
}
// Why: the "about" block (what the provider unlocks, which plugins carry it)
// is server-rendered from the services tree and not part of the snapshot, so
// a re-render carries the previous card's block instead of rebuilding it.
function about(account, previous) {
    const kept = previous?.querySelector('.sp-connector__about');
    if (kept) return kept;
    const block = element('div', '', 'sp-connector__about');
    block.append(element('p', account.configured ? 'Authorize once; every client you connect uses the same account.' : 'Provider setup required.', 'sp-connector__blurb'));
    return block;
}
function card(account, previous) {
    const family = glyph(account.provider);
    const row = element('li', '', 'sp-connector'); row.id = `connector-${account.provider}`; row.dataset.status = account.status; row.dataset.provider = account.provider; row.dataset.glyph = family;
    const head = element('div', '', 'sp-connector__head');
    const mark = element('span', monogram(family, displayName(account)), 'sp-connector__mark'); mark.setAttribute('aria-hidden', 'true');
    const title = element('div', '', 'sp-connector__title');
    title.append(element('h3', displayName(account), 'sp-connector__name'), element('span', kind(family), 'sp-connector__kind'));
    const badge = element('span', labels[account.status] || 'Unavailable', `sp-badge sp-connector__status sp-badge--${tones[account.status] || 'muted'}`);
    head.append(mark, title, badge); row.append(head, about(account, previous), facts(account));
    const text = note(account); if (text) row.append(element('p', text, 'sp-connector__note'));
    row.append(actionsFor(account));
    // Why: a check panel that is open survives the 15 s re-render, otherwise a
    // result would vanish while the person is still reading it.
    const check = previous?.querySelector('.sp-connector__check');
    if (check) row.append(check);
    return row;
}
function section(group, cards) {
    const node = element('section', '', 'sp-connector-group'); node.dataset.group = group.key; node.setAttribute('aria-label', group.title);
    const head = element('div', '', 'sp-section__head');
    const count = element('span', String(cards.length), 'sp-section__count'); count.dataset.groupCount = '';
    head.append(element('h2', group.title, 'sp-section__title'), count, element('p', group.note, 'sp-section__sub'));
    const list = element('ul', '', 'sp-connector-grid'); list.dataset.groupList = ''; list.append(...cards);
    node.append(head, list);
    return node;
}
function summarise(accounts) {
    const counts = { configured: accounts.length, connected: 0, needs_attention: 0, not_connected: 0 };
    let broken = null; let ready = null;
    for (const account of accounts) {
        if (account.status === 'connected') counts.connected += 1;
        else if (attention.includes(account.status)) { counts.needs_attention += 1; broken ??= displayName(account); }
        else if (account.status === 'not_connected') { counts.not_connected += 1; if (account.entitled) ready ??= displayName(account); }
    }
    for (const [key, value] of Object.entries(counts)) {
        const tile = root.querySelector(`[data-summary="${key}"] [data-value]`); if (tile) tile.textContent = String(value);
    }
    const pct = n => counts.configured ? `${Math.round(n * 100 / counts.configured)}%` : '0%';
    root.querySelector('[data-summary-bar="connected"]')?.style.setProperty('--sp-fill', pct(counts.connected));
    root.querySelector('[data-summary-bar="needs_attention"]')?.style.setProperty('--sp-fill', pct(counts.needs_attention));
    const headline = counts.configured === 0 ? 'No connectors on this gateway' : counts.configured === counts.connected ? 'Everything is connected' : `${counts.connected} of ${counts.configured} connected`;
    root.querySelector('[data-summary-headline]').textContent = headline;
    const next = root.querySelector('[data-summary-next]');
    next.textContent = broken ? `Reconnect ${broken} to get it working again.` : ready ? `Connect ${ready} to unlock it in every client.` : '';
    next.hidden = !next.textContent;
}
function render(data) {
    const expected = new URLSearchParams(window.location.search).get('expected_user');
    if (expected && expected !== data.user_id) { root.querySelector('[data-connector-groups]')?.replaceChildren(); message('Sign in to the same Systemprompt account as your bridge.', 'err'); return; }
    if (data.version !== 1) { message('Update the server and bridge to manage connected accounts.', 'err'); return; }
    if (data.revision < revision) return;
    revision = data.revision;
    summarise(data.connections.filter(account => account.configured && account.requires_auth !== false));
    const container = root.querySelector('[data-connector-groups]'); if (!container) return;
    const sections = groups.map(group => {
        const cards = data.connections.filter(account => groupOf(account) === group.key).map(account => card(account, container.querySelector(`#connector-${account.provider}`)));
        return cards.length ? section(group, cards) : null;
    }).filter(Boolean);
    container.replaceChildren(...sections);
}
async function refresh() {
    if (!root || pending || root.querySelector('form')) return;
    const request = ++sequence;
    try {
        const response = await rawResponse('/api/public/account/connections', { credentials: 'same-origin', cache: 'no-store' });
        if (response.status === 401) {
            root.querySelectorAll('button').forEach(button => { button.disabled = true; });
            message('Your login session is no longer active. Sign in again to manage connectors.', 'err');
            const link = element('a', 'Sign in again', 'sp-btn sp-btn--sm sp-btn--primary'); link.href = '/admin/login';
            root.querySelector('[data-connection-message]').append(' ', link); return;
        }
        if (!response.ok) throw new Error(await errorMessage(response));
        const data = await response.json();
        if (request !== sequence) return;
        render(data); lastChecked = new Date().toLocaleTimeString(); message('');
    } catch (error) { message(`Last known state${lastChecked ? ` (${lastChecked})` : ''}. ${error.message}`, 'warn'); }
}
function field(form, label, name, type = 'text') {
    const wrapper = element('label', label); const input = element('input', '', 'sp-field-input'); input.name = name; input.type = type;
    input.autocomplete = 'off'; wrapper.append(input); form.append(wrapper); return input;
}
function formFor(provider, action, user) {
    const row = root.querySelector(`#connector-${provider}`);
    if (row.querySelector('form')) return;
    const form = element('form', '', 'sp-connector__form');
    if (action === 'manual_token') {
        field(form, 'Personal token', 'token', 'password').required = true;
        if (provider === 'atlassian') field(form, 'Atlassian account email', 'email', 'email').required = true;
    }
    if (provider === 'atlassian') {
        const site = field(form, 'Atlassian site address', 'resource_id', 'url');
        site.value = 'https://astounddigital.atlassian.net'; site.required = true;
    }
    const actions = element('div', '', 'sp-connector__actions'); form.append(actions);
    const submit = element('button', action === 'manual_token' ? 'Verify and save' : 'Continue to Atlassian', 'sp-btn sp-btn--sm sp-btn--primary'); submit.type = 'submit'; actions.append(submit);
    const cancel = element('button', 'Cancel', 'sp-btn sp-btn--sm sp-btn--ghost'); cancel.type = 'button'; cancel.addEventListener('click', () => form.remove()); actions.append(cancel);
    form.addEventListener('submit', async (event) => {
        event.preventDefault(); const values = Object.fromEntries(new FormData(form));
        if (action !== 'manual_token') { connect(provider, user, values.resource_id); return; }
        await mutate(provider, 'manual-token', values); form.reset(); form.remove(); await refresh();
    }); row.append(form);
}
function connect(provider, user, resource = '') {
    const url = new URL(`/api/public/connectors/${provider}/start`, window.location.origin);
    url.searchParams.set('expected_user', user);
    if (resource) url.searchParams.set('resource_id', resource);
    window.location.assign(url);
}
async function mutate(provider, action, body) {
    pending = true; ++sequence;
    try {
        const response = await rawResponse(`/api/public/account/connections/${provider}/${action}`, {
            method: 'POST', credentials: 'same-origin', cache: 'no-store',
            headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body || {})
        });
        if (!response.ok) throw new Error(await errorMessage(response));
        const data = await response.json();
        render(data); message('Account updated on the server. Enrolled bridges will refresh automatically.');
        return data;
    } catch (error) { message(error.message, 'err'); return null; } finally { pending = false; }
}
function checkPanel(provider) {
    const row = root.querySelector(`#connector-${provider}`);
    let panel = row.querySelector('.sp-connector__check');
    if (!panel) { panel = element('div', '', 'sp-connector__check'); row.append(panel); }
    panel.hidden = false; panel.replaceChildren();
    const list = element('ol', '', 'sp-connector__steps');
    for (const [index, step] of steps.entries()) {
        const item = element('li', '', `sp-connector__step ${index === 0 ? 'is-running' : 'is-pending'}`); item.dataset.step = step.id;
        item.append(element('span', '', 'sp-connector__step-mark'), element('span', step.title, 'sp-connector__step-title'), element('span', step.pending, 'sp-connector__step-detail'), element('span', '', 'sp-connector__step-ms'));
        list.append(item);
    }
    const summary = element('p', 'Testing the connection…', 'sp-connector__summary'); summary.setAttribute('role', 'status');
    panel.append(list, summary);
    return panel;
}
function paint(panel, report, account, user) {
    const items = [...panel.querySelectorAll('.sp-connector__step')];
    let failed = false;
    for (const item of items) {
        const result = report.steps.find(step => step.step === item.dataset.step);
        if (result) {
            item.className = `sp-connector__step ${result.ok ? 'is-ok' : 'is-fail'}`;
            item.querySelector('.sp-connector__step-detail').textContent = result.detail;
            item.querySelector('.sp-connector__step-ms').textContent = `${result.duration_ms} ms`;
            if (!result.ok) failed = true;
        } else {
            item.className = 'sp-connector__step is-skipped';
            item.querySelector('.sp-connector__step-detail').textContent = failed ? 'Not reached' : 'Skipped';
        }
    }
    const total = report.steps.reduce((sum, step) => sum + step.duration_ms, 0);
    const summary = panel.querySelector('.sp-connector__summary');
    summary.className = `sp-connector__summary ${report.ok ? 'is-ok' : 'is-fail'}`;
    summary.textContent = report.ok ? `Verified${account?.account_name ? ` as ${account.account_name}` : ''} in ${(total / 1000).toFixed(1)} s. Every connected client can use this provider.` : `${report.error || 'Verification failed.'} Reconnect to authorize again.`;
    const actions = element('div', '', 'sp-connector__actions');
    const retry = element('button', 'Run again', 'sp-btn sp-btn--sm sp-btn--outline'); retry.type = 'button'; retry.addEventListener('click', () => test(account?.provider || report.provider, user)); actions.append(retry);
    if (!report.ok && account?.actions.some(a => ['connect', 'reconnect'].includes(a))) {
        const reconnect = element('button', 'Reconnect', 'sp-btn sp-btn--sm sp-btn--primary'); reconnect.type = 'button'; reconnect.addEventListener('click', () => connect(account.provider, user)); actions.append(reconnect);
    }
    const close = element('button', 'Close', 'sp-btn sp-btn--sm sp-btn--ghost'); close.type = 'button'; close.addEventListener('click', () => { panel.hidden = true; }); actions.append(close);
    panel.append(actions);
}
async function test(provider, user) {
    const panel = checkPanel(provider);
    const data = await mutate(provider, 'test');
    const live = root.querySelector(`#connector-${provider} .sp-connector__check`) || panel;
    if (!data) {
        const text = root.querySelector('[data-connection-message]').textContent;
        paint(live, { ok: false, steps: [{ step: 'token', ok: false, detail: text, duration_ms: 0 }], error: text, provider }, null, user);
        return;
    }
    message('');
    const account = data.connections.find(entry => entry.provider === provider);
    paint(live, data.verification || { ok: false, steps: [], error: 'The server returned no verification report.', provider }, account, user);
}
async function perform(provider, action, user) {
    if (pending) return;
    if (action === 'test') { await test(provider, user); return; }
    if (action === 'manual_token' || (provider === 'atlassian' && ['connect', 'reconnect'].includes(action))) { formFor(provider, action, user); return; }
    if (['connect', 'reconnect'].includes(action)) { connect(provider, user); return; }
    if (action === 'disconnect') { await showConfirmDialog('Disconnect account', `Disconnect ${names[provider] || provider} on every enrolled device?`, 'Disconnect', () => mutate(provider, action)); return; }
    await mutate(provider, action);
}
if (root) {
    // Why: buttons are server-rendered on first paint and rebuilt on every
    // refresh, so one delegated listener on the page root serves both.
    root.addEventListener('click', (event) => {
        const button = event.target.closest('.sp-connector > .sp-connector__actions > [data-action]');
        if (button) perform(button.closest('.sp-connector').dataset.provider, button.dataset.action, root.dataset.userId);
    });
    refresh();
    window.setInterval(() => { if (!document.hidden) refresh(); }, 15000);
    window.addEventListener('focus', refresh);
}
