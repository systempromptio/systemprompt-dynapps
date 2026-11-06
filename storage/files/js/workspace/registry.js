// The workspace's one extension surface. Format editors, drawer tabs, commands
// and live-event handlers all register here from their own modules, and the
// shell builds the name menu, the palette, the drawer and the event feed from
// what was registered, so no feature is wired into the shell by name.
// Every `when(ctx)` and `run`/`load` receives the page context:
// { root, doc, view, access, canEdit, canShare, canComment, isOwner, userId,
//   documentId, peopleApi, openPane(id), refresh() }.
const editors = new Map();
const tabs = new Map();
const commands = new Map();
const live = [];

const allowed = (entry, ctx) => !entry.when || Boolean(entry.when(ctx));

export const registerEditor = (formats, mount) => {
  for (const format of formats) editors.set(format, mount);
};

export const editorFor = (format) => editors.get(format);

// A tab registered again under the same id replaces the first, so a format
// editor can take over a pane (the doc editor's anchored comments, say).
// `live` names the event prefixes that reload the pane while it is shown.
export const registerDrawerTab = (tab) => {
  tabs.set(tab.id, { order: 100, live: [], ...tab });
};

export const drawerTab = (id) => tabs.get(id);

export const drawerTabs = (ctx) => [...tabs.values()]
  .filter((tab) => allowed(tab, ctx))
  .sort((a, b) => a.order - b.order);

// `menu` puts the command in the name menu; every command is in the palette.
// `keys` is shown as written, and a `⌘<letter>` binds that shortcut.
export const registerCommand = (command) => {
  commands.set(command.id, { label: command.id, keys: '', menu: false, ...command });
};

export const availableCommands = (ctx) => [...commands.values()].filter((c) => allowed(c, ctx));

export const runCommand = async (id, ctx) => {
  const command = commands.get(id);
  if (!command || !allowed(command, ctx)) return;
  try {
    await command.run(ctx.root, ctx);
  } catch {
    // The transport or the pane has already shown the failure.
  }
};

export const registerLiveHandler = (prefix, fn) => {
  live.push({ prefix, fn });
};

export const dispatchLive = (name, value) => {
  for (const { prefix, fn } of live) {
    if (name.startsWith(prefix)) fn(name, value);
  }
};

// A pane whose route answers 404 (the feature is off on this installation)
// says so with this event; the drawer or the rail hides it.
export const paneUnavailable = (pane) => {
  pane.dispatchEvent(new CustomEvent('ws:pane-unavailable', { bubbles: true }));
};
