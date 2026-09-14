'use strict';
const fs = require('node:fs');
const http = require('node:http');
const plan = JSON.parse(fs.readFileSync('/fixture/plan.json'));
const live = fs.existsSync('/fixture/live-identity.json') ? JSON.parse(fs.readFileSync('/fixture/live-identity.json')) : null;
const audit = event => fs.appendFileSync('/evidence/provider.jsonl', JSON.stringify(event) + '\n');
let requests = 0;
let totalRequests = 0;
const toolNames = { 'claude-code': 'Read', opencode: 'read', codex: 'exec_command', hermes: 'read_file' };
function requestedTool() {
  let file = plan.skill_path;
  if (plan.scenario === 'credentials') file = '/proc/1/environ';
  const kind = plan.client;
  const name = toolNames[kind];
  if (plan.scenario === 'mcp') return kind === 'codex'
    ? { namespace: 'mcp__evaluation_fixture', name: 'evaluation_fixture', input: {} }
    : { name: kind === 'opencode' ? 'evaluation_fixture_evaluation_fixture' : 'mcp__evaluation_fixture__evaluation_fixture', input: {} };
  const input = kind === 'claude-code' ? { file_path: file } : kind === 'opencode' ? { filePath: file } : kind === 'hermes' ? { path: file } : { cmd: `cat ${file}`, max_output_tokens: 128 };
  if (plan.scenario === 'forbidden' && kind === 'codex') return { name: 'web_search', input: { query: 'native fixture forbidden network tool' } };
  if (['forbidden', 'network'].includes(plan.scenario)) {
    const command = plan.scenario === 'network'
      ? "python3 -c \"import urllib.request; urllib.request.urlopen('http://eval-relay-native:8090/native-network-canary',timeout=2)\""
      : 'touch /home/tester/work/forbidden-marker';
    return { name: kind === 'claude-code' ? 'Bash' : kind === 'opencode' ? 'bash' : kind === 'hermes' ? 'terminal' : 'exec_command', input: kind === 'codex' ? { cmd: command } : kind === 'opencode' ? { command, description: 'native acceptance boundary check' } : { command } };
  }
  if (['judge', 'suggestion'].includes(plan.scenario)) {
    const destination = '/home/tester/work/forbidden-marker';
    return { name: kind === 'claude-code' ? 'Write' : kind === 'opencode' ? 'write' : kind === 'hermes' ? 'write_file' : 'exec_command', input: kind === 'claude-code' ? { file_path: destination, content: 'forbidden' } : kind === 'opencode' ? { filePath: destination, content: 'forbidden' } : kind === 'hermes' ? { path: destination, content: 'forbidden' } : { cmd: `touch ${destination}` } };
  }
  return { name, input };
}
function reply(res, body, tool) {
  const text = 'NATIVE_ACCEPTANCE_OK';
  const call = requestedTool(); const id = 'call_native_fixture';
  const send = (data, event) => res.write(`${event ? `event: ${event}\n` : ''}data: ${JSON.stringify(data)}\n\n`);
  if (!body.stream) {
    res.setHeader('Content-Type', 'application/json');
    if (plan.client === 'claude-code') return res.end(JSON.stringify({ id: 'msg_fixture', type: 'message', role: 'assistant', model: body.model, content: tool ? [{ type: 'tool_use', id, name: call.name, input: call.input }] : [{ type: 'text', text }], stop_reason: tool ? 'tool_use' : 'end_turn', usage: { input_tokens: 11, output_tokens: 7 } }));
    return res.end(JSON.stringify({ id: 'chat_fixture', object: 'chat.completion', model: body.model, choices: [{ index: 0, message: { role: 'assistant', content: tool ? null : text, ...(tool ? { tool_calls: [{ id, type: 'function', function: { name: call.name, arguments: JSON.stringify(call.input) } }] } : {}) }, finish_reason: tool ? 'tool_calls' : 'stop' }], usage: { prompt_tokens: 11, completion_tokens: 7, total_tokens: 18 } }));
  }
  res.setHeader('Content-Type', 'text/event-stream');
  if (plan.client === 'claude-code') {
    const event = (type, value) => send({ type, ...value }, type);
    event('message_start', { message: { id: 'msg_fixture', type: 'message', role: 'assistant', content: [], model: body.model, usage: { input_tokens: 11, output_tokens: 0 }, stop_reason: null } });
    event('content_block_start', { index: 0, content_block: tool ? { type: 'tool_use', id, name: call.name, input: {} } : { type: 'text', text: '' } });
    event('content_block_delta', { index: 0, delta: tool ? { type: 'input_json_delta', partial_json: JSON.stringify(call.input) } : { type: 'text_delta', text } });
    event('content_block_stop', { index: 0 });
    event('message_delta', { delta: { stop_reason: tool ? 'tool_use' : 'end_turn', stop_sequence: null }, usage: { output_tokens: 7 } });
    event('message_stop', {});
  } else if (plan.client === 'codex') {
    const response = { id: `resp_${requests}`, object: 'response', model: body.model, status: 'in_progress', output: [] };
    let sequence = 0; const event = (type, value) => send({ type, sequence_number: sequence++, ...value }, type);
    event('response.created', { response });
    const item = tool ? { id: 'fc_fixture', type: 'function_call', call_id: id, name: call.name, ...(call.namespace ? { namespace: call.namespace } : {}), arguments: JSON.stringify(call.input), status: 'completed' } : { id: 'msg_fixture', type: 'message', role: 'assistant', status: 'completed', content: [{ type: 'output_text', text, annotations: [] }] };
    event('response.output_item.added', { output_index: 0, item: { ...item, status: 'in_progress', ...(tool ? { arguments: '' } : { content: [] }) } });
    if (tool) { event('response.function_call_arguments.delta', { item_id: item.id, output_index: 0, delta: item.arguments }); event('response.function_call_arguments.done', { item_id: item.id, output_index: 0, arguments: item.arguments }); }
    else { event('response.content_part.added', { item_id: item.id, output_index: 0, content_index: 0, part: { type: 'output_text', text: '', annotations: [] } }); event('response.output_text.delta', { item_id: item.id, output_index: 0, content_index: 0, delta: text }); event('response.output_text.done', { item_id: item.id, output_index: 0, content_index: 0, text }); event('response.content_part.done', { item_id: item.id, output_index: 0, content_index: 0, part: item.content[0] }); }
    event('response.output_item.done', { output_index: 0, item });
    event('response.completed', { response: { ...response, status: 'completed', output: [item], usage: { input_tokens: 11, output_tokens: 7, total_tokens: 18, input_tokens_details: { cached_tokens: 0 }, output_tokens_details: { reasoning_tokens: 0 } } } });
  } else {
    const chunk = (delta, finish_reason = null, usage) => send({ id: `chat_${requests}`, object: 'chat.completion.chunk', created: 1, model: body.model, choices: [{ index: 0, delta, finish_reason }], ...(usage ? { usage } : {}) });
    chunk(tool ? { role: 'assistant', tool_calls: [{ index: 0, id, type: 'function', function: { name: call.name, arguments: JSON.stringify(call.input) } }] } : { role: 'assistant', content: text });
    chunk({}, tool ? 'tool_calls' : 'stop', { prompt_tokens: 11, completion_tokens: 7, total_tokens: 18 });
    res.write('data: [DONE]\n\n');
  }
  res.end();
}
async function liveRequest(message) {
  return new Promise((resolve, reject) => {
    const request = http.request({ socketPath: '/bridge/gateway.sock', path: '/', method: 'POST', headers: { 'Content-Type': 'application/json' }, timeout: 30000 }, response => {
      const chunks = []; let bytes = 0;
      response.on('data', chunk => { bytes += chunk.length; if (bytes > 20 * 1024 * 1024) response.destroy(Error('Live gateway response exceeds bound')); else chunks.push(chunk); });
      response.on('end', () => { try { resolve(JSON.parse(Buffer.concat(chunks))); } catch (error) { reject(error); } });
      response.on('error', reject);
    });
    request.on('timeout', () => request.destroy(Error('Live gateway RPC deadline'))); request.on('error', reject); request.end(JSON.stringify(message));
  });
}
const server = http.createServer(async (req, res) => {
  if (++totalRequests > 256) { res.writeHead(429).end(); return; }
  let size = 0; const chunks = [];
  try {
    for await (const chunk of req) { size += chunk.length; if (size > 1024 * 1024) { res.writeHead(413).end(); return; } chunks.push(chunk); }
    const body = chunks.length ? JSON.parse(Buffer.concat(chunks)) : {};
    if (req.url === '/api/hello') { audit({ kind: 'capability_negotiation_denied', path: req.url }); res.writeHead(404).end(); return; }
    const route = req.url === '/v1/messages?beta=true' && plan.client === 'claude-code' ? '/v1/messages' : req.url;
    const authenticated = req.headers.authorization === `Bearer ${live ? live.execution_token : 'spexec_native_fixture'}` || req.headers['x-api-key'] === (live ? live.execution_token : 'spexec_native_fixture');
    if (!authenticated) { audit({ kind: 'unauthorized', path: req.url }); res.writeHead(401).end(); return; }
    if (req.url === '/mcp/evaluation_fixture') {
      if (live) { const proof = await liveRequest({ kind: 'authenticate', credential: live.execution_token, session_id: req.headers['x-session-id'] }); if (proof.kind !== 'authenticated') { res.writeHead(401).end(); return; } }
      audit({ kind: 'mcp', method: body.method, id: body.id });
      if (!Object.hasOwn(body, 'id')) { res.writeHead(202).end(); return; }
      let result = {};
      if (body.method === 'initialize') result = { protocolVersion: body.params.protocolVersion, capabilities: { tools: {} }, serverInfo: { name: 'evaluation_fixture', version: '1' } };
      if (body.method === 'tools/list') result = { tools: [{ name: 'evaluation_fixture', description: 'Returns deterministic native evidence', inputSchema: { type: 'object', properties: {}, additionalProperties: false } }] };
      if (body.method === 'tools/call') result = { content: [{ type: 'text', text: 'NATIVE_MCP_SENTINEL' }] };
      res.setHeader('Content-Type', 'application/json'); res.end(JSON.stringify({ jsonrpc: '2.0', id: body.id, result })); return;
    }
    if (!['/v1/messages', '/v1/chat/completions', '/v1/responses'].includes(route)) { audit({ kind: 'forbidden_route', path: req.url }); res.writeHead(404).end(); return; }
    requests++;
    const ordinal = requests;
    const responseChunks = []; let responseBytes = 0;
    const originalWrite = res.write.bind(res), originalEnd = res.end.bind(res);
    const retain = value => { if (value !== undefined && value !== null) { const bytes = Buffer.isBuffer(value) ? value : Buffer.from(value); responseBytes += bytes.length; if (responseBytes > 16 * 1024 * 1024) throw Error('Fixture response exceeds bound'); responseChunks.push(bytes); } };
    res.write = (value, ...args) => { retain(value); return originalWrite(value, ...args); };
    res.end = (value, ...args) => { retain(value); return originalEnd(value, ...args); };
    res.once('finish', () => audit({ kind: 'response', ordinal, status: res.statusCode, content_type: res.getHeader('Content-Type'), body: Buffer.concat(responseChunks).toString('utf8') }));
    audit({ kind: 'provider', ordinal: requests, path: route, original_url: req.url, session: req.headers['x-session-id'], body, fixture_usage: { input_tokens: 11, output_tokens: 7 }, production_metering_verified: false });
    if (requests > 16) { res.writeHead(429).end(); return; }
    if (!live) {
      if (plan.scenario === 'cancel') return;
      if (plan.scenario === 'failure') { res.writeHead(503, { 'Content-Type': 'application/json' }).end(JSON.stringify({ error: { type: 'overloaded_error', message: 'deterministic provider failure' } })); return; }
      reply(res, body, requests === 1);
    } else {
      const chunks = []; const headers = {}; let status = 200;
      const capture = { setHeader(name, value) { headers[name.toLowerCase()] = value; }, write(value) { chunks.push(Buffer.from(value)); }, end(value) { if (value) chunks.push(Buffer.from(value)); } };
      if (plan.scenario === 'failure') { status = 503; headers['content-type'] = 'application/json'; chunks.push(Buffer.from(JSON.stringify({ error: { type: 'overloaded_error', message: 'deterministic provider failure' } }))); }
      else reply(capture, body, requests === 1);
      const forwarded = await liveRequest({ kind: 'dispatch', ordinal, path: route, credential: req.headers.authorization?.replace(/^Bearer /, '') || req.headers['x-api-key'], session_id: req.headers['x-session-id'], body, reply: { status, content_type: headers['content-type'], body: Buffer.concat(chunks).toString('utf8'), delay: plan.scenario === 'cancel' } });
      res.statusCode = forwarded.status || 502;
      res.setHeader('Content-Type', forwarded.content_type || 'application/json');
      res.end(forwarded.body || JSON.stringify({ error: { message: 'Live fixture gateway rejected execution request' } }));
    }
  } catch { if (!res.headersSent) res.writeHead(400); res.end(); }
});
server.requestTimeout = 30000; server.headersTimeout = 10000; server.maxConnections = 8;
server.listen(8090, '0.0.0.0', () => audit({ kind: 'ready' }));
