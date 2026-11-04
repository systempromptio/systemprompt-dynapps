// The Drive listing's "New document" dialog: a template from the install's
// catalog, a name and the template's title, created in the open folder.
import { rawFetch } from './api.js';

const createBody = (form, folderId) => {
  const data = new FormData(form);
  const values = {};
  for (const input of form.querySelectorAll('[data-param]')) {
    if (input.value.trim()) values[input.dataset.param] = input.value.trim();
  }
  return {
    template_id: data.get('template_id'),
    name: String(data.get('name')).trim(),
    folder_id: folderId ?? null,
    params: { values },
  };
};

const showError = (dialog, message) => {
  const slot = dialog.querySelector('[data-slot="drive-create-error"]');
  slot.textContent = message;
  slot.hidden = false;
};

const submit = async (dialog, listing) => {
  const form = dialog.querySelector('[data-slot="drive-create-form"]');
  const { documentsApi, baseUrl } = listing.root.dataset;
  const button = form.querySelector('button[type="submit"]');
  button.disabled = true;
  try {
    const { document: created } = await rawFetch(documentsApi, {
      method: 'POST',
      body: JSON.stringify(createBody(form, listing.folderId)),
    });
    window.location.assign(`${baseUrl}/${encodeURIComponent(created.id)}`);
  } catch (err) {
    button.disabled = false;
    showError(dialog, err.message);
  }
};

export const initCreateDialog = (dialog, listing) => {
  document.querySelector('[data-action="drive-create-open"]')?.addEventListener('click', () => {
    dialog.querySelector('[data-slot="drive-create-error"]').hidden = true;
    dialog.showModal();
  });
  dialog.querySelector('[data-action="drive-create-cancel"]').addEventListener('click', () => dialog.close());
  dialog.querySelector('[data-slot="drive-create-form"]').addEventListener('submit', (evt) => {
    evt.preventDefault();
    submit(dialog, listing);
  });
};
