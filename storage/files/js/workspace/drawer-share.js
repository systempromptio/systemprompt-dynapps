// The Sharing tab's grant form: who (a subject kind from core's list, found
// through the picker, or typed when the kind cannot be listed), at which level
// (never above the sharer's own), until when, and an optional message.
import { createPeoplePicker } from './people-picker.js';
import { subjectsApiOf } from './people.js';
import { el } from './pane-dom.js';

const LEVELS = ['view', 'comment', 'edit', 'share'];
const RANK = { view: 1, comment: 2, edit: 3, share: 4, own: 5 };
const LEVEL_LABELS = { view: 'Can view', comment: 'Can comment', edit: 'Can edit', share: 'Can share' };
const KIND_LABELS = { user: 'Person', role: 'Role', group: 'Group' };
const MAX_MESSAGE = 1024;

const field = (labelText, control) => {
  const label = el('label', 'sp-ws-share__field');
  label.append(el('span', 'sp-form-label', labelText), control);
  return label;
};

const select = (name, values, labels, chosen) => {
  const box = el('select', 'sp-select');
  box.name = name;
  box.append(...values.map((v) => new Option(labels[v] ?? v, v, false, v === chosen)));
  return box;
};

export const shareForm = (ctx, kinds, onShare) => {
  const form = el('form', 'sp-ws-share');
  const kind = select('kind', kinds, KIND_LABELS, kinds[0]);
  const subject = el('input', 'sp-field-input');
  Object.assign(subject, { name: 'subject', autocomplete: 'off', required: true, placeholder: 'Name, email or id' });
  const hint = el('p', 'sp-ws__muted');
  hint.hidden = true;
  const own = RANK[ctx.access] ?? 0;
  const level = select('level', LEVELS.filter((l) => RANK[l] <= own), LEVEL_LABELS, 'view');
  const expires = el('input', 'sp-field-input');
  Object.assign(expires, { name: 'expires', type: 'datetime-local' });
  const message = el('textarea', 'sp-field-input');
  Object.assign(message, { name: 'message', rows: 2, maxLength: MAX_MESSAGE, placeholder: 'Message (optional)' });
  const submit = el('button', 'sp-btn sp-btn--sm sp-btn--primary', 'Share');
  submit.type = 'submit';

  let chosen = null;
  let searchable = true;
  const subjectBox = field('Share with', subject);
  subjectBox.classList.add('sp-ws__share');
  const picker = createPeoplePicker({
    input: subject,
    subjectsApi: subjectsApiOf(ctx.root),
    documentId: ctx.documentId,
    kind: () => kind.value,
    onSearchable: (yes) => {
      searchable = yes;
      hint.hidden = yes;
      hint.textContent = yes ? '' : `Enter the exact ${KIND_LABELS[kind.value] ?? kind.value} value.`;
    },
    onPick: (p) => {
      chosen = p;
      subject.value = p.display_name || p.email || p.user_id;
    },
  });
  subject.addEventListener('input', () => {
    chosen = null;
    picker.search(subject.value.trim());
  });
  subject.addEventListener('keydown', (evt) => picker.handleKey(evt));
  kind.addEventListener('change', () => {
    picker.close();
    chosen = null;
    searchable = true;
    subject.value = '';
    hint.hidden = true;
  });

  const valueOf = () => {
    if (chosen) return chosen.user_id;
    if (kind.value === 'user') return null;
    return searchable && !subject.value.trim() ? null : subject.value.trim();
  };

  form.addEventListener('submit', (evt) => {
    evt.preventDefault();
    const subjectValue = valueOf();
    if (!subjectValue) {
      hint.hidden = false;
      hint.textContent = 'Choose someone from the list.';
      return;
    }
    onShare({
      subjectKind: kind.value,
      subjectValue,
      level: level.value,
      validUntil: expires.value ? new Date(expires.value).toISOString() : null,
      message: message.value.trim(),
    }).then(() => {
      form.reset();
      chosen = null;
    });
  });

  form.append(...(kinds.length > 1 ? [field('Kind', kind)] : []), subjectBox, hint, field('Access', level), field('Expires', expires), field('Message', message), submit);
  return form;
};
