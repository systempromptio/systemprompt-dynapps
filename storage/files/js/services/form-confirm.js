// Every server-rendered form that asks before it acts carries
// `data-confirm="<question>"` and, optionally, `data-confirm-label` for the
// button. The dialog is the same one the JSON pages use, so a destructive
// action reads the same wherever it lives.

import { showConfirmDialog } from './confirm.js';

const CONFIRMED = 'yes';

const onSubmit = (event) => {
  const form = event.target.closest('form[data-confirm]');
  if (!form || form.dataset.confirmed === CONFIRMED) return;
  event.preventDefault();
  const label = form.dataset.confirmLabel || 'Confirm';
  const title = form.dataset.confirmTitle || label;
  showConfirmDialog(title, form.dataset.confirm, label, () => {
    form.dataset.confirmed = CONFIRMED;
    form.requestSubmit();
  });
};

export const initFormConfirm = () => {
  document.addEventListener('submit', onSubmit);
};
