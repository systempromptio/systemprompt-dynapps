// Registers the spreadsheet editors. Excel sheets edit live: the editor and
// its vendored engines (Loro, IronCalc) load on first use, joining in comment
// or view mode below edit access. OpenDocument sheets have no live session:
// they draw read-only from the projection and re-read it when a revision lands.
import { rawFetch } from '../../services/api.js';
import { registerEditor, registerLiveHandler } from '../registry.js';

registerEditor(['xlsx'], async (slot, ctx) => {
  const { mount } = await import('./mount.js');
  return mount(slot, ctx);
});

registerEditor(['ods'], async (slot, { root }) => {
  const { createProjectionSheet } = await import('./projection.js');
  const read = async () => (await rawFetch(root.dataset.projectionApi)).projection;
  const page = Object.assign(document.createElement('div'), { className: 'sp-ws-sheet' });
  slot.replaceChildren(page);
  const sheet = createProjectionSheet(page);
  sheet.render(await read());
  registerLiveHandler('document.revision.created', () => {
    read().then(sheet.update).catch(() => null);
  });
  const save = root.querySelector('[data-field="save"]');
  if (save) save.textContent = 'View only';
  return { readOnly: true, hint: sheet.hint };
});
