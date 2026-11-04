// The spreadsheet editor: a grid over the live session's Loro sheet replica,
// which is the only source of truth. Local edits write cell inputs into Loro
// (the session posts the update); updates from anyone else arrive on the
// document's live channel and repaint. IronCalc evaluates in the browser, so
// no computed value ever travels. Below edit access the grid is read-only.
import { loadEngine } from '../doc/loro.js';
import { createSession } from '../doc/session.js';
import { loadCalc, createCalc } from './formula.js';
import { readSheets, addSheet, writeCells, inputAt, carryCells } from './replica.js';
import { createSelection } from './selection.js';
import { createGrid } from './grid.js';
import { createCellEditor } from './cell-editor.js';
import { bindInput } from './keys.js';
import { bindClipboard } from './clipboard.js';
import { createPresence } from './presence.js';
import { layout, renderTabs } from './layout.js';

const SPARE_ROWS = 50;
const SPARE_COLS = 10;
const MIN_ROWS = 200;
const MIN_COLS = 26;

export const mount = async (slot, { root, readOnly }) => {
  await Promise.all([loadEngine(), loadCalc()]);
  const api = root.dataset.documentApi;
  const saveLabel = root.querySelector('[data-field="save"]');
  const parts = layout(slot);
  const calc = createCalc();
  const view = { sheets: [], active: 0 };
  let grid = null;
  let editor = null;
  let presence = null;

  const sheet = () => view.sheets[view.active];

  const refresh = () => {
    view.sheets = readSheets(session.state.doc);
    if (!view.sheets.length && session.state.mode === 'edit') {
      addSheet(session.state.doc, 'Sheet1');
      session.state.doc.commit();
      view.sheets = readSheets(session.state.doc);
    }
    view.active = Math.min(view.active, Math.max(0, view.sheets.length - 1));
    calc.sync(view.sheets);
    renderTabs(parts.tabs, view, (index) => {
      view.active = index;
      selection.set({ row: 0, col: 0 });
    });
    editor?.show();
    if (presence) grid.setPeers(presence.ranges(sheet()));
    else grid?.schedule();
  };

  const session = createSession({
    mode: readOnly ? 'comment' : 'edit',
    collabApi: `${api}/collab`,
    liveApi: `${api}/live`,
    onState: (text) => {
      if (saveLabel) saveLabel.textContent = text;
    },
    onReset: (s, carried) => {
      if (carried) carryCells(s.doc, carried);
      refresh();
    },
    onRemote: () => refresh(),
    onEvent: (name, data) => {
      if (name === 'document.awareness') presence?.receive(data);
    },
  });

  const selection = createSelection(() => {
    grid?.reveal(selection.head);
    editor?.show();
    presence?.moved(sheet(), selection);
  });

  const source = {
    selection,
    canEdit: () => session.state.mode === 'edit',
    focus: () => parts.viewport.focus({ preventScroll: true }),
    extent: () => ({
      rows: Math.max(MIN_ROWS, (sheet()?.rows.length ?? 0) + SPARE_ROWS),
      cols: Math.max(MIN_COLS, (sheet()?.cols.length ?? 0) + SPARE_COLS),
    }),
    input: (row, col) => (sheet() ? inputAt(sheet(), row, col) : ''),
    cell: (row, col) => {
      const text = calc.display(view.active, row, col);
      return { text, isNumber: text !== '' && calc.isNumber(text) };
    },
    write: (writes) => {
      if (!sheet() || !source.canEdit()) return;
      if (!writeCells(session.state.doc, sheet().id, writes)) return;
      session.flush();
      refresh();
    },
  };

  await session.start();
  grid = createGrid(parts, source);
  editor = createCellEditor({ ...parts, selection, grid, source });
  presence = createPresence({
    api: `${api}/collab/awareness`,
    session,
    selfId: root.dataset.userId,
    onPeers: () => grid.setPeers(presence.ranges(sheet())),
  });
  bindInput(parts, { selection, editor, grid, source });
  bindClipboard(parts.viewport, { selection, editor, source });
  refresh();
  parts.bar.readOnly = !source.canEdit();
  presence.moved(sheet(), selection);
  return {
    readOnly: !source.canEdit(),
    hint: 'Click a cell and type. Enter or a double-click edits; arrows and Tab move; Shift extends the range.',
  };
};
