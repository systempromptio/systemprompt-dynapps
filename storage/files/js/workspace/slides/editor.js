// Registers the deck editor for the presentation formats. It and its
// vendored engines load on first use, so a page whose vendor head is
// missing still opens with the read-only preview.
import { registerEditor } from '../registry.js';

registerEditor(['pptx', 'odp'], async (slot, ctx) => {
  const { mount } = await import('./mount.js');
  return mount(slot, ctx);
});
