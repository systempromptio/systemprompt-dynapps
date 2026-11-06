// Copy, cut and paste as tab-separated text, the format every spreadsheet
// and text editor exchanges: rows by newline, cells by tab. Copy carries the
// cell inputs (formulas included), so a paste back into this grid keeps them.
// A paste lands at the active cell and writes in one Loro commit.
const toTsv = (selection, source) => {
  const { top, left, bottom, right } = selection.range();
  const lines = [];
  for (let row = top; row <= bottom; row += 1) {
    const cells = [];
    for (let col = left; col <= right; col += 1) cells.push(source.input(row, col));
    lines.push(cells.join('\t'));
  }
  return lines.join('\n');
};

const fromTsv = (text) => {
  const lines = text.replace(/\r\n?/g, '\n').replace(/\n$/, '').split('\n');
  return lines.map((line) => line.split('\t'));
};

export const bindClipboard = (viewport, { selection, editor, source }) => {
  const copy = (event, cut) => {
    if (editor.isEditing()) return;
    event.clipboardData.setData('text/plain', toTsv(selection, source));
    event.preventDefault();
    if (!cut || !source.canEdit()) return;
    const { top, left, bottom, right } = selection.range();
    const writes = [];
    for (let row = top; row <= bottom; row += 1) {
      for (let col = left; col <= right; col += 1) writes.push({ row, col, value: '' });
    }
    source.write(writes);
  };

  // A focused element with no text selection is not a copy target, so the
  // browser fires clipboard events at the document; the grid takes them
  // while it holds focus.
  const ours = () => document.activeElement === viewport;
  document.addEventListener('copy', (event) => ours() && copy(event, false));
  document.addEventListener('cut', (event) => ours() && copy(event, true));
  document.addEventListener('paste', (event) => {
    if (!ours() || editor.isEditing() || !source.canEdit()) return;
    const rows = fromTsv(event.clipboardData.getData('text/plain'));
    event.preventDefault();
    const { head } = selection;
    const writes = rows.flatMap((cells, r) => cells.map((value, c) => ({ row: head.row + r, col: head.col + c, value })));
    source.write(writes);
    const width = Math.max(...rows.map((cells) => cells.length));
    selection.set({ row: head.row, col: head.col });
    selection.set({ row: head.row + rows.length - 1, col: head.col + width - 1 }, true);
  });
};
