'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const http = require('node:http');
const { createRelay, childEnvironment, nativeArguments, validateOptions } = require('../../deploy/evaluator/codex-runner.cjs');

const options = () => ({ relay: 'http://eval-relay-fixture:8090', token: 'spexec_fixture.signature',
  session: 'fixture-session', model: 'fixture-model', maxRequests: 2, maxOutputTokens: 512,
  localToken: 'a'.repeat(64), purpose: 'execution' });
const listen = server => new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
async function fixture(t, handler, settings = {}) {
  const upstream = http.createServer(handler);
  await listen(upstream);
  const relay = createRelay({ ...options(), ...settings }, (request, callback) => http.request({ ...request,
    hostname: '127.0.0.1', port: upstream.address().port }, callback));
  await listen(relay.server);
  t.after(async () => { await relay.close(); upstream.closeAllConnections(); await new Promise(resolve => upstream.close(resolve)); });
  const send = (body, path = '/v1/responses', headers = {}, method = 'POST') => new Promise((resolve, reject) => {
    const request = http.request({ hostname: '127.0.0.1', port: relay.server.address().port, method, path,
      headers: { authorization: `Bearer ${options().localToken}`, 'content-type': 'application/json', ...headers } }, response => {
      const chunks = []; response.on('data', chunk => chunks.push(chunk));
      response.on('end', () => resolve({ status: response.statusCode, body: Buffer.concat(chunks).toString() }));
      response.on('error', reject);
    });
    request.on('error', reject);
    request.end(typeof body === 'string' ? body : JSON.stringify(body));
  });
  return { relay, send };
}

test('provider bounds and authoritative credentials precede upstream forwarding', async t => {
  const observed = [];
  const { relay, send } = await fixture(t, async (req, res) => {
    const chunks = []; for await (const chunk of req) chunks.push(chunk);
    observed.push({ headers: req.headers, body: JSON.parse(Buffer.concat(chunks)) });
    res.writeHead(500); res.end('{}');
  });
  assert.equal((await send({ model: 'fixture-model', max_output_tokens: 10000 }, undefined,
    { 'x-session-id': 'forged', authorization: `Bearer ${options().localToken}` })).status, 500);
  assert.equal((await send({ model: 'fixture-model', max_output_tokens: 256 })).status, 500);
  assert.equal((await send({ model: 'fixture-model' })).status, 429);
  assert.equal(relay.attempts(), 3);
  assert.equal(observed.length, 2);
  assert.equal(observed[0].body.max_output_tokens, 512);
  assert.equal(observed[1].body.max_output_tokens, 256);
  assert.equal(observed[0].body.store, false);
  assert.equal(observed[0].headers.authorization, 'Bearer spexec_fixture.signature');
  assert.equal(observed[0].headers['x-session-id'], 'fixture-session');
  assert.equal(observed[0].headers['x-inference-protocol'], 'openai-responses');
});

test('forged local auth, paths, models and malformed limits never reach upstream', async t => {
  let calls = 0;
  const { send } = await fixture(t, (_req, res) => { calls++; res.end('{}'); }, { maxRequests: 10 });
  assert.equal((await send({}, undefined, { authorization: 'Bearer forged' })).status, 401);
  assert.equal((await send({}, 'http://evil.invalid/v1/responses')).status, 404);
  assert.equal((await send({}, '/v1/responses?forward=evil')).status, 404);
  assert.equal((await send({ model: 'other' })).status, 400);
  assert.equal((await send({ model: 'fixture-model', max_output_tokens: -1 })).status, 400);
  assert.equal((await send('not-json')).status, 400);
  assert.equal((await send('x'.repeat(1024 * 1024 + 1))).status, 413);
  assert.equal(calls, 0);
});

test('MCP allowlist rejects unrelated tools and review-purpose fixture access', async t => {
  let calls = 0;
  const { send } = await fixture(t, (_req, res) => { calls++; res.end('{}'); });
  assert.equal((await send({ jsonrpc: '2.0', method: 'tools/call', params: { name: 'shell' } }, '/mcp/evaluation_fixture')).status, 403);
  assert.equal((await send({ jsonrpc: '2.0', method: 'tools/call', params: { name: 'evaluation_fixture' } }, '/mcp/evaluation_fixture')).status, 200);
  assert.equal(calls, 1);
  const review = await fixture(t, (_req, res) => res.end('{}'), { purpose: 'review' });
  assert.equal((await review.send({ method: 'initialize' }, '/mcp/evaluation_fixture')).status, 403);
});

