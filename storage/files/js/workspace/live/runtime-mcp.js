// Part two of the page runtime (see runtime-core.js): the `mcp` namespace.
// Calls, watches and their per-identity flights, polling and invalidation.
// A function expression the boot file calls with core's helpers.
(function governanceMcp(core) {
  'use strict';
  const {
    ask, fail, later, checkCall, identityOf, inputOf, shaped, cacheOption, DENIALS, POLL_FLOOR_MS, MAX_WATCHES,
  } = core;
  const entries = new Map();
  const groups = new Map();
  let watchCount = 0;

  const remember = (identity, result) => {
    const storedAt = result.cache && typeof result.cache.storedAt === 'number' ? result.cache.storedAt : Date.now();
    entries.set(identity, { result, storedAt });
  };

  const deliver = (group, event) => {
    group.watches.forEach((watch) => {
      if (watch.active) watch.handler(event);
    });
  };

  const callTool = (server, tool, input, options) => {
    const invalid = checkCall(server, tool, input);
    if (invalid) return Promise.reject(invalid);
    const signal = options && options.signal;
    if (signal && signal.aborted) return Promise.reject(fail('cancelled', 'the call was aborted before it started'));
    const cache = cacheOption(options);
    const params = { server, tool, input: inputOf(input), cache };
    return ask('mcp.callTool', params, signal).then((raw) => {
      const result = shaped(server, raw);
      if (cache === false) return result;
      const identity = identityOf(server, tool, input);
      remember(identity, result);
      const group = groups.get(identity);
      if (group) deliver(group, { type: 'data', result });
      return result;
    });
  };

  // One flight per identity and mode: a `peek` may answer from the viewer's
  // stored entry however old, a `refresh` always runs the tool.
  const execute = (group, mode) => {
    if (group.flights[mode]) return group.flights[mode];
    const params = { server: group.server, tool: group.tool, input: group.input, mode, cache: group.cache };
    group.flights[mode] = ask('mcp.watch', params)
      .then((raw) => {
        const result = shaped(group.server, raw);
        remember(group.identity, result);
        return result;
      })
      .finally(() => { group.flights[mode] = null; });
    return group.flights[mode];
  };

  const refresh = (group) => {
    if (group.refreshing) return group.refreshing;
    group.refreshing = execute(group, 'refresh').then(
      (result) => deliver(group, { type: 'data', result }),
      (error) => {
        if (DENIALS.includes(error.code)) entries.delete(group.identity);
        deliver(group, { type: 'error', error });
      },
    ).finally(() => { group.refreshing = null; });
    return group.refreshing;
  };

  const stale = (group, storedAt) => Date.now() - storedAt >= group.staleTime;

  const replay = (group, watch) => {
    const tell = (event) => { if (watch.active) watch.handler(event); };
    const entry = entries.get(group.identity);
    if (entry) {
      const old = stale(group, entry.storedAt);
      tell({ type: 'data', result: Object.assign({}, entry.result, { cache: { storedAt: entry.storedAt, revalidating: old } }) });
      if (old) refresh(group);
      return;
    }
    execute(group, 'peek').then(
      (result) => {
        const old = result.cache ? stale(group, result.cache.storedAt) : false;
        tell({ type: 'data', result: old ? Object.assign({}, result, { cache: { storedAt: result.cache.storedAt, revalidating: true } }) : result });
        if (old) refresh(group);
      },
      (error) => tell({ type: 'error', error }),
    );
  };

  const schedule = (group) => {
    clearInterval(group.timer);
    group.timer = null;
    const intervals = [...group.watches].map((watch) => watch.interval).filter((ms) => ms > 0);
    if (intervals.length === 0) return;
    group.timer = setInterval(() => {
      if (document.visibilityState === 'hidden') {
        group.missed = true;
        return;
      }
      refresh(group);
    }, Math.max(POLL_FLOOR_MS, Math.min(...intervals)));
  };

  document.addEventListener('visibilitychange', () => {
    if (document.visibilityState === 'hidden') return;
    groups.forEach((group) => {
      if (!group.missed) return;
      group.missed = false;
      refresh(group);
    });
  });

  const watchTool = (server, tool, input, handler, options) => {
    if (typeof handler !== 'function') throw new TypeError('watchTool needs a handler function');
    const watch = { handler, active: true, interval: 0 };
    const unsubscribe = () => {
      if (!watch.active) return;
      watch.active = false;
      const group = groups.get(watch.identity);
      if (!group || !group.watches.delete(watch)) return;
      watchCount -= 1;
      if (group.watches.size > 0) {
        schedule(group);
        return;
      }
      clearInterval(group.timer);
      groups.delete(watch.identity);
    };
    const refuse = (error) => {
      later(() => { if (watch.active) handler({ type: 'error', error }); });
      return unsubscribe;
    };
    const invalid = checkCall(server, tool, input);
    if (invalid) return refuse(invalid);
    if (watchCount >= MAX_WATCHES) return refuse(fail('bad_request', 'this view already holds 64 watches; unsubscribe unused ones', { server }));
    watch.identity = identityOf(server, tool, input);
    const known = groups.get(watch.identity);
    if (known && [...known.watches].some((other) => other.handler === handler)) {
      return refuse(fail('bad_request', 'this handler already watches this call', { server }));
    }
    const cache = cacheOption(options) || {};
    const group = known || {
      identity: watch.identity, server, tool, input: inputOf(input), watches: new Set(),
      flights: {}, refreshing: null, timer: null, missed: false, staleTime: 0, cache: {},
    };
    group.staleTime = Math.max(group.staleTime, cache.staleTime || 0);
    group.cache = { staleTime: cache.staleTime, gcTime: cache.gcTime };
    const interval = options && typeof options.refetchInterval === 'number' ? options.refetchInterval : 0;
    watch.interval = interval > 0 ? Math.max(POLL_FLOOR_MS, interval) : 0;
    group.watches.add(watch);
    groups.set(watch.identity, group);
    watchCount += 1;
    schedule(group);
    later(() => { if (watch.active) replay(group, watch); });
    return unsubscribe;
  };

  const invalidate = (server, tool, input) => {
    if (input !== undefined && (typeof server !== 'string' || typeof tool !== 'string')) {
      return Promise.reject(fail('bad_request', '`input` needs `server` and `tool`'));
    }
    const params = {};
    if (server !== undefined) params.server = server;
    if (tool !== undefined) params.tool = tool;
    if (input !== undefined) params.input = inputOf(input);
    const matches = (group) => (server === undefined || group.server === server)
      && (tool === undefined || group.tool === tool)
      && (input === undefined || group.identity === identityOf(server, tool, input));
    return ask('mcp.invalidate', params).then(() => {
      [...entries.keys()].forEach((identity) => {
        const [s, t] = identity.split('\u0000');
        if ((server === undefined || s === server) && (tool === undefined || t === tool)
          && (input === undefined || identity === identityOf(server, tool, input))) entries.delete(identity);
      });
      groups.forEach((group) => { if (matches(group)) refresh(group); });
    });
  };

  const listTools = (server) => {
    if (server !== undefined && typeof server !== 'string') return Promise.reject(fail('bad_request', '`server` is a string'));
    return ask('mcp.listTools', server === undefined ? {} : { server });
  };

  const RESERVED = ['then', 'toJSON'];
  const serverHandle = (name) => listTools(name).then((listed) => {
    const info = (listed.servers || []).find((item) => item.server === name);
    if (!info || !Array.isArray(info.tools) || info.tools.length === 0) {
      throw fail('server_not_connected', `no server "${name}" is reachable from this view`, { server: name });
    }
    if (info.authStatus === 'needs_reauth') throw fail('needs_reauth', `reconnect "${name}"`, { server: name });
    const handle = {};
    info.tools
      .map((tool) => tool.name)
      .filter((tool) => typeof tool === 'string' && !RESERVED.includes(tool) && !(tool in Object.prototype))
      .forEach((tool) => {
        handle[tool] = (input, options) => callTool(name, tool, input, options)
          .then((result) => (result.payload !== undefined ? result.payload : result));
      });
    return Object.freeze(handle);
  });

  const describeTool = (server, tool) => {
    const invalid = checkCall(server, tool, undefined);
    if (invalid) return Promise.reject(invalid);
    return ask('mcp.describeTool', { server, tool });
  };

  return { callTool, watchTool, invalidate, listTools, serverHandle, describeTool };
})
