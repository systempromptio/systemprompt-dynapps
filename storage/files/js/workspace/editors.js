// The editor slot. Format editors register a mount function in the registry,
// and a format with no editor yet, or whose editor cannot start (no live engine on
// this installation, say), shows core's sanitised preview as a read-only page.
// An editor is handed `readOnly` when the viewer holds less than edit access.
import { showPaper } from '../services/drive-paper.js';
import { editorFor } from './registry.js';

const showPreview = async (slot, root) => {
  const frame = slot.querySelector('[data-slot="ws-paper"]');
  try {
    await showPaper(frame, root.dataset.previewApi);
  } catch {
    frame.replaceWith(Object.assign(document.createElement('p'), {
      className: 'sp-ws__muted',
      textContent: 'This document could not be rendered.',
    }));
  }
  return { readOnly: true };
};

export const mountEditor = (ctx) => {
  const { root, doc } = ctx;
  const slot = root.querySelector('[data-slot="ws-editor"]');
  const mount = editorFor(doc.format);
  if (!mount) return showPreview(slot, root);
  return Promise.resolve()
    .then(() => mount(slot, { ...ctx, readOnly: !ctx.canEdit }))
    .catch(() => showPreview(slot, root));
};
