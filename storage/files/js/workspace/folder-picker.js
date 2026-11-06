// The folder picker behind "Move to folder": a dialog that walks the folder
// tree one level at a time, so a person chooses a place by its name.
import { listFolders } from '../documents/documents-api.js';
import { el } from './pane-dom.js';

const PAGE = 200;

const childrenOf = async (folderId) => {
  const listing = await listFolders({ scope: 'folder', folder: folderId ?? undefined, sort: 'name', limit: PAGE });
  return listing.folders ?? [];
};

const crumbs = (trail, onPick) => {
  const nav = el('nav', 'sp-ws-folders__trail');
  nav.setAttribute('aria-label', 'Current folder');
  trail.forEach((step, at) => {
    const link = el('button', 'sp-btn sp-btn--ghost', step.name);
    link.type = 'button';
    link.disabled = at === trail.length - 1;
    link.addEventListener('click', () => onPick(at));
    nav.append(link);
  });
  return nav;
};

const folderList = (folders, onOpen) => {
  if (!folders.length) return el('p', 'sp-ws__muted', 'No folders here');
  const list = el('ul', 'sp-ws-folders__list');
  for (const folder of folders) {
    const row = el('button', 'sp-ws-folders__row', folder.name);
    row.type = 'button';
    row.dataset.folder = folder.id;
    row.addEventListener('click', () => onOpen(folder));
    const item = el('li');
    item.append(row);
    list.append(item);
  }
  return list;
};

/** Resolves with the chosen folder id (null for the top level), or undefined on cancel. */
export const pickFolder = ({ title, currentId }) => new Promise((resolve) => {
  const dialog = el('dialog', 'sp-ws-folders');
  dialog.setAttribute('aria-label', title);
  const body = el('div', 'sp-ws-folders__body');
  const alert = el('p', 'sp-ws__muted');
  const trail = [{ id: null, name: 'Drive' }];
  let chosen;

  const here = () => trail[trail.length - 1];

  const moveHere = el('button', 'sp-btn sp-btn--primary', 'Move here');
  moveHere.type = 'button';
  moveHere.dataset.action = 'move-here';
  moveHere.addEventListener('click', () => {
    chosen = here().id;
    dialog.close();
  });
  const cancel = el('button', 'sp-btn sp-btn--outline', 'Cancel');
  cancel.type = 'button';
  cancel.addEventListener('click', () => dialog.close());

  const draw = async () => {
    moveHere.disabled = true;
    alert.textContent = '';
    let folders = [];
    try {
      folders = await childrenOf(here().id);
    } catch (error) {
      alert.textContent = error.message ?? 'The folders could not be loaded.';
    }
    moveHere.disabled = (here().id ?? null) === (currentId ?? null);
    body.replaceChildren(
      crumbs(trail, (at) => { trail.length = at + 1; draw(); }),
      folderList(folders, (folder) => { trail.push({ id: folder.id, name: folder.name }); draw(); }),
      alert,
    );
  };

  const actions = el('div', 'sp-ws-pane__actions');
  actions.append(moveHere, cancel);
  dialog.append(el('h2', 'sp-ws-folders__title', title), body, actions);
  dialog.addEventListener('close', () => {
    dialog.remove();
    resolve(chosen);
  });
  document.body.append(dialog);
  dialog.showModal();
  draw();
});
