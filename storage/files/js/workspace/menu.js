// The name menu and the top-bar buttons, both built from the registered
// commands the viewer may run: the menu lists every `menu` command, and a
// toolbar button whose command is not available is hidden. Any element with
// `data-command` inside the workspace runs that command when clicked, and a
// command's `⌘<letter>` keys bind that shortcut.
import { availableCommands, runCommand } from './registry.js';

const menuItem = (command) => {
  const template = document.querySelector('template[data-template="ws-menu-item"]');
  const item = template.content.firstElementChild.cloneNode(true);
  const button = item.querySelector('button');
  button.dataset.command = command.id;
  item.querySelector('[data-field="label"]').textContent = command.label;
  item.querySelector('[data-field="keys"]').textContent = command.keys;
  return item;
};

const shortcutOf = (keys) => (/^⌘(.)$/u.exec(keys ?? '')?.[1] ?? '').toLowerCase();

export const initMenu = (ctx, openPalette) => {
  const { root } = ctx;
  const commands = availableCommands(ctx);
  const ids = new Set(commands.map((c) => c.id));
  const toggle = root.querySelector('[data-action="ws-name-menu"]');
  const menu = root.querySelector('[data-slot="ws-menu"]');
  menu.replaceChildren(...commands.filter((c) => c.menu).map(menuItem));
  root.querySelectorAll('.sp-ws__bar [data-command]').forEach((b) => { b.hidden = !ids.has(b.dataset.command); });

  const setOpen = (open) => {
    menu.hidden = !open;
    toggle.setAttribute('aria-expanded', String(open));
    if (open) menu.querySelector('[role="menuitem"]')?.focus();
  };
  toggle.addEventListener('click', () => setOpen(menu.hidden));
  menu.addEventListener('keydown', (event) => {
    if (event.key === 'Escape') {
      setOpen(false);
      toggle.focus();
    }
  });
  document.addEventListener('pointerdown', (event) => {
    if (!menu.hidden && !event.target.closest('.sp-ws__title')) setOpen(false);
  });
  root.addEventListener('click', (event) => {
    const id = event.target.closest('button[data-command]')?.dataset.command;
    if (!id) return;
    setOpen(false);
    runCommand(id, ctx);
  });

  const shortcuts = new Map(commands.map((c) => [shortcutOf(c.keys), c.id]).filter(([key]) => key));
  document.addEventListener('keydown', (event) => {
    if (!(event.metaKey || event.ctrlKey)) return;
    const key = event.key.toLowerCase();
    if (key === 'k') openPalette();
    else if (shortcuts.has(key)) runCommand(shortcuts.get(key), ctx);
    else return;
    event.preventDefault();
  });
};
