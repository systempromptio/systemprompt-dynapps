// Mounts registered drawer panes outside the workspace drawer: the block
// editor page's rail holds the same Sharing, Comments and Proposals panes in
// `[data-pane-mount="<tab id>"]` cards. Each pane is loaded once, reloaded on
// its tab's live events, and its card is hidden when the feature is off.
import { drawerTab } from './registry.js';

export const mountPanes = (container, ctx) => {
  const mounted = [];
  for (const pane of container.querySelectorAll('[data-pane-mount]')) {
    const tab = drawerTab(pane.dataset.paneMount);
    if (!tab) continue;
    const load = () => Promise.resolve().then(() => tab.load(pane, ctx)).catch(() => undefined);
    pane.addEventListener('ws:pane-unavailable', () => {
      const card = pane.closest('section') ?? pane;
      card.hidden = true;
    });
    mounted.push({ tab, load });
    load();
  }
  return (name) => {
    for (const { tab, load } of mounted) {
      if (tab.live.some((prefix) => name.startsWith(prefix))) load();
    }
  };
};
