// Markdown input rules and the `/` slash menu. Typing `/` at the start of
// an empty block opens a filtered list of block types; Enter applies the
// highlighted one, Escape closes it.
import { inputRules, textblockTypeInputRule, InputRule } from 'prosemirror-inputrules';
import { Plugin } from 'prosemirror-state';
import { schema } from './schema.js';
import { ACTIONS, SLASH_ITEMS } from './actions.js';

const n = schema.nodes;

const markRule = (pattern, type, delim) => new InputRule(pattern, (state, match, _start, end) => {
  const text = match[1];
  if (!text) return null;
  const from = end - text.length - 2 * delim + 1;
  return state.tr.replaceWith(from, end, schema.text(text, [type.create()])).removeStoredMark(type);
});

const atomRule = (pattern, type) => new InputRule(pattern, (state, _match, start, end) => {
  const $start = state.doc.resolve(start);
  return state.tr.replaceRangeWith($start.before(1), state.doc.resolve(end).after(1), type.create());
});

export const markdownRules = () => inputRules({
  rules: [
    textblockTypeInputRule(/^(#{1,6})\s$/, n.heading, (match) => ({ level: match[1].length })),
    textblockTypeInputRule(/^\s*([-+*])\s$/, n.list_item, { level: 1, ordered: false }),
    textblockTypeInputRule(/^(\d+)\.\s$/, n.list_item, { level: 1, ordered: true }),
    textblockTypeInputRule(/^>\s$/, n.quote),
    textblockTypeInputRule(/^```$/, n.code_block),
    atomRule(/^---$/, n.divider),
    markRule(/\*\*([^*]+)\*\*$/, schema.marks.bold, 2),
    markRule(/(?:^|[^*])\*([^*]+)\*$/, schema.marks.italic, 1),
    markRule(/`([^`]+)`$/, schema.marks.code, 1),
    markRule(/~~([^~]+)~~$/, schema.marks.strike, 2),
  ],
});

const slashQuery = (state) => {
  const { $from, empty } = state.selection;
  if (!empty || $from.depth < 1) return null;
  const text = $from.parent.textContent;
  return /^\/[\w ]*$/.test(text) && $from.parentOffset === text.length ? text.slice(1).toLowerCase() : null;
};

export const slashPlugin = (menu) => {
  let items = [];
  let active = 0;
  const render = (view, query) => {
    items = SLASH_ITEMS.filter((id) => ACTIONS[id].label.toLowerCase().includes(query));
    if (!items.length) {
      menu.hidden = true;
      return;
    }
    active = Math.min(active, items.length - 1);
    menu.replaceChildren(...items.map((id, i) => {
      const item = document.createElement('button');
      item.type = 'button';
      item.className = `sp-doc-slash__item${i === active ? ' sp-doc-slash__item--active' : ''}`;
      item.textContent = ACTIONS[id].label;
      item.addEventListener('mousedown', (event) => {
        event.preventDefault();
        apply(view, id);
      });
      return item;
    }));
    const box = view.coordsAtPos(view.state.selection.from);
    const host = menu.offsetParent?.getBoundingClientRect() ?? { top: 0, left: 0 };
    menu.style.top = `${box.bottom - host.top + 4}px`;
    menu.style.left = `${box.left - host.left}px`;
    menu.hidden = false;
  };
  const apply = (view, id) => {
    const { $from } = view.state.selection;
    view.dispatch(view.state.tr.delete($from.start(), $from.pos));
    ACTIONS[id].run(view.state, view.dispatch);
    menu.hidden = true;
    view.focus();
  };
  return new Plugin({
    view: () => ({
      update: (view) => {
        const query = slashQuery(view.state);
        if (query === null) menu.hidden = true;
        else render(view, query);
      },
    }),
    props: {
      handleKeyDown: (view, event) => {
        if (menu.hidden || !items.length) return false;
        if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
          active = (active + (event.key === 'ArrowDown' ? 1 : items.length - 1)) % items.length;
          render(view, slashQuery(view.state) ?? '');
          return true;
        }
        if (event.key === 'Enter') {
          apply(view, items[active]);
          return true;
        }
        if (event.key === 'Escape') {
          menu.hidden = true;
          return true;
        }
        return false;
      },
    },
  });
};
