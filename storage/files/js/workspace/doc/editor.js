// Registers the doc editor for the rich-text formats. The editor and its
// vendored engines load on first use, so a page whose vendor head is
// missing still opens with the read-only preview. Below edit access the
// editor joins the live session in comment or view mode. An HTML document
// opens as the page its author designed, with the editor one switch away.
import { registerEditor } from '../registry.js';

const openEditor = async (slot, ctx) => {
  const { mount } = await import('./mount.js');
  return mount(slot, ctx);
};

registerEditor(['md', 'docx', 'odt'], openEditor);

registerEditor(['html'], async (slot, ctx) => {
  const { mountRendered } = await import('./rendered.js');
  return mountRendered(slot, ctx, openEditor);
});
