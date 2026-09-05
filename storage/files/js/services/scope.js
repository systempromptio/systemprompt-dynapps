// The AI-activity scope contract: `group`, `project`, `user_id` and `preset`
// are the four parameters every page in that group reads, and a person who set
// them on one page means them on the next. The server renders each page for
// the parameters it was given; this module is what carries them across.

const SCOPE_KEYS = ['group', 'project', 'user_id', 'preset'];

const currentScope = () => {
  const params = new URLSearchParams(window.location.search);
  const scope = new Map();
  for (const key of SCOPE_KEYS) {
    const value = params.get(key);
    if (value) scope.set(key, value);
  }
  return scope;
};

const withScope = (href, scope) => {
  const url = new URL(href, window.location.origin);
  for (const [key, value] of scope) url.searchParams.set(key, value);
  return `${url.pathname}${url.search}`;
};

// Why: only the AI-activity links carry the scope. Marking them in the sidebar
// rather than matching on path keeps the group's membership in one place — the
// template that draws it.
const carryScopeIntoNav = (scope) => {
  if (scope.size === 0) return;
  for (const link of document.querySelectorAll('a[data-scope-nav]')) {
    link.href = withScope(link.getAttribute('href'), scope);
  }
};

// The range links are rendered as `?preset=…` alone so they work with no JS.
// Here they regain the rest of the current query, and the active one is marked.
const bindRanges = (scope) => {
  const active = scope.get('preset');
  for (const link of document.querySelectorAll('[data-scope-range]')) {
    const range = link.dataset.scopeRange;
    const url = new URL(window.location.href);
    url.searchParams.set('preset', range);
    url.searchParams.delete('page');
    link.href = `${url.pathname}${url.search}`;
    link.classList.toggle('sp-scope-filter__range--active', range === active);
    if (range === active) {
      link.setAttribute('aria-current', 'true');
    } else {
      link.removeAttribute('aria-current');
    }
  }
};

// Why: changing a scope select submits immediately. An Apply button people
// must find after choosing is the single most reported friction on filter
// bars, and the button stays for the no-JS path.
const bindSelects = (form) => {
  for (const select of form.querySelectorAll('[data-scope-key]')) {
    select.addEventListener('change', () => form.submit());
  }
};

export const initScope = () => {
  const scope = currentScope();
  carryScopeIntoNav(scope);
  const form = document.querySelector('[data-scope-selector]');
  if (!form) return;
  bindRanges(scope);
  bindSelects(form);
};
