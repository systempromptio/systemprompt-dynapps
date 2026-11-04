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
const GLYPHS = {
  undo: '↶', redo: '↷', bold: 'B', italic: 'I', underline: 'U', strike: 'S', code: '</>', link: '🔗',
  bullet: '•', numbered: '1.', outdent: '⇤', indent: '⇥', divider: '—', page_break: '⤓', table: '▦', clear: '⌫',
};

const button = (id, label, onRun) => {
  const el = document.createElement('button');
  el.type = 'button';
  el.className = `sp-doc-tool sp-doc-tool--${id}`;
  el.dataset.tool = id;
  el.title = label;
  el.setAttribute('aria-label', label);
  el.textContent = GLYPHS[id] ?? label;
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
