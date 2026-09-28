import { on, onOutsideClick } from './events.js';
import { closeSidebar } from './sidebar.js';

let portal = null;
let activeDropdown = null;
let activeMenu = null;

const closeOnOutsideClick = (e) => {
  if (!activeDropdown) return;
  if (activeDropdown.contains(e.target)) return;
  if (e.target.closest('[data-action="menu"]')) return;
  close();
};

const init = () => {
  if (portal) return;
  portal = document.createElement('div');
  portal.id = 'dropdown-portal';
  portal.classList.add('sp-dropdown-portal');
  document.body.append(portal);
  onOutsideClick(closeOnOutsideClick);
};

export const open = (triggerBtn) => {
  close();
  const menu = triggerBtn.closest('.sp-actions-menu');
  if (menu) {
    const dropdown = menu.querySelector('.sp-actions-dropdown');
    if (dropdown) {
      const rect = triggerBtn.getBoundingClientRect();
      const clone = dropdown.cloneNode(true);
      clone.classList.add('sp-dropdown-menu');
      clone.setAttribute('data-portal-dropdown', 'true');
      const isPhone = window.matchMedia('(max-width: 48rem)').matches;
      if (isPhone) {
        clone.classList.add('is-sheet');
      } else {
        clone.style.top = (rect.bottom + 4) + 'px';
        clone.style.right = (window.innerWidth - rect.right) + 'px';
      }

      portal.append(clone);
      // A menu opened near the viewport foot would run off screen, so it flips above its trigger.
      const box = clone.getBoundingClientRect();
      if (!isPhone && box.bottom > window.innerHeight - 8) {
        clone.style.top = Math.max(8, rect.top - 4 - box.height) + 'px';
      }
      if (!isPhone && box.left < 8) {
        clone.style.right = Math.max(8, window.innerWidth - box.width - 8) + 'px';
      }
      activeMenu = menu;
      activeDropdown = clone;
      menu.classList.add('is-open');
    }
  }
};

export const close = () => {
  activeDropdown?.remove();
  activeDropdown = null;
  activeMenu?.classList.remove('is-open');
  activeMenu = null;
};

export const closeAllMenus = () => {
  close();
  for (const m of document.querySelectorAll('.sp-actions-menu.is-open')) {
    m.classList.remove('is-open');
  }
  const installMenu = document.getElementById('install-menu');
  if (installMenu?.classList.contains('is-open')) {
    installMenu.classList.remove('is-open');
    installMenu.querySelector('.sp-install-trigger')?.setAttribute('aria-expanded', 'false');
  }
  const headerActions = document.getElementById('header-actions');
  if (headerActions?.classList.contains('is-open')) {
    headerActions.classList.remove('is-open');
    headerActions.querySelector('.sp-topbar__actions-toggle')?.setAttribute('aria-expanded', 'false');
  }
  closeSidebar();
  for (const p of document.querySelectorAll('.sf-action-menu--portal')) p.remove();
};

let dropdownReady = false;

export const initDropdown = () => {
  if (dropdownReady) return;
  dropdownReady = true;
  init();
  on('click', '[data-action="menu"]', (e, trigger) => {
    e.stopPropagation();
    open(trigger);
  }, { exclusive: true });
};
