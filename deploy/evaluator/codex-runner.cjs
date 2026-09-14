'use strict';
const http = require('node:http');
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const { spawn } = require('node:child_process');

const MAX_BODY = 1024 * 1024;
const MAX_RESPONSE = 16 * 1024 * 1024;
const EXECUTABLE = '/opt/systemprompt/codex/bin/codex';

function validateOptions(options) {
  if (['relay', 'token', 'session', 'model', 'localToken'].some(key => typeof options[key] !== 'string') || !/^http:\/\/eval-relay-[a-zA-Z0-9_-]{1,128}:8090$/.test(options.relay) ||
      !/^[a-zA-Z0-9._-]{1,128}$/.test(options.token) ||
      !/^[a-zA-Z0-9._-]{1,128}$/.test(options.session) ||
      !/^[a-zA-Z0-9._/:-]{1,128}$/.test(options.model) ||
      !Number.isInteger(options.maxRequests) || options.maxRequests < 1 || options.maxRequests > 100 ||
      !Number.isInteger(options.maxOutputTokens) || options.maxOutputTokens < 256 || options.maxOutputTokens > 32768 ||
      !/^[a-f0-9]{64}$/.test(options.localToken) || !['execution', 'review'].includes(options.purpose)) {
    throw new Error('Invalid bounded Codex runner configuration');
  }
}

function childEnvironment(source, localToken) {
  const environment = {
    HOME: '/home/tester', USERPROFILE: '/home/tester', CODEX_HOME: '/home/tester/.codex',
    PATH: '/opt/systemprompt/codex/codex-path:/usr/local/bin:/usr/bin:/bin',
    XDG_CONFIG_HOME: '/home/tester/.config', XDG_CACHE_HOME: '/home/tester/.cache',
    XDG_DATA_HOME: '/home/tester/.local/share', XDG_STATE_HOME: '/home/tester/.local/state',
    CODEX_LOCAL_PROXY_TOKEN: localToken,
  };
  for (const key of ['TZ', 'SYSTEMPROMPT_FIXTURE_CLOCK', 'SYSTEMPROMPT_EXECUTION_ID']) {
    if (source[key] && source[key].length <= 256 && !/[\x00-\x1f\x7f]/.test(source[key])) environment[key] = source[key];
  }
  return environment;
}

function nativeArguments(argv, environment) {
  const timezone = environment.TZ;
  if (argv[0] !== 'exec' || typeof timezone !== 'string' || !timezone.length || timezone.length > 256 || /[\x00-\x1f\x7f]/.test(timezone)) {
    throw new Error('Invalid frozen native timezone');
  }
  return ['exec', '-c', `shell_environment_policy.set.TZ=${JSON.stringify(timezone)}`, ...argv.slice(1)];
}

