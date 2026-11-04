// Every block, table, row and cell keeps a unique, stable id: a split or a paste can
// copy one, so the first holder keeps it and the copy gets a fresh id. The
// server refuses an update with an empty or repeated id (`bad_block_id`).
import { Plugin } from 'prosemirror-state';

const prefix = `c${Math.random().toString(36).slice(2, 8)}`;
let counter = 0;

export const freshId = () => {
  counter += 1;
  return `${prefix}-${counter}`;
};

export const idsPlugin = () => new Plugin({
  appendTransaction: (trs, _old, state) => {
    if (!trs.some((tr) => tr.docChanged)) return null;
    const seen = new Set();
    let tr = null;
    state.doc.descendants((node, pos) => {
      if (!('id' in node.attrs)) return true;
      const { id } = node.attrs;
      if (id && !seen.has(id)) {
        seen.add(id);
      } else {
        const next = freshId();
        seen.add(next);
        tr ??= state.tr;
        tr.setNodeMarkup(pos, null, { ...node.attrs, id: next });
      }
      return !node.isTextblock;
    });
    return tr;
  },
});

// Opaque blocks (tables, images, native markup) belong to the server: a
// local change may not delete them, so refuse a transaction that would.
export const keepOpaquePlugin = () => new Plugin({
  filterTransaction: (tr, state) => {
    if (!tr.docChanged || tr.getMeta('remote')) return true;
    const ids = (doc) => {
      const out = [];
      doc.descendants((node) => { if (node.type.name === 'opaque') out.push(node.attrs.id); });
      return out;
    };
    const after = new Set(ids(tr.doc));
    return ids(state.doc).every((id) => after.has(id));
  },
});
