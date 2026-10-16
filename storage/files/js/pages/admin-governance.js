import { on, initDelegation } from '../services/events.js';
import { initSyncPlane } from '../components/sp-sync-plane.js';

const submit = (el) => el.closest('form')?.requestSubmit();

export const init = () => {
  initDelegation();
  on('change', '[data-governance-filters] select', (event, el) => submit(el));
};

init();
initSyncPlane();
