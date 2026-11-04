// The folder Class dialog, opened from the "Class…" entry this registers in
// each folder's menu on the Drive listing.
import { registerFolderMenuItem } from './drive-listing-registry.js';
import { showPromptDialog } from './confirm.js';
import {
  agentsCanText, isLoosening, loadClass, loadClassOptions, rankOf, saveClass,
} from './document-class.js';

const KIND = 'folder';

const render = (dialog, options, view) => {
  const select = dialog.querySelector('[data-slot="drive-folder-class"]');
  const floor = rankOf(options, view.class_id);
  const choices = options.filter((o, rank) => view.can_loosen || rank >= floor);
  select.replaceChildren(...choices.map((o) => new Option(o.label, o.id, false, o.id === view.class_id)));
  select.disabled = !view.can_change;
  dialog.querySelector('[data-field="drive-folder-agents"]').textContent = agentsCanText(view);
  dialog.dataset.classId = view.class_id;
};

const change = async (dialog, ctx, next) => {
  const status = dialog.querySelector('[data-field="drive-folder-class-status"]');
  const apply = async (reason) => {
    try {
      render(dialog, ctx.options, await saveClass(KIND, ctx.folder.id, next, reason));
      status.textContent = '';
      ctx.listing.root.dispatchEvent(new CustomEvent('sp-document-class-changed'));
    } catch (err) {
      status.textContent = err.message;
      render(dialog, ctx.options, await loadClass(KIND, ctx.folder.id));
    }
  };
  if (!isLoosening(ctx.options, dialog.dataset.classId, next)) return apply(null);
  return showPromptDialog('Loosen folder class', 'Reason (recorded with the change)', '', apply);
};

const open = async (dialog, ctx) => {
  const status = dialog.querySelector('[data-field="drive-folder-class-status"]');
  dialog.querySelector('[data-field="drive-folder-name"]').textContent = ctx.folder.name;
  status.textContent = '';
  dialog.showModal();
  try {
    ctx.options ??= await loadClassOptions();
    render(dialog, ctx.options, await loadClass(KIND, ctx.folder.id));
  } catch (err) {
    status.textContent = err.message;
  }
};

export const initFolderClass = (dialog) => {
  const ctx = { options: null, folder: null, listing: null };
  dialog.querySelector('[data-slot="drive-folder-class"]').addEventListener('change', (evt) => {
    change(dialog, ctx, evt.target.value);
  });
  dialog.querySelector('[data-action="drive-folder-class-close"]').addEventListener('click', () => dialog.close());
  registerFolderMenuItem({
    id: 'class',
    label: 'Class…',
    run: ({ folder, listing }) => {
      ctx.folder = folder;
      ctx.listing = listing;
      open(dialog, ctx);
    },
  });
};
