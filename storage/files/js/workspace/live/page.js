// Entry of `/drive/{document_id}/live`: mounts the viewer shell on the
// server-rendered root with the paths and flags the page carries.
import { mountLiveShell } from './shell.js';

const root = document.querySelector('[data-live-root]');

if (root) {
  const data = root.dataset;
  const mounted = mountLiveShell(root.querySelector('[data-slot="live-page"]'), {
    title: data.title,
    embed: data.embed === 'true',
    frameUrl: data.frameUrl,
    signIn: data.signIn,
    connectors: data.connectors,
    urls: { api: data.api, people: data.peopleApi, search: data.searchApi, document: data.documentId },
  });
  const button = root.querySelector('[data-action="live-permissions"]');
  if (button) button.addEventListener('click', async () => (await mounted).permissions());
}
