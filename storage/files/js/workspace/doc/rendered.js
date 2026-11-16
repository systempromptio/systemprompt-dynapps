// An HTML document shown as designed: core serves the stored file at
// `/rendered` under a sandbox policy with no same origin, and the frame
// navigates to it. The frame must load by `src`: a `srcdoc` document
// inherits this page's content policy, which would strip the file's own
// styles, and would not carry the response's sandbox. The rich-text editor
// mounts beside the frame the first time Edit is chosen and keeps its live
// session; switching back reloads the frame so it shows what was saved.
import { showToast } from '../../services/toast.js';

const el = (tag, className, attrs = {}) => Object.assign(document.createElement(tag), { className, ...attrs });

const renderedFrame = (root, name) => {
  const frame = el('iframe', 'sp-doc-rendered__frame', { title: name });
  frame.setAttribute('sandbox', 'allow-scripts');
  frame.src = `${root.dataset.documentApi}/rendered`;
  return frame;
};

const fullScreen = async (frame) => {
  try {
    await frame.requestFullscreen();
  } catch {
    showToast('Full screen is not available in this browser.', 'error');
  }
};

export const mountRendered = (slot, ctx, openEditor) => {
  const { root, doc, readOnly } = ctx;
  const bar = el('div', 'sp-doc-rendered__bar');
  const frame = renderedFrame(root, doc.name);
  const editor = el('div', 'sp-doc-rendered__editor', { hidden: true });
  const toggle = el('button', 'sp-btn sp-btn--sm sp-btn--outline', { type: 'button', textContent: 'Edit' });
  toggle.setAttribute('aria-pressed', 'false');
  let mounted = false;

  const show = async (editing) => {
    if (editing && !mounted) {
      await openEditor(editor, ctx);
      mounted = true;
    }
    if (!editing) frame.src = `${root.dataset.documentApi}/rendered`;
    frame.hidden = editing;
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
    await fullScreen(frame);
  });
  bar.append(el('span', 'sp-ws__muted', { textContent: 'Shown as designed' }));
  if (!readOnly) bar.append(toggle);
  slot.replaceChildren(bar, frame, editor);
  return { readOnly: true };
};
