// The built-in workspace commands. Each registers once with the access it
// needs, so the name menu, the toolbar buttons, the palette and the shortcuts
// offer only what the viewer may do, and one id always means one behaviour.
// Core's levels: rename and copy need edit; move and trash need ownership.
import { errorMessage, rawFetch, rawResponse } from '../services/api.js';
import { showConfirmDialog, showPromptDialog } from '../services/confirm.js';
import { showToast } from '../services/toast.js';
import { toggleDrawer } from './drawer.js';
import { pickFolder } from './folder-picker.js';
import { registerCommand } from './registry.js';

const enc = encodeURIComponent;
const DECKS = new Set(['pptx', 'odp']);

const patch = (root, body) => rawFetch(root.dataset.documentApi, { method: 'PATCH', body: JSON.stringify(body) });

const go = (url) => window.location.assign(url);

const canEdit = (ctx) => ctx.canEdit;
const isOwner = (ctx) => ctx.isOwner;

const rename = (root, ctx) => showPromptDialog('Rename document', 'New name', ctx.doc.name, async (name) => {
  await patch(root, { name: name.trim() });
  ctx.refresh();
});

const duplicate = async (root, ctx) => {
  const created = await rawFetch(`${root.dataset.documentApi}/copy`, {
    method: 'POST',
    body: JSON.stringify({ name: `Copy of ${ctx.doc.name}` }),
  });
  go(`${root.dataset.baseUrl}/${enc(created.document.id)}`);
};

const move = async (root, ctx) => {
  const folder = await pickFolder({ title: `Move “${ctx.doc.name}”`, currentId: ctx.doc.folder_id });
  if (folder === undefined) return;
  try {
    await patch(root, { folder_id: folder });
    ctx.refresh();
    showToast('Moved', 'success');
  } catch (error) {
    showToast(error.message ?? 'The document could not be moved.', 'error');
  }
};

const download = (root) => {
  const link = Object.assign(document.createElement('a'), { href: root.dataset.contentUrl, download: '' });
  link.click();
};

const exportPdf = (root) => {
  const link = Object.assign(document.createElement('a'), { href: `${root.dataset.contentUrl}?format=pdf`, download: '' });
  link.click();
};

const fileNameOf = (response) => /filename="?([^";]+)"?/.exec(response.headers.get('content-disposition') ?? '')?.[1] ?? 'export.md';

// Fetched rather than linked, because a link cannot read the header naming
// what the file left out, and the person must be told.
const exportMarkdown = async (root) => {
  const response = await rawResponse(`${root.dataset.documentApi}/export/markdown`, { credentials: 'same-origin' });
  if (!response.ok) {
    showToast(await errorMessage(response), 'error');
    return;
  }
  const url = URL.createObjectURL(await response.blob());
  Object.assign(document.createElement('a'), { href: url, download: fileNameOf(response) }).click();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
  const leftOut = response.headers.get('x-export-left-out');
  if (leftOut) showToast(`Exported. Markdown could not hold: ${leftOut}.`, 'info');
};

const SHEETS = new Set(['xlsx', 'ods']);
const isSheet = (ctx) => SHEETS.has(ctx.doc?.format);
const isDeck = (ctx) => DECKS.has(ctx.doc?.format);
const isText = (ctx) => canRenderPdf(ctx) && !isSheet(ctx) && !isDeck(ctx);

// Core renders every format to PDF except an opaque file, which has no text.
const canRenderPdf = (ctx) => ctx.doc?.format !== 'file';

const trash = (root, ctx) => showConfirmDialog('Move to trash', `“${ctx.doc.name}” moves to the trash and can be restored.`, 'Move to trash', async () => {
  await rawFetch(root.dataset.documentApi, { method: 'DELETE' });
  go(root.dataset.baseUrl);
});

const toggleComments = (root) => {
  const button = root.querySelector('[data-command="comments"]');
  const open = button?.getAttribute('aria-pressed') !== 'true';
  button?.setAttribute('aria-pressed', String(open));
  toggleDrawer(root, 'comments', open);
};

const present = (root) => root.querySelector('[data-slot="ws-editor"]').requestFullscreen?.();

[
  { id: 'rename', label: 'Rename', menu: true, when: canEdit, run: rename },
  { id: 'duplicate', label: 'Make a copy', menu: true, when: canEdit, run: duplicate },
  { id: 'move', label: 'Move to folder', menu: true, when: isOwner, run: move },
  { id: 'download', label: 'Download', menu: true, run: download },
  { id: 'export-pdf', label: 'Export as PDF', menu: true, when: canRenderPdf, run: exportPdf },
  { id: 'export-md', label: 'Export as Markdown', menu: true, when: isText, run: exportMarkdown },
  { id: 'export-md-tables', label: 'Export as Markdown (one table per sheet)', menu: true, when: isSheet, run: exportMarkdown },
  { id: 'export-md-outline', label: 'Export as Markdown (text outline of the slides)', menu: true, when: isDeck, run: exportMarkdown },
  { id: 'history', label: 'Version history', menu: true, run: (root) => toggleDrawer(root, 'versions', true) },
  { id: 'share', label: 'Share', menu: true, when: (ctx) => ctx.canShare, run: (root) => toggleDrawer(root, 'sharing', true) },
  { id: 'details', label: 'Details', keys: '⌘I', menu: true, run: (root) => toggleDrawer(root, 'details') },
  { id: 'proposals', label: 'Proposals', run: (root) => toggleDrawer(root, 'proposals', true) },
  { id: 'blocks', label: 'Block editor', menu: true, run: (root) => go(root.dataset.blocksUrl) },
  { id: 'trash', label: 'Move to trash', menu: true, when: isOwner, run: trash },
  { id: 'comments', label: 'Show comments', run: toggleComments },
  { id: 'present', label: 'Present', when: (ctx) => DECKS.has(ctx.doc?.format), run: present },
  { id: 'home', label: 'Back to Drive', run: (root) => go(root.dataset.baseUrl) },
].forEach((command) => registerCommand(command));
