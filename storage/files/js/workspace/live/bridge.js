// What the viewer shell does for each request a live page makes: it is the
// only party between the page's sandboxed frame and the viewer's session.
// Every request goes to the artifacts broker under
// `/api/v1/artifacts/{document_id}/` with the session the shell runs in,
// and every answer is reshaped into the codes of Anthropic's artifact runtime
// contract 0.2.99. The shell asks the viewer once per connector before a
// first call on it, and again for every call to a tool that is not a read,
// while a watch asks for the connector but never approves a write. A server
// is shown to the viewer by the name the broker gives it (`nameOf`).
import { summaryOf } from './labels.js';
import { createViewer } from './viewer.js';

const BROKER_CODES = {
  reauth: 'upstream_error',
  cross_site: 'upstream_error',
  internal: 'upstream_error',
  not_found: 'not_granted',
};
const MAX_MANAGE = 5;
const DEFAULT_GC_MS = 300000;

const fail = (code, message, extra = {}) => ({ code, message, ...extra });

const brokerError = (reply, server) => {
  const body = reply.body || {};
  const code = BROKER_CODES[body.error_key] || body.error_key || 'upstream_error';
  const error = fail(code, body.error || `the request failed (${reply.status})`, server ? { server } : {});
  if (code === 'server_unavailable') error.retryable = true;
  if (body.live) error.live = body.live;
  return error;
};

const scopeOf = (server) => `mcp:${server}`;

