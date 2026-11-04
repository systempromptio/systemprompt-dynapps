// The owner's two destructive actions on the Sharing tab: hand the document to
// another person, and revoke every grant at once. Each asks for confirmation
// in place, as an alert dialog inside the pane, before it runs.
import { createPeoplePicker } from './people-picker.js';
import { subjectsApiOf } from './people.js';
import { button, el, heading } from './pane-dom.js';

const confirmBox = (question, onConfirm, onCancel) => {
  const box = el('div', 'sp-ws-confirm');
  box.setAttribute('role', 'alertdialog');
  box.setAttribute('aria-label', 'Confirm');
  const text = el('p', 'sp-ws-confirm__text', question);
  text.id = `ws-confirm-${Date.now()}`;
  box.setAttribute('aria-describedby', text.id);
  const yes = button('Confirm', 'sp-btn--danger', onConfirm);
  box.append(text, yes, button('Cancel', 'sp-btn--ghost', onCancel));
  queueMicrotask(() => yes.focus());
  return box;
};

export const ownerActions = (ctx, { onTransfer, onRevokeAll }) => {
  const box = el('section', 'sp-ws__pane');
  const slot = el('div', '');
  const ask = (question, work) => {
    slot.replaceChildren(confirmBox(question, () => {
      slot.replaceChildren();
      work();
    }, () => slot.replaceChildren()));
  };

  const form = el('form', 'sp-ws__share');
  const input = el('input', 'sp-field-input');
  Object.assign(input, { autocomplete: 'off', placeholder: 'New owner', required: true });
  input.setAttribute('aria-label', 'New owner');
  let chosen = null;
  const picker = createPeoplePicker({
    input,
    subjectsApi: subjectsApiOf(ctx.root),
    documentId: ctx.documentId,
    onPick: (p) => {
      chosen = p;
      input.value = p.display_name || p.email || p.user_id;
    },
  });
  input.addEventListener('input', () => {
    chosen = null;
    picker.search(input.value.trim());
  });
  input.addEventListener('keydown', (evt) => picker.handleKey(evt));
  const submit = el('button', 'sp-btn sp-btn--sm sp-btn--outline', 'Transfer ownership');
  submit.type = 'submit';
  form.append(input, submit);
  form.addEventListener('submit', (evt) => {
    evt.preventDefault();
    if (!chosen) return;
    const target = chosen;
    ask(`Transfer ownership to ${input.value}? You keep edit access and cannot undo this.`, () => onTransfer(target.user_id));
  });

  const revokeAll = button('Revoke all', 'sp-btn--danger', () => ask('Revoke every grant on this document?', onRevokeAll));
  box.append(heading('Owner'), form, revokeAll, slot);
  return {
    node: box,
    update: (access) => { revokeAll.hidden = !(access.grants?.length); },
  };
};
