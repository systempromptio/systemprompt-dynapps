// A read-only grid drawn from a document projection, for sheet formats with
// no live session (OpenDocument: core's formula engine cannot read it). The
// same grid, selection and keyboard model as the live editor, so navigation
// and copy behave identically; values are the projection's cell text, and a
// new revision redraws them.
import { layout, renderTabs } from './layout.js';
import { createSelection } from './selection.js';
import { createGrid } from './grid.js';
import { createCellEditor } from './cell-editor.js';
import { bindInput } from './keys.js';
import { bindClipboard } from './clipboard.js';

const NUMBER = /^-?\d+(\.\d+)?$/;
const MIN_ROWS = 100;
const MIN_COLS = 26;

const parseRef = (path) => {
  const match = /\/([A-Z]+)(\d+)$/.exec(path ?? '');
  if (!match) return null;
  const col = [...match[1]].reduce((acc, ch) => acc * 26 + ch.charCodeAt(0) - 64, 0) - 1;
  return { row: Number(match[2]) - 1, col };
};

const readProjection = (projection) => (projection?.root ?? [])
  .filter((node) => node.kind?.type === 'sheet')
  .map((sheet, i) => {
    const cells = new Map();
    let rows = 0;
    let cols = 0;
    for (const row of sheet.children ?? []) {
      for (const cell of row.children ?? []) {
        const at = parseRef(cell.attrs?.path);
        if (!at) continue;
        cells.set(`${at.row}:${at.col}`, cell.text ?? '');
        rows = Math.max(rows, at.row + 1);
        cols = Math.max(cols, at.col + 1);
      }
    }
    return { name: sheet.attrs?.name ?? sheet.text ?? `Sheet${i + 1}`, cells, rows, cols };
  });

export const createProjectionSheet = (root) => {
  const parts = layout(root);
  const view = { sheets: [], active: 0 };
  const sheet = () => view.sheets[view.active];
  let grid = null;
  let editor = null;

  const selection = createSelection(() => {
    grid?.reveal(selection.head);
    editor?.show();
  });
  const source = {
    selection,
    canEdit: () => false,
    focus: () => parts.viewport.focus({ preventScroll: true }),
    extent: () => ({ rows: Math.max(MIN_ROWS, (sheet()?.rows ?? 0) + 20), cols: Math.max(MIN_COLS, (sheet()?.cols ?? 0) + 5) }),
    input: (row, col) => sheet()?.cells.get(`${row}:${col}`) ?? '',
    cell: (row, col) => {
      const text = source.input(row, col);
      return { text, isNumber: NUMBER.test(text.trim()) };
    },
    write: () => undefined,
  };
  grid = createGrid(parts, source);
  editor = createCellEditor({ ...parts, selection, grid, source });
  bindInput(parts, { selection, editor, grid, source });
  bindClipboard(parts.viewport, { selection, editor, source });
  parts.bar.readOnly = true;

  const render = (projection) => {
    view.sheets = readProjection(projection);
    view.active = Math.min(view.active, Math.max(0, view.sheets.length - 1));
    renderTabs(parts.tabs, view, (index) => {
      view.active = index;
      selection.set({ row: 0, col: 0 });
    });
    editor.show();
    grid.schedule();
  };

  return { render, update: render, hint: 'This spreadsheet is read-only here. Arrows move; Shift extends; copy works.' };
};
