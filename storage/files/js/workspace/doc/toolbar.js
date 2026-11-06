// The format toolbar above the page and the bubble that floats over a text
// selection. Both run the shared ACTIONS and reflect the selection's state.
import { Plugin } from 'prosemirror-state';
import { schema } from './schema.js';
import { ACTIONS, blockName, markActive, setLink } from './actions.js';
import { showPromptDialog } from '../../services/confirm.js';

const STYLES = ['paragraph', 'h1', 'h2', 'h3', 'quote', 'code_block'];
const GROUPS = [
  ['undo', 'redo'],
  ['bold', 'italic', 'underline', 'strike', 'code'],
  ['link'],
  ['bullet', 'numbered', 'outdent', 'indent'],
  ['divider', 'page_break', 'table', 'clear'],
];
// Letter tools stay type, styled by the stylesheet; the rest are 16px stroke
// icons, so no tool depends on the viewer's emoji or symbol fonts.
const LETTERS = { bold: 'B', italic: 'I', underline: 'U', strike: 'S' };
const ICONS = {
  undo: 'M6 3 3 6l3 3M3 6h6.5a3.5 3.5 0 0 1 0 7H6',
  redo: 'M10 3l3 3-3 3M13 6H6.5a3.5 3.5 0 0 0 0 7H10',
  code: 'M6 4 2 8l4 4M10 4l4 4-4 4',
  link: 'M6.5 9.5l3-3M7 4.5l1-1a3 3 0 0 1 4.2 4.2l-1 1M9 11.5l-1 1a3 3 0 0 1-4.2-4.2l1-1',
  bullet: 'M6 4h8M6 8h8M6 12h8M2.5 4h.01M2.5 8h.01M2.5 12h.01',
  numbered: 'M7 4h7M7 8h7M7 12h7M2.5 3 3.5 2.5V6M2.5 10.2c.6-.8 2-.5 2 .4 0 .8-2 1.6-2 2.4h2',
  outdent: 'M3 3h10M8 7h5M8 10h5M3 13h10M5.5 6.5 3 8.5l2.5 2',
  indent: 'M3 3h10M8 7h5M8 10h5M3 13h10M3 6.5 5.5 8.5 3 10.5',
  divider: 'M2 8h12',
  page_break: 'M3 2v3h10V2M3 14v-3h10v3M2 8h2M6 8h4M12 8h2',
  table: 'M2.5 3h11v10h-11zM2.5 6.5h11M2.5 10h11M8 3v10',
  clear: 'M4 3h9M8.5 3 6.5 13M3 13h6M11 10l3 3M14 10l-3 3',
};
const SVG_NS = 'http://www.w3.org/2000/svg';

const icon = (id) => {
  const svg = document.createElementNS(SVG_NS, 'svg');
  svg.setAttribute('viewBox', '0 0 16 16');
  svg.setAttribute('class', 'sp-doc-tool__icon');
  svg.setAttribute('aria-hidden', 'true');
  const path = document.createElementNS(SVG_NS, 'path');
  path.setAttribute('d', ICONS[id]);
  svg.append(path);
  return svg;
};

const button = (id, label, onRun) => {
  const el = document.createElement('button');
  el.type = 'button';
  el.className = `sp-doc-tool sp-doc-tool--${id}`;
  el.dataset.tool = id;
  el.title = label;
  el.setAttribute('aria-label', label);
  if (ICONS[id]) el.append(icon(id));
  else el.textContent = LETTERS[id] ?? label;
  el.addEventListener('mousedown', (event) => {
    event.preventDefault();
    onRun(id);
  });
  return el;
};

const askLink = (view) => {
  const current = view.state.selection.$from.marks().find((mk) => mk.type === schema.marks.link)?.attrs.href ?? 'https://';
  showPromptDialog('Link', 'Address (https, mailto or a page path)', current, (href) => {
    setLink(href.trim())(view.state, view.dispatch);
    view.focus();
  });
};

export const createToolbar = (bar, getView, extra = {}) => {
  const run = (id) => {
    const view = getView();
    if (!view) return;
    if (id === 'link') askLink(view);
    else if (extra[id]) extra[id](view);
    else ACTIONS[id]?.run(view.state, view.dispatch);
    view.focus();
  };
  const select = document.createElement('select');
  select.className = 'sp-doc-style';
  select.setAttribute('aria-label', 'Paragraph style');
  for (const id of STYLES) select.append(new Option(ACTIONS[id].label, id));
  select.addEventListener('change', () => run(select.value));
  const groups = GROUPS.map((ids) => {
    const group = document.createElement('div');
    group.className = 'sp-doc-toolbar__group';
    group.append(...ids.map((id) => button(id, ACTIONS[id]?.label ?? 'Link', run)));
    return group;
  });
  const tail = document.createElement('div');
  tail.className = 'sp-doc-toolbar__group';
  for (const [id, label] of Object.entries(extra.labels ?? {})) tail.append(button(id, label, run));
  bar.replaceChildren(select, ...groups, tail);

  return new Plugin({
    view: () => ({
      update: (view) => {
        const name = blockName(view.state);
        select.value = STYLES.includes(name) ? name : 'paragraph';
        for (const key of ['bold', 'italic', 'underline', 'strike', 'code', 'link']) {
          const tool = bar.querySelector(`[data-tool="${key}"]`);
          tool?.setAttribute('aria-pressed', String(markActive(view.state, schema.marks[key])));
        }
        bar.querySelectorAll('button, select').forEach((el) => { el.disabled = !view.editable; });
      },
    }),
  });
};

export const bubblePlugin = (bubble, onComment) => {
  const tools = ['bold', 'italic', 'link'].map((id) => button(id, ACTIONS[id]?.label ?? 'Link', (tool) => {
    const view = bubble.view;
    if (tool === 'link') askLink(view);
    else ACTIONS[tool].run(view.state, view.dispatch);
  }));
  const comment = button('comment', 'Comment', () => onComment(bubble.view));
  comment.textContent = 'Comment';
  bubble.replaceChildren(...tools, comment);
  return new Plugin({
    view: (view) => {
      bubble.view = view;
      return {
        update: () => {
          const { empty, from, to } = view.state.selection;
          if (empty || !view.hasFocus()) {
            bubble.hidden = true;
            return;
          }
          const start = view.coordsAtPos(from);
          const end = view.coordsAtPos(to);
          const host = bubble.offsetParent?.getBoundingClientRect() ?? { top: 0, left: 0 };
          bubble.hidden = false;
          bubble.style.top = `${start.top - host.top - bubble.offsetHeight - 6}px`;
          bubble.style.left = `${(start.left + end.left) / 2 - host.left - bubble.offsetWidth / 2}px`;
          tools.forEach((el) => { el.hidden = !view.editable; });
        },
      };
    },
  });
};
