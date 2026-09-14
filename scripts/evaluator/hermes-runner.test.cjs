'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const http = require('node:http');
const { nativeConfiguration, nativeEnvironment, sandboxArguments, readUsage, resultEvidence } = require('../../deploy/evaluator/hermes-runner.cjs');
const { createRelay } = require('../../deploy/evaluator/codex-runner.cjs');
const options = { relay: 'http://eval-relay-fixture:8090', token: 'scope-token', session: 'session',
  model: 'fixture', maxRequests: 4, maxOutputTokens: 512, localToken: 'b'.repeat(64), purpose: 'execution', wire: 'chat' };
const listen = server => new Promise(resolve => server.listen(0, '127.0.0.1', resolve));

test('Hermes native namespace hides parent processes and locks config and skills', () => {
  const args = sandboxArguments('/tmp/config', '/tmp/evidence', '/tmp/skills', 'execution', ['--oneshot=case']);
  assert.ok(args.includes('--unshare-pid')); assert.ok(args.includes('--die-with-parent'));
  assert.deepEqual(args.slice(args.indexOf('--proc'), args.indexOf('--proc') + 2), ['--proc', '/proc']);
  assert.deepEqual(args.slice(args.indexOf('/tmp/skills') - 1, args.indexOf('/tmp/skills') + 2), ['--ro-bind', '/tmp/skills', '/home/tester/.hermes/skills']);
  const review = sandboxArguments('/tmp/config', '/tmp/evidence', '/tmp/empty', 'review', []);
  const work = review.indexOf('/home/tester/work'); assert.equal(review[work - 1], '--ro-bind');
  const env = nativeEnvironment({ HERMES_EVALUATION_TOKEN: 'secret', OPENAI_API_KEY: 'host', HTTPS_PROXY: 'host', TZ: 'Asia/Tokyo' }, 'local');
  assert.equal(env.OPENAI_API_KEY, undefined); assert.equal(env.OPENAI_BASE_URL, undefined); assert.equal(env.HERMES_EVALUATION_TOKEN, undefined); assert.equal(env.HTTPS_PROXY, undefined); assert.equal(env.TZ, 'Asia/Tokyo');
});

test('native usage symlinks, hardlinks, oversized and malformed files are rejected', t => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'hermes-usage-test-')); t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  const report = path.join(directory, 'report'); fs.writeFileSync(report, '{"completed":true}');
  assert.equal(readUsage(report).value.completed, true);
  const symlink = path.join(directory, 'link'); fs.symlinkSync(report, symlink); assert.throws(() => readUsage(symlink));
  const hardlink = path.join(directory, 'hardlink'); fs.linkSync(report, hardlink); assert.throws(() => readUsage(report)); fs.unlinkSync(hardlink);
  fs.writeFileSync(report, 'x'.repeat(65537)); assert.throws(() => readUsage(report));
  fs.writeFileSync(report, 'malformed'); assert.throws(() => readUsage(report));
});

test('usage evidence never claims independent metering or overrides failed native exit', () => {
  const usage = { value: { model: 'fixture', session_id: 'native', completed: true, failed: false, input_tokens: 10 }, sha256: 'a'.repeat(64) };
  const failed = resultEvidence(usage, { code: 1, signal: null }, 'fixture', ['read_file']);
  assert.equal(failed.process_exit_code, 1); assert.equal(failed.usage.completed, true); assert.equal(failed.usage_independently_verified, false);
  assert.equal(resultEvidence(usage, { code: 0, signal: null }, 'different', []).usage_identity_matches, false);
  assert.equal(resultEvidence(null, { code: 0, signal: null }, 'fixture', []).usage_identity_matches, false);
});

