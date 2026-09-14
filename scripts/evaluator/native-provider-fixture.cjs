'use strict';
const fs = require('node:fs');
const http = require('node:http');
const plan = JSON.parse(fs.readFileSync('/fixture/plan.json'));
const audit = event => fs.appendFileSync('/evidence/provider.jsonl', JSON.stringify(event) + '\n');
let requests = 0;
let totalRequests = 0;
const toolNames = { 'claude-code': 'Read', opencode: 'read', codex: 'exec_command', hermes: 'read_file' };
function requestedTool() {
  let file = plan.skill_path;
  if (plan.scenario === 'credentials') file = '/proc/1/environ';
  const kind = plan.client;
  const name = toolNames[kind];
  if (plan.scenario === 'mcp') return { name: kind === 'opencode' ? 'evaluation_fixture_evaluation_fixture' : 'mcp__evaluation_fixture__evaluation_fixture', input: {} };
  const input = kind === 'claude-code' ? { file_path: file } : kind === 'opencode' ? { filePath: file } : kind === 'hermes' ? { path: file } : { cmd: `cat ${file}`, max_output_tokens: 128 };
  if (plan.scenario === 'forbidden' && kind === 'codex') return { name: 'web_search', input: { query: 'native fixture forbidden network tool' } };
  if (plan.scenario === 'network' && kind === 'codex') return { name: 'exec_command', input: { cmd: "python3 -c \"import urllib.request; urllib.request.urlopen('http://eval-relay-native:8090/native-network-canary',timeout=2)\"" } };
  if (['forbidden', 'network'].includes(plan.scenario)) return { name: kind === 'claude-code' ? 'Bash' : kind === 'opencode' ? 'bash' : kind === 'hermes' ? 'terminal' : 'exec_command', input: { command: 'touch /home/tester/work/forbidden-marker', cmd: 'touch /home/tester/work/forbidden-marker' } };
  if (['judge', 'suggestion'].includes(plan.scenario)) return { name: kind === 'claude-code' ? 'Write' : kind === 'opencode' ? 'write' : kind === 'hermes' ? 'write_file' : 'exec_command', input: { file_path: '/home/tester/work/forbidden-marker', filePath: '/home/tester/work/forbidden-marker', path: '/home/tester/work/forbidden-marker', content: 'forbidden', cmd: 'touch /home/tester/work/forbidden-marker' } };
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
    const item = tool ? { id: 'fc_fixture', type: 'function_call', call_id: id, name: call.name, arguments: JSON.stringify(call.input), status: 'completed' } : { id: 'msg_fixture', type: 'message', role: 'assistant', status: 'completed', content: [{ type: 'output_text', text, annotations: [] }] };
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
const server = http.createServer(async (req, res) => {
  if (++totalRequests > 256) { res.writeHead(429).end(); return; }
  let size = 0; const chunks = [];
  try {
    for await (const chunk of req) { size += chunk.length; if (size > 1024 * 1024) { res.writeHead(413).end(); return; } chunks.push(chunk); }
    const body = chunks.length ? JSON.parse(Buffer.concat(chunks)) : {};
    const authenticated = req.headers.authorization === 'Bearer spexec_native_fixture' || req.headers['x-api-key'] === 'spexec_native_fixture';
    if (!authenticated) { audit({ kind: 'unauthorized', path: req.url }); res.writeHead(401).end(); return; }
    if (req.url === '/mcp/evaluation_fixture') {
      audit({ kind: 'mcp', method: body.method });
      if (!body.id) { res.writeHead(202).end(); return; }
      let result = {};
      if (body.method === 'initialize') result = { protocolVersion: body.params.protocolVersion, capabilities: { tools: {} }, serverInfo: { name: 'evaluation_fixture', version: '1' } };
      if (body.method === 'tools/list') result = { tools: [{ name: 'evaluation_fixture', description: 'Returns deterministic native evidence', inputSchema: { type: 'object', properties: {}, additionalProperties: false } }] };
      if (body.method === 'tools/call') result = { content: [{ type: 'text', text: 'NATIVE_MCP_SENTINEL' }] };
      res.setHeader('Content-Type', 'application/json'); res.end(JSON.stringify({ jsonrpc: '2.0', id: body.id, result })); return;
    }
    if (!['/v1/messages', '/v1/chat/completions', '/v1/responses'].includes(req.url)) { audit({ kind: 'forbidden_route', path: req.url }); res.writeHead(404).end(); return; }
    requests++;
    const ordinal = requests;
    const responseChunks = []; let responseBytes = 0;
    const originalWrite = res.write.bind(res), originalEnd = res.end.bind(res);
    const retain = value => { if (value !== undefined && value !== null) { const bytes = Buffer.isBuffer(value) ? value : Buffer.from(value); responseBytes += bytes.length; if (responseBytes > 16 * 1024 * 1024) throw Error('Fixture response exceeds bound'); responseChunks.push(bytes); } };
    res.write = (value, ...args) => { retain(value); return originalWrite(value, ...args); };
    res.end = (value, ...args) => { retain(value); return originalEnd(value, ...args); };
    res.once('finish', () => audit({ kind: 'response', ordinal, status: res.statusCode, content_type: res.getHeader('Content-Type'), body: Buffer.concat(responseChunks).toString('utf8') }));
    audit({ kind: 'provider', ordinal: requests, path: req.url, session: req.headers['x-session-id'], body, fixture_usage: { input_tokens: 11, output_tokens: 7 }, production_metering_verified: false });
    if (requests > 16) { res.writeHead(429).end(); return; }
    if (plan.scenario === 'cancel') return;
    if (plan.scenario === 'failure') { res.writeHead(503, { 'Content-Type': 'application/json' }).end(JSON.stringify({ error: { type: 'overloaded_error', message: 'deterministic provider failure' } })); return; }
    reply(res, body, requests === 1);
  } catch { if (!res.headersSent) res.writeHead(400); res.end(); }
});
server.requestTimeout = 30000; server.headersTimeout = 10000; server.maxConnections = 8;
server.listen(8090, '0.0.0.0', () => audit({ kind: 'ready' }));
