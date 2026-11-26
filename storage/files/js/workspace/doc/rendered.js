// An HTML document shown as designed: core serves the stored file at
// `/rendered` under a sandbox policy with no same origin, and the frame
// navigates to it. The frame must load by `src`: a `srcdoc` document
// inherits this page's content policy, which would strip the file's own
// styles, and would not carry the response's sandbox. A page that declares
// live capabilities is shown through the viewer shell instead (`js/workspace/live/`),
// which frames it the same way and answers its runtime as the viewer. The
// rich-text editor mounts beside the frame the first time Edit is chosen and
// keeps its live session; switching back reloads the page so it shows
// what was saved there.
import { showToast } from '../../services/toast.js';

const el = (tag, className, attrs = {}) => Object.assign(document.createElement(tag), { className, ...attrs });

const renderedView = (root, name) => {
  const frame = el('iframe', 'sp-doc-rendered__frame', { title: name });
  frame.setAttribute('sandbox', 'allow-scripts');
  frame.src = `${root.dataset.documentApi}/rendered`;
  return { node: frame, reload: () => { frame.src = `${root.dataset.documentApi}/rendered`; } };
};

const liveView = (root, name) => {
  const node = el('div', 'sp-doc-rendered__frame');
  const documentId = root.dataset.documentApi.split('/').pop();
  const mounted = import('../live/shell.js').then(({ mountLiveShell }) => mountLiveShell(node, {
    title: name,
    embed: true,
    frameUrl: root.dataset.liveFrame,
    signIn: `/admin/login?redirect=${encodeURIComponent(window.location.pathname)}`,
    connectors: '/admin/connectors',
    urls: { api: root.dataset.liveApi, people: root.dataset.peopleApi, search: `${root.dataset.documentsApi}/people`, document: decodeURIComponent(documentId) },
  }));
  return {
    node,
    reload: async () => (await mounted).reload(),
    permissions: async () => (await mounted).permissions(),
  };
};

const fullScreen = async (node) => {
  try {
    await node.requestFullscreen();
  } catch {
    showToast('Full screen is not available in this browser.', 'error');
  }
};

export const mountRendered = (slot, ctx, openEditor) => {
  const { root, doc, readOnly } = ctx;
  const live = Boolean(root.dataset.liveFrame);
  const bar = el('div', 'sp-doc-rendered__bar');
  const view = live ? liveView(root, doc.name) : renderedView(root, doc.name);
  const editor = el('div', 'sp-doc-rendered__editor', { hidden: true });
  const toggle = el('button', 'sp-btn sp-btn--sm sp-btn--outline', { type: 'button', textContent: 'Edit' });
  toggle.setAttribute('aria-pressed', 'false');
  let mounted = false;

  const show = async (editing) => {
    if (editing && !mounted) {
      await openEditor(editor, ctx);
      mounted = true;
    }
    if (!editing) await view.reload();
    view.node.hidden = editing;
    editor.hidden = !editing;
    toggle.setAttribute('aria-pressed', String(editing));
  };

  const onToggle = async () => {
    try {
      await show(toggle.getAttribute('aria-pressed') !== 'true');
    } catch {
      showToast('The editor could not be opened.', 'error');
    }
  };

  toggle.addEventListener('click', onToggle);
  root.addEventListener('ws:present', async () => {
    await show(false);
    await fullScreen(view.node);
  });
  bar.append(el('span', 'sp-ws__muted', { textContent: live ? 'Live page' : 'Shown as designed' }));
  if (live) {
    const permissions = el('button', 'sp-btn sp-btn--sm sp-btn--ghost', { type: 'button', textContent: 'Permissions' });
    permissions.addEventListener('click', () => view.permissions());
    bar.append(permissions);
  }
  if (!readOnly) bar.append(toggle);
  slot.replaceChildren(bar, view.node, editor);
  return { readOnly: true };
};
