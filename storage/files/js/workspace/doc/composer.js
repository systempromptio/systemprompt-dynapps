// The comment composer: an inline form in the comments pane that turns `@`
// into a people search and the chosen person into a mention token. Core
// shares the document (as commenter) with anyone mentioned who cannot see
// it, in the same request that stores the comment; the note under the field
// says so before posting, and blocks the post when the author cannot share.
import { createPeoplePicker } from '../people-picker.js';
import { documentIdOf, permits, subjectsApiOf } from '../people.js';
import { mentionIds, mentionToken } from './mentions.js';

const AT_TERM = /(?:^|\s)@([^\s[\]()]{1,60})$/;

const node = (tag, className, text) => {
  const el = document.createElement(tag);
  el.className = className;
  if (text != null) el.textContent = text;
  return el;
};

const buildForm = () => {
  const form = node('form', 'sp-doc-composer');
  form.hidden = true;
  const title = node('p', 'sp-doc-composer__title');
  const input = node('textarea', 'sp-field-input sp-doc-composer__input');
  input.rows = 3;
  input.setAttribute('aria-label', 'Comment');
  input.placeholder = 'Comment, or @ to mention someone';
  const field = node('div', 'sp-doc-composer__field');
  field.append(input);
  const note = node('p', 'sp-doc-composer__note');
  note.hidden = true;
  note.setAttribute('aria-live', 'polite');
  const actions = node('div', 'sp-doc-composer__actions');
  const cancel = node('button', 'sp-btn sp-btn--xs sp-btn--ghost', 'Cancel');
  cancel.type = 'button';
  const post = node('button', 'sp-btn sp-btn--xs sp-btn--primary', 'Post');
  post.type = 'submit';
  actions.append(cancel, post);
  form.append(title, field, note, actions);
  return { form, title, input, note, post, cancel };
};

export const createComposer = (root) => {
  const { form, title, input, note, post, cancel } = buildForm();
  const picked = new Map();
  let settle = null;

  const unshared = () => mentionIds(input.value).map((id) => picked.get(id)).filter((p) => p && !p.access_level);

  const refreshNote = () => {
    const people = unshared();
    const names = people.map((p) => p.display_name || p.email || p.user_id).join(', ');
    const canShare = !root.dataset.myAccess || permits(root.dataset.myAccess, 'share');
    note.hidden = !people.length;
    note.classList.toggle('sp-doc-composer__note--blocked', !canShare);
    note.textContent = canShare
      ? `${names} will get comment access to this document.`
      : `${names} can't see this document, and only someone who can share it can mention them. Ask an owner to share it.`;
    post.disabled = Boolean(people.length) && !canShare;
    return post.disabled;
  };

  const insert = (chosen) => {
    picked.set(chosen.user_id, chosen);
    const caret = input.selectionStart;
    const term = input.value.slice(0, caret).match(AT_TERM)?.[1] ?? '';
    const start = caret - term.length - 1;
    const token = `${mentionToken(chosen)} `;
    input.setRangeText(token, start, caret, 'end');
    input.focus();
    refreshNote();
  };

  const picker = createPeoplePicker({
    input, subjectsApi: subjectsApiOf(root), documentId: documentIdOf(root), onPick: insert,
  });

  const done = (result) => {
    picker.close();
    form.hidden = true;
    note.hidden = true;
    settle?.(result);
    settle = null;
  };

  const submit = () => {
    const body = input.value.trim();
    if (!body || refreshNote()) return;
    done({ body, mentions: mentionIds(body) });
  };

  input.addEventListener('input', () => {
    const term = input.value.slice(0, input.selectionStart).match(AT_TERM)?.[1];
    if (term) picker.search(term);
    else picker.close();
    refreshNote();
  });
  input.addEventListener('keydown', (evt) => {
    if (picker.handleKey(evt)) return;
    if (evt.key === 'Escape') done(null);
    if (evt.key === 'Enter' && (evt.metaKey || evt.ctrlKey)) form.requestSubmit();
  });
  form.addEventListener('submit', (evt) => {
    evt.preventDefault();
    submit();
  });
  cancel.addEventListener('click', () => done(null));

  const compose = (heading) => {
    settle?.(null);
    title.textContent = heading;
    input.value = '';
    picked.clear();
    refreshNote();
    form.hidden = false;
    input.focus();
    return new Promise((resolve) => { settle = resolve; });
  };

  return { form, compose };
};
