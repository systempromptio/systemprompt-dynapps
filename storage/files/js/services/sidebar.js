// Why: two nav links can render for the same page id — `/admin` and
// `/admin/analytics` are both the dashboard — so the server marks a best guess
// and the exact pathname settles it here. Only one link may be active, so a
// match clears the others. Record rows (`.sp-nav-child`) are left out: their
// href is the page itself, and the server already marked them.
const markActiveLink = (sidebar) => {
  const links = [...sidebar.querySelectorAll('nav a:not(.sp-nav-child)')];
  const exact = links.find((link) => new URL(link.href, window.location.origin).pathname === window.location.pathname);
  if (!exact) return;
  for (const link of links) {
    link.classList.toggle('is-active', link === exact);
    if (link === exact) {
      link.setAttribute('aria-current', 'page');
    } else {
      link.removeAttribute('aria-current');
    }
  }
};

const FOCUSABLE = 'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), [tabindex]:not([tabindex="-1"])';
const DESKTOP = '(min-width: 48.0625rem)';

let opener = null;
let clickInside = false;

const parts = () => ({
  sidebar: document.getElementById('sp-admin-sidebar'),
  overlay: document.getElementById('sp-sidebar-overlay'),
  toggle: document.querySelector('.sp-topbar__nav-toggle'),
  main: document.getElementById('main')
});

// Why: the drawer is modal, so the page behind it is made inert and the
// scroll locked; focus moves into the drawer and returns to whatever opened it.
const openSidebar = () => {
  const { sidebar, overlay, toggle, main } = parts();
  if (!sidebar || sidebar.classList.contains('is-open')) return;
  opener = document.activeElement;
  sidebar.classList.add('is-open');
  overlay?.classList.add('is-open');
  toggle?.setAttribute('aria-expanded', 'true');
  document.documentElement.classList.add('is-drawer-open');
  main?.setAttribute('inert', '');
  sidebar.querySelector('.sp-sidebar-close-btn')?.focus();
};

// Why: closeAllMenus runs this on every click no delegated handler claimed,
// and the nav-group toggles are not exclusive, so a click inside the drawer
// must not count as a click outside it.
export const closeSidebar = () => {
  const { sidebar, overlay, toggle, main } = parts();
  if (!sidebar?.classList.contains('is-open') || clickInside) return;
  sidebar.classList.remove('is-open');
  overlay?.classList.remove('is-open');
  toggle?.setAttribute('aria-expanded', 'false');
  document.documentElement.classList.remove('is-drawer-open');
  main?.removeAttribute('inert');
  const back = opener;
  opener = null;
  back?.focus?.();
};

// Why: with the page inert, Tab past the last link would leave for the browser
// chrome, so the drawer wraps focus between its own first and last controls.
const trapFocus = (sidebar) => (event) => {
  if (event.key !== 'Tab' || !sidebar.classList.contains('is-open')) return;
  const items = [...sidebar.querySelectorAll(FOCUSABLE)].filter((el) => el.offsetParent !== null);
  if (items.length === 0) return;
  const first = items[0];
  const last = items[items.length - 1];
  if (event.shiftKey && document.activeElement === first) {
    event.preventDefault();
    last.focus();
  } else if (!event.shiftKey && document.activeElement === last) {
    event.preventDefault();
    first.focus();
  }
};

export const initSidebar = () => {
  const { sidebar, overlay, toggle } = parts();
  const closeBtn = sidebar?.querySelector('.sp-sidebar-close-btn');
  if (sidebar) markActiveLink(sidebar);
  if (!toggle || !sidebar) return;

  toggle.addEventListener('click', (e) => {
    e.stopPropagation();
    if (sidebar.classList.contains('is-open')) {
      closeSidebar();
    } else {
      openSidebar();
    }
  });
  closeBtn?.addEventListener('click', (e) => {
    e.stopPropagation();
    closeSidebar();
  });
  overlay?.addEventListener('click', (e) => {
    e.stopPropagation();
    closeSidebar();
  });
  sidebar.addEventListener('keydown', trapFocus(sidebar));
  sidebar.addEventListener('click', () => {
    clickInside = true;
    setTimeout(() => { clickInside = false; });
  });
  for (const link of sidebar.querySelectorAll('nav a')) {
    link.addEventListener('click', closeSidebar);
  }
  // Why: widening past the breakpoint turns the drawer back into a static
  // column, and an inert main left behind would freeze the whole page.
  window.matchMedia(DESKTOP).addEventListener('change', (e) => {
    if (e.matches) closeSidebar();
  });
};