export const createBridge = ({ urls, transport, prompts, onPublished, onFailure }) => {
  let snapshot = { capabilities: null, revision: null, writer: false, states: {}, viewer: null, servers: [] };
  const declined = new Set();
  let manageCount = 0;

  const send = async (url, options, server) => {
    let reply;
    try {
      reply = await transport(url, options);
    } catch {
      throw fail('server_unavailable', 'the broker could not be reached', { retryable: true, ...(server ? { server } : {}) });
    }
    if (reply.status === 401) onFailure('reauth');
    return reply;
  };
  const post = (path, body, server) => send(`${urls.api}${path}`, { method: 'POST', body: JSON.stringify(body) }, server);

  const load = async () => {
    const reply = await send(`${urls.api}/permissions`, { method: 'GET' });
    if (reply.status !== 200) throw brokerError(reply);
    snapshot = reply.body;
    return snapshot;
  };

  const nameOf = (server) => {
    const named = (snapshot.servers || []).find((item) => item.id === server);
    return named && named.name ? named.name : server;
  };

  const declared = () => ((snapshot.capabilities && snapshot.capabilities.mcp && snapshot.capabilities.mcp.servers) || []).map((item) => item.server);
  const publishes = () => {
    const artifact = snapshot.capabilities && snapshot.capabilities.artifact;
    return Boolean(artifact) && artifact.publish !== false;
  };

  const states = () => {
    const out = { ...snapshot.states, user: 'granted' };
    declined.forEach((scope) => { if (out[scope] === 'prompt') out[scope] = 'denied'; });
    const each = declared().map((server) => out[scopeOf(server)]);
    if (each.length > 0) {
      out.mcp = each.every((state) => state === 'granted') ? 'granted' : (each.includes('prompt') ? 'prompt' : 'denied');
    }
    return out;
  };

  const decide = async (scope, decision) => {
    const reply = decision === 'prompt'
      ? await send(`${urls.api}/permissions/${encodeURIComponent(scope)}`, { method: 'DELETE' })
      : await post('/permissions', { scope, decision });
    if (reply.status !== 200) throw brokerError(reply);
    snapshot = reply.body;
    declined.delete(scope);
    return states();
  };

  const askConsent = async (servers, call) => {
    const allowed = await prompts.consent(servers, call);
    if (!allowed) {
      servers.forEach((server) => declined.add(scopeOf(server)));
      return false;
    }
    await servers.reduce((done, server) => done.then(() => decide(scopeOf(server), 'granted')), Promise.resolve());
    return true;
  };

  // Resolves true when the viewer allowed the connector just now, in a
  // prompt that showed `call`: that one Allow also approves this call.
  const ensureConsent = async (server, call) => {
    if (!declared().includes(server)) return false;
    const refused = fail('not_in_manifest', `the viewer has not allowed "${server}" for this page`, { server });
    const state = states()[scopeOf(server)];
    if (state === 'granted') return false;
    if (state === 'denied' || !(await askConsent([server], call))) throw refused;
    return Boolean(call);
  };

  const callTool = async ({ server, tool, input, cache }) => {
    const call = { server, tool, summary: summaryOf(input) };
    const allowedNow = await ensureConsent(server, call);
    const body = { server, tool, input, ...(cache === undefined ? {} : { cache }) };
    let reply = await post('/mcp/call', body, server);
    if (reply.status === 428 && states()[scopeOf(server)] === 'granted') {
      if (!allowedNow && !(await prompts.confirm(call))) {
        throw fail('approval_required', 'the viewer did not approve this call', { server });
      }
      const approved = await post('/mcp/approve', { server, tool, input }, server);
      if (approved.status !== 200) throw brokerError(approved, server);
      reply = await post('/mcp/call', { ...body, approval: approved.body.approval }, server);
    }
    if (reply.status !== 200) throw brokerError(reply, server);
    return reply.body;
  };

  const watch = async ({ server, tool, input, mode, cache }) => {
    await ensureConsent(server);
    const gcTime = cache && typeof cache.gcTime === 'number' ? cache.gcTime : DEFAULT_GC_MS;
    const tuned = mode === 'peek' ? { staleTime: gcTime, gcTime } : { gcTime, refresh: true };
    const reply = await post('/mcp/call', { server, tool, input, cache: tuned }, server);
    if (reply.status !== 200) throw brokerError(reply, server);
    return reply.body;
  };

  const viewer = createViewer({ urls, send, snapshot: () => snapshot });

  const plain = (path) => async (params) => {
    const reply = await post(path, params, params.server);
    if (reply.status === 204) return null;
    if (reply.status !== 200) throw brokerError(reply, params.server);
    return reply.body;
  };

  const publish = async ({ html }) => {
    if (!publishes()) throw fail('not_granted', 'this page does not declare publishing itself');
    const reply = await post('/artifact/publish', { html, base_revision: snapshot.revision });
    if (reply.status !== 200) {
      const error = brokerError(reply);
      if (error.code === 'conflict' && error.live) onPublished(error.live);
      throw error;
    }
    onPublished(reply.body.version);
    return { version: reply.body.version };
  };

  const handlers = {
    'permissions.state': async ({ name }) => (name === undefined ? states() : states()[name] || 'unavailable'),
    'permissions.request': async ({ names }) => {
      const now = states();
      const wanted = names || Object.keys(now);
      const servers = new Set();
      wanted.forEach((name) => {
        if (name === 'mcp') declared().forEach((server) => { if (now[scopeOf(server)] === 'prompt') servers.add(server); });
        if (name.startsWith('mcp:') && now[name] === 'prompt') servers.add(name.slice(4));
      });
      if (servers.size > 0) await askConsent([...servers]);
      const after = states();
      if (!names) return after;
      return Object.fromEntries(names.map((name) => [name, after[name] || 'unavailable']));
    },
    'permissions.manage': async () => {
      if (prompts.busy() || manageCount >= MAX_MANAGE) throw fail('unavailable', 'the permissions panel cannot open now');
      manageCount += 1;
      await prompts.panel({ servers: declared(), states: states(), decide });
      return null;
    },
    'mcp.callTool': callTool,
    'mcp.watch': watch,
    'mcp.invalidate': plain('/mcp/invalidate'),
    'mcp.listTools': plain('/mcp/tools'),
    'mcp.describeTool': plain('/mcp/describe'),
    'user.me': async () => viewer.me(),
    'user.can': async ({ capability }) => capability === 'files.write' && Boolean(snapshot.writer) && publishes(),
    'user.profiles': viewer.profiles,
    'user.search': viewer.search,
    'artifact.publish': publish,
    'artifact.edit': async () => { throw fail(snapshot.writer ? 'not_granted' : 'not_writer', 'a live page cannot be edited from itself; publish it instead'); },
  };

  return {
    load,
    snapshot: () => snapshot,
    nameOf,
    panel: () => prompts.panel({ servers: declared(), states: states(), decide }),
    served: () => ['permissions', 'user', 'artifact', ...(declared().length > 0 ? ['mcp'] : [])],
    declared,
    serverOf: (tool) => {
      const servers = ((snapshot.capabilities && snapshot.capabilities.mcp && snapshot.capabilities.mcp.servers) || [])
        .filter((item) => item.tools.includes(tool));
      return servers.length === 1 ? servers[0].server : null;
    },
    handle: (method, params) => {
      const handler = handlers[method];
      if (!handler) return Promise.reject(fail('capability_removed', `"${method}" is not part of this runtime`));
      return handler(params || {});
    },
  };
};
