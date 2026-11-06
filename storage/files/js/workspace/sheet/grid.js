// The virtualised grid: only the rows and columns in view (plus an overscan)
// exist as DOM, positioned on one scroll layer by CSS custom properties. The
// headers sit outside the scroller and follow its offset, so they stay put on
// both axes without sticky positioning inside an absolutely laid-out layer.
// Row height and column width come from the stylesheet, measured once.
import { columnName, inRange } from './selection.js';

const OVERSCAN = 4;

const el = (tag, className) => Object.assign(document.createElement(tag), { className });

const place = (node, x, y) => {
  node.style.setProperty('--sp-sheet-x', `${x}px`);
  node.style.setProperty('--sp-sheet-y', `${y}px`);
  return node;
};

const applyLook = (node, look) => {
  if (look.fill) {
    node.style.setProperty('--sp-sheet-fill', look.fill);
    node.classList.add('sp-sheet__cell--filled');
  }
  if (look.ink) {
    node.style.setProperty('--sp-sheet-ink', look.ink);
    node.classList.add('sp-sheet__cell--inked');
  }
  node.classList.toggle('sp-sheet__cell--bold', look.bold);
  node.classList.toggle('sp-sheet__cell--italic', look.italic);
  node.classList.toggle('sp-sheet__cell--underline', look.underline);
  node.classList.toggle('sp-sheet__cell--strike', look.strike);
};

const measure = (layer) => {
  const probe = el('div', 'sp-sheet__cell');
  layer.append(probe);
  const size = { row: probe.offsetHeight || 24, col: probe.offsetWidth || 104 };
  probe.remove();
  return size;
};

export const createGrid = ({ viewport, layer, colHeads, rowHeads, sizer }, source) => {
  let size = null;
  let frame = 0;
  let peers = [];

  const span = () => {
    const { scrollTop: top, clientHeight: height, scrollLeft: left, clientWidth: width } = viewport;
    return {
      r0: Math.max(0, Math.floor(top / size.row) - OVERSCAN),
      r1: Math.ceil((top + height) / size.row) + OVERSCAN,
      c0: Math.max(0, Math.floor(left / size.col) - OVERSCAN),
      c1: Math.ceil((left + width) / size.col) + OVERSCAN,
    };
  };

  const cell = (row, col, range) => {
    const { text, isNumber, look } = source.cell(row, col);
    const node = place(el('div', 'sp-sheet__cell'), col * size.col, row * size.row);
    node.textContent = text;
    node.dataset.ref = `${columnName(col)}${row + 1}`;
    if (isNumber) node.classList.add('sp-sheet__cell--number');
    if (look) applyLook(node, look);
    if (inRange(range, row, col)) node.classList.add('is-selected');
    return node;
  };

  const box = (range, className) => {
    const node = place(el('div', className), range.left * size.col, range.top * size.row);
    node.style.setProperty('--sp-sheet-w', `${(range.right - range.left + 1) * size.col}px`);
    node.style.setProperty('--sp-sheet-h', `${(range.bottom - range.top + 1) * size.row}px`);
    return node;
  };

  const heads = (v, range) => {
    const cols = [];
    const rows = [];
    for (let c = v.c0; c <= v.c1; c += 1) {
      const node = place(el('div', 'sp-sheet__head'), c * size.col, 0);
      node.textContent = columnName(c);
      node.classList.toggle('is-active', c >= range.left && c <= range.right);
      cols.push(node);
    }
    for (let r = v.r0; r <= v.r1; r += 1) {
      const node = place(el('div', 'sp-sheet__head sp-sheet__head--row'), 0, r * size.row);
      node.textContent = String(r + 1);
      node.classList.toggle('is-active', r >= range.top && r <= range.bottom);
      rows.push(node);
    }
    colHeads.replaceChildren(...cols);
    rowHeads.replaceChildren(...rows);
  };

  const paint = () => {
    frame = 0;
    size ??= measure(layer);
    const { rows, cols } = source.extent();
    sizer.style.setProperty('--sp-sheet-w', `${cols * size.col}px`);
    sizer.style.setProperty('--sp-sheet-h', `${rows * size.row}px`);
    const v = span();
    const range = source.selection.range();
    const nodes = [];
    for (let r = v.r0; r <= Math.min(v.r1, rows - 1); r += 1) {
      for (let c = v.c0; c <= Math.min(v.c1, cols - 1); c += 1) nodes.push(cell(r, c, range));
    }
    for (const peer of peers) {
      const node = box(peer.range, `sp-sheet__peer sp-doc-select--tone-${peer.tone}`);
      node.dataset.name = peer.name;
      nodes.push(node);
    }
    nodes.push(box(range, 'sp-sheet__range'));
    const { head } = source.selection;
    nodes.push(box({ top: head.row, left: head.col, bottom: head.row, right: head.col }, 'sp-sheet__active'));
    layer.replaceChildren(...nodes);
    heads(v, range);
    colHeads.style.setProperty('--sp-sheet-sx', `${-viewport.scrollLeft}px`);
    rowHeads.style.setProperty('--sp-sheet-sy', `${-viewport.scrollTop}px`);
  };

  const schedule = () => {
    frame ||= requestAnimationFrame(paint);
  };

  viewport.addEventListener('scroll', schedule, { passive: true });
  new ResizeObserver(schedule).observe(viewport);

  return {
    schedule,
    setPeers: (next) => {
      peers = next;
      schedule();
    },
    cellAt: (clientX, clientY) => {
      const rect = layer.getBoundingClientRect();
      size ??= measure(layer);
      return { row: Math.floor((clientY - rect.top) / size.row), col: Math.floor((clientX - rect.left) / size.col) };
    },
    rectOf: ({ row, col }) => ({ x: col * size.col, y: row * size.row, w: size.col, h: size.row }),
    reveal: ({ row, col }) => {
      size ??= measure(layer);
      const { scrollTop: top, scrollLeft: left, clientHeight: h, clientWidth: w } = viewport;
      if (row * size.row < top) viewport.scrollTop = row * size.row;
      else if ((row + 1) * size.row > top + h) viewport.scrollTop = (row + 1) * size.row - h;
      if (col * size.col < left) viewport.scrollLeft = col * size.col;
      else if ((col + 1) * size.col > left + w) viewport.scrollLeft = (col + 1) * size.col - w;
      schedule();
    },
  };
};
