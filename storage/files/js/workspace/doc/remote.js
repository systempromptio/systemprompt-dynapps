// Remote Loro changes → the ProseMirror view, outside the undo history.
// Each replace covers only the smallest range that differs, down to
// characters inside a block, so a remote keystroke never moves the local
// caret out of the block it sits in (a whole-block replace would map it to
// the block's end). An import that touched only some blocks rebuilds only
// those; a change to the block list itself, or a block the view cannot
// find, falls back to diffing the whole document.
import { blockList, BLOCKS } from './loro.js';
import { blockNode, readDoc } from './model.js';

const diffInto = (tr, base, prev, next) => {
  const start = prev.content.findDiffStart(next.content);
  if (start == null) return;
  let { a: endPrev, b: endNext } = prev.content.findDiffEnd(next.content);
  const overlap = start - Math.min(endPrev, endNext);
  if (overlap > 0) {
    endPrev += overlap;
    endNext += overlap;
  }
  tr.replace(base + start, base + endPrev, next.slice(start, endNext));
};

const dispatchRemote = (view, tr) => {
  if (!tr.docChanged) return;
  view.dispatch(tr.setMeta('remote', true).setMeta('addToHistory', false));
};

export const syncFromLoro = (view, loroDoc) => {
  const { tr } = view.state;
  diffInto(tr, 0, view.state.doc, readDoc(loroDoc));
  dispatchRemote(view, tr);
};

// Top-level block indexes an event batch touched, or null when the block
// list itself changed (path `['blocks']`) or a path is not under it.
const touchedIndexes = (batch) => {
  const out = new Set();
  for (const { path } of batch?.events ?? []) {
    if (path?.[0] !== BLOCKS || typeof path[1] !== 'number') return null;
    out.add(path[1]);
  }
  return out;
};

const positions = (doc) => {
  const out = new Map();
  doc.forEach((node, offset) => out.set(node.attrs.id, { node, offset }));
  return out;
};

export const applyRemote = (view, loroDoc, batch) => {
  const indexes = touchedIndexes(batch);
  const list = blockList(loroDoc);
  if (!indexes || [...indexes].some((i) => i >= list.length)) {
    syncFromLoro(view, loroDoc);
    return;
  }
  const at = positions(view.state.doc);
  const changes = [...indexes].map((i) => blockNode(list.get(i)))
    .map((next) => ({ next, prev: at.get(next.attrs.id) }));
  if (changes.some((c) => !c.prev)) {
    syncFromLoro(view, loroDoc);
    return;
  }
  const { tr } = view.state;
  changes.sort((x, y) => y.prev.offset - x.prev.offset);
  for (const { next, prev: { node, offset } } of changes) {
    if (node.sameMarkup(next)) diffInto(tr, offset + 1, node, next);
    else tr.replaceWith(offset, offset + node.nodeSize, next);
  }
  dispatchRemote(view, tr);
};
