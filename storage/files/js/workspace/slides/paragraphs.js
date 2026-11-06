// A shape's paragraphs ⇄ ProseMirror. Reading reuses the doc editor's block
// reader; writing mirrors its smallest-change rule on the shape's own list
// (core addresses paragraphs by id, so ids move instead of being re-created).
import { schema, TEXT_KINDS } from '../doc/schema.js';
import { blockNode, kindOf } from '../doc/model.js';
import { newBlock, textOf, FLAG_MARKS } from '../doc/loro.js';
import { itemsOf } from './deck.js';

const MARK_KEYS = ['bold', 'italic', 'underline', 'strike', 'code', 'link', 'color', 'highlight', 'comment'];
const ATTR_OF = { link: 'href', color: 'value', highlight: 'value', comment: 'thread' };

export const readParagraphs = (list) => {
  const blocks = itemsOf(list).map(blockNode);
  return schema.nodes.doc.create(null, blocks.length ? blocks : [schema.nodes.paragraph.create()]);
};

const flatten = (node) => {
  let text = '';
  const marks = [];
  node.forEach((child) => {
    const str = child.isText ? child.text : '\n';
    const set = {};
    for (const mark of child.marks) {
      const { name } = mark.type;
      set[name] = FLAG_MARKS.has(name) ? true : mark.attrs[ATTR_OF[name]];
    }
    text += str;
    for (let i = 0; i < str.length; i += 1) marks.push(set);
  });
  return { text, marks };
};

const splice = (text, next) => {
  const prev = text.toString();
  if (prev === next) return;
  let start = 0;
  while (start < prev.length && start < next.length && prev[start] === next[start]) start += 1;
  let end = 0;
  while (end < prev.length - start && end < next.length - start
    && prev[prev.length - 1 - end] === next[next.length - 1 - end]) end += 1;
  if (prev.length - start - end > 0) text.delete(start, prev.length - start - end);
  if (next.length - start - end > 0) text.insert(start, next.slice(start, next.length - end));
};

const loroMarks = (text) => text.toDelta().flatMap((op) => (
  typeof op.insert === 'string' ? Array.from(op.insert, () => op.attributes ?? {}) : []));

const applyRuns = (text, key, want, have) => {
  let i = 0;
  while (i < want.length) {
    const target = want[i][key] ?? null;
    if (target === (have[i]?.[key] ?? null)) { i += 1; continue; }
    let j = i + 1;
    while (j < want.length && (want[j][key] ?? null) === target && target !== (have[j]?.[key] ?? null)) j += 1;
    if (target === null) text.unmark({ start: i, end: j }, key);
    else text.mark({ start: i, end: j }, key, target);
    i = j;
  }
};

const writeBlock = (map, node) => {
  const kind = JSON.stringify(kindOf(node));
  if (node.type.name !== 'opaque' && map.get('kind') !== kind) map.set('kind', kind);
  const text = textOf(map);
  if (!text || !TEXT_KINDS.has(node.type.name)) return;
  const want = flatten(node);
  splice(text, want.text);
  const have = loroMarks(text);
  for (const key of MARK_KEYS) applyRuns(text, key, want.marks, have);
};

const place = (list, at, node) => {
  const current = at < list.length ? list.get(at) : null;
  if (current?.get('id') === node.attrs.id) return current;
  const from = itemsOf(list).findIndex((m, j) => j > at && m.get('id') === node.attrs.id);
  if (from < 0) return newBlock(list, at, node.attrs.id, kindOf(node));
  list.move(from, at);
  return list.get(at);
};

export const writeParagraphs = (list, pm, seen) => {
  const wanted = [];
  pm.forEach((node) => wanted.push(node));
  const ids = new Set(wanted.map((n) => n.attrs.id));
  for (let i = list.length - 1; i >= 0; i -= 1) {
    if (!ids.has(list.get(i).get('id'))) list.delete(i, 1);
  }
  wanted.forEach((node, at) => {
    const map = place(list, at, node);
    if (seen.get(node.attrs.id) === node) return;
    writeBlock(map, node);
    seen.set(node.attrs.id, node);
  });
};
