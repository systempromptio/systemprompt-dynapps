// In-cell and formula-bar editing over the one selection. A cell is either
// selected (typing replaces it, Enter or a double-click edits it) or being
// edited, and both inputs edit the same draft: whichever one the user types
// in, the other mirrors it. A draft is written only on commit, so a remote
// change to the cell under the caret never overwrites what is being typed.
import { refOf } from './selection.js';

export const createCellEditor = ({ input, bar, ref, selection, grid, source }) => {
  let editing = null;

  const show = () => {
    const { head } = selection;
    ref.textContent = refOf(head);
    if (!editing) bar.value = source.input(head.row, head.col);
  };

  const begin = (initial = null, from = 'cell') => {
    if (!source.canEdit()) return;
    const { head } = selection;
    const value = initial ?? source.input(head.row, head.col);
    editing = { row: head.row, col: head.col };
    const { x, y, w, h } = grid.rectOf(head);
    input.style.setProperty('--sp-sheet-x', `${x}px`);
    input.style.setProperty('--sp-sheet-y', `${y}px`);
    input.style.setProperty('--sp-sheet-w', `${w}px`);
    input.style.setProperty('--sp-sheet-h', `${h}px`);
    input.value = value;
    bar.value = value;
    input.hidden = false;
    if (from === 'cell') {
      input.focus();
      input.setSelectionRange(value.length, value.length);
    }
  };

  const end = () => {
    editing = null;
    input.hidden = true;
    source.focus();
  };

  const commit = (dRow = 0, dCol = 0) => {
    if (!editing) return;
    const { row, col } = editing;
    const value = input.hidden ? bar.value : input.value;
    if (value !== source.input(row, col)) source.write([{ row, col, value }]);
    end();
    selection.set({ row: row + dRow, col: col + dCol });
  };

  const cancel = () => {
    if (!editing) return;
    end();
    show();
  };

  input.addEventListener('input', () => {
    bar.value = input.value;
  });
  bar.addEventListener('focus', () => {
    if (!editing) begin(bar.value, 'bar');
  });
  bar.addEventListener('input', () => {
    input.value = bar.value;
  });
  input.addEventListener('blur', (event) => {
    if (editing && event.relatedTarget !== bar) commit();
  });
  bar.addEventListener('blur', (event) => {
    if (editing && event.relatedTarget !== input) commit();
  });

  return { begin, commit, cancel, show, isEditing: () => editing !== null };
};
