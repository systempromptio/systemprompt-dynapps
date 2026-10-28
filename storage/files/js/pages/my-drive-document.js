import '/assets/documents/js/documents.js';

export const init = () => {
  const dialog = document.querySelector('[data-drive-share]');
  const trigger = document.querySelector('[data-action="drive-share-open"]');
  if (!dialog || !trigger) return;
  trigger.addEventListener('click', () => dialog.showModal());
};

init();
