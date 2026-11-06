// The command palette (⌘K): every registered command the viewer may run,
// filtered as you type and run with Enter or a click.
import { availableCommands, runCommand } from './registry.js';

const visible = (list) => [...list.querySelectorAll('[data-command]')].filter((item) => !item.hidden);

const select = (list, index) => {
  const items = visible(list);
  items.forEach((item, i) => item.setAttribute('aria-selected', String(i === index)));
  items[index]?.scrollIntoView({ block: 'nearest' });
};

const selectedIndex = (list) => visible(list).findIndex((item) => item.getAttribute('aria-selected') === 'true');

const paletteItem = (command) => {
  const template = document.querySelector('template[data-template="ws-palette-item"]');
  const item = template.content.firstElementChild.cloneNode(true);
  item.dataset.command = command.id;
  item.dataset.label = command.label;
  item.querySelector('[data-field="label"]').textContent = command.label;
  item.querySelector('[data-field="keys"]').textContent = command.keys;
  return item;
};

export const initPalette = (ctx) => {
  const dialog = ctx.root.querySelector('[data-slot="ws-palette"]');
  const input = dialog.querySelector('input');
  const list = dialog.querySelector('[data-slot="ws-palette-list"]');
  list.replaceChildren(...availableCommands(ctx).map(paletteItem));

  const filter = () => {
    const needle = input.value.trim().toLowerCase();
    list.querySelectorAll('[data-command]').forEach((item) => {
      item.hidden = Boolean(needle) && !item.dataset.label.toLowerCase().includes(needle);
    });
    select(list, 0);
  };

  const choose = (command) => {
    dialog.close();
    if (command) runCommand(command, ctx);
  };

  input.addEventListener('input', filter);
  input.addEventListener('keydown', (event) => {
    const items = visible(list);
    const index = selectedIndex(list);
    if (event.key === 'ArrowDown') select(list, Math.min(items.length - 1, index + 1));
    else if (event.key === 'ArrowUp') select(list, Math.max(0, index - 1));
    else if (event.key === 'Enter') choose(items[index]?.dataset.command);
    else return;
    event.preventDefault();
  });
  list.addEventListener('click', (event) => choose(event.target.closest('[data-command]')?.dataset.command));

  return () => {
    input.value = '';
    filter();
    dialog.showModal();
    input.focus();
  };
};