function createRelay(options, transport = http.request) {
  validateOptions(options);
  const wire = options.wire ?? 'responses';
  if (!['responses', 'chat'].includes(wire)) throw new Error('Unsupported native wire protocol');
  const chatClient = options.chatClient ?? 'hermes';
  if (!['hermes', 'opencode'].includes(chatClient) || (wire !== 'chat' && options.chatClient !== undefined)) throw new Error('Unsupported native chat client');
  const providerPath = wire === 'responses' ? '/v1/responses' : '/v1/chat/completions';
  const toolEvidence = new Map();
  const upstream = new URL(options.relay);
  const upstreamTimeoutMs = options.upstreamTimeoutMs ?? 30000;
  if (!Number.isInteger(upstreamTimeoutMs) || upstreamTimeoutMs < 1 || upstreamTimeoutMs > 30000) throw new Error('Invalid relay timeout');
  let attempts = 0;
  let mcpAttempts = 0;
  let active = 0;
  const pending = new Set();
  const server = http.createServer(async (request, response) => {
    const fail = (code, message) => {
      if (response.writableEnded || response.destroyed) return;
      if (response.headersSent) { response.destroy(); return; }
      response.writeHead(code, { 'content-type': 'application/json', connection: 'close' });
      response.end(JSON.stringify({ error: { message, type: 'evaluation_bound' } }));
    };
    if (request.headers.authorization !== `Bearer ${options.localToken}`) { fail(401, 'Unauthenticated local request'); return; }
    const provider = request.url === providerPath && request.method === 'POST';
    const mcp = request.url === '/mcp/evaluation_fixture' && ['GET', 'POST', 'DELETE'].includes(request.method);
    if (!provider && !mcp) { fail(404, 'Endpoint unavailable'); return; }
    if (provider && ++attempts > options.maxRequests) { fail(429, 'Native provider attempt limit reached'); return; }
    if (mcp && (options.purpose !== 'execution' || ++mcpAttempts > 256)) { fail(403, 'Fixture requests unavailable'); return; }
    if (active >= 4) { fail(429, 'Native concurrency limit reached'); return; }
    active++;
    let outgoing;
    let released = false;
    const release = () => { if (!released) { released = true; active--; } };
    response.once('close', () => { outgoing?.destroy(); release(); });
    response.once('finish', release);
    try {
      let bytes = 0;
      const chunks = [];
      for await (const chunk of request.iterator({ destroyOnReturn: false })) {
        bytes += chunk.length;
        if (bytes > MAX_BODY) { request.resume(); fail(413, 'Native request body limit reached'); return; }
        chunks.push(chunk);
      }
      let body = Buffer.concat(chunks);
      if (provider || (mcp && request.method === 'POST')) {
        let value;
        try { value = JSON.parse(body.toString('utf8')); } catch { fail(400, 'Malformed native JSON request'); return; }
        if (!value || typeof value !== 'object' || Array.isArray(value)) { fail(400, 'Invalid native request'); return; }
        if (provider) {
          if (value.model !== options.model) { fail(400, 'Native model differs from admitted experiment'); return; }
          const fields = wire === 'responses' ? ['max_output_tokens'] : ['max_tokens', 'max_completion_tokens'];
          if (wire === 'responses' && ('max_tokens' in value || 'max_completion_tokens' in value) || wire === 'chat' && 'max_output_tokens' in value) {
            fail(400, 'Output limit does not match native wire protocol'); return;
          }
          for (const field of fields) {
            if (value[field] !== undefined && (!Number.isInteger(value[field]) || value[field] <= 0)) {
              fail(400, 'Invalid native output limit'); return;
            }
            if (value[field] !== undefined) value[field] = Math.min(value[field], options.maxOutputTokens);
          }
          if (fields.every(field => value[field] === undefined)) value[fields[0]] = options.maxOutputTokens;
          if (wire === 'responses') value.store = false;
          if (wire === 'chat') {
            const allowed = new Set(chatClient === 'opencode' ? (options.purpose === 'review' ? ['read', 'glob', 'grep'] : ['read', 'glob', 'grep', 'edit', 'write', 'apply_patch', 'skill', 'evaluation_fixture_evaluation_fixture']) : ['read_file', 'write_file', 'patch', 'search_files', 'mcp__evaluation_fixture__evaluation_fixture']);
            if (value.tools !== undefined && (!Array.isArray(value.tools) || value.tools.length > 32 || value.tools.some(tool => tool?.type !== 'function' || !allowed.has(tool.function?.name)))) {
              fail(403, 'Native tool routing unavailable'); return;
            }
            if (options.purpose === 'review' && value.tools?.some(tool => (tool.function.name.startsWith('mcp_') || tool.function.name === 'evaluation_fixture_evaluation_fixture'))) { fail(403, 'Review fixture routing unavailable'); return; }
            const calls = new Map();
            for (const message of Array.isArray(value.messages) ? value.messages : []) {
              for (const call of Array.isArray(message.tool_calls) ? message.tool_calls : []) {
                if (typeof call.id === 'string' && call.id.length <= 256 && allowed.has(call.function?.name)) calls.set(call.id, call.function.name);
              }
              if (message.role === 'tool' && calls.has(message.tool_call_id)) {
                const name = calls.get(message.tool_call_id);
                if (toolEvidence.has(message.tool_call_id) && toolEvidence.get(message.tool_call_id) !== name) { fail(400, 'Conflicting native tool attribution'); return; }
                toolEvidence.set(message.tool_call_id, name);
              }
            }
            if (toolEvidence.size > 1000) { fail(413, 'Native tool evidence limit reached'); return; }
          }
        } else {
          if (!['initialize', 'notifications/initialized', 'tools/list', 'tools/call', 'ping'].includes(value.method) ||
              (value.method === 'tools/call' && value.params?.name !== 'evaluation_fixture')) {
            fail(403, 'Fixture operation unavailable'); return;
          }
        }
        body = Buffer.from(JSON.stringify(value));
      }
      if (body.length > MAX_BODY) { fail(413, 'Serialized native request body limit reached'); return; }
      const headers = {
        authorization: `Bearer ${options.token}`, 'x-session-id': options.session,
        'content-type': 'application/json', accept: provider ? 'text/event-stream' : 'application/json, text/event-stream',
        'content-length': body.length,
      };
      if (provider) headers['x-inference-protocol'] = wire === 'responses' ? 'openai-responses' : 'openai-chat';
      for (const key of ['mcp-session-id', 'mcp-protocol-version']) {
        const value = request.headers[key];
        if (mcp && typeof value === 'string' && value.length <= 256 && !/[\x00-\x1f\x7f]/.test(value)) headers[key] = value;
      }
      outgoing = transport({ hostname: upstream.hostname, port: 8090, path: request.url, method: request.method, headers, agent: false }, incoming => {
        if (incoming.statusCode >= 300 && incoming.statusCode < 400) { incoming.destroy(); fail(502, 'Relay redirects are forbidden'); return; }
        const outputHeaders = { 'content-type': incoming.headers['content-type'] || 'application/json' };
        for (const key of ['mcp-session-id', 'mcp-protocol-version', 'x-request-id']) {
          const value = incoming.headers[key];
          if (typeof value === 'string' && value.length <= 256 && !/[\x00-\x1f\x7f]/.test(value)) outputHeaders[key] = value;
        }
        response.writeHead(incoming.statusCode || 502, outputHeaders);
        let total = 0;
        incoming.on('data', chunk => {
          total += chunk.length;
          if (total > MAX_RESPONSE) { incoming.destroy(); fail(502, 'Native response body limit reached'); return; }
          if (!response.write(chunk)) incoming.pause();
        });
        response.on('drain', () => incoming.resume());
        incoming.once('end', () => response.end());
        incoming.once('error', () => fail(502, 'Relay response failed'));
      });
      pending.add(outgoing);
      outgoing.once('close', () => pending.delete(outgoing));
      outgoing.once('error', () => fail(502, 'Execution relay unavailable'));
      outgoing.setTimeout(upstreamTimeoutMs, () => { outgoing.destroy(); fail(504, 'Execution relay timed out'); });
      outgoing.end(body);
    } catch { fail(400, 'Native request interrupted'); }
    finally { if (response.writableEnded || response.destroyed) release(); }
  });
  server.requestTimeout = 30000;
  server.headersTimeout = 10000;
  server.maxHeadersCount = 32;
  server.maxConnections = 8;
  server.on('clientError', (_error, socket) => socket.destroy());
  return {
    server,
    close: () => new Promise(resolve => {
      for (const request of pending) request.destroy();
      server.closeAllConnections();
      server.close(resolve);
    }),
    attempts: () => attempts,
    toolCalls: () => [...toolEvidence.values()],
  };
}

