// The single place the suite names a class from the admin design system.
//
// Page objects and specs never hard-code a class: they go through this map, so
// a rename in the design system is one edit here rather than a sweep through
// every spec. Keys are the component vocabulary; values are what the markup
// actually ships.
export const SEL = {
  shell: '.sp-shell',
  nav: '.sp-nav',
  navLink: '.sp-nav__link',
  navLinkActive: '.sp-nav__link.active',
  pageHeader: '.sp-page-header',
  // A selector list, not a single class: the design-system header is preferred,
  // and the bare h1 keeps the harness usable on pages that have not been
  // rebuilt onto it yet.
  heading: '.sp-page-header h1, main h1, h1',
  breadcrumb: '.sp-breadcrumb',
  breadcrumbItem: '.sp-breadcrumb li',
  toolbar: '.sp-toolbar',
  table: '.sp-table',
  tableHead: '.sp-table thead',
  tableHeaderCell: '.sp-table thead th',
  tableSort: '.sp-table thead th [data-sort]',
  tableRow: '.sp-table tbody tr',
  pagination: '.sp-pagination',
  paginationNext: '.sp-pagination [data-page="next"]',
  paginationPrev: '.sp-pagination [data-page="prev"]',
  kpi: '.sp-kpi',
  kpiLabel: '.sp-kpi__label',
  kpiValue: '.sp-kpi__value',
  badge: '.sp-badge',
  empty: '.sp-empty',
  tabs: '.sp-tabs',
  tab: '.sp-tabs [role="tab"]',
  dialog: '.sp-dialog',
  dialogConfirm: '.sp-dialog [data-action="confirm"]',
  dialogCancel: '.sp-dialog [data-action="cancel"]',
  toast: '.sp-toast',
  filterInput: '[data-filter]',
  csvLink: '[data-export="csv"]',
  scopeSelect: '[data-scope]',
  rangeSelect: '[data-range]',
} as const;

/** The token prefix every computed-style assertion checks for. */
export const TOKEN_PREFIX = '--sp-';
