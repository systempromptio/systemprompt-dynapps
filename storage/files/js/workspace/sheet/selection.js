// The one selection model the grid, the formula bar, the clipboard and the
// presence feed all read: an anchor where the range began and a head (the
// active cell) where it ends. Positions are 0-based row and column indexes
// into the sheet's id lists; nothing here touches the DOM.
export const createSelection = (onChange) => {
  const s = { anchor: { row: 0, col: 0 }, head: { row: 0, col: 0 } };

  const clamp = ({ row, col }) => ({ row: Math.max(0, row), col: Math.max(0, col) });

  const set = (head, extend = false) => {
    s.head = clamp(head);
    if (!extend) s.anchor = s.head;
    onChange(s);
  };

  return {
    get head() {
      return s.head;
    },
    get anchor() {
      return s.anchor;
    },
    set,
    move: (dRow, dCol, extend = false) => set({ row: s.head.row + dRow, col: s.head.col + dCol }, extend),
    range: () => ({
      top: Math.min(s.anchor.row, s.head.row),
      left: Math.min(s.anchor.col, s.head.col),
      bottom: Math.max(s.anchor.row, s.head.row),
      right: Math.max(s.anchor.col, s.head.col),
    }),
    isSingle: () => s.anchor.row === s.head.row && s.anchor.col === s.head.col,
  };
};

export const columnName = (index) => {
  let out = '';
  let n = index + 1;
  while (n > 0) {
    const r = (n - 1) % 26;
    out = String.fromCharCode(65 + r) + out;
    n = Math.floor((n - 1) / 26);
  }
  return out;
};

export const refOf = ({ row, col }) => `${columnName(col)}${row + 1}`;

export const inRange = (range, row, col) =>
  row >= range.top && row <= range.bottom && col >= range.left && col <= range.right;
