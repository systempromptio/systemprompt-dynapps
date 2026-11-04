// Loro blocks → ProseMirror nodes: one node per block Map from its `kind`
// JSON and its marked text; a table block from its rows and cells
// (`table-model.js`). Writing back is `write.js`.
import { schema } from './schema.js';
import { opaqueLabel } from './opaque.js';
import { blockList, childList, listItems, textOf, FLAG_MARKS } from './loro.js';
import { tableNode } from './table-model.js';

export const MARK_KEYS = ['bold', 'italic', 'underline', 'strike', 'code', 'link', 'color', 'highlight', 'comment'];
export const ATTR_OF = { link: 'href', color: 'value', highlight: 'value', comment: 'thread' };

const readKind = (map) => {
  try {
    return JSON.parse(map.get('kind') ?? '{}');
  } catch {
    return { type: 'raw', source: '' };
  }
};

const marksOf = (attributes = {}) => MARK_KEYS.filter((key) => attributes[key] != null && attributes[key] !== false)
  .map((key) => (FLAG_MARKS.has(key) ? schema.marks[key].create() : schema.marks[key].create({ [ATTR_OF[key]]: String(attributes[key]) })));

const inlineOf = (text, code) => {
  if (code) {
    const str = text?.toString() ?? '';
    return str ? [schema.text(str)] : [];
  }
  const out = [];
  for (const op of text?.toDelta() ?? []) {
    if (typeof op.insert !== 'string' || !op.insert) continue;
    const marks = marksOf(op.attributes);
    op.insert.split('\n').forEach((part, i) => {
      if (i > 0) out.push(schema.nodes.hard_break.create());
      if (part) out.push(schema.text(part, marks));
    });
  }
  return out;
};

// A table inside a cell is outside the model; it shows as opaque.
export const blockNode = (map, inCell = false) => {
  const id = map.get('id');
  const kind = readKind(map);
  if (!inCell && childList(map, 'rows')) {
    const table = tableNode(map, blockNode);
    if (table) return table;
  }
  const text = textOf(map);
  switch (kind.type) {
    case 'paragraph': return schema.nodes.paragraph.create({ id }, inlineOf(text));
    case 'heading': return schema.nodes.heading.create({ id, level: kind.level }, inlineOf(text));
    case 'list_item': return schema.nodes.list_item.create({ id, level: kind.level, ordered: kind.ordered }, inlineOf(text));
    case 'quote': return schema.nodes.quote.create({ id }, inlineOf(text));
    case 'code': return schema.nodes.code_block.create({ id, language: kind.language ?? null }, inlineOf(text, true));
    case 'divider': return schema.nodes.divider.create({ id });
    case 'page_break': return schema.nodes.page_break.create({ id });
    default: return schema.nodes.opaque.create({ id, kind: JSON.stringify(kind), label: opaqueLabel(kind) });
  }
};

export const readDoc = (doc) => {
  const blocks = listItems(blockList(doc)).map((map) => blockNode(map));
  return schema.nodes.doc.create(null, blocks.length ? blocks : [schema.nodes.paragraph.create()]);
};

export const kindOf = (node) => {
  const { attrs } = node;
  switch (node.type.name) {
    case 'heading': return { type: 'heading', level: attrs.level };
    case 'list_item': return { type: 'list_item', level: attrs.level, ordered: attrs.ordered };
    case 'code_block': return attrs.language ? { type: 'code', language: attrs.language } : { type: 'code' };
    case 'opaque': return JSON.parse(attrs.kind);
    default: return { type: node.type.name };
  }
};
