// Editing actions shared by the toolbar, the selection bubble, the slash
// menu and the keymap. Each is a ProseMirror command `(state, dispatch)`.
import { setBlockType, toggleMark } from 'prosemirror-commands';
import { undo, redo } from 'prosemirror-history';
import { schema } from './schema.js';

const n = schema.nodes;
const m = schema.marks;

const blockType = (type, attrs) => (state, dispatch) => {
  const { $from } = state.selection;
  const keep = { id: $from.depth >= 1 ? $from.node(1).attrs.id : null };
  return setBlockType(type, { ...keep, ...attrs })(state, dispatch);
};

const insertAtom = (type) => (state, dispatch) => {
  if (dispatch) {
    const tr = state.tr.replaceSelectionWith(type.create());
    dispatch(tr.scrollIntoView());
  }
  return true;
};

// A header row over two body rows, three columns; never inside a cell,
// since core's model has no nested tables.
const insertTable = (state, dispatch) => {
  const { $from } = state.selection;
  for (let d = $from.depth; d > 0; d -= 1) if ($from.node(d).type === n.table) return false;
  if (dispatch) {
    const cell = (type) => type.create(null, n.paragraph.create());
    const row = (type) => n.table_row.create(null, [cell(type), cell(type), cell(type)]);
    const table = n.table.create({ props: JSON.stringify({ header_rows: 1 }) }, [row(n.table_header), row(n.table_cell), row(n.table_cell)]);
    dispatch(state.tr.replaceSelectionWith(table).scrollIntoView());
  }
  return true;
};

const clearMarks = (state, dispatch) => {
  const { from, to, empty } = state.selection;
  if (empty) return false;
  if (dispatch) dispatch(state.tr.removeMark(from, to).setStoredMarks([]));
  return true;
};

const indent = (delta) => (state, dispatch) => {
  const { $from } = state.selection;
  const block = $from.depth >= 1 ? $from.node(1) : null;
  if (block?.type !== n.list_item) return false;
  const level = Math.max(1, Math.min(6, block.attrs.level + delta));
  if (dispatch) dispatch(state.tr.setNodeMarkup($from.before(1), null, { ...block.attrs, level }));
  return true;
};

export const ACTIONS = {
  undo: { label: 'Undo', run: undo },
  redo: { label: 'Redo', run: redo },
  paragraph: { label: 'Text', run: blockType(n.paragraph), hint: 'Plain paragraph' },
  h1: { label: 'Heading 1', run: blockType(n.heading, { level: 1 }), hint: '# ' },
  h2: { label: 'Heading 2', run: blockType(n.heading, { level: 2 }), hint: '## ' },
  h3: { label: 'Heading 3', run: blockType(n.heading, { level: 3 }), hint: '### ' },
  bullet: { label: 'Bulleted list', run: blockType(n.list_item, { level: 1, ordered: false }), hint: '- ' },
  numbered: { label: 'Numbered list', run: blockType(n.list_item, { level: 1, ordered: true }), hint: '1. ' },
  quote: { label: 'Quote', run: blockType(n.quote), hint: '> ' },
  code_block: { label: 'Code block', run: blockType(n.code_block), hint: '```' },
  divider: { label: 'Divider', run: insertAtom(n.divider), hint: '---' },
  page_break: { label: 'Page break', run: insertAtom(n.page_break) },
  table: { label: 'Table', run: insertTable },
  bold: { label: 'Bold', run: toggleMark(m.bold), keys: 'Mod-b' },
  italic: { label: 'Italic', run: toggleMark(m.italic), keys: 'Mod-i' },
  underline: { label: 'Underline', run: toggleMark(m.underline), keys: 'Mod-u' },
  strike: { label: 'Strikethrough', run: toggleMark(m.strike), keys: 'Mod-Shift-x' },
  code: { label: 'Inline code', run: toggleMark(m.code), keys: 'Mod-e' },
  clear: { label: 'Clear formatting', run: clearMarks },
  indent: { label: 'Indent', run: indent(1), keys: 'Tab' },
  outdent: { label: 'Outdent', run: indent(-1), keys: 'Shift-Tab' },
};

export const SLASH_ITEMS = ['paragraph', 'h1', 'h2', 'h3', 'bullet', 'numbered', 'quote', 'code_block', 'divider', 'page_break', 'table'];

export const setLink = (href) => (state, dispatch) => {
  const { from, to, empty } = state.selection;
  if (empty) return false;
  if (dispatch) {
    const tr = state.tr.removeMark(from, to, m.link);
    dispatch(href ? tr.addMark(from, to, m.link.create({ href })) : tr);
  }
  return true;
};

export const markActive = (state, type) => {
  const { from, to, empty, $from } = state.selection;
  return empty ? Boolean(type.isInSet(state.storedMarks || $from.marks())) : state.doc.rangeHasMark(from, to, type);
};

export const blockName = (state) => {
  const { $from } = state.selection;
  const block = $from.depth >= 1 ? $from.node(1) : null;
  if (!block) return 'paragraph';
  if (block.type === n.heading) return `h${Math.min(block.attrs.level, 3)}`;
  if (block.type === n.list_item) return block.attrs.ordered ? 'numbered' : 'bullet';
  if (block.type === n.code_block) return 'code_block';
  return block.type.name;
};

export const keymapBindings = () => {
  const keys = { 'Mod-z': undo, 'Mod-y': redo, 'Shift-Mod-z': redo };
  for (const action of Object.values(ACTIONS)) if (action.keys) keys[action.keys] = action.run;
  return keys;
};
