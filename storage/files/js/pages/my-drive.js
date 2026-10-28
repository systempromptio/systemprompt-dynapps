import '/assets/documents/js/documents.js';
import { rawFetch } from '../services/api.js';

const DOCUMENTS_API = '/api/v1/documents';

const createBody = (form) => {
  const data = new FormData(form);
  const values = {};
  for (const input of form.querySelectorAll('[data-param]')) {
    if (input.value.trim()) values[input.dataset.param] = input.value.trim();
  }
  return {
    template_id: data.get('template_id'),
    name: String(data.get('name')).trim(),
    params: { values },
  };
};

const showError = (dialog, message) => {
  const slot = dialog.querySelector('[data-slot="drive-create-error"]');
  slot.textContent = message;
  slot.hidden = false;
};

const submitCreate = async (dialog, baseUrl) => {
  const form = dialog.querySelector('[data-slot="drive-create-form"]');
  try {
    const { document: created } = await rawFetch(DOCUMENTS_API, {
      method: 'POST',
      body: JSON.stringify(createBody(form)),
    });
    window.location.assign(`${baseUrl}/${encodeURIComponent(created.id)}`);
  } catch (err) {
    showError(dialog, err.message);
  }
};

const bindCreate = (root, dialog) => {
  const { baseUrl } = root.dataset;
  document.querySelector('[data-action="drive-create-open"]')?.addEventListener('click', () => {
    dialog.querySelector('[data-slot="drive-create-error"]').hidden = true;
    dialog.showModal();
  });
  dialog.querySelector('[data-action="drive-create-cancel"]').addEventListener('click', () => dialog.close());
  dialog.querySelector('[data-slot="drive-create-form"]').addEventListener('submit', (evt) => {
    evt.preventDefault();
    submitCreate(dialog, baseUrl);
  });
};

export const init = () => {
  const root = document.querySelector('[data-my-drive]');
  if (!root) return;
  root.addEventListener('sp-document-open', (evt) => {
    window.location.assign(`${root.dataset.baseUrl}/${encodeURIComponent(evt.detail.id)}`);
  });
  const dialog = document.querySelector('[data-drive-create]');
  if (dialog) bindCreate(root, dialog);
};

init();
