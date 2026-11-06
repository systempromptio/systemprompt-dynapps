// The spreadsheet's static frame: formula bar, the corner and both header
// tracks around the scrolling viewport, and the sheet tab strip.
const el = (tag, className, attrs = {}) => Object.assign(document.createElement(tag), { className, ...attrs });

export const layout = (slot) => {
  const frame = el('div', 'sp-sheet');
  const barRow = el('div', 'sp-sheet__bar');
  const ref = el('output', 'sp-sheet__ref', { ariaLabel: 'Selected cell' });
  const bar = el('input', 'sp-sheet__formula', { type: 'text', ariaLabel: 'Cell contents', spellcheck: false });
  barRow.append(ref, bar);

  const body = el('div', 'sp-sheet__body');
  const corner = el('div', 'sp-sheet__corner');
  const colFrame = el('div', 'sp-sheet__cols');
  const colHeads = el('div', 'sp-sheet__track sp-sheet__track--cols');
  colFrame.append(colHeads);
  const rowFrame = el('div', 'sp-sheet__rows');
  const rowHeads = el('div', 'sp-sheet__track sp-sheet__track--rows');
  rowFrame.append(rowHeads);
  const viewport = el('div', 'sp-sheet__viewport', { tabIndex: 0, role: 'grid', ariaLabel: 'Spreadsheet' });
  const sizer = el('div', 'sp-sheet__sizer');
  const layer = el('div', 'sp-sheet__layer');
  const input = el('input', 'sp-sheet__input', { type: 'text', hidden: true, ariaLabel: 'Edit cell', spellcheck: false });
  sizer.append(layer, input);
  viewport.append(sizer);
  body.append(corner, colFrame, rowFrame, viewport);

  const tabs = el('div', 'sp-sheet__tabs', { role: 'tablist', ariaLabel: 'Sheets' });
  frame.append(barRow, body, tabs);
  slot.replaceChildren(frame);
  return { frame, ref, bar, viewport, sizer, layer, input, colHeads, rowHeads, tabs };
};

export const renderTabs = (tabs, view, onPick) => {
  const nodes = view.sheets.map((sheet, index) => {
    const tab = el('button', 'sp-sheet__tab', { type: 'button', role: 'tab', textContent: sheet.name });
    tab.setAttribute('aria-selected', String(index === view.active));
    tab.addEventListener('click', () => {
      if (index === view.active) return;
      onPick(index);
      renderTabs(tabs, view, onPick);
    });
    return tab;
  });
  tabs.replaceChildren(...nodes);
};
