// The live page runtime: `window.governance.use(name)`, with `window.claude`
// as the same frozen object, as Anthropic's artifact runtime contract 0.2.99
// defines it (names, timing, error codes). It is three files read as one
// script, in this order: runtime-core.js (this one: the transport to the
// shell and the argument rules), runtime-mcp.js and runtime-use.js. The
// Drive's live frame route inlines them at the top of the page's head and
// replaces the HOST marker at the end with `{origins, contract}`. The page
// sits in an opaque-origin sandbox with no network: everything it asks goes
// to its parent, the viewer shell, by postMessage, and only answers from
// that parent are read. No credential ever reaches this frame.
(function governanceRuntime(host) {
  'use strict';
  if (Object.getOwnPropertyDescriptor(window, 'governance')) return;

  const PROTOCOL = 'governance-live/1';
  const HOST_WAIT_MS = 10000;
  const HELLO_EVERY_MS = 1000;
  const DEFAULT_BUDGET_MS = 130000;
  const POLL_FLOOR_MS = 30000;
  const MAX_WATCHES = 64;
  const DENIALS = ['needs_reauth', 'server_not_connected', 'blocked_by_policy', 'approval_required', 'not_in_manifest', 'not_granted'];
  const CAPABILITIES = ['permissions', 'mcp', 'user', 'artifact'];
  const origins = host && Array.isArray(host.origins) ? host.origins : [];
  const framed = window.parent !== window;

  const fail = (code, message, extra) => Object.assign(new Error(message), { code, message }, extra || {});
  const later = (fn) => Promise.resolve().then(fn);

  let budgetMs = DEFAULT_BUDGET_MS;
  let sequence = 0;
  const pending = new Map();
  const post = (message) => window.parent.postMessage(Object.assign({ governance: PROTOCOL }, message), '*');

  let settleServed = null;
  const served = new Promise((resolve) => { settleServed = resolve; });
  if (!framed) {
    settleServed(null);
  } else {
    const hello = () => post({ type: 'hello', contract: host ? host.contract : null });
    hello();
    const helloTimer = setInterval(hello, HELLO_EVERY_MS);
    setTimeout(() => { clearInterval(helloTimer); settleServed(null); }, HOST_WAIT_MS);
    served.then(() => clearInterval(helloTimer));
  }

  const armBudget = (id) => {
    const entry = pending.get(id);
    if (!entry) return;
    clearTimeout(entry.timer);
    entry.timer = setTimeout(() => {
      pending.delete(id);
      entry.reject(fail('upstream_error', 'the viewer did not answer in time'));
    }, budgetMs);
  };

  window.addEventListener('message', (event) => {
    if (event.source !== window.parent) return;
    if (origins.length > 0 && !origins.includes(event.origin)) return;
    const data = event.data;
    if (!data || data.governance !== PROTOCOL) return;
    if (data.type === 'welcome') {
      if (typeof data.budgetMs === 'number' && data.budgetMs > 0) budgetMs = data.budgetMs;
      settleServed(Array.isArray(data.served) ? data.served : []);
      return;
    }
    const entry = pending.get(data.id);
    if (!entry) return;
    if (data.type === 'progress') {
      armBudget(data.id);
      return;
    }
    if (data.type !== 'reply') return;
    pending.delete(data.id);
    clearTimeout(entry.timer);
    if (data.error) entry.reject(fail(data.error.code || 'upstream_error', data.error.message || 'the request failed', data.error));
    else entry.resolve(data.result);
  });

  const ask = (method, params, signal) => new Promise((resolve, reject) => {
    const id = `${PROTOCOL}:${++sequence}`;
    pending.set(id, { resolve, reject, timer: null });
    armBudget(id);
    if (signal) {
      signal.addEventListener('abort', () => {
        const entry = pending.get(id);
        if (!entry) return;
        pending.delete(id);
        clearTimeout(entry.timer);
        post({ type: 'cancel', id });
        reject(fail('cancelled', 'the call was aborted; the tool may still have run'));
      }, { once: true });
    }
    post({ type: 'request', id, method, params });
  });

  const isPlainObject = (value) => {
    if (value === null || typeof value !== 'object') return false;
    const proto = Object.getPrototypeOf(value);
    return proto === Object.prototype || proto === null;
  };

  const plainJson = (value, depth) => {
    if (depth > 64) return false;
    if (value === null || typeof value === 'string' || typeof value === 'boolean') return true;
    if (typeof value === 'number') return Number.isFinite(value);
    if (Array.isArray(value)) return value.every((item) => plainJson(item, depth + 1));
    if (!isPlainObject(value)) return false;
    return Object.keys(value).every((key) => plainJson(value[key], depth + 1));
  };

  const hasFile = (value) => {
    if (Array.isArray(value)) return value.some(hasFile);
    if (!isPlainObject(value)) return false;
    if (Object.prototype.hasOwnProperty.call(value, '$file')) return true;
    return Object.keys(value).some((key) => hasFile(value[key]));
  };

  const canonical = (value) => {
    if (Array.isArray(value)) return `[${value.map(canonical).join(',')}]`;
    if (isPlainObject(value)) {
      return `{${Object.keys(value).sort().map((key) => `${JSON.stringify(key)}:${canonical(value[key])}`).join(',')}}`;
    }
    return JSON.stringify(value);
  };

  const inputOf = (input) => (input === undefined || input === null ? {} : input);
  const identityOf = (server, tool, input) => `${server}\u0000${tool}\u0000${canonical(inputOf(input))}`;

  const checkCall = (server, tool, input) => {
    if (typeof server !== 'string' || typeof tool !== 'string' || server === '' || tool === '') {
      return fail('bad_request', '`server` and `tool` are non-empty strings', { server });
    }
    if (input !== undefined && hasFile(input)) {
      return fail('capability_disabled', 'file arguments are not available in this view', { server });
    }
    if (input !== undefined && !plainJson(input, 0)) {
      return fail('bad_request', '`input` must be plain JSON', { server });
    }
    return null;
  };

  const firstText = (result) => {
    const block = Array.isArray(result.content) ? result.content.find((item) => item && item.type === 'text') : null;
    return block ? String(block.text) : null;
  };

  // A tool failure rejects with `tool_error` (contract 0.2.99: results no
  // longer resolve with an `isError` flag), and a success carries `payload`.
  const shaped = (server, result) => {
    if (!isPlainObject(result)) return { content: [], payload: result };
    if (result.isError === true) {
      throw fail('tool_error', firstText(result) || 'the tool reported a failure', { server, result });
    }
    const out = Object.assign({}, result);
    delete out.isError;
    if (out.structuredContent !== undefined) {
      out.payload = out.structuredContent;
      return out;
    }
    const text = firstText(out);
    if (text !== null) {
      try {
        out.payload = JSON.parse(text);
      } catch {
        out.payload = text;
      }
    }
    return out;
  };

  const cacheOption = (options) => {
    const cache = options && options.cache;
    if (cache === false) return false;
    if (!isPlainObject(cache)) return undefined;
    const tuned = {};
    ['staleTime', 'gcTime'].forEach((key) => {
      if (typeof cache[key] === 'number' && Number.isFinite(cache[key])) tuned[key] = cache[key];
    });
    if (cache.refresh === true) tuned.refresh = true;
    return tuned;
  };
