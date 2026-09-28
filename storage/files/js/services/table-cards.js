import { on } from './events.js';

// Why: on a phone every table row is drawn as a card by CSS, and a card cell
// needs the name of its column beside the value. The server renders one header
// row and no per-cell labels across well over a hundred tables, so the labels
// are read from the header here and written onto each cell as `data-label`.
// Without this module the tables stay ordinary scrolling tables.

const TYPED_CELL = '.sp-col-date, .sp-col-status, .sp-table__cell--date, .sp-table__cell--num, .sp-table__cell--actions';
const SKIPPED_ROW = 'sp-table__row-detail';
const EMPTY_ROW = 'sp-table__empty';

// A cell holding nothing but a selection box is the row's checkbox. It gets no
// label, because a label beside a lone checkbox only repeats the header.
const isCheckOnly = (cell) => {
  if (cell.children.length !== 1) return false;
  if (!cell.children[0].matches('input[type="checkbox"]')) return false;
  return cell.textContent.trim() === '';
};

const cleanText = (th) => {
  const copy = th.cloneNode(true);
  for (const node of copy.querySelectorAll('.sp-table__sort-icon, [aria-hidden="true"]')) node.remove();
  return copy.textContent.replace(/\s+/g, ' ').trim();
};

// The same precedence a screen reader would use, with the tooltip last so an
// icon-only header such as the artifacts column still names itself.
const headerLabel = (th) => {
  if (isCheckOnly(th)) return '';
  return th.dataset.label
    || th.getAttribute('aria-label')
    || cleanText(th)
    || th.getAttribute('title')
    || '';
};

const columnLabels = (table) => {
  const head = table.tHead?.rows[0];
  if (!head) return null;
  const labels = [];
  for (const th of head.cells) {
    const label = headerLabel(th);
    for (let i = 0; i < th.colSpan; i += 1) labels.push(th.colSpan > 1 ? '' : label);
  }
  return labels;
};

// The identity column is the first one the row explicitly marks as primary,
// otherwise the first untyped cell after any checkbox. That is the desktop rule
// in 06-table.css, so the card title is the column that was widest on desktop.
const titleCell = (cells) => {
  const marked = cells.find((cell) => cell.classList.contains('sp-table__cell--primary'));
  if (marked) return marked;
  const first = cells.find((cell) => !isCheckOnly(cell));
  if (!first || first.colSpan > 1) return null;
  if (!first.matches(TYPED_CELL)) return first;
  // Why: a table that opens with a date or a count still has an identity
  // column further along; the first untyped cell is it, so the card is never
  // left without a heading.
  return cells.find((cell) => !isCheckOnly(cell) && cell.colSpan === 1 && !cell.matches(TYPED_CELL)) || null;
};

const setLabel = (cell, label) => {
  if (label) cell.dataset.label = label;
  else delete cell.dataset.label;
};

const labelRow = (row, labels) => {
  const cells = Array.from(row.cells);
  const title = titleCell(cells);
  let column = 0;
  for (const cell of cells) {
    const span = cell.colSpan;
    delete cell.dataset.cardCell;
    if (isCheckOnly(cell)) cell.dataset.cardCell = 'check';
    else if (cell === title) cell.dataset.cardCell = 'title';
    const wanted = span > 1 || cell.dataset.cardCell ? '' : labels[column] || '';
    setLabel(cell, wanted);
    column += span;
  }
};

const cardWrapper = (table) => {
  const scroll = table.parentElement;
  if (!scroll?.classList.contains('sp-table__scroll')) return null;
  const wrapper = scroll.parentElement;
  if (!wrapper?.classList.contains('sp-table')) return null;
  if (wrapper.classList.contains('sp-table--scroll')) return null;
  return wrapper;
};

const isCardTable = (table) => !table.classList.contains('sp-table__el--nested')
  && !table.classList.contains('sp-audience-matrix');

// Sorting lives in the header links, and the header row is hidden on a phone,
// so the sortable columns are offered again as one select above the cards.
const sortOptions = (table) => {
  const options = [];
  for (const th of table.tHead.rows[0].cells) {
    const link = th.querySelector('a.sp-table__sort');
    const direction = th.getAttribute('aria-sort');
    if (!link || !direction) continue;
    const label = cleanText(th);
    if (direction === 'none') {
      options.push(new Option(label, link.href));
      continue;
    }
    const reverse = direction === 'descending' ? 'ascending' : 'descending';
    options.push(new Option(`${label}, ${direction}`, '', true, true));
    options.push(new Option(`${label}, ${reverse}`, link.href));
  }
  return options;
};

const addSortSelect = (wrapper, table) => {
  if (wrapper.querySelector(':scope > .sp-table__sort-bar')) return;
  const options = sortOptions(table);
  if (options.length < 2) return;
  const bar = document.createElement('label');
  bar.className = 'sp-table__sort-bar';
  const text = document.createElement('span');
  text.textContent = 'Sort by';
  const select = document.createElement('select');
  select.className = 'sp-table__sort-select';
  select.append(...options);
  bar.append(text, select);
  wrapper.prepend(bar);
};

export const labelTable = (table) => {
  if (!isCardTable(table)) return;
  const wrapper = cardWrapper(table);
  if (!wrapper) return;
  const labels = columnLabels(table);
  if (!labels) return;
  for (const section of [...table.tBodies, table.tFoot].filter(Boolean)) {
    for (const row of section.rows) {
      if (row.classList.contains(SKIPPED_ROW) || row.classList.contains(EMPTY_ROW)) continue;
      labelRow(row, labels);
    }
  }
  addSortSelect(wrapper, table);
  wrapper.dataset.cards = 'ready';
};

export const labelTables = (root = document) => {
  for (const table of root.querySelectorAll('.sp-table__el')) labelTable(table);
};

// A table rendered after load (a filtered reload, a probe result) is found
// through the subtree it arrived in, so unrelated DOM churn such as chart
// tooltips never walks every table on the page again.
const tablesTouchedBy = (records) => {
  const found = new Set();
  for (const record of records) {
    for (const node of record.addedNodes) {
      if (node.nodeType !== Node.ELEMENT_NODE) continue;
      const host = node.closest('.sp-table__el:not(.sp-table__el--nested)');
      if (host) found.add(host);
      for (const table of node.querySelectorAll('.sp-table__el')) found.add(table);
    }
  }
  return found;
};

export const initTableCards = () => {
  labelTables(document);
  new MutationObserver((records) => {
    for (const table of tablesTouchedBy(records)) labelTable(table);
  }).observe(document.body, { childList: true, subtree: true });
  on('change', '.sp-table__sort-select', (e, select) => {
    if (select.value) window.location.assign(select.value);
  });
};
