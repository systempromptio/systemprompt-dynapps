import { initSyncPlane } from '../components/sp-sync-plane.js';

// The editor is a plain form and works without this file. One enhancement:
// a window's custom length field only shows when the length is "custom".
// The delete confirmation is the shell's services/form-confirm.js.

const CUSTOM = 'custom';

const syncCustomField = (select) => {
  const row = select.closest('[data-window]');
  if (!row) return;
  const custom = row.querySelector('[data-window-custom]');
  if (custom) custom.hidden = select.value !== CUSTOM;
};

document.querySelectorAll('[data-window-choice]').forEach((select) => {
  syncCustomField(select);
  select.addEventListener('change', () => syncCustomField(select));
});

// Why: the Sync tab renders the shared sync-plane component; its buttons
// are bound here so the tab works without a second page script.
initSyncPlane();