test('upstream redirects and timeout errors retain generic bounded diagnostics', async t => {
  const redirects = await fixture(t, (_req, res) => { res.writeHead(307, { location: 'http://evil.invalid/?token=secret' }); res.end(); });
  const result = await redirects.send({ model: 'fixture-model' });
  assert.equal(result.status, 502);
  assert.ok(!result.body.includes('secret'));
  const stalled = await fixture(t, () => {}, { upstreamTimeoutMs: 20 });
  assert.equal((await stalled.send({ model: 'fixture-model' })).status, 504);
});

test('native child receives no host or upstream credentials and relay config is exact', () => {
  const env = childEnvironment({ HOME: '/host', OPENAI_API_KEY: 'host-secret', HTTPS_PROXY: 'host-proxy',
    CODEX_EVALUATION_TOKEN: 'upstream-secret', CODEX_EVALUATION_RELAY: 'private', TZ: 'UTC' }, 'a'.repeat(64));
  assert.equal(env.HOME, '/home/tester');
  assert.equal(env.CODEX_HOME, '/home/tester/.codex');
  assert.equal(env.CODEX_LOCAL_PROXY_TOKEN, 'a'.repeat(64));
  for (const key of ['OPENAI_API_KEY', 'HTTPS_PROXY', 'CODEX_EVALUATION_TOKEN', 'CODEX_EVALUATION_RELAY']) assert.equal(env[key], undefined);
  for (const patch of [{ relay: 'https://eval-relay-fixture:8090' }, { relay: 'http://eval-relay-fixture:8090/path' },
    { token: undefined }, { session: 'session\nInjected: value' }, { maxRequests: 0 }, { maxOutputTokens: 0 }]) {
    assert.throws(() => validateOptions({ ...options(), ...patch }));
  }
});

test('shutdown cancels pending upstream work and releases the local listener', async t => {
  let started;
  const reached = new Promise(resolve => { started = resolve; });
  const { relay, send } = await fixture(t, () => started());
  const request = send({ model: 'fixture-model' }).catch(error => error);
  await reached;
  const address = relay.server.address();
  await relay.close();
  await request;
  assert.equal(relay.server.listening, false);
  const replacement = http.createServer();
  await new Promise((resolve, reject) => { replacement.once('error', reject); replacement.listen(address.port, '127.0.0.1', resolve); });
  await new Promise(resolve => replacement.close(resolve));
});

test('response size and concurrent forwarding stay bounded', async t => {
  const oversized = await fixture(t, (_req, res) => res.end(Buffer.alloc(16 * 1024 * 1024 + 1, 'x')));
  await assert.rejects(oversized.send({ model: 'fixture-model' }));
  let accepted = 0;
  let ready;
  const allStarted = new Promise(resolve => { ready = resolve; });
  const busy = await fixture(t, () => { if (++accepted === 4) ready(); }, { maxRequests: 10 });
  const pending = Array.from({ length: 4 }, () => busy.send({ model: 'fixture-model' }).catch(error => error));
  await allStarted;
  assert.equal((await busy.send({ model: 'fixture-model' })).status, 429);
  assert.equal(accepted, 4);
  await busy.relay.close();
  await Promise.all(pending);
});


test('serialization cannot inflate an accepted provider body beyond the forwarding bound', async t => {
  let calls = 0;
  const { send } = await fixture(t, (_req, res) => { calls++; res.end('{}'); });
  const request = { model: 'fixture-model', padding: '' };
  request.padding = 'x'.repeat(1024 * 1024 - Buffer.byteLength(JSON.stringify(request)));
  assert.equal(Buffer.byteLength(JSON.stringify(request)), 1024 * 1024);
  const result = await send(request);
  assert.equal(result.status, 413);
  assert.equal(calls, 0);
});


test('sandboxed tools receive the same frozen nonUTC timezone as native execution', () => {
  const environment = childEnvironment({ TZ: 'Asia/Tokyo', OPENAI_API_KEY: 'host-secret' }, 'a'.repeat(64));
  const args = nativeArguments(['exec', '--json', '--', 'case'], environment);
  assert.equal(environment.TZ, 'Asia/Tokyo');
  assert.deepEqual(args.slice(0, 3), ['exec', '-c', 'shell_environment_policy.set.TZ="Asia/Tokyo"']);
  assert.deepEqual(args.slice(-2), ['--', 'case']);
  assert.equal(environment.OPENAI_API_KEY, undefined);
  assert.throws(() => nativeArguments(['exec'], { TZ: 'UTC\nINJECTED=true' }));
});
