// A local ProseMirror document → the Loro block lists, with the smallest
// change per list: moves, inserts and deletes by id, then per block a text
// splice and mark runs, or a table's rows and cells the same way down.
import { TEXT_KINDS } from './schema.js';
import { blockList, newBlock, newTable, textOf } from './loro.js';
import { kindOf, MARK_KEYS, ATTR_OF } from './model.js';
import { writeTable } from './table-model.js';

// One entry per UTF-16 unit: the block's text and, per mark key, its value.
const flatten = (node) => {
  let text = '';
  const marks = [];
  node.forEach((child) => {
    const str = child.isText ? child.text : '\n';
    const set = {};
    for (const mark of child.marks) set[mark.type.name] = ATTR_OF[mark.type.name] ? mark.attrs[ATTR_OF[mark.type.name]] : true;
    text += str;
    for (let i = 0; i < str.length; i += 1) marks.push(set);
  });
  return { text, marks };
};

const loroMarks = (text) => {
  const marks = [];
  for (const op of text.toDelta()) {
    if (typeof op.insert !== 'string') continue;
    for (let i = 0; i < op.insert.length; i += 1) marks.push(op.attributes ?? {});
  }
  return marks;
};

const spliceText = (text, next) => {
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

// An opaque block's `kind` is written only when this editor changed it:
// re-serialising one it never touched could rewrite the server's spelling
// of the same value.
const writeBlock = (map, node, before) => {
  const opaque = node.type.name === 'opaque';
  const kind = opaque ? node.attrs.kind : JSON.stringify(kindOf(node));
  const touched = !opaque || (before && before.attrs.kind !== node.attrs.kind);
  if (touched && map.get('kind') !== kind) map.set('kind', kind);
  const text = textOf(map);
  if (!text || !TEXT_KINDS.has(node.type.name)) return;
  const want = flatten(node);
  spliceText(text, want.text);
  const have = loroMarks(text);
  for (const key of MARK_KEYS) applyRuns(text, key, want.marks, have);
};

const moveTo = (list, at, id) => {
  for (let j = at + 1; j < list.length; j += 1) {
    if (list.get(j).get('id') !== id) continue;
    list.move(j, at);
    return list.get(at);
  }
  return null;
};

// Reconcile a list of Maps with `nodes` by id. `seen` maps id → the node
// last synced, so a node whose object did not change costs nothing.
const syncList = (seen, list, nodes, { create, write }) => {
  const ids = new Set(nodes.map((n) => n.attrs.id));
  for (let i = list.length - 1; i >= 0; i -= 1) {
    if (!ids.has(list.get(i).get('id'))) list.delete(i, 1);
  }
  nodes.forEach((node, at) => {
    const { id } = node.attrs;
    const current = at < list.length ? list.get(at) : null;
    let map = current?.get('id') === id ? current : null;
    if (map && seen.get(id) === node) return;
    map ??= moveTo(list, at, id) ?? create(list, at, node);
    write(map, node, at);
    seen.set(id, node);
  });
};

const writeBlocks = (seen, list, parent) => {
  const nodes = [];
  parent.forEach((node) => nodes.push(node));
  const ctx = {
    sync: (l, n, create, write) => syncList(seen, l, n, { create, write }),
    writeBlocks: (l, p) => writeBlocks(seen, l, p),
  };
  syncList(seen, list, nodes, {
    create: (l, at, node) => (node.type.name === 'table'
      ? newTable(l, at, node.attrs.id)
      : newBlock(l, at, node.attrs.id, kindOf(node))),
    write: (map, node) => (node.type.name === 'table'
      ? writeTable(map, node, ctx)
      : writeBlock(map, node, seen.get(node.attrs.id))),
  });
};

export const writeDoc = (doc, pm, seen) => writeBlocks(seen, blockList(doc), pm);

export const remember = (seen, pm) => pm.descendants((node) => {
  if (node.attrs.id) seen.set(node.attrs.id, node);
  return !node.isTextblock;
});
