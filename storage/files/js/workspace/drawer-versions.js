// The Versions tab: the revision list, narrowed to one author chosen with the
// people picker; any earlier revision compared with the head as changed
// lines; and, for an editor, restored as a new head revision.
import { listRevisions, opId, readView, restoreRevision } from '../documents/documents-api.js';
import { formatDate, shortId } from '../services/drive-rows.js';
import { createPeoplePicker } from './people-picker.js';
import { person, resolvePeople, subjectsApiOf } from './people.js';
import { alertLine, attempt, button, el, heading, lineList } from './pane-dom.js';
import { registerDrawerTab } from './registry.js';

const panes = new WeakMap();

const authorFilter = (ctx, state, reload) => {
  const form = el('form', 'sp-ws__share');
  form.setAttribute('role', 'search');
  const input = el('input', 'sp-field-input');
  Object.assign(input, { name: 'author', placeholder: 'Filter by author', autocomplete: 'off' });
  input.setAttribute('aria-label', 'Author');
  const clear = el('button', 'sp-btn sp-btn--sm sp-btn--ghost', 'Clear');
  clear.type = 'reset';
  form.append(input, clear);
  const picker = createPeoplePicker({
    input,
    subjectsApi: subjectsApiOf(ctx.root),
    onPick: (p) => {
      state.author = p.user_id;
      input.value = p.display_name || p.email || p.user_id;
      reload();
    },
  });
  input.addEventListener('input', () => picker.search(input.value.trim()));
  input.addEventListener('keydown', (evt) => picker.handleKey(evt));
  form.addEventListener('submit', (evt) => evt.preventDefault());
  form.addEventListener('reset', () => {
    picker.close();
    state.author = null;
    reload();
  });
  return form;
};

const revisionRow = (revision, isHead, ctx, state, reload) => {
  const item = el('li', 'sp-ws__entry');
  item.append(
    el('span', 'sp-ws__entry-title', revision.message || `Revision ${shortId(revision.id)}`),
    el('span', 'sp-ws__muted', `${person(revision.author_user_id)} · ${revision.origin_kind} · ${formatDate(revision.created_at)}`),
  );
  const actions = el('div', 'sp-ws-pane__actions');
  if (isHead) actions.append(el('span', 'sp-badge sp-badge--sm sp-badge--muted', 'Current'));
  else {
    actions.append(button('Compare with head', 'sp-btn--ghost', () => attempt(state.alert, async () => {
      const { lines } = await readView(ctx.documentId, { view: 'changes', since_revision: revision.id });
      state.diff = { from: revision, lines };
    }, reload)));
    if (ctx.canEdit) {
      actions.append(button('Restore', 'sp-btn--outline', () => attempt(state.alert, () => restoreRevision(ctx.documentId, {
        revisionId: revision.id, headRevision: state.head, opId: opId(),
      }), reload)));
    }
  }
  item.append(actions);
  return item;
};

const diffView = (state, reload) => {
  const box = el('section', 'sp-ws-pane__diff');
  box.setAttribute('aria-label', 'Changes since revision');
  box.append(
    heading(`Changes since ${formatDate(state.diff.from.created_at)}`),
    lineList(state.diff.lines, 'No changes'),
    button('Close', 'sp-btn--ghost', () => {
      state.diff = null;
      reload();
    }),
  );
  return box;
};

const load = async (pane, ctx) => {
  if (!panes.has(pane)) {
    const state = { alert: alertLine(), author: null, diff: null, head: null, body: el('div', 'sp-ws__pane') };
    const reload = () => load(pane, ctx).catch(() => undefined);
    pane.replaceChildren(authorFilter(ctx, state, reload), state.alert.node, state.body);
    panes.set(pane, { state, reload });
  }
  const { state, reload } = panes.get(pane);
  const { revisions } = await listRevisions(ctx.documentId);
  state.head = revisions[0]?.id ?? null;
  await resolvePeople(ctx.peopleApi, revisions.map((r) => r.author_user_id));
  const shown = state.author ? revisions.filter((r) => r.author_user_id === state.author) : revisions;
  const list = el('ol', 'sp-ws__list');
  list.append(...shown.map((r) => revisionRow(r, r.id === state.head, ctx, state, reload)));
  state.body.replaceChildren(list, ...(state.diff ? [diffView(state, reload)] : []));
};

registerDrawerTab({ id: 'versions', label: 'Versions', order: 20, live: ['document.revision.created'], load });
