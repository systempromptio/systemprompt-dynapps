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
  path: string;
  /** Its page has no route yet; nothing may assert it resolves. */
  mounted: boolean;
}

export interface NavSection {
  heading: string;
  items: NavItem[];
}

const item = (label: string, path: string, mounted = true): NavItem => ({ label, path, mounted });

export const NAV: NavSection[] = [
  {
    heading: 'Overview',
    items: [item('Overview', PATHS.overview)],
  },
  {
    heading: 'People & access',
    items: [
      item('Users', PATHS.users),
      item('Groups', PATHS.groups),
      item('Projects', PATHS.projects),
      item('Roles & permissions', UNMOUNTED_PATHS.roles, false),
      item('Access control', PATHS.accessControl),
      item('Devices & enrolments', UNMOUNTED_PATHS.devices, false),
    ],
  },
  {
    heading: 'AI activity',
    items: [
      item('Analytics', PATHS.analytics),
      item('Requests', PATHS.requests),
      item('Sessions', PATHS.sessions),
      item('Traces', PATHS.traces),
      item('Contexts', PATHS.contexts),
    ],
  },
  {
    heading: 'Governance',
    items: [
      item('Governance', PATHS.governance),
      item('Approvals', UNMOUNTED_PATHS.approvals, false),
      item('Secrets audit', UNMOUNTED_PATHS.secrets, false),
    ],
  },
  {
    heading: 'Platform',
    items: [
      item('MCP servers', PATHS.mcp),
      item('Marketplaces', PATHS.marketplaces),
      item('Plugins', PATHS.plugins),
      item('Skills', PATHS.skills),
      item('Gateway', UNMOUNTED_PATHS.gateway, false),
    ],
  },
  {
    heading: 'Account',
    items: [
      item('Profile', PATHS.profile),
      item('My conversations', PATHS.history),
      item('Settings', PATHS.settings),
    ],
  },
];

/** Every nav item whose page is routed today. */
export const MOUNTED_NAV_ITEMS: NavItem[] = NAV.flatMap((s) => s.items).filter((i) => i.mounted);
