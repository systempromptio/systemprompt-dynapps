// The viewer shell of a live Drive page: it frames the page in an
// opaque-origin sandbox (`allow-scripts` only), answers the page's runtime
// and MCP Apps messages through the bridge, and draws the viewer's prompts,
// the permissions panel and a thin status line. A message is read only when
// its source is that one frame; replies go back to the frame alone and carry
// what the page asked for, never a cookie, a token or the session. In embed
// mode (`?embed=1`) the shell's own chrome is hidden and the shell tells a
// parent window of the same origin its state and height.
import { createBridge } from './bridge.js';
import { createPrompts } from './prompts.js';
import { connectorHint } from './labels.js';
import { rawResponse } from '../../services/api.js';
import { PROTOCOL, createRouter, normalized } from './router.js';

const transport = async (url, options) => {
  const resp = await rawResponse(url, { credentials: 'same-origin', ...options });
  if (resp.status === 204) return { status: 204, body: null };
  try {
    return { status: resp.status, body: await resp.json() };
  } catch {
    return { status: resp.status, body: null };
  }
};

const el = (tag, className, attrs = {}) => Object.assign(document.createElement(tag), { className, ...attrs });

export const mountLiveShell = async (root, options) => {
  const status = el('p', 'sp-live__status', { role: 'status' });
  const frame = el('iframe', 'sp-live__frame', { title: options.title || 'Live page' });
  frame.setAttribute('sandbox', 'allow-scripts');
  frame.setAttribute('referrerpolicy', 'no-referrer');
  root.classList.add('sp-live');
  root.classList.toggle('sp-live--embed', Boolean(options.embed));
  root.replaceChildren(frame, status);

  let state = 'loading';
  let height = 0;
  const tell = () => {
    if (!options.embed || window.parent === window) return;
    window.parent.postMessage({ governance: PROTOCOL, type: 'live.status', state, height }, window.location.origin);
  };
  const show = (text, link) => {
    status.replaceChildren(text);
    if (link) status.append(' ', el('a', 'sp-live__link', { href: link.href, textContent: link.label }));
  };

  const prompts = createPrompts(root, (open) => { state = open ? 'asking' : 'ready'; tell(); });
  const load = (revision) => {
    frame.src = revision ? `${options.frameUrl}?revision=${encodeURIComponent(revision)}` : options.frameUrl;
  };
  const bridge = createBridge({
    urls: options.urls,
    transport,
    prompts,
    onPublished: (version) => setTimeout(async () => {
      await bridge.load().catch(() => null);
      load(version);
    }, 0),
    onFailure: () => {
      state = 'signed-out';
      show('Your session has ended.', { href: options.signIn, label: 'Sign in again' });
      tell();
    },
  });

  const onActivity = (method, params, outcome) => {
    if (!method.startsWith('mcp.') || state === 'signed-out') return;
    const hint = outcome.error && connectorHint(outcome.error.code, params.server);
    if (hint) {
      show(hint.text, { href: options.connectors, label: hint.label });
      return;
    }
    if (outcome.result && outcome.result.content) {
      const at = outcome.result.cache ? new Date(outcome.result.cache.storedAt) : new Date();
      show(`Live data · updated ${at.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}`);
    }
  };
  const route = createRouter({
    frame: () => frame.contentWindow,
    bridge,
    onSize: (reported) => {
      if (typeof reported !== 'number') return;
      height = Math.ceil(reported) + status.offsetHeight;
      tell();
    },
    onActivity,
  });
  window.addEventListener('message', route);

  try {
    const snapshot = await bridge.load();
    load(snapshot.revision);
    state = 'ready';
    show(bridge.declared().length > 0 ? 'Live page' : 'Shown as designed');
  } catch (error) {
    state = 'error';
    show(normalized(error).message);
  }
  tell();

  return {
    frame,
    permissions: () => bridge.panel(),
    reload: async () => load((await bridge.load()).revision),
    destroy: () => window.removeEventListener('message', route),
  };
};
