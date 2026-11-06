// The small DOM vocabulary the drawer panes share: elements built with text
// only, buttons, a pane's alert line and a numbered list of changed lines.
// No pane ever sets markup from a string, so a name or a comment body
// always reaches the page as text.
export const el = (tag, className, text) => {
  const node = document.createElement(tag);
  if (className) node.className = className;
  if (text != null) node.textContent = text;
  return node;
};

export const button = (label, className, onClick) => {
  const node = el('button', `sp-btn sp-btn--xs ${className ?? 'sp-btn--ghost'}`, label);
  node.type = 'button';
  if (onClick) node.addEventListener('click', onClick);
  return node;
};

export const errorText = (err) => (err?.message ? String(err.message) : 'The request failed.');

export const alertLine = () => {
  const line = el('p', 'sp-ws-pane__error');
  line.setAttribute('role', 'alert');
  line.hidden = true;
  return {
    node: line,
    show: (err) => {
      line.textContent = err ? errorText(err) : '';
      line.hidden = !err;
    },
  };
};

export const lineList = (lines, empty) => {
  const list = el('ol', 'sp-ws-lines');
  const items = (lines ?? []).map((line) => {
    const item = el('li', 'sp-ws-lines__line', line);
    if (/^[+\-~]/.test(line)) item.dataset.mark = line[0];
    return item;
  });
  list.append(...(items.length ? items : [el('li', 'sp-ws-lines__line sp-ws-lines__line--empty', empty)]));
  return list;
};

export const heading = (text) => el('h3', 'sp-ws-pane__heading', text);

export const relativeTime = (iso) => {
  if (!iso) return '';
  const date = new Date(iso);
  return Number.isNaN(date.valueOf()) ? '' : date.toLocaleString();
};

// Runs a pane action, shows its failure on the pane's alert line and reloads
// the pane either way, so what is shown is always what the server holds.
export const attempt = async (alert, work, reload) => {
  try {
    await work();
    alert.show(null);
  } catch (err) {
    alert.show(err);
  }
  await reload();
};
