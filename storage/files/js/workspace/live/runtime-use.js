// Part three of the page runtime (see runtime-core.js): every namespace
// `use()` hands out, the frozen `window.governance` with its alias
// `window.claude`, and the page height the shell is told about. A function
// expression the boot file calls with core's helpers and the mcp namespace.
(function governanceUse(core, mcp) {
  'use strict';
  const { ask, post, fail, later, served, framed, isPlainObject, CAPABILITIES } = core;
  const { callTool, watchTool, invalidate, listTools, serverHandle, describeTool } = mcp;

  const quiet = (method, params, fallback) => ask(method, params).catch(() => fallback);

  const BUILDERS = {
    permissions: () => ({
      state: (name) => ask('permissions.state', name === undefined ? {} : { name: String(name) }),
      request: (names) => ask('permissions.request', Array.isArray(names) ? { names: names.map(String) } : {}),
      manage: () => ask('permissions.manage', {}),
    }),
    mcp: () => ({ callTool, watchTool, invalidate, listTools, server: serverHandle, describeTool }),
    user: () => ({
      me: () => quiet('user.me', {}, { id: null, name: '', avatarUrl: '', color: '', email: null, isOwner: false, canEdit: false }),
      isOwner: () => quiet('user.me', {}, {}).then((me) => me.isOwner === true),
      canEdit: () => quiet('user.me', {}, {}).then((me) => me.canEdit === true),
      can: (capability) => quiet('user.can', { capability: String(capability) }, null),
      id: () => quiet('user.me', {}, {}).then((me) => me.id || null),
      name: () => quiet('user.me', {}, {}).then((me) => me.name || ''),
      avatarUrl: () => quiet('user.me', {}, {}).then((me) => (me.id ? me.avatarUrl : null)),
      email: () => quiet('user.me', {}, {}).then((me) => me.email || null),
      profiles: (ids, options) => {
        const list = [...new Set((typeof ids === 'string' ? [ids] : Array.from(ids || [])).map(String))];
        return quiet('user.profiles', { ids: list, roles: Boolean(options && options.roles) }, {});
      },
      search: (query) => quiet('user.search', { query: String(query) }, []),
    }),
    artifact: () => ({
      publish: (content) => {
        if (typeof content === 'string') return ask('artifact.publish', { html: content });
        if (isPlainObject(content)) return Promise.reject(fail('capability_disabled', 'the files form of publish is not available in this view'));
        return Promise.reject(fail('transform_error', 'publish takes the whole page as a string'));
      },
      edit: () => ask('artifact.edit', {}),
      sync: (fn) => Promise.resolve().then(() => (typeof fn === 'function' ? fn() : undefined)).then(() => {
        if (document.querySelector('artifact-sync, [artifact-sync]')) {
          throw fail('invalid_content', 'artifact sync saves nothing; publish the page instead');
        }
      }),
    }),
  };

  const namespaces = new Map();
  const namespace = (name) => {
    if (!namespaces.has(name)) namespaces.set(name, Object.freeze(BUILDERS[name]()));
    return namespaces.get(name);
  };

  const memo = new Map();
  const use = (name) => {
    const key = name === 'self' ? 'artifact' : name;
    if (!CAPABILITIES.includes(key)) return later(() => null);
    if (!memo.has(key)) {
      memo.set(key, served.then((names) => (names && names.includes(key) ? namespace(key) : null)));
    }
    return memo.get(key);
  };

  const api = Object.freeze({ use });
  Object.defineProperty(window, 'governance', { value: api, enumerable: true });
  Object.defineProperty(window, 'claude', { value: api, enumerable: true });

  if (framed && typeof ResizeObserver === 'function') {
    let last = 0;
    new ResizeObserver(() => {
      const height = document.documentElement.scrollHeight;
      if (height === last) return;
      last = height;
      post({ type: 'size', height });
    }).observe(document.documentElement);
  }
})
