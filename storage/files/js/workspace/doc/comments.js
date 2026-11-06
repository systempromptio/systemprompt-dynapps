// The comments sidebar: threads anchored to a block and the quoted words,
// with reply and resolve. A comment made from a selection also marks the
// quoted range with a `comment` mark naming the thread, so the canvas can
// highlight it and the mark travels with the text through concurrent edits.
import { rawFetch } from '../../services/api.js';
import { showToast } from '../../services/toast.js';
import { formatDate } from '../../services/drive-rows.js';
import { toggleDrawer } from '../drawer.js';
import { person, resolvePeople } from '../people.js';
import { createComposer } from './composer.js';
import { mentionIds, renderBody } from './mentions.js';
import { schema } from './schema.js';

const el = (tag, className, text) => {
  const node = document.createElement(tag);
  node.className = className;
  if (text != null) node.textContent = text;
  return node;
};

const bodyOf = (text) => {
  const body = el('p', 'sp-doc-thread__body');
  body.append(renderBody(text));
  return body;
};

const idle = { load: async () => undefined, fromSelection: () => undefined, onEvent: () => undefined };

export const createComments = ({ root, api, getView, canComment }) => {
  const pane = root.querySelector('[data-pane="comments"]');
  if (!pane) return idle;
  const composer = createComposer(root);
  const threads = el('div', '');
  pane.replaceChildren(composer.form, threads);

  let linked = new URLSearchParams(window.location.search).get('comment');

  // A notification links here with ?comment=<id>: once the threads are in,
  // open the pane on that comment, once.
  const revealLinked = () => {
    if (!linked) return;
    queueMicrotask(() => {
      const entry = threads.querySelector(`[data-comment-id="${CSS.escape(linked)}"]`);
      if (!entry) return;
      linked = null;
      toggleDrawer(root, 'comments', true);
      entry.classList.add('sp-doc-thread__entry--linked');
      entry.scrollIntoView({ block: 'center' });
    });
  };

  const write = async (heading) => {
    toggleDrawer(root, 'comments', true);
    return composer.compose(heading);
  };

  const focusBlock = (id) => {
    const view = getView();
    if (!view) return;
    view.state.doc.forEach((node, offset) => {
      if (node.attrs.id === id) view.nodeDOM(offset)?.scrollIntoView({ block: 'center', behavior: 'smooth' });
    });
  };

  const post = async (body) => rawFetch(api, { method: 'POST', body: JSON.stringify(body) });

  const reply = async (thread) => {
    const written = await write('Reply');
    if (!written) return;
    await post({ anchor: thread.anchor, parent_id: thread.id, ...written });
    load();
  };

  const resolve = async (thread) => {
    await rawFetch(`${api}/${encodeURIComponent(thread.id)}/resolve`, { method: 'POST', body: '{}' });
    load();
  };

  const threadView = (thread, replies) => {
    const item = el('li', `sp-doc-thread${thread.resolved_at ? ' sp-doc-thread--resolved' : ''}`);
    if (thread.anchor?.find) item.append(el('blockquote', 'sp-doc-thread__quote', thread.anchor.find));
    for (const c of [thread, ...replies]) {
      const entry = el('div', 'sp-doc-thread__entry');
      entry.dataset.commentId = c.id;
      entry.append(el('span', 'sp-doc-thread__author', person(c.author_user_id)), el('span', 'sp-ws__muted', formatDate(c.created_at)), bodyOf(c.body));
      item.append(entry);
    }
    const actions = el('div', 'sp-doc-thread__actions');
    const go = el('button', 'sp-btn sp-btn--xs sp-btn--ghost', 'Show');
    go.type = 'button';
    go.addEventListener('click', () => focusBlock(thread.anchor?.node));
    actions.append(go);
    if (!thread.resolved_at && canComment) {
      const r = el('button', 'sp-btn sp-btn--xs sp-btn--ghost', 'Reply');
      r.type = 'button';
      r.addEventListener('click', () => reply(thread).catch(() => undefined));
      const done = el('button', 'sp-btn sp-btn--xs sp-btn--outline', 'Resolve');
      done.type = 'button';
      done.addEventListener('click', () => resolve(thread).catch(() => undefined));
      actions.append(r, done);
    }
    item.append(actions);
    return item;
  };

  async function load() {
    const body = await rawFetch(api);
    const all = body?.comments ?? [];
    await resolvePeople(root.dataset.peopleApi, all.flatMap((c) => [c.author_user_id, ...(c.mentions ?? []), ...mentionIds(c.body)]));
    const roots = all.filter((c) => !c.parent_id);
    const list = el('ol', 'sp-doc-threads');
    list.append(...roots.sort((a, b) => Boolean(a.resolved_at) - Boolean(b.resolved_at))
      .map((t) => threadView(t, all.filter((c) => c.parent_id === t.id))));
    revealLinked();
    threads.replaceChildren(roots.length ? list : el('p', 'sp-ws__muted', 'No comments yet. Select text and choose Comment.'));
  }

  const fromSelection = async (view) => {
    if (!canComment) return;
    const { from, to, $from, $to } = view.state.selection;
    if ($from.depth < 1 || $from.node(1) !== $to.node(1)) {
      showToast('Select text inside one paragraph to comment on it.', 'info');
      return;
    }
    const block = $from.node(1);
    const quote = view.state.doc.textBetween(from, to);
    const written = await write(quote ? `On “${quote.slice(0, 80)}”` : 'Comment on this paragraph');
    if (!written) return;
    const created = await post({ anchor: { node: block.attrs.id, find: quote || undefined }, ...written });
    if (created?.id && to > from) {
      const tr = view.state.tr.addMark(from, to, schema.marks.comment.create({ thread: created.id }));
      view.dispatch(tr);
    }
    load();
  };

  return { load, fromSelection: (view) => fromSelection(view).catch(() => undefined), onEvent: () => load().catch(() => undefined) };
};
