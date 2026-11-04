// The spreadsheet's Loro replica, in the shape core's sheet engine reads:
// root `sheets` MovableList of sheet Maps {id, name, rows, cols, cells,
// merges}. Rows and columns are ordered lists of stable ids, and a cell is
// keyed "<rowId>:<colId>", so an inserted row moves no other cell's key and
// two people typing in different cells never touch the same container.
// Computed values are never written here: every replica evaluates for itself.
import { LoroMap, LoroMovableList, LoroList } from 'loro-crdt';

export const SHEETS = 'sheets';

const prefix = `s${Math.random().toString(36).slice(2, 8)}`;
let counter = 0;

const freshId = () => {
  counter += 1;
  return `${prefix}${counter.toString(36)}`;
};

export const cellKey = (rowId, colId) => `${rowId}:${colId}`;

export const sheetList = (doc) => doc.getMovableList(SHEETS);

// An axis as read: a repeated id is kept once, since two replicas growing
// the axis at once append the same grown ids.
const ids = (list) => [...new Set(list ? list.toJSON() : [])];

// The id a replica gives the entry it grows at `known.length`, derived as
// core's `sheet::edit::grown_id` derives it, so two replicas growing the same
// axis at once name the same rows and write the same cells.
const grownId = (sheetId, axis, known) => {
  const base = `${sheetId}.${axis === 'cols' ? 'c' : 'r'}${known.length}`;
  let id = base;
  for (let n = 1; known.includes(id); n += 1) id = `${base}.${n}`;
  return id;
};

const cellsOf = (map) => {
  const out = new Map();
  const cells = map.get('cells');
  if (!cells) return out;
  for (const key of cells.keys()) {
    const cell = cells.get(key);
    const v = cell instanceof LoroMap ? cell.get('v') : null;
    if (typeof v === 'string' && v !== '') out.set(key, v);
  }
  return out;
};

export const readSheets = (doc) => {
  const list = sheetList(doc);
  const out = [];
  for (let i = 0; i < list.length; i += 1) {
    const map = list.get(i);
    if (!(map instanceof LoroMap)) continue;
    out.push({
      id: map.get('id'),
      name: map.get('name') || `Sheet${i + 1}`,
      rows: ids(map.get('rows')),
      cols: ids(map.get('cols')),
      cells: cellsOf(map),
    });
  }
  return out;
};

export const addSheet = (doc, name) => {
  const list = sheetList(doc);
  const map = list.insertContainer(list.length, new LoroMap());
  map.set('id', freshId());
  map.set('name', name);
  map.setContainer('rows', new LoroMovableList());
  map.setContainer('cols', new LoroMovableList());
  map.setContainer('cells', new LoroMap());
  map.setContainer('merges', new LoroList());
  return map;
};

const sheetMap = (doc, sheetId) => {
  const list = sheetList(doc);
  for (let i = 0; i < list.length; i += 1) {
    const map = list.get(i);
    if (map instanceof LoroMap && map.get('id') === sheetId) return map;
  }
  return null;
};

// A cell past the last row or column the replica knows grows the axis to
// reach it with grown ids, the ones every replica gives that position.
const grow = (map, axis, index) => {
  const list = map.get(axis) ?? map.ensureMergeableMovableList(axis);
  const known = ids(list);
  while (known.length <= index) {
    const id = grownId(map.get('id'), axis, known);
    list.push(id);
    known.push(id);
  }
  return known[index];
};

export const writeCells = (doc, sheetId, writes) => {
  const map = sheetMap(doc, sheetId);
  if (!map) return false;
  const cells = map.get('cells') ?? map.ensureMergeableMap('cells');
  for (const { row, col, value } of writes) {
    const key = cellKey(grow(map, 'rows', row), grow(map, 'cols', col));
    if (value === '') {
      if (cells.get(key) !== undefined) cells.delete(key);
      continue;
    }
    const held = cells.get(key);
    const cell = held instanceof LoroMap ? held : cells.ensureMergeableMap(key);
    if (cell.get('v') !== value) cell.set('v', value);
  }
  doc.commit();
  return true;
};

export const indexOf = (sheet) => ({
  rows: new Map(sheet.rows.map((id, i) => [id, i])),
  cols: new Map(sheet.cols.map((id, i) => [id, i])),
});

export const inputAt = (sheet, row, col) => {
  const rowId = sheet.rows[row];
  const colId = sheet.cols[col];
  if (rowId == null || colId == null) return '';
  return sheet.cells.get(cellKey(rowId, colId)) ?? '';
};

// A re-join's unsent edits, merged over the new snapshot in `carried`, are
// written again into `doc` cell by cell under the new peer: a value that
// differs is set, and a cell `carried` no longer holds was cleared locally.
export const carryCells = (doc, carried) => {
  for (const was of readSheets(carried)) {
    const now = readSheets(doc).find((sheet) => sheet.id === was.id);
    if (!now) continue;
    const at = indexOf(was);
    const writes = [];
    const place = (key, value) => {
      const [rowId, colId] = key.split(':');
      const row = at.rows.get(rowId);
      const col = at.cols.get(colId);
      if (row != null && col != null) writes.push({ row, col, value });
    };
    for (const [key, value] of was.cells) if (now.cells.get(key) !== value) place(key, value);
    for (const key of now.cells.keys()) if (!was.cells.has(key)) place(key, '');
    if (writes.length) writeCells(doc, was.id, writes);
  }
};
