// One ProseMirror view per shape over its `paragraphs` list. Local edits
// write through and commit (the session posts them); a remote import
// replaces only the differing range so the local caret stays put.
import { EditorState } from 'prosemirror-state';
import { EditorView } from 'prosemirror-view';
import { history, undo, redo } from 'prosemirror-history';
import { keymap } from 'prosemirror-keymap';
import { baseKeymap, toggleMark } from 'prosemirror-commands';
import { schema } from '../doc/schema.js';
import { idsPlugin } from '../doc/ids.js';
import { readParagraphs, writeParagraphs } from './paragraphs.js';

const sync = (view, next) => {
  const prev = view.state.doc;
  const start = prev.content.findDiffStart(next.content);
  if (start == null) return;
  let { a: endPrev, b: endNext } = prev.content.findDiffEnd(next.content);
  const overlap = start - Math.min(endPrev, endNext);
  if (overlap > 0) {
    endPrev += overlap;
    endNext += overlap;
  }
  const tr = view.state.tr.replace(start, endPrev, next.slice(start, endNext));
  view.dispatch(tr.setMeta('remote', true).setMeta('addToHistory', false));
};

const KEYS = {
  'Mod-z': undo,
  'Mod-y': redo,
  'Shift-Mod-z': redo,
  'Mod-b': toggleMark(schema.marks.bold),
  'Mod-i': toggleMark(schema.marks.italic),
  'Mod-u': toggleMark(schema.marks.underline),
};

export const mountShapeText = (host, { shape, session, onSelect, onEdit }) => {
  const seen = new Map();
  const list = () => shape.get('paragraphs');
  const remember = (pm) => pm.forEach((node) => seen.set(node.attrs.id, node));
  const initial = readParagraphs(list());
  remember(initial);
  const view = new EditorView(host, {
    state: EditorState.create({ schema, doc: initial, plugins: [idsPlugin(), keymap(KEYS), keymap(baseKeymap), history()] }),
    editable: () => session.state.mode === 'edit',
    attributes: { class: 'sp-deck-shape__text', spellcheck: 'true' },
    dispatchTransaction: (tr) => {
      const state = view.state.apply(tr);
      view.updateState(state);
      if (tr.selectionSet || tr.docChanged) onSelect(view);
      if (!tr.docChanged || tr.getMeta('remote') || !list()) return;
      writeParagraphs(list(), state.doc, seen);
      session.state.doc.commit();
      onEdit?.();
    },
  });
  return {
    view,
    refresh: (nextShape) => {
      shape = nextShape;
      if (!list()) return;
      sync(view, readParagraphs(list()));
      remember(view.state.doc);
    },
    destroy: () => view.destroy(),
  };
};
