// The doc editor for HTML, Markdown, Word and OpenDocument text: a
// ProseMirror view over the live session's Loro document. Local
// transactions write through to Loro (which posts the update); imported
// updates rebuild only the blocks that changed (`remote.js`). Tables are
// prosemirror-tables nodes: column resize, cell selection, Tab between cells.
import { EditorState } from 'prosemirror-state';
import { EditorView } from 'prosemirror-view';
import { history } from 'prosemirror-history';
import { keymap } from 'prosemirror-keymap';
import { baseKeymap } from 'prosemirror-commands';
import { dropCursor } from 'prosemirror-dropcursor';
import { gapCursor } from 'prosemirror-gapcursor';
import { loadEngine } from './loro.js';
import { schema } from './schema.js';
import { columnResizing, tableEditing, goToNextCell } from 'prosemirror-tables';
import { setMediaBase } from './opaque.js';
import { readDoc } from './model.js';
import { writeDoc, remember } from './write.js';
import { syncFromLoro, applyRemote } from './remote.js';
import { createSession } from './session.js';
import { createAwareness } from './awareness.js';
import { createComments } from './comments.js';
import { createToolbar, bubblePlugin } from './toolbar.js';
import { markdownRules, slashPlugin } from './menus.js';
import { idsPlugin, keepOpaquePlugin } from './ids.js';
import { keymapBindings } from './actions.js';
import { toMarkdown } from './source.js';
import { commentsTab } from '../drawer-comments.js';
import { permits } from '../people.js';
import { registerDrawerTab } from '../registry.js';

const el = (tag, className, attrs = {}) => Object.assign(document.createElement(tag), { className, ...attrs });

const layout = (slot, format) => {
  const toolbar = el('div', 'sp-doc-toolbar', { role: 'toolbar', ariaLabel: 'Formatting' });
  const page = el('div', `sp-doc-page sp-doc-page--${format}`);
  const editor = el('div', 'sp-doc-editor');
  const source = el('pre', 'sp-doc-source', { hidden: true });
  const bubble = el('div', 'sp-doc-bubble', { hidden: true });
  const slash = el('div', 'sp-doc-slash', { hidden: true, role: 'listbox' });
  page.append(editor, source, bubble, slash);
  slot.replaceChildren(toolbar, page);
  return { toolbar, page, editor, source, bubble, slash };
};

export const mount = async (slot, { root, doc, access, readOnly }) => {
  await loadEngine();
  const api = root.dataset.documentApi;
  setMediaBase(api);
  const selfId = root.dataset.userId;
  let view = null;
  const seen = new Map();
  const saveLabel = root.querySelector('[data-field="save"]');
  const parts = {};
  let awareness = null;
  let comments = null;

  const session = createSession({
    mode: readOnly ? 'comment' : 'edit',
    collabApi: `${api}/collab`,
    liveApi: `${api}/live`,
    onState: (text) => { saveLabel.textContent = text; },
    // `carried` holds the edits the last session never sent, merged over
    // the new snapshot; writing its state into the new doc re-issues them
    // as this join's peer, which is the only peer core accepts.
    onReset: (s, carried) => {
      if (carried) {
        writeDoc(s.doc, readDoc(carried), new Map());
        s.doc.commit();
      }
      if (!view) return;
      syncFromLoro(view, s.doc);
      seen.clear();
      remember(seen, view.state.doc);
      view.setProps({ editable: () => s.mode === 'edit' });
    },
    onRemote: (loroDoc, batch) => {
      if (!view) return;
      applyRemote(view, loroDoc, batch);
      remember(seen, view.state.doc);
    },
    onEvent: (name, data) => {
      if (name === 'document.awareness') awareness?.receive(data);
      else if (name.startsWith('document.comment.')) comments?.onEvent();
    },
  });
  await session.start();

  awareness = createAwareness({ api: `${api}/collab/awareness`, session, selfId });
  comments = createComments({ root, api: `${api}/comments`, getView: () => view, canComment: permits(access, 'comment') });
  registerDrawerTab({ ...commentsTab, eager: false, load: () => comments.load() });

  Object.assign(parts, layout(slot, doc.format));
  const showSource = (v) => {
    const on = parts.source.hidden;
    parts.source.textContent = on ? toMarkdown(v.state.doc) : '';
    parts.source.hidden = !on;
    parts.editor.hidden = on;
  };
  const extra = doc.format === 'md' ? { labels: { source: 'Markdown source' }, source: showSource } : {};
  const plugins = [
    idsPlugin(),
    keepOpaquePlugin(),
    markdownRules(),
    slashPlugin(parts.slash),
    columnResizing(),
    tableEditing(),
    keymap({ Tab: goToNextCell(1), 'Shift-Tab': goToNextCell(-1) }),
    keymap(keymapBindings()),
    keymap(baseKeymap),
    history(),
    dropCursor(),
    gapCursor(),
    createToolbar(parts.toolbar, () => view, extra),
    bubblePlugin(parts.bubble, (v) => comments.fromSelection(v)),
    awareness.plugin,
  ];

  const initial = readDoc(session.state.doc);
  remember(seen, initial);
  view = new EditorView(parts.editor, {
    state: EditorState.create({ schema, doc: initial, plugins }),
    editable: () => session.state.mode === 'edit',
    attributes: { class: 'sp-doc-prose', spellcheck: 'true' },
    dispatchTransaction: (tr) => {
      const state = view.state.apply(tr);
      view.updateState(state);
      if (!tr.docChanged || tr.getMeta('remote')) return;
      writeDoc(session.state.doc, state.doc, seen);
      session.state.doc.commit();
    },
  });
  awareness.hello();
  comments.load().catch(() => undefined);
  return { readOnly: session.state.mode !== 'edit', view };
};
