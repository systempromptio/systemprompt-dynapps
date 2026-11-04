// Formula evaluation with IronCalc, the same engine (and version) core runs,
// so a value seen in the browser is the value an agent or an export reads.
// The model is a cache rebuilt from the Loro replica: cell inputs replay into
// it by position, and only the cells whose input changed are re-entered.
// A row or column reorder changes every position, so it rebuilds the model.
import init, { Model } from '@ironcalc/wasm';
import { rawResponse } from '../../services/api.js';
import { cellKey } from './replica.js';

const ENGINE = '@ironcalc/wasm';
const NUMBER = /^-?\d+(\.\d+)?(e-?\d+)?%?$/i;

let ready = null;

const wasmLink = () => {
  const node = document.getElementById('sp-vendor-wasm');
  const link = (node ? JSON.parse(node.textContent) : {})[ENGINE];
  if (!link) throw new Error('The formula engine is not in the vendor head.');
  return link;
};

export const loadCalc = () => {
  ready ??= (async () => {
    const { url, integrity } = wasmLink();
    const response = await rawResponse(url, { headers: {}, integrity });
    if (!response.ok) throw new Error(`Formula engine failed to load (${response.status}).`);
    await init({ module_or_path: response });
  })();
  return ready;
};

const shapeOf = (sheets) => sheets.map((s) => `${s.id}|${s.name}|${s.rows.join(',')}|${s.cols.join(',')}`).join('\n');

const enter = (model, at, input) => {
  try {
    model.setUserInput(at.sheet, at.row, at.col, input);
  } catch {
    // An input IronCalc cannot parse stays as typed; the cell shows it raw.
  }
};

const positions = (sheet, index) => {
  const rows = new Map(sheet.rows.map((id, i) => [id, i + 1]));
  const cols = new Map(sheet.cols.map((id, i) => [id, i + 1]));
  return (key) => {
    const [rowId, colId] = key.split(':');
    const row = rows.get(rowId);
    const col = cols.get(colId);
    return row && col ? { sheet: index, row, col } : null;
  };
};

const nameSheets = (model, sheets) => {
  sheets.forEach((sheet, i) => {
    if (i > 0) model.newSheet();
    try {
      model.renameSheet(i, sheet.name);
    } catch {
      // A name IronCalc refuses keeps its default; references by name to it miss.
    }
  });
};

export const createCalc = () => {
  let model = null;
  let shape = '';
  let seen = [];

  const rebuild = (sheets) => {
    model?.free();
    model = new Model('workbook', 'en', 'UTC', 'en');
    model.pauseEvaluation();
    nameSheets(model, sheets);
    sheets.forEach((sheet, i) => {
      const at = positions(sheet, i);
      for (const [key, input] of sheet.cells) {
        const pos = at(key);
        if (pos) enter(model, pos, input);
      }
    });
    model.resumeEvaluation();
    model.evaluate();
  };

  const patch = (sheets) => {
    model.pauseEvaluation();
    sheets.forEach((sheet, i) => {
      const at = positions(sheet, i);
      const before = seen[i].cells;
      for (const [key, input] of sheet.cells) {
        const pos = before.get(key) !== input && at(key);
        if (pos) enter(model, pos, input);
      }
      for (const key of before.keys()) {
        const pos = !sheet.cells.has(key) && at(key);
        if (pos) model.rangeClearContents(i, pos.row, pos.col, pos.row, pos.col);
      }
    });
    model.resumeEvaluation();
    model.evaluate();
  };

  return {
    sync: (sheets) => {
      const next = shapeOf(sheets);
      if (!model || next !== shape) rebuild(sheets);
      else patch(sheets);
      shape = next;
      seen = sheets;
    },
    display: (sheetIndex, row, col) => {
      const sheet = seen[sheetIndex];
      const rowId = sheet?.rows[row];
      const colId = sheet?.cols[col];
      if (rowId == null || colId == null || !sheet.cells.has(cellKey(rowId, colId))) return '';
      try {
        return model.getFormattedCellValue(sheetIndex, row + 1, col + 1);
      } catch {
        return sheet.cells.get(cellKey(rowId, colId));
      }
    },
    isNumber: (text) => NUMBER.test(text.replace(/,/g, '')),
  };
};
