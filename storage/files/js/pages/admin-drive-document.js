// One drive document in the block editor: the canvas for its format (read-only
// below edit access), a faithful preview from core, any earlier revision on
// demand, rename in place, and a rail of details, activity and the workspace's
// own sharing, comments and proposals panes. The canvas is the workspace's
// registered editor for the format, so this page and the workspace edit through
// the same live sessions; other people's commits and presence arrive on the
// page's one document event stream and reach the editor as live events.
import { subscribeDocument } from '../documents/documents-events.js';
import { loadEditors, readJson } from '../services/drive-rows.js';
import { rawFetch } from '../services/api.js';
import { showPaper } from '../services/drive-paper.js';
import {
  resolvePeople, showActivity, showDetails, showEditors,
} from '../services/drive-rail.js';
import { documentIdOf, permits } from '../workspace/people.js';
import { mountPanes } from '../workspace/panes.js';
import { dispatchLive, editorFor } from '../workspace/registry.js';
import '../workspace/doc/editor.js';
import '../workspace/sheet/editor.js';
import '../workspace/slides/editor.js';
import '../workspace/drawer-sharing.js';
import '../workspace/drawer-comments.js';
import '../workspace/drawer-proposals.js';

const FORMAT_LABEL = {
  docx: 'Word', odt: 'Document', xlsx: 'Excel', ods: 'Sheet', pptx: 'PowerPoint',
  odp: 'Slides', html: 'Web page', markdown: 'Markdown', md: 'Markdown', text: 'Text',
};

const page = { root: null, doc: null };

const slot = (name) => page.root.querySelector(`[data-slot="${name}"]`);

const setStatus = (text, tone) => {
  const el = slot('doc-status');
  el.textContent = text;
  el.dataset.tone = tone;
};

const stem = (name) => name.replace(/\.[^.]+$/, '');
const extension = (name) => (/\.[^.]+$/.exec(name)?.[0] ?? '');

const refreshRail = async () => {
  const { root } = page;
  const [{ document: doc, folder_path: folderPath }, { revisions }] = await Promise.all([
    readJson(root.dataset.driveDocument),
    readJson(root.dataset.revisionsApi),
  ]);
  page.doc = doc;
  await resolvePeople(root.dataset.peopleApi, [doc.owner_user_id, ...revisions.map((r) => r.author_user_id)]);
  showDetails(root, doc, folderPath);
  showActivity(slot('doc-activity'), revisions, viewRevision);
};

const contextOf = (root) => {
  const access = root.dataset.myAccess || 'view';
  return {
    root,
    access,
    canEdit: root.dataset.canEdit === 'true',
    canShare: root.dataset.canShare === 'true',
    canComment: permits(access, 'comment'),
    isOwner: access === 'own',
    userId: root.dataset.userId,
    documentId: documentIdOf(root),
    peopleApi: root.dataset.peopleApi,
  };
};

const followEvents = (ctx) => {
  const reloadPanes = mountPanes(page.root.querySelector('#doc-rail'), ctx);
  subscribeDocument(ctx.documentId, (name, value) => {
    reloadPanes(name);
    dispatchLive(name, value);
    if (name === 'document.revision.created') refreshRail().catch(() => null);
    else if (name === 'document.presence') refreshEditors().catch(() => null);
  });
};

const refreshEditors = async () => {
  const editors = await loadEditors(page.root.dataset.presenceApi);
  await resolvePeople(page.root.dataset.peopleApi, editors.map((e) => e.user_id));
  showEditors(slot('doc-editors'), editors);
};

const setMode = (mode, revision = null) => {
  const editing = mode === 'edit';
  slot('doc-canvas').hidden = !editing;
  slot('doc-frame').hidden = editing;
  slot('doc-hint').hidden = !editing;
  page.root.querySelector('[data-action="doc-edit"]').setAttribute('aria-pressed', String(editing));
  page.root.querySelector('[data-action="doc-preview"]').setAttribute('aria-pressed', String(!editing));
  const banner = slot('doc-banner');
  banner.hidden = !revision;
  if (revision) slot('doc-banner-text').textContent = `Viewing the revision from ${new Date(revision.created_at).toLocaleString()}. It is read-only.`;
  if (!editing) {
    showPaper(slot('doc-frame'), page.root.dataset.previewApi, page.doc.format, revision?.id)
      .catch(() => setStatus('The preview could not be rendered', 'warn'));
  }
};

function viewRevision(revision) {
  setMode('preview', revision);
}

const rename = async (input) => {
  const next = input.value.trim();
  if (!next || next === stem(page.doc.name)) {
    input.value = stem(page.doc.name);
    return;
  }
  const body = await rawFetch(page.root.dataset.driveDocument, {
    method: 'PATCH',
    body: JSON.stringify({ name: `${next}${extension(page.doc.name)}` }),
  }).catch(() => null);
  if (body) setStatus('Renamed', 'ok');
  await refreshRail();
  input.value = stem(page.doc.name);
};

const wireHeader = (ctx) => {
  const { root } = page;
  const name = slot('doc-name');
  name.value = stem(page.doc.name);
  name.readOnly = !ctx.canEdit;
  name.addEventListener('change', () => rename(name));
  name.addEventListener('keydown', (e) => { if (e.key === 'Enter') name.blur(); });
  root.querySelector('[data-action="doc-edit"]').addEventListener('click', () => setMode('edit'));
  root.querySelector('[data-action="doc-preview"]').addEventListener('click', () => setMode('preview'));
  root.querySelector('[data-action="doc-current"]').addEventListener('click', () => setMode('edit'));
  const railToggle = root.querySelector('[data-action="doc-rail"]');
  railToggle.addEventListener('click', () => {
    const open = railToggle.getAttribute('aria-pressed') !== 'true';
    railToggle.setAttribute('aria-pressed', String(open));
    root.classList.toggle('sp-p-doc--rail-closed', !open);
  });
};

// A format with no registered editor, or one whose editor cannot start, opens
// on the preview alone.
const mountCanvas = async (ctx) => {
  const mount = editorFor(page.doc.format);
  if (!mount) return null;
  try {
    return await mount(slot('doc-canvas'), { ...ctx, doc: page.doc, readOnly: !ctx.canEdit });
  } catch {
    return null;
  }
};

export const init = async () => {
  page.root = document.querySelector('[data-drive-document]');
  if (!page.root) return;
  const ctx = contextOf(page.root);
  try {
    await refreshRail();
  } catch {
    setStatus('This document could not be opened', 'warn');
    return;
  }
  const { format } = page.doc;
  slot('doc-badge').textContent = FORMAT_LABEL[format] ?? format;
  slot('doc-badge').dataset.format = format;
  page.root.querySelector('.sp-p-doc__desk').dataset.format = format;
  wireHeader(ctx);
  followEvents(ctx);
  refreshEditors().catch(() => null);
  const editor = await mountCanvas(ctx);
  if (!editor) {
    page.root.querySelector('[data-action="doc-edit"]').disabled = true;
    setMode('preview');
    return;
  }
  slot('doc-hint').textContent = editor.hint ?? '';
  if (editor.readOnly) setStatus('View only', 'ok');
};

init();
