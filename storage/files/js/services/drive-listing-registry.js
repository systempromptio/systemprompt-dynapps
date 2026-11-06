// The Drive listing's extension points. A feature registers here, at import
// time or later, and the listing picks it up without the page knowing it.
// registerRowExtension(extend) takes extend({ kind, item }), where kind is
// 'folder' or 'document' and item the API row, and that returns attrs (data-*
// attributes for the row) and content (a Node, a string or an array of both
// for the row's extension slot; strings become text, never HTML), or null.
// The call is synchronous: a feature that loads data returns nothing for an
// uncached row and calls listing.invalidate() once it has the data.
// registerFolderMenuItem and registerDocumentMenuItem take an id, label and
// run, and call run({ folder | document, listing }) from the row's menu and its
// right-click menu; a second item with the same id replaces the first one.
// registerListingAction({ id, label, run, admin }) adds a header button that
// calls run({ listing }), shown only to administrators when admin is true.
// The listing a callback receives carries root (the listing element, where
// listing events are dispatched), folderId, isAdmin, invalidate and reload.

const rowExtensions = [];
const menuItems = { folder: new Map(), document: new Map() };
const listingActions = new Map();
const listeners = new Set();

const changed = (what) => listeners.forEach((listener) => listener(what));

export const onRegistryChange = (listener) => {
  listeners.add(listener);
  return () => listeners.delete(listener);
};

export const registerRowExtension = (extend) => {
  rowExtensions.push(extend);
  changed('rows');
};

const registerMenuItem = (kind, { id, label, run }) => {
  menuItems[kind].set(id, { id, label, run });
  changed('menu');
};

export const registerFolderMenuItem = (item) => registerMenuItem('folder', item);

export const registerDocumentMenuItem = (item) => registerMenuItem('document', item);

export const registerListingAction = ({ id, label, run, admin = false }) => {
  listingActions.set(id, { id, label, run, admin });
  changed('actions');
};

export const menuItemsFor = (kind) => [...(menuItems[kind]?.values() ?? [])];

export const listingActionsFor = (isAdmin) => [...listingActions.values()].filter((a) => isAdmin || !a.admin);

const DATA_ATTR = /^data-[a-z0-9-]+$/;

const toNodes = (content) => {
  const parts = Array.isArray(content) ? content : [content];
  return parts
    .filter((part) => part !== null && part !== undefined && part !== '')
    .map((part) => (part instanceof Node ? part : document.createTextNode(String(part))));
};

// Runs every row extension for one row: clears what earlier runs set, then
// applies each result. A throwing extension leaves its part of the row bare.
export const applyRowExtensions = (row, slot, kind, item) => {
  for (const name of (row.dataset.extAttrs ?? '').split(' ').filter(Boolean)) row.removeAttribute(name);
  const applied = [];
  const nodes = [];
  for (const extend of rowExtensions) {
    let result = null;
    try {
      result = extend({ kind, item });
    } catch {
      result = null;
    }
    for (const [name, value] of Object.entries(result?.attrs ?? {})) {
      if (!DATA_ATTR.test(name)) continue;
      row.setAttribute(name, String(value));
      applied.push(name);
    }
    nodes.push(...toNodes(result?.content));
  }
  row.dataset.extAttrs = applied.join(' ');
  slot?.replaceChildren(...nodes);
};
