// Registers the doc editor for the rich-text formats. The editor and its
// vendored engines load on first use, so a page whose vendor head is
// missing still opens with the read-only preview. Below edit access the
// editor joins the live session in comment or view mode.
import { registerEditor } from '../registry.js';

registerEditor(['html', 'md', 'docx', 'odt'], async (slot, ctx) => {
  const { mount } = await import('./mount.js');
  return mount(slot, ctx);
});
