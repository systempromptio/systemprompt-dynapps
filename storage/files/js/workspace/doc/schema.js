// The ProseMirror schema that mirrors core's RichText model one to one: a
// list of top-level blocks, each carrying the stable block `id` the server,
// comments and agents address it by. A table is real table/row/cell nodes
// (prosemirror-tables), each with its own id, and a cell holds any block but
// another table (`cell_block`). Images and `raw` native markup are `opaque`
// atoms, shown and never edited here.
import { Schema } from 'prosemirror-model';
import { tableNodes } from 'prosemirror-tables';
import { opaqueDom, opaqueLabel, readKindAttr } from './opaque.js';

export { opaqueLabel };

const id = { id: { default: null } };
const BLOCK = 'block cell_block';
const textBlock = (tag, extra = {}) => ({
  group: BLOCK,
  content: 'inline*',
  attrs: { ...id, ...extra },
});

// `props` carries the core props the editor does not model as attrs
// (table style and header_rows, cell shade and alignment) as a JSON string.
const tables = () => {
  const props = { default: '{}' };
  const made = tableNodes({
    tableGroup: 'block',
    cellContent: 'cell_block+',
    cellAttributes: { id: { default: null }, props },
  });
  return {
    ...made,
    table: { ...made.table, attrs: { ...id, props } },
    table_row: { ...made.table_row, attrs: id },
  };
};

const nodes = {
  doc: { content: 'block+' },
  paragraph: { ...textBlock(), parseDOM: [{ tag: 'p' }], toDOM: () => ['p', 0] },
  heading: {
    ...textBlock('h', { level: { default: 1 } }),
    defining: true,
    parseDOM: [1, 2, 3, 4, 5, 6].map((level) => ({ tag: `h${level}`, attrs: { level } })),
    toDOM: (node) => [`h${node.attrs.level}`, 0],
  },
  list_item: {
    ...textBlock('li', { level: { default: 1 }, ordered: { default: false } }),
    parseDOM: [{ tag: 'li' }],
    toDOM: (node) => ['p', {
      class: `sp-doc__li sp-doc__li--${node.attrs.ordered ? 'ordered' : 'bullet'} sp-doc__li--level-${Math.min(node.attrs.level, 6)}`,
    }, 0],
  },
  quote: { ...textBlock(), parseDOM: [{ tag: 'blockquote' }], toDOM: () => ['blockquote', 0] },
  code_block: {
    group: BLOCK,
    content: 'text*',
    marks: '',
    code: true,
    defining: true,
    attrs: { ...id, language: { default: null } },
    parseDOM: [{ tag: 'pre', preserveWhitespace: 'full' }],
    toDOM: () => ['pre', ['code', 0]],
  },
  divider: { group: BLOCK, atom: true, attrs: id, parseDOM: [{ tag: 'hr' }], toDOM: () => ['hr'] },
  page_break: {
    group: BLOCK,
    atom: true,
    attrs: id,
    toDOM: () => ['div', { class: 'sp-doc__break', 'aria-label': 'Page break' }],
  },
  opaque: {
    group: BLOCK,
    atom: true,
    selectable: true,
    attrs: { ...id, kind: { default: '{}' }, label: { default: '' } },
    toDOM: (node) => opaqueDom(readKindAttr(node.attrs), node.attrs.label),
  },
  ...tables(),
  text: { group: 'inline' },
  hard_break: { inline: true, group: 'inline', selectable: false, toDOM: () => ['br'] },
};

const flag = (tag, cls) => ({ parseDOM: [{ tag }], toDOM: () => [tag, cls ? { class: cls } : {}, 0] });
const valued = (attr, cls) => ({
  attrs: { [attr]: {} },
  inclusive: false,
  toDOM: (mark) => ['span', { class: cls, 'data-value': mark.attrs[attr] }, 0],
});

const marks = {
  bold: flag('strong'),
  italic: flag('em'),
  underline: flag('u'),
  strike: flag('s'),
  code: { ...flag('code'), inclusive: false },
  link: {
    attrs: { href: {} },
    inclusive: false,
    toDOM: (mark) => ['a', { href: mark.attrs.href, rel: 'noopener noreferrer nofollow', target: '_blank' }, 0],
  },
  color: valued('value', 'sp-doc__color'),
  highlight: valued('value', 'sp-doc__highlight'),
  comment: { ...valued('thread', 'sp-doc__comment'), excludes: '' },
};

export const schema = new Schema({ nodes, marks });

// Block kinds the model owns as text, keyed by ProseMirror node name.
export const TEXT_KINDS = new Set(['paragraph', 'heading', 'list_item', 'quote', 'code_block']);
