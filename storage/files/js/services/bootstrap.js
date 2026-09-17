import { initDelegation, setCloseMenus } from './events.js';
import { initDropdown, closeAllMenus } from './dropdown.js';
import { initSidebar } from './sidebar.js';
import { initNavGroups } from './nav-groups.js';
import { initTableExpand } from './table-expand.js';
import { initHeaderActions } from './header-actions.js';
import { initHeaderSearch } from './header-search.js';
import { initLogout, initUserDisplay, getUserContext } from './auth.js';
import { initFilterRibbon } from './filter-ribbon.js';
import { initScope } from './scope.js';
import { initExport } from './export.js';
import { showToast } from './toast.js';

const run = (init) => {
  try {
    const result = init();
    if (result instanceof Promise) {
      result.catch((err) => showToast(err.message || 'Initialisation failed', 'error'));
    }
  } catch (err) {
    showToast(err.message || 'Initialisation failed', 'error');
  }
};

setCloseMenus(closeAllMenus);

for (const init of [
  initDelegation,
  initDropdown,
  initSidebar,
  initNavGroups,
  initTableExpand,
  initHeaderActions,
  initHeaderSearch,
  initFilterRibbon,
  initScope,
  initExport,
  initLogout,
  initUserDisplay,
  getUserContext
]) {
  run(init);
}
