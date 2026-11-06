// The Sharing tab's request form for someone who can only view: the level
// they are asking for (above view) and an optional message to whoever can
// share; `onRequest` answers whether the request went through. Built once, so a live reload never clears what is being typed.
import { el, heading } from './pane-dom.js';

const LEVELS = ['comment', 'edit', 'share'];
const LEVEL_LABELS = { comment: 'Can comment', edit: 'Can edit', share: 'Can share' };
const MAX_MESSAGE = 1024;

const field = (labelText, control) => {
  const label = el('label', 'sp-ws-share__field');
  label.append(el('span', 'sp-form-label', labelText), control);
  return label;
};

export const requestAccessForm = (onRequest) => {
  const form = el('form', 'sp-ws-share');
  const level = el('select', 'sp-select');
  level.name = 'level';
  level.append(...LEVELS.map((l) => new Option(LEVEL_LABELS[l], l, false, l === 'edit')));
  const message = el('textarea', 'sp-field-input');
  Object.assign(message, { name: 'message', rows: 2, maxLength: MAX_MESSAGE, placeholder: 'Why you need it (optional)' });
  const sent = el('p', 'sp-ws__muted');
  sent.hidden = true;
  const submit = el('button', 'sp-btn sp-btn--sm sp-btn--primary', 'Request access');
  submit.type = 'submit';

  form.addEventListener('submit', (evt) => {
    evt.preventDefault();
    submit.disabled = true;
    onRequest({ level: level.value, message: message.value.trim() })
      .then((ok) => {
        if (!ok) return;
        form.reset();
        sent.textContent = 'Request sent.';
        sent.hidden = false;
      })
      .finally(() => {
        submit.disabled = false;
      });
  });

  form.append(heading('Request access'), field('Access', level), field('Message', message), sent, submit);
  return form;
};
