// Keyboard and pointer input for the grid, in the spreadsheet convention:
// arrows and Tab move (Shift extends), Enter or F2 or a double-click edits,
// a printable key replaces the cell, Delete clears the range, Escape drops a
// draft. While a cell is being edited, only the keys that end the edit are
// taken; everything else belongs to the input.
const ARROWS = { ArrowUp: [-1, 0], ArrowDown: [1, 0], ArrowLeft: [0, -1], ArrowRight: [0, 1] };

const endKeys = (event, editor) => {
  if (event.key === 'Enter') editor.commit(event.shiftKey ? -1 : 1, 0);
  else if (event.key === 'Tab') editor.commit(0, event.shiftKey ? -1 : 1);
  else if (event.key === 'Escape') editor.cancel();
  else return false;
  return true;
};

const clearRange = (selection, source) => {
  const { top, left, bottom, right } = selection.range();
  const writes = [];
  for (let row = top; row <= bottom; row += 1) {
    for (let col = left; col <= right; col += 1) {
      if (source.input(row, col) !== '') writes.push({ row, col, value: '' });
    }
  }
  if (writes.length) source.write(writes);
};

const selectKeys = (event, { selection, editor, source }) => {
  const arrow = ARROWS[event.key];
  if (arrow) selection.move(arrow[0], arrow[1], event.shiftKey);
  else if (event.key === 'Tab') selection.move(0, event.shiftKey ? -1 : 1);
  else if (event.key === 'Enter' || event.key === 'F2') editor.begin();
  else if (event.key === 'Delete' || event.key === 'Backspace') clearRange(selection, source);
  else if (event.key.length === 1 && !event.ctrlKey && !event.metaKey && !event.altKey) editor.begin(event.key);
  else return false;
  return true;
};

export const bindInput = ({ viewport, layer, input, bar }, ctx) => {
  const { selection, editor, grid } = ctx;
  let dragging = false;

  const onKey = (event) => {
    const handled = editor.isEditing() ? endKeys(event, editor) : selectKeys(event, ctx);
    if (handled) event.preventDefault();
  };
  viewport.addEventListener('keydown', onKey);
  // The input sits inside the viewport: an Enter that commits here would
  // bubble on, find nothing being edited, and open the editor again.
  input.addEventListener('keydown', (event) => {
    onKey(event);
    event.stopPropagation();
  });
  bar.addEventListener('keydown', (event) => {
    if (endKeys(event, editor)) event.preventDefault();
  });

  layer.addEventListener('pointerdown', (event) => {
    if (event.button !== 0) return;
    if (editor.isEditing()) editor.commit();
    selection.set(grid.cellAt(event.clientX, event.clientY), event.shiftKey);
    dragging = true;
    layer.setPointerCapture(event.pointerId);
    viewport.focus({ preventScroll: true });
    event.preventDefault();
  });
  layer.addEventListener('pointermove', (event) => {
    if (dragging) selection.set(grid.cellAt(event.clientX, event.clientY), true);
  });
  layer.addEventListener('pointerup', () => {
    dragging = false;
  });
  layer.addEventListener('dblclick', () => editor.begin());
};
