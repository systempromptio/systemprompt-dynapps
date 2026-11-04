// The Comments tab for every format without its own anchored comments (sheets,
// decks, the preview): threads with replies and resolve, each anchored to the
// sheet tab or slide that was selected when it was written. Choosing an
// anchor's Show reveals that node in the canvas. Hidden when comments are off.
import { listComments, postComment, resolveComment } from '../documents/documents-api.js';
import { permits, person, resolvePeople } from './people.js';
import { alertLine, attempt, button, el, relativeTime } from './pane-dom.js';
import { paneUnavailable, registerDrawerTab } from './registry.js';

const MAX_BODY = 4000;
const panes = new WeakMap();
const labels = new Map();

const textBox = (label, placeholder) => {
  const box = el('textarea', 'sp-field-input sp-ws-comment__body');
  Object.assign(box, { rows: 2, maxLength: MAX_BODY, placeholder, required: true });
  box.setAttribute('aria-label', label);
  return box;
};

const anchorLabel = (anchor) => (anchor?.node ? labels.get(anchor.node) ?? 'a part of the document' : null);

const composer = (ctx, state, reload) => {
  const form = el('form', 'sp-ws-comment__form');
  const body = textBox('New comment', 'Add a comment');
  const chip = el('p', 'sp-ws__muted');
  const clear = button('Clear anchor', 'sp-btn--ghost', () => {
    state.anchor = null;
    showChip();
  });
  function showChip() {
    chip.textContent = state.anchor ? `On ${state.anchor.label}` : 'On the whole document';
    clear.hidden = !state.anchor;
  }
  showChip();
  state.showChip = showChip;
  const send = el('button', 'sp-btn sp-btn--sm sp-btn--primary', 'Comment');
  send.type = 'submit';
  form.append(body, chip, clear, send);
  form.addEventListener('submit', (evt) => {
    evt.preventDefault();
    const anchor = state.anchor ? { node: state.anchor.node } : undefined;
    attempt(state.alert, async () => {
      await postComment(ctx.documentId, { anchor, body: body.value.trim() });
      form.reset();
    }, reload);
  });
  return form;
};

const replyForm = (thread, ctx, state, reload) => {
  const form = el('form', 'sp-ws-comment__form');
  const body = textBox('Reply', 'Reply');
  const send = el('button', 'sp-btn sp-btn--xs sp-btn--outline', 'Reply');
  send.type = 'submit';
  form.append(body, send);
  form.addEventListener('submit', (evt) => {
    evt.preventDefault();
    attempt(state.alert, () => postComment(ctx.documentId, {
      anchor: thread.anchor ?? undefined, body: body.value.trim(), parentId: thread.id,
    }), reload);
  });
  return form;
};

const threadView = (thread, replies, ctx, state, reload) => {
  const item = el('li', `sp-doc-thread${thread.resolved_at ? ' sp-doc-thread--resolved' : ''}`);
  const on = anchorLabel(thread.anchor);
  if (on) {
    const quote = el('p', 'sp-doc-thread__quote', `On ${on}`);
    quote.append(' ', button('Show', 'sp-btn--ghost', () => ctx.root.dispatchEvent(
      new CustomEvent('ws:reveal', { detail: { node: thread.anchor.node } }),
    )));
    item.append(quote);
  }
  for (const c of [thread, ...replies]) {
    const entry = el('div', 'sp-doc-thread__entry');
    entry.dataset.commentId = c.id;
    entry.append(el('span', 'sp-doc-thread__author', person(c.author_user_id)), el('span', 'sp-ws__muted', relativeTime(c.created_at)), el('p', 'sp-doc-thread__body', c.body));
    item.append(entry);
  }
  if (thread.resolved_at) item.append(el('p', 'sp-ws__muted', 'Resolved'));
  else if (state.canComment) {
    item.append(replyForm(thread, ctx, state, reload), button('Resolve', 'sp-btn--outline', () => attempt(
      state.alert, () => resolveComment(ctx.documentId, thread.id), reload,
    )));
  }
  return item;
};

const listenForAnchors = (ctx, state) => {
  const take = (evt) => {
    const { node, label } = evt.detail ?? {};
    if (!node) return;
    if (label) labels.set(node, label);
    state.anchor = { node, label: labels.get(node) ?? 'the selection' };
    state.showChip?.();
  };
  ctx.root.addEventListener('ws:anchor', take);
  ctx.root.addEventListener('ws:reveal', take);
};

const revealLinked = (pane, ctx, state) => {
  if (!state.linked) return;
  const entry = pane.querySelector(`[data-comment-id="${CSS.escape(state.linked)}"]`);
  if (!entry) return;
  state.linked = null;
  ctx.openPane?.('comments');
  entry.classList.add('sp-doc-thread__entry--linked');
  entry.scrollIntoView({ block: 'center' });
};

const load = async (pane, ctx) => {
  if (!panes.has(pane)) {
    const state = {
      alert: alertLine(), anchor: null, canComment: permits(ctx.access, 'comment'),
      linked: new URLSearchParams(window.location.search).get('comment'), body: el('ol', 'sp-doc-threads'),
    };
    const reload = () => load(pane, ctx).catch(() => undefined);
    listenForAnchors(ctx, state);
    pane.replaceChildren(state.alert.node, ...(state.canComment ? [composer(ctx, state, reload)] : []), state.body);
    panes.set(pane, { state, reload });
  }
  const { state, reload } = panes.get(pane);
  let comments;
  try {
    ({ comments } = await listComments(ctx.documentId));
  } catch (err) {
    if (err.status === 404) paneUnavailable(pane);
    else state.alert.show(err);
    return;
  }
  await resolvePeople(ctx.peopleApi, comments.map((c) => c.author_user_id));
  const roots = comments.filter((c) => !c.parent_id)
    .sort((a, b) => Boolean(a.resolved_at) - Boolean(b.resolved_at));
  state.body.replaceChildren(...(roots.length
    ? roots.map((t) => threadView(t, comments.filter((c) => c.parent_id === t.id), ctx, state, reload))
    : [el('li', 'sp-ws__muted', 'No comments yet.')]));
  revealLinked(pane, ctx, state);
};

export const commentsTab = {
  id: 'comments',
  label: 'Comments',
  order: 40,
  eager: true,
  live: ['document.comment'],
  load,
};

registerDrawerTab(commentsTab);
