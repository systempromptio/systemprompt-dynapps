// The notification bell: the signed-in user's feed from core's notification
// fabric (mentions, shares and stream items), kept live from core's per-user
// AG-UI stream (`data-live-stream`). Read state is stored per user in core, so this bell, the bridge
// and the documents MCP inbox show one unread count.
import { rawFetch, rawResponse } from './api.js';
import { cloneRow, formatDate } from './drive-rows.js';
import { initDelegation, onOutsideClick } from './events.js';

const LIST_LIMIT = 20;
const UNREAD_CAP = 1000;

const KIND_TEXT = {
  'document.comment.mentioned': 'mentioned you in a comment',
  'document.shared': 'shared a document with you',
};

const actorOf = (n) => {
  const name = n.actor_display ?? 'Someone';
  return n.via_agent ? `${name}'s agent` : name;
};

const textOf = (n) => {
  const folded = n.coalesce_count > 1 ? ` (${n.coalesce_count})` : '';
  if (n.kind === 'stream.published') return `${actorOf(n)} posted in #${n.stream}${folded}`;
  return `${actorOf(n)} ${KIND_TEXT[n.kind] ?? n.kind}${folded}`;
};

// EventSource reconnects by itself, but a frame sent while it was down is
// lost, so every open after the first re-reads the feed.
const REOPENED = 'stream.reopened';

const listen = (url, onEvent) => {
  const stream = new EventSource(url);
  let opened = false;
  stream.addEventListener('message', (message) => {
    let event;
    try {
      event = JSON.parse(message.data);
    } catch {
      return;
    }
    if (event?.type !== 'CUSTOM' || !String(event.name).startsWith('notification.')) return;
    onEvent(event.name, event.value);
  });
  stream.addEventListener('open', () => {
    if (opened) onEvent(REOPENED, null);
    opened = true;
  });
};

const ATTENTION_TEXT = { human: 'Needs you', agent: 'For your agent', silent: '' };

const createBell = (root) => {
  const { inboxApi } = root.dataset;
  const button = root.querySelector('[data-action="bell-toggle"]');
  const panel = root.querySelector('.sp-bell__panel');
  const list = root.querySelector('[data-slot="bell-list"]');
  const count = root.querySelector('[data-slot="bell-count"]');

  const setUnread = (unread, capped = unread >= UNREAD_CAP) => {
    const label = capped ? `${UNREAD_CAP}+` : String(unread);
    count.hidden = !unread;
    count.textContent = unread > 99 ? '99+' : label;
    button.setAttribute('aria-label', unread ? `Notifications, ${label} unread` : 'Notifications');
  };

  const markRead = (ids) => rawFetch(`${inboxApi}/read`, {
    method: 'POST',
    keepalive: true,
    body: JSON.stringify(ids ? { ids } : { all: true }),
  });

  const item = (n) => {
    const row = cloneRow('bell-item', {
      text: textOf(n),
      excerpt: n.excerpt ?? '',
      attention: ATTENTION_TEXT[n.attention] ?? '',
      time: formatDate(n.last_event_at),
    });
    row.dataset.id = n.id;
    row.classList.toggle('sp-bell__item--unread', !n.read_at);
    row.classList.toggle('sp-bell__item--human', n.attention === 'human');
    const link = row.querySelector('[data-field="link"]');
    if (n.action?.open_url) link.href = n.action.open_url;
    link.addEventListener('click', () => {
      if (!n.read_at) markRead([n.id]).catch(() => undefined);
    });
    return row;
  };

  const showEmpty = () => {
    root.querySelector('[data-slot="bell-empty"]').hidden = list.children.length > 0;
  };

  // Read without the transport's error toast: a binary built without the
  // notifications feature answers 404 on every page, and the bell hides.
  const load = async () => {
    const resp = await rawResponse(`${inboxApi}?limit=${LIST_LIMIT}`);
    root.hidden = !resp.ok;
    if (!resp.ok) return;
    const body = await resp.json();
    list.replaceChildren(...(body?.notifications ?? []).map(item));
    setUnread(body?.unread ?? 0, body?.unread_capped);
    showEmpty();
  };

  const readLocally = (ids) => {
    for (const row of list.children) {
      if (!ids.length || ids.includes(row.dataset.id)) row.classList.remove('sp-bell__item--unread');
    }
  };

  const onEvent = (name, value) => {
    if (name === 'notification.created') {
      list.querySelector(`[data-id="${CSS.escape(value.notification.id)}"]`)?.remove();
      list.prepend(item(value.notification));
      if (list.children.length > LIST_LIMIT) list.lastElementChild.remove();
      setUnread(value.unread);
      showEmpty();
    } else if (name === 'notification.read') {
      readLocally(value.ids);
      setUnread(value.unread);
    } else if (name === 'notification.escalated' || name === REOPENED) {
      load().catch(() => undefined);
    }
  };

  const toggle = (open) => {
    panel.hidden = !open;
    button.setAttribute('aria-expanded', String(open));
    if (!open) return;
    load().catch(() => undefined);
    panel.querySelector('a, button')?.focus();
  };

  button.addEventListener('click', () => toggle(panel.hidden));
  root.addEventListener('keydown', (evt) => {
    if (evt.key !== 'Escape' || panel.hidden) return;
    toggle(false);
    button.focus();
  });
  onOutsideClick((evt) => {
    if (!panel.hidden && !root.contains(evt.target)) toggle(false);
  });
  root.querySelector('[data-action="bell-read-all"]').addEventListener('click', async () => {
    const { unread } = await markRead(null);
    readLocally([]);
    setUnread(unread);
  });

  return { load, onEvent };
};

export const initNotificationBell = () => {
  const root = document.querySelector('[data-notification-bell]');
  if (!root) return;
  initDelegation();
  const bell = createBell(root);
  bell.load().catch(() => undefined);
  if (root.dataset.liveStream) listen(root.dataset.liveStream, bell.onEvent);
};
