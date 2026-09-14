'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const http = require('node:http');
const { createRelay } = require('../../deploy/evaluator/codex-runner.cjs');
const { nativeEnvironment, nativeConfiguration, sandboxArguments, invocation } = require('../../deploy/evaluator/opencode-runner.cjs');
const localToken = 'a'.repeat(64);
const options = { relay: 'http://eval-relay-fixture:8090', token: 'spexec_fixture.signature', session: 'session-fixture', model: 'gpt-5',
  maxRequests: 1, maxOutputTokens: 512, localToken, purpose: 'execution', wire: 'chat', chatClient: 'opencode' };
async function fixture(t, status, override = {}) {
  const observed = [];
  const upstream = http.createServer(async (req, res) => {
    const chunks = []; for await (const chunk of req) chunks.push(chunk);
    observed.push({ headers: req.headers, body: JSON.parse(Buffer.concat(chunks)) });
    res.writeHead(status, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ choices: [{ message: { role: 'assistant', tool_calls: [{ id: 'native', type: 'function', function: { name: 'read', arguments: '{}' } }] }, finish_reason: 'tool_calls' }] }));
  });
  await new Promise(resolve => upstream.listen(0, '127.0.0.1', resolve));
  const relay = createRelay({ ...options, ...override }, (request, callback) => http.request({ ...request, hostname: '127.0.0.1', port: upstream.address().port }, callback));
  await new Promise(resolve => relay.server.listen(0, '127.0.0.1', resolve));
  t.after(async () => { await relay.close(); upstream.closeAllConnections(); await new Promise(resolve => upstream.close(resolve)); });
  const send = (body, requestPath = '/v1/chat/completions') => new Promise((resolve, reject) => {
    const req = http.request({ hostname: '127.0.0.1', port: relay.server.address().port, path: requestPath, method: 'POST', headers: { authorization: `Bearer ${localToken}` } }, res => {
      res.resume(); res.once('end', () => resolve(res.statusCode)); res.once('error', reject);
    });
    req.once('error', reject); req.end(JSON.stringify(body));
  });
  return { send, observed, relay };
}
test('OpenCode extra tool turns and failed-request retries cannot exceed the hard forwarding ceiling', async t => {
  for (const status of [200, 503]) {
    const f = await fixture(t, status);
    const body = { model: 'gpt-5', max_tokens: 10000, tools: [{ type: 'function', function: { name: 'read' } }], messages: [] };
    assert.equal(await f.send(body), status);
    assert.equal(await f.send(body), 429);
    assert.equal(f.observed.length, 1);
    assert.equal(f.relay.attempts(), 2);
    assert.equal(f.observed[0].body.max_tokens, 512);
    assert.equal(f.observed[0].headers.authorization, 'Bearer spexec_fixture.signature');
    assert.equal(f.observed[0].headers['x-session-id'], 'session-fixture');
  }
});
test('OpenCode tool profile stays distinct from Hermes and review-purpose writes', async t => {
  const f = await fixture(t, 200, { maxRequests: 10 });
  assert.equal(await f.send({ model: 'gpt-5', tools: [{ type: 'function', function: { name: 'read_file' } }] }), 403);
  assert.equal(await f.send({ model: 'gpt-5', tools: [{ type: 'function', function: { name: 'bash' } }] }), 403);
  assert.equal(await f.send({ model: 'gpt-5' }, '/v1/responses'), 404);
  const review = await fixture(t, 200, { purpose: 'review' });
  assert.equal(await review.send({ model: 'gpt-5', tools: [{ type: 'function', function: { name: 'apply_patch' } }] }), 403);
  assert.equal(f.observed.length, 0); assert.equal(review.observed.length, 0);
  assert.throws(() => createRelay({ ...options, chatClient: 'unknown' }));
});
test('OpenCode child receives only local provider credentials behind private PID and config mounts', () => {
  const base = { enabled_providers: ['systemprompt'], provider: { systemprompt: { options: { apiKey: 'upstream-secret', baseURL: 'http://eval-relay-fixture:8090/v1' } } }, mcp: { evaluation_fixture: { headers: { Authorization: 'Bearer upstream-secret' } } } };
  const config = nativeConfiguration(base, localToken, 'execution');
  assert.equal(config.provider.systemprompt.options.apiKey, localToken);
  assert.equal(config.provider.systemprompt.options.baseURL, 'http://127.0.0.1:8091/v1');
  assert.equal(config.mcp.evaluation_fixture.headers.Authorization, `Bearer ${localToken}`);
  assert.ok(!JSON.stringify(config).includes('upstream-secret'));
  assert.deepEqual(nativeConfiguration(base, 'b'.repeat(64), 'review').mcp, {});
  const env = nativeEnvironment({ OPENAI_API_KEY: 'host-secret', HTTPS_PROXY: 'host-proxy', OPENCODE_CONFIG: '/host', TZ: 'Asia/Tokyo' }, '{}');
  assert.equal(env.OPENAI_API_KEY, undefined); assert.equal(env.HTTPS_PROXY, undefined); assert.equal(env.TZ, 'Asia/Tokyo');
  assert.equal(env.OPENCODE_CONFIG, '/home/tester/.config/opencode/opencode.json');
  const args = sandboxArguments('/tmp/owned/config.json', '/tmp/owned/skills', 'review', ['--version']);
  assert.ok(args.includes('--unshare-pid')); assert.ok(args.includes('--die-with-parent'));
  assert.ok(args.some((value, i) => value === '--tmpfs' && args[i + 1] === '/home/tester/.config/opencode'));
  assert.ok(args.some((value, i) => value === '--ro-bind' && args[i + 1] === '/home/tester/work' && args[i + 2] === '/home/tester/work'));
});
test('OpenCode hard limits must match prepared native settings before launch', () => {
  const config = JSON.stringify({ agent: { evaluation: { steps: 1 } }, provider: { systemprompt: { models: { 'gpt-5': { limit: { output: 512 } } } } } });
  const args = ['1', '512', 'gpt-5', 'execution', config, '--pure', 'run', '--format', 'json', '--model', 'systemprompt/gpt-5', '--agent', 'evaluation', '--', '--untrusted-prompt'];
  assert.equal(invocation(args).turns, 1);
  assert.equal(invocation(args).nativeArgs.at(-1), '--untrusted-prompt');
  assert.throws(() => invocation(['2', ...args.slice(1)]));
  assert.throws(() => invocation([...args, '--extra']));
});