test('chat wire preserves route isolation, clamps both chat limits and rejects unsafe tools', async t => {
  const seen = [];
  const upstream = http.createServer(async (req, res) => {
    const chunks = []; for await (const chunk of req) chunks.push(chunk);
    seen.push({ path: req.url, headers: req.headers, body: JSON.parse(Buffer.concat(chunks)) }); res.end('{}');
  });
  await listen(upstream);
  const relay = createRelay(options, (request, callback) => http.request({ ...request, hostname: '127.0.0.1', port: upstream.address().port }, callback));
  await listen(relay.server);
  t.after(async () => { await relay.close(); upstream.closeAllConnections(); await new Promise(resolve => upstream.close(resolve)); });
  const send = (body, route = '/v1/chat/completions') => new Promise((resolve, reject) => {
    const req = http.request({ hostname: '127.0.0.1', port: relay.server.address().port, path: route, method: 'POST', headers: { authorization: `Bearer ${options.localToken}` } }, res => {
      res.resume(); res.once('end', () => resolve(res.statusCode));
    }); req.once('error', reject); req.end(JSON.stringify(body));
  });
  assert.equal(await send({ model: 'fixture' }, '/v1/responses'), 404);
  assert.equal(await send({ model: 'fixture', max_output_tokens: 512 }), 400);
  assert.equal(await send({ model: 'fixture', tools: [{ type: 'function', function: { name: 'terminal' } }] }), 403);
  assert.equal(await send({ model: 'fixture', max_tokens: 9000, max_completion_tokens: 256,
    tools: [{ type: 'function', function: { name: 'read_file' } }],
    messages: [{ role: 'assistant', tool_calls: [{ id: 'call1', function: { name: 'read_file' } }] }, { role: 'tool', tool_call_id: 'call1', content: 'file' }] }), 200);
  assert.equal(seen.length, 1); assert.equal(seen[0].body.max_tokens, 512); assert.equal(seen[0].body.max_completion_tokens, 256);
  assert.equal(seen[0].path, '/v1/chat/completions'); assert.equal(seen[0].headers['x-inference-protocol'], 'openai-chat'); assert.deepEqual(relay.toolCalls(), ['read_file']);
  assert.throws(() => createRelay({ ...options, wire: 'arbitrary' }));
});


test('Hermes custom-provider credentials are explicit, local, rotated and absent from child environment', () => {
  const source = { model: { provider: 'cloud', base_url: 'https://invalid.example', api_key: 'stale-host-key' },
    fallback_providers: ['cloud'], mcp_servers: { ambient: { url: 'https://invalid.example' } }, agent: { disabled_toolsets: ['terminal'] } };
  const first = nativeConfiguration(source, { turns: 2, model: 'fixture', purpose: 'execution', localToken: options.localToken });
  const rotated = nativeConfiguration(source, { turns: 2, model: 'fixture', purpose: 'review', localToken: 'c'.repeat(64) });
  assert.equal(first.model.api_key, options.localToken);
  assert.equal(first.model.provider, 'custom'); assert.equal(first.model.base_url, 'http://127.0.0.1:8091/v1');
  assert.equal(first.model.api_mode, 'chat_completions');
  assert.equal(first.mcp_servers.evaluation_fixture.headers.Authorization, `Bearer ${first.model.api_key}`);
  assert.equal(rotated.model.api_key, 'c'.repeat(64)); assert.deepEqual(rotated.mcp_servers, {});
  assert.deepEqual(first.agent.disabled_toolsets, ['terminal']); assert.deepEqual(first.fallback_providers, []);
  assert.equal(source.model.api_key, 'stale-host-key', 'runtime token does not mutate retained configuration');
  assert.ok(!JSON.stringify(first).includes('stale-host-key'));
  const env = nativeEnvironment({ HERMES_EVALUATION_TOKEN: 'upstream-secret', OPENAI_API_KEY: 'host-key' }, options.localToken);
  assert.equal(env.HERMES_EVALUATION_TOKEN, undefined); assert.equal(env.OPENAI_API_KEY, undefined);
  assert.ok(!Object.values(env).includes(options.localToken));
  assert.throws(() => nativeConfiguration(source, { turns: 2, model: 'fixture', purpose: 'execution', localToken: 'no-key-required' }));
});

test('the configured Hermes bearer reaches relay; placeholder and previous tokens cannot', async t => {
  const seen = [];
  const upstream = http.createServer(async (req, res) => {
    for await (const _chunk of req) { /* consume fixture request */ }
    seen.push(req.headers.authorization); res.end('{}');
  });
  await listen(upstream);
  const relay = createRelay(options, (request, callback) => http.request({ ...request, hostname: '127.0.0.1', port: upstream.address().port }, callback));
  await listen(relay.server);
  t.after(async () => { await relay.close(); upstream.closeAllConnections(); await new Promise(resolve => upstream.close(resolve)); });
  const config = nativeConfiguration({}, { turns: 2, model: 'fixture', purpose: 'execution', localToken: options.localToken });
  const send = token => new Promise((resolve, reject) => {
    const req = http.request({ hostname: '127.0.0.1', port: relay.server.address().port, path: '/v1/chat/completions', method: 'POST', headers: { authorization: `Bearer ${token}` } }, res => {
      res.resume(); res.once('end', () => resolve(res.statusCode));
    }); req.once('error', reject); req.end(JSON.stringify({ model: 'fixture', messages: [] }));
  });
  assert.equal(await send('no-key-required'), 401);
  assert.equal(await send('c'.repeat(64)), 401);
  assert.equal(await send(config.model.api_key), 200);
  assert.deepEqual(seen, [`Bearer ${options.token}`], 'parent swaps local token for scoped execution capability only upstream');
});
