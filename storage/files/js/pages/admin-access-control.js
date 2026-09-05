import { loadState, showPane } from './admin-access-control-state.js';
import { renderBandEditor } from './admin-access-control-editors.js';
import { renderUserMatrix } from './admin-access-control-matrix.js';
import { bindDialogs } from './admin-access-control-modals.js';

const SELECTORS = '[data-action="select-group"], [data-action="select-project"], [data-action="select-user"]';

const clearActive = (root) => {
  for (const btn of root.querySelectorAll('[aria-pressed="true"]')) {
    btn.setAttribute('aria-pressed', 'false');
  }
};

const selectTarget = (root, target) => {
  clearActive(root);
  target.setAttribute('aria-pressed', 'true');
  const { action, group, groupLabel, project, projectLabel, userId, userDisplay } = target.dataset;
  if (action === 'select-group') {
    renderBandEditor('group', group, groupLabel);
  } else if (action === 'select-project') {
    renderBandEditor('project', project, projectLabel);
  } else {
    renderUserMatrix(userId, userDisplay);
  }
};

const bindTree = (root) => {
  const activate = (ev) => {
    const target = ev.target.closest(SELECTORS);
    if (!target) return;
    ev.preventDefault();
    selectTarget(root, target);
  };
  root.addEventListener('click', activate);
  root.addEventListener('keydown', (ev) => {
    if (ev.key === 'Enter' || ev.key === ' ') activate(ev);
  });
};

const matchesQuery = (row, query) => {
  const name = (row.dataset.userDisplay ?? '').toLowerCase();
  const email = (row.dataset.userEmail ?? '').toLowerCase();
  return !query || name.includes(query) || email.includes(query);
};

const filterTree = (root, query) => {
  for (const branch of root.querySelectorAll('.ac-tree-group')) {
    let anyMatch = false;
    for (const row of branch.querySelectorAll('.ac-user-row')) {
      const visible = matchesQuery(row, query);
      row.parentElement.hidden = !visible;
      if (visible) anyMatch = true;
    }
    const label = (branch.dataset.group ?? branch.dataset.project ?? '').toLowerCase();
    branch.hidden = Boolean(query) && !anyMatch && !label.includes(query);
  }
};

const bindSearch = (root) => {
  const input = document.querySelector('[data-ac="search"]');
  if (!input) return;
  input.addEventListener('input', () => filterTree(root, input.value.trim().toLowerCase()));
};

const focusRequestedUser = (root) => {
  const userId = new URLSearchParams(window.location.search).get('user');
  const target = userId
    ? root.querySelector(`.ac-user-row[data-user-id="${CSS.escape(userId)}"]`)
    : null;
  if (target) {
    selectTarget(root, target);
    return;
  }
  showPane('welcome');
};

const init = () => {
  const root = document.querySelector('[data-ac="layout"]');
  if (!root) return;
  loadState();
  bindTree(root);
  bindSearch(root);
  bindDialogs();
  focusRequestedUser(root);
};

init();
