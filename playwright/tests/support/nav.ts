// The sidebar's information architecture: six sections, in render order, each
// with its items as they are labelled and where they go.
//
// A page spec asserts its own nav item is the active one; this map is what lets
// it do that by label rather than by a hard-coded href, and what lets a shell
// spec prove the whole structure in one pass.
//
// Active state is carried twice and in lockstep: the handlebars helper emits
// class="is-active" aria-current="page" from the page id, and services/sidebar.js
// then re-marks the link whose href pathname equals the location exactly,
// clearing every other — setting and removing both together. Either is a safe
// assertion; SEL.navLinkActive uses the ARIA form because it is also what a
// screen reader is told. The JS correction exists because /admin and
// /admin/analytics render the same page id.
import { PATHS, UNMOUNTED_PATHS } from './paths';

export interface NavItem {
  label: string;
  /** The key naming this route in PATHS, or in UNMOUNTED_PATHS until it lands. */
  key: string;
  path: string;
  /** Its page has a route today; false while its author has not landed it. */
  mounted: boolean;
}

export interface NavSection {
  heading: string;
  items: NavItem[];
}

type RouteTable = Record<string, unknown>;

// Why look the route up rather than name the table: an item is promoted by its
// author moving the key from UNMOUNTED_PATHS to PATHS, and a map that named the
// table by hand went red the moment gateway was promoted. Asking where the key
// lives makes promotion a no-op here and keeps `mounted` true by construction.
function item(label: string, key: string): NavItem {
  const mounted = (PATHS as RouteTable)[key];
  if (typeof mounted === 'string') return { label, key, path: mounted, mounted: true };
  const pending = (UNMOUNTED_PATHS as RouteTable)[key];
  if (typeof pending === 'string') return { label, key, path: pending, mounted: false };
  throw new Error(
    `nav item "${label}" names route "${key}", which is in neither PATHS nor ` +
      `UNMOUNTED_PATHS. Add it to one, or fix the key.`,
  );
}

export const NAV: NavSection[] = [
  {
    heading: 'Overview',
    items: [item('Overview', 'overview')],
  },
  {
    heading: 'People & access',
    items: [
      item('Users', 'users'),
      item('Groups', 'groups'),
      item('Projects', 'projects'),
      item('Roles & permissions', 'roles'),
      item('Access control', 'accessControl'),
      item('Devices & enrolments', 'devices'),
    ],
  },
  {
    heading: 'AI activity',
    items: [
      item('Analytics', 'analytics'),
      item('Requests', 'requests'),
      item('Sessions', 'sessions'),
      item('Traces', 'traces'),
      item('Contexts', 'contexts'),
    ],
  },
  {
    heading: 'Governance',
    items: [
      item('Governance', 'governance'),
      item('Approvals', 'approvals'),
      item('Secrets audit', 'secrets'),
    ],
  },
  {
    heading: 'Platform',
    items: [
      item('MCP servers', 'mcp'),
      item('Marketplaces', 'marketplaces'),
      item('Plugins', 'plugins'),
      item('Skills', 'skills'),
      item('Gateway', 'gateway'),
    ],
  },
  {
    heading: 'Account',
    items: [
      item('Profile', 'profile'),
      item('My conversations', 'history'),
      item('Settings', 'settings'),
    ],
  },
];

/** Every nav item whose page is routed today. */
export const MOUNTED_NAV_ITEMS: NavItem[] = NAV.flatMap((s) => s.items).filter((i) => i.mounted);
