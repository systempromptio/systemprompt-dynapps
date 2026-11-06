// The row menu of the Drive listing: the items registered for a folder or a
// document, opened from the row's ⋯ button or by right-clicking the row (or
// the context-menu key on it). Arrow keys move between items, Escape and a
// click elsewhere close it, and focus returns to what opened it.
import { menuItemsFor } from './drive-listing-registry.js';

let active = null;

const close = ({ restore = true } = {}) => {
  if (!active) return;
  const { menu, opener } = active;
  active = null;
  menu.remove();
  if (restore) opener?.focus();
};

const items = (menu) => [...menu.querySelectorAll('[role="menuitem"]')];

const move = (menu, step) => {
  const all = items(menu);
  const at = all.indexOf(document.activeElement);
  all[(at + step + all.length) % all.length]?.focus();
};

const onKey = (event) => {
  if (!active) return;
  const { menu } = active;
  if (event.key === 'Escape') {
    event.preventDefault();
    close();
  } else if (event.key === 'ArrowDown') {
    event.preventDefault();
    move(menu, 1);
  } else if (event.key === 'ArrowUp') {
    event.preventDefault();
    move(menu, -1);
  } else if (event.key === 'Tab') {
    close({ restore: false });
  }
};

const onPointer = (event) => {
  if (active && !active.menu.contains(event.target)) close({ restore: false });
};

const place = (menu, point) => {
  const width = menu.offsetWidth;
  const height = menu.offsetHeight;
  menu.style.left = `${Math.max(4, Math.min(point.x, window.innerWidth - width - 4))}px`;
  menu.style.top = `${Math.max(4, Math.min(point.y, window.innerHeight - height - 4))}px`;
};

const build = (kind, item, listing) => {
  const template = document.querySelector('template[data-template="drive-row-menu"]');
  const menu = template.content.firstElementChild.cloneNode(true);
  const entry = menu.querySelector('[role="menuitem"]');
  entry.remove();
  for (const registered of menuItemsFor(kind)) {
    const button = entry.cloneNode(true);
    button.textContent = registered.label;
    button.dataset.menuItem = registered.id;
    button.addEventListener('click', () => {
      close({ restore: false });
      registered.run({ [kind]: item, listing });
    });
    menu.append(button);
  }
  return menu;
};

export const hasMenu = (kind) => menuItemsFor(kind).length > 0;

export const openRowMenu = ({ kind, item, listing, opener, point }) => {
  close({ restore: false });
  if (!hasMenu(kind)) return;
  const menu = build(kind, item, listing);
  document.body.append(menu);
  const rect = opener.getBoundingClientRect();
  place(menu, point ?? { x: rect.left, y: rect.bottom + 4 });
  active = { menu, opener };
  items(menu)[0]?.focus();
};

let wired = false;

export const initRowMenus = () => {
  if (wired) return;
  wired = true;
  document.addEventListener('keydown', onKey);
  document.addEventListener('pointerdown', onPointer);
  window.addEventListener('resize', () => close({ restore: false }));
};
