// The block editor: one textarea per text block of the head revision's
// projection. Saving sends one edits call against the revision the editor
// was loaded from; core refuses it if someone else committed first, and the
// editor reloads from the new head so nothing is written over their change.
import { rawFetch } from './api.js';

const TEXT_KINDS = new Set(['paragraph', 'heading', 'list_item']);

const kindLabel = (kind) => {
  if (kind.type === 'heading') return `Heading ${kind.level}`;
  if (kind.type === 'list_item') return 'List item';
  if (kind.type === 'paragraph') return 'Paragraph';
  return kind.type.replace(/_/g, ' ');
};

const flatten = (nodes, out = []) => {
  for (const node of nodes ?? []) {
    out.push(node);
    if (!TEXT_KINDS.has(node.kind.type)) flatten(node.children, out);
  }
  return out;
};

const grow = (area) => {
  area.style.blockSize = 'auto';
  area.style.blockSize = `${area.scrollHeight}px`;
};

const blockRow = (block) => {
  const template = document.querySelector('template[data-template="drive-block"]');
  const row = template.content.firstElementChild.cloneNode(true);
  row.querySelector('[data-field="kind"]').textContent = kindLabel(block.kind);
  const area = row.querySelector('[data-field="text"]');
  area.value = block.text ?? '';
  area.readOnly = !block.editable;
  area.addEventListener('input', () => grow(area));
  if (block.kind.type === 'heading') row.classList.add('sp-p-drive__block--heading');
  row.querySelectorAll('[data-action^="drive-add"], [data-action="drive-remove"]').forEach((b) => {
    b.hidden = !block.editable;
  });
  row.dataset.key = block.key;
  return row;
};

export const createEditor = ({ form, projectionApi, editsApi, onSaved }) => {
  const list = form.querySelector('[data-slot="drive-blocks"]');
  const note = form.querySelector('[data-slot="drive-editor-note"]');
  const state = { base: null, blocks: [], nextKey: 0 };

  const render = () => {
    list.replaceChildren(...state.blocks.filter((b) => !b.removed).map(blockRow));
    list.querySelectorAll('textarea').forEach(grow);
  };

  const load = async () => {
    const { revision_id: base, projection } = await rawFetch(projectionApi);
    state.base = base;
    state.blocks = flatten(projection.root).map((node) => ({
      key: `n${state.nextKey++}`,
      id: node.id,
      hash: node.hash8,
      kind: node.kind,
      text: node.text ?? '',
      original: node.text ?? '',
      editable: TEXT_KINDS.has(node.kind.type),
    }));
    const locked = state.blocks.filter((b) => !b.editable).length;
    note.textContent = locked
      ? `${locked} block(s) such as tables, cells or slides are shown read-only here; edit those through the MCP tools or the desktop file.`
      : '';
    render();
  };

  const find = (key) => state.blocks.find((b) => b.key === key);

  const capture = () => {
    for (const row of list.children) {
      find(row.dataset.key).text = row.querySelector('textarea').value;
    }
  };

  const insertAfter = (key, kind) => {
    capture();
    const index = state.blocks.findIndex((b) => b.key === key);
    const anchor = state.blocks[index];
    state.blocks.splice(index + 1, 0, {
      key: `n${state.nextKey++}`,
      anchor: anchor.id ?? anchor.anchor,
      kind,
      text: '',
      editable: true,
      fresh: true,
    });
    render();
    list.querySelector(`[data-key="${state.blocks[index + 1].key}"] textarea`)?.focus();
  };

  const ops = () => {
    capture();
    const out = [];
    for (const block of state.blocks) {
      if (block.fresh && !block.removed && block.text.trim()) {
        out.push({ op: 'insert_after', node: block.anchor, kind: block.kind, text: block.text });
      } else if (!block.fresh && block.removed) {
        out.push({ op: 'delete', node: block.id, expected_hash: block.hash });
      } else if (!block.fresh && block.editable && block.text !== block.original) {
        out.push({ op: 'replace_text', node: block.id, expected_hash: block.hash, text: block.text });
      }
    }
    return out;
  };

  const save = async () => {
    const pending = ops();
    if (!pending.length) {
      note.textContent = 'Nothing changed.';
      return;
    }
    const message = form.elements.message.value.trim() || null;
    try {
      await rawFetch(editsApi, {
        method: 'POST',
        body: JSON.stringify({ base_revision: state.base, ops: pending, message }),
      });
      form.elements.message.value = '';
      await onSaved();
      await load();
    } catch {
      await load();
      note.textContent = 'The save was refused; the editor now shows the current revision. Re-apply your change.';
    }
  };

  const keyOf = (target) => target.closest('[data-key]')?.dataset.key;

  form.addEventListener('click', (event) => {
    const action = event.target.closest('[data-action]')?.dataset.action;
    const key = keyOf(event.target);
    if (action === 'drive-add-paragraph') insertAfter(key, { type: 'paragraph' });
    if (action === 'drive-add-heading') insertAfter(key, { type: 'heading', level: 2 });
    if (action === 'drive-remove') {
      capture();
      find(key).removed = true;
      render();
    }
  });
  form.addEventListener('submit', (event) => {
    event.preventDefault();
    save();
  });

  return { load };
};
