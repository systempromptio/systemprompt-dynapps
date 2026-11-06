// A core table block ⇄ prosemirror-tables nodes. The table Map holds
// `props` and `rows`; a row Map holds `cells`; a cell Map holds `props` and
// `blocks` (the same block Maps as the document, never another table).
// Props are core's native types: table `columns` a JSON string (widths in
// points; the editor carries them as each cell's `colwidth` in pixels),
// `header_rows` an integer, `style` a string; cell `colspan`/`rowspan`
// integers left out at 1, `shade`/`align`/`valign` strings. A header cell is
// whatever `header_rows` covers. Reading also takes the earlier spelling,
// where every value was a JSON string.
import { schema } from './schema.js';
import { childList, childMap, ensureList, ensureMap, listItems, newRow, newCell } from './loro.js';

const PT_PER_PX = 0.75;

const INTEGER_KEYS = new Set(['header_rows', 'colspan', 'rowspan']);

const propValue = (key, value) => {
  if (typeof value !== 'string') return value;
  if (INTEGER_KEYS.has(key)) return Number(value) || 0;
  if (key !== 'columns' && !value.startsWith('"')) return value;
  try {
    return JSON.parse(value);
  } catch {
    return value;
  }
};

const encode = (key, value) => {
  if (key === 'columns') return JSON.stringify(value);
  if (INTEGER_KEYS.has(key)) return Math.round(Number(value) || 0);
  return typeof value === 'string' ? value : JSON.stringify(value);
};

const readProps = (map) => {
  const out = {};
  for (const [key, value] of Object.entries(map?.toJSON() ?? {})) out[key] = propValue(key, value);
  return out;
};

const parseAttr = (value) => {
  try {
    return JSON.parse(value || '{}');
  } catch {
    return {};
  }
};

const widthsOf = (columns) => (Array.isArray(columns) ? columns : [])
  .map((column) => (column?.width ? Math.round(column.width / PT_PER_PX) : 0));

const cellNode = (map, { header, widths, col, readBlock }) => {
  const { colspan = 1, rowspan = 1, header: flag, ...extra } = readProps(childMap(map, 'props'));
  const span = Math.max(1, Number(colspan) || 1);
  const colwidth = widths.slice(col, col + span);
  const blocks = listItems(childList(map, 'blocks')).map((block) => readBlock(block, true));
  const type = (flag ?? header) ? schema.nodes.table_header : schema.nodes.table_cell;
  return type.create({
    id: map.get('id'),
    colspan: span,
    rowspan: Math.max(1, Number(rowspan) || 1),
    colwidth: colwidth.some(Boolean) ? colwidth : null,
    props: JSON.stringify(extra),
  }, blocks.length ? blocks : [schema.nodes.paragraph.create()]);
};

// `covered[c]` counts the rows still spanned into column c by a rowspan from
// above, so a cell's widths are read from the column it actually sits in.
const rowNode = (row, ctx) => {
  const { covered } = ctx;
  let col = 0;
  const skip = () => { while (covered[col] > 0) col += 1; };
  const cells = listItems(childList(row, 'cells')).map((cell) => {
    skip();
    const node = cellNode(cell, { ...ctx, col });
    for (let k = 0; k < node.attrs.colspan; k += 1) covered[col + k] = Math.max(covered[col + k] ?? 0, node.attrs.rowspan);
    col += node.attrs.colspan;
    return node;
  });
  for (let c = 0; c < covered.length; c += 1) if (covered[c] > 0) covered[c] -= 1;
  return schema.nodes.table_row.create({ id: row.get('id') }, cells);
};

// Null when the Map has no rows: the caller shows it as an opaque block.
export const tableNode = (map, readBlock) => {
  const { columns, ...rest } = readProps(childMap(map, 'props'));
  const widths = widthsOf(columns);
  const headerRows = Number(rest.header_rows ?? 0);
  const covered = [];
  const rows = listItems(childList(map, 'rows'))
    .map((row, r) => rowNode(row, { header: r < headerRows, widths, readBlock, covered }));
  if (!rows.length) return null;
  // Width-less columns ride along untouched until a resize replaces them.
  const props = widths.some(Boolean) || !columns ? rest : { ...rest, columns };
  return schema.nodes.table.create({ id: map.get('id'), props: JSON.stringify(props) }, rows);
};

// `columns` compares by value so core's own spelling of the same widths
// is not rewritten; every other key also rewrites an earlier JSON spelling.
const syncProps = (map, want) => {
  const have = readProps(map);
  for (const key of Object.keys(have)) if (!(key in want)) map.delete(key);
  for (const [key, value] of Object.entries(want)) {
    const next = encode(key, value);
    const same = key === 'columns' ? JSON.stringify(have[key]) === JSON.stringify(value) : map.get(key) === next;
    if (!same) map.set(key, next);
  }
};

const columnsOf = (table) => {
  const out = [];
  table.firstChild?.forEach((cell) => {
    for (let k = 0; k < cell.attrs.colspan; k += 1) {
      const px = cell.attrs.colwidth?.[k];
      out.push(px ? { width: Math.round(px * PT_PER_PX) } : {});
    }
  });
  return out.some((column) => column.width) ? out : null;
};

const children = (node) => {
  const out = [];
  node.forEach((child) => out.push(child));
  return out;
};

const writeCell = (map, cell, ctx) => {
  const { colspan, rowspan } = cell.attrs;
  syncProps(ensureMap(map, 'props'), {
    ...parseAttr(cell.attrs.props),
    ...(colspan > 1 ? { colspan } : {}),
    ...(rowspan > 1 ? { rowspan } : {}),
  });
  ctx.writeBlocks(ensureList(map, 'blocks'), cell);
};

// The context carries two writers: `sync(list, nodes, create, write)` reconciles
// a Map list by id, and `writeBlocks(list, parent)` writes the blocks of a cell.
export const writeTable = (map, node, ctx) => {
  const want = parseAttr(node.attrs.props);
  const columns = columnsOf(node);
  syncProps(ensureMap(map, 'props'), columns ? { ...want, columns } : want);
  ctx.sync(ensureList(map, 'rows'), children(node), (list, at, row) => newRow(list, at, row.attrs.id),
    (rowMap, row) => ctx.sync(ensureList(rowMap, 'cells'), children(row),
      (list, at, cell) => newCell(list, at, cell.attrs.id),
      (cellMap, cell) => writeCell(cellMap, cell, ctx)));
};
