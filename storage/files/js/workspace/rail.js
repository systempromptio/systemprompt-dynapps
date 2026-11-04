// The navigation rail: headings for documents, sheet tabs for workbooks and
// slides for decks, read from the head revision's projection. Choosing an
// entry asks the mounted editor to reveal that node.
import { readJson } from '../services/drive-rows.js';

const DECKS = new Set(['pptx', 'odp']);
const SHEETS = new Set(['xlsx', 'ods']);

const railTitle = (format) => {
  if (DECKS.has(format)) return 'Slides';
  if (SHEETS.has(format)) return 'Sheets';
  return 'Outline';
};

const collect = (nodes, keep, out = []) => {
  for (const node of nodes ?? []) {
    if (keep(node)) out.push(node);
    else collect(node.children, keep, out);
  }
  return out;
};

const entryLabel = (node, index) => {
  if (node.kind.type === 'slide') return node.attrs?.title || node.text || `Slide ${index + 1}`;
  if (node.kind.type === 'sheet') return node.attrs?.name || node.text || `Sheet ${index + 1}`;
  return node.text || 'Untitled heading';
};

const keepFor = (format) => {
  if (DECKS.has(format)) return (node) => node.kind.type === 'slide';
  if (SHEETS.has(format)) return (node) => node.kind.type === 'sheet';
  return (node) => node.kind.type === 'heading';
};

const outlineItem = (node, index) => {
  const template = document.querySelector('template[data-template="ws-outline-item"]');
  const item = template.content.firstElementChild.cloneNode(true);
  const button = item.querySelector('[data-field="text"]');
  button.textContent = entryLabel(node, index);
  button.dataset.node = node.id;
  if (node.kind.level) item.dataset.level = String(node.kind.level);
  return item;
};

export const showRail = async (root, format) => {
  root.querySelector('[data-field="rail-title"]').textContent = railTitle(format);
  const list = root.querySelector('[data-slot="ws-outline"]');
  const empty = root.querySelector('[data-slot="ws-outline-empty"]');
  try {
    const { projection } = await readJson(root.dataset.projectionApi);
    const entries = collect(projection.root, keepFor(format));
    list.replaceChildren(...entries.map(outlineItem));
    empty.hidden = entries.length > 0;
  } catch {
    list.replaceChildren();
    empty.hidden = false;
  }
  const mark = (event) => {
    const node = event.detail?.node;
    list.querySelectorAll('[data-node]').forEach((b) => {
      if (b.dataset.node === node) b.setAttribute('aria-current', 'true');
      else b.removeAttribute('aria-current');
    });
  };
  root.addEventListener('ws:anchor', mark);
  root.addEventListener('ws:reveal', mark);
  list.addEventListener('click', (event) => {
    const entry = event.target.closest('[data-node]');
    if (entry) root.dispatchEvent(new CustomEvent('ws:reveal', { detail: { node: entry.dataset.node, label: entry.textContent } }));
  });
};