// Native deny mounts need existing directory targets; absent .codex paths can
// otherwise become synthetic file mounts in the pinned Linux sandbox.
function prepareDeniedDirectories(home = '/home/tester') {
  for (const directory of [path.join(home, '.codex'), path.join(home, 'work', '.codex')]) {
    try { fs.mkdirSync(directory, { mode: 0o700 }); }
    catch (error) { if (error.code !== 'EEXIST') throw error; }
    const metadata = fs.lstatSync(directory);
    if (!metadata.isDirectory() || metadata.isSymbolicLink()) throw new Error('Reserved Codex directory is not a regular directory');
  }
}

async function main(argv, source = process.env) {
  if (argv.length < 7 || argv[4] !== '--' || argv[5] !== 'exec') throw new Error('Invalid Codex runner arguments');
  const localToken = crypto.randomBytes(32).toString('hex');
  const relay = createRelay({ relay: source.CODEX_EVALUATION_RELAY, token: source.CODEX_EVALUATION_TOKEN,
    session: source.CODEX_EVALUATION_SESSION, model: argv[2], purpose: argv[3],
    maxRequests: Number(argv[0]), maxOutputTokens: Number(argv[1]), localToken });
  await new Promise((resolve, reject) => { relay.server.once('error', reject); relay.server.listen(8091, '127.0.0.1', resolve); });
  let child;
  try {
    prepareDeniedDirectories();
    const environment = childEnvironment(source, localToken);
    child = spawn(EXECUTABLE, nativeArguments(argv.slice(5), environment), { env: environment, stdio: ['ignore', 'inherit', 'inherit'], detached: true });
    const terminate = () => {
      try { process.kill(-child.pid, 'SIGTERM'); } catch {}
      const force = setTimeout(() => { try { process.kill(-child.pid, 'SIGKILL'); } catch {} }, 2000);
      force.unref();
    };
    process.once('SIGINT', terminate); process.once('SIGTERM', terminate);
    try {
      const result = await new Promise((resolve, reject) => { child.once('error', reject); child.once('exit', (code, signal) => resolve(signal ? 1 : (code ?? 1))); });
      process.exitCode = result;
    } finally { process.removeListener('SIGINT', terminate); process.removeListener('SIGTERM', terminate); terminate(); }
  } finally { await relay.close(); }
}

module.exports = { createRelay, childEnvironment, nativeArguments, validateOptions, prepareDeniedDirectories };
if (require.main === module) main(process.argv.slice(2)).catch(() => {
  process.stdout.write(JSON.stringify({ type: 'error', message: 'Codex runner failed before verified completion' }) + '\n');
  process.exitCode = 1;
});
