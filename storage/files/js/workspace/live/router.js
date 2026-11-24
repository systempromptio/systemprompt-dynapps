// Routes the messages of one live page's frame to the viewer shell's
// bridge. Only a message whose source is that frame is read: anything from
// another frame, the shell itself or an extension is dropped before it is
// looked at. Two dialects arrive: the page runtime's own requests (the
// `governance-live/1` protocol behind `window.governance.use`) and MCP Apps
// JSON-RPC (`ui/initialize`, `tools/call`, size notifications), so a page
// written as an MCP App runs the same way. Replies go to that frame alone.
export const PROTOCOL = 'governance-live/1';
const HEARTBEAT_MS = 30000;
const BUDGET_MS = 130000;
const APPS_VERSION = '2026-01-26';
const CONTRACT = '0.2.99';
const RPC_METHOD_NOT_FOUND = -32601;
const RPC_INVALID_PARAMS = -32602;
const RPC_FAILED = -32000;

export const normalized = (error) => {
  if (error && typeof error.code === 'string') return error;
  return { code: 'upstream_error', message: (error && error.message) || 'the request failed' };
};

// Routes one frame's messages. `frame()` is the framed page's window; any
// message from another source (another frame, the shell itself, an
// extension) is dropped before it is read.
export const createRouter = ({ frame, bridge, onSize, onActivity }) => {
  const cancelled = new Set();
  const send = (message) => {
    const target = frame();
    if (target) target.postMessage(message, '*');
  };

  const answer = async (id, method, params) => {
    const heartbeat = setInterval(() => send({ governance: PROTOCOL, type: 'progress', id }), HEARTBEAT_MS);
    try {
      const result = await bridge.handle(method, params);
      onActivity(method, params, { result });
      if (!cancelled.delete(id)) send({ governance: PROTOCOL, type: 'reply', id, result });
    } catch (error) {
      const failure = normalized(error);
      onActivity(method, params, { error: failure });
      if (!cancelled.delete(id)) send({ governance: PROTOCOL, type: 'reply', id, error: failure });
    } finally {
      clearInterval(heartbeat);
    }
  };

  const rpc = async (data) => {
    const reply = (body) => send({ jsonrpc: '2.0', id: data.id, ...body });
    if (data.method === 'ui/initialize') {
      reply({ result: { protocolVersion: APPS_VERSION, hostInfo: { name: 'Governance', version: CONTRACT }, hostCapabilities: {}, hostContext: {} } });
      return;
    }
    if (data.method !== 'tools/call') {
      reply({ error: { code: RPC_METHOD_NOT_FOUND, message: `"${data.method}" is not supported here` } });
      return;
    }
    const params = data.params || {};
    const server = bridge.serverOf(params.name);
    if (!server) {
      reply({ error: { code: RPC_INVALID_PARAMS, message: 'the page declares no single server with this tool' } });
      return;
    }
    const call = { server, tool: params.name, input: params.arguments || {} };
    try {
      const result = await bridge.handle('mcp.callTool', call);
      onActivity('mcp.callTool', call, { result });
      reply({ result });
    } catch (error) {
      const failure = normalized(error);
      onActivity('mcp.callTool', call, { error: failure });
      reply({ error: { code: RPC_FAILED, message: failure.message, data: failure } });
    }
  };

  return (event) => {
    const source = frame();
    if (!source || event.source !== source) return;
    const data = event.data;
    if (!data || typeof data !== 'object') return;
    if (data.jsonrpc === '2.0') {
      if (data.method === 'ui/notifications/size-changed') onSize(data.params && data.params.height);
      else if (data.id !== undefined && typeof data.method === 'string') rpc(data);
      return;
    }
    if (data.governance !== PROTOCOL) return;
    if (data.type === 'hello') send({ governance: PROTOCOL, type: 'welcome', served: bridge.served(), budgetMs: BUDGET_MS });
    if (data.type === 'request' && typeof data.method === 'string') answer(data.id, data.method, data.params);
    if (data.type === 'cancel') cancelled.add(data.id);
    if (data.type === 'size') onSize(data.height);
  };
};
