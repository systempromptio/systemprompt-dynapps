import { errorMessage, rawResponse } from '/js/services/api.js';
// Account state comes only from the server. Credentials are submitted once and
// never kept in browser storage, URL parameters, or rendered response payloads.
const root = document.querySelector('[data-connected-accounts]');
const labels = { not_configured: 'Not configured', not_connected: 'Not connected', connected: 'Connected', reconnect_required: 'Reconnect required', temporarily_unavailable: 'Temporarily unavailable' };
const names = { atlassian: 'Atlassian', github: 'GitHub', salesforce: 'Salesforce' };
let revision = -1;
let sequence = 0;
let pending = false;
let lastChecked = '';
function element(tag, text) { const node = document.createElement(tag); if (text) node.textContent = text; return node; }
function message(text) { root.querySelector('[data-connection-message]').textContent = text; }
function render(data) {
    const expected = new URLSearchParams(window.location.search).get('expected_user');
    if (expected && expected !== data.user_id) { root.querySelector('[data-connection-list]').replaceChildren(); message('Sign in to the same Systemprompt account as your bridge.'); return; }
    if (data.version !== 1) { message('Update the server and bridge to manage connected accounts.'); return; }
    if (data.revision < revision) return;
    revision = data.revision;
    const list = root.querySelector('[data-connection-list]');
    list.replaceChildren();
    for (const account of data.connections) {
        const card = element('article'); card.className = 'sp-card'; card.id = `connection-${account.provider}`;
        card.append(element('h3', names[account.provider]), element('p', labels[account.status] || 'Unavailable'));
        if (account.account_name) card.append(element('p', account.account_name));
        if (account.resource_name) card.append(element('p', account.resource_name));
        if (account.verified_at) card.append(element('p', `Last verified: ${new Date(account.verified_at).toLocaleString()}`));
        if (!account.entitled) card.append(element('p', 'India Development marketplace access required.'));
        for (const action of account.actions) {
            const label = { connect: 'Connect', reconnect: 'Reconnect', disconnect: 'Disconnect', test: 'Test connection', manual_token: 'Use personal token' }[action];
            if (!label) continue;
            const button = element('button', label); button.type = 'button'; button.className = 'sp-btn sp-btn--sm sp-btn--outline';
            button.addEventListener('click', () => perform(account.provider, action, data.user_id)); card.append(button);
        }
        list.append(card);
    }
}
async function refresh() {
    if (!root || pending || root.querySelector('form')) return;
    const request = ++sequence;
    try {
        const response = await rawResponse('/api/public/account/connections', { credentials: 'same-origin', cache: 'no-store' });
        if (!response.ok) throw new Error(await errorMessage(response));
        const data = await response.json();
        if (request !== sequence) return;
        render(data); lastChecked = new Date().toLocaleTimeString(); message('');
    } catch (error) { message(`Last known state${lastChecked ? ` (${lastChecked})` : ''}. ${error.message}`); }
}
function field(form, label, name, type = 'text') {
    const wrapper = element('label', label); const input = element('input'); input.name = name; input.type = type;
    input.autocomplete = 'off'; wrapper.append(input); form.append(wrapper); return input;
}
function formFor(provider, action, user) {
    const card = root.querySelector(`#connection-${provider}`);
    if (card.querySelector('form')) return;
    const form = element('form');
    if (action === 'manual_token') {
        field(form, 'Personal token', 'token', 'password').required = true;
        if (provider === 'atlassian') field(form, 'Atlassian account email', 'email', 'email').required = true;
    }
    if (provider === 'atlassian') field(form, 'Atlassian site address (optional)', 'resource_id');
    const submit = element('button', action === 'manual_token' ? 'Verify and save' : 'Continue to Atlassian'); submit.type = 'submit'; submit.className = 'sp-btn'; form.append(submit);
    const cancel = element('button', 'Cancel'); cancel.type = 'button'; cancel.addEventListener('click', () => form.remove()); form.append(cancel);
    form.addEventListener('submit', async (event) => {
        event.preventDefault(); const values = Object.fromEntries(new FormData(form));
        if (action !== 'manual_token') { connect(provider, user, values.resource_id); return; }
        await mutate(provider, 'manual-token', values); form.reset(); form.remove(); await refresh();
    }); card.append(form);
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
        render(await response.json()); message('Account updated on the server. Enrolled bridges will refresh automatically.');
    } catch (error) { message(error.message); } finally { pending = false; }
}
async function perform(provider, action, user) {
    if (pending) return;
    if (action === 'manual_token' || (provider === 'atlassian' && ['connect', 'reconnect'].includes(action))) { formFor(provider, action, user); return; }
    if (['connect', 'reconnect'].includes(action)) { connect(provider, user); return; }
    if (action === 'disconnect' && !window.confirm(`Disconnect ${names[provider]} on every enrolled device?`)) return;
    await mutate(provider, action);
}
if (root) {
    refresh();
    window.setInterval(() => { if (!document.hidden) refresh(); }, 15000);
    window.addEventListener('focus', refresh);
}
