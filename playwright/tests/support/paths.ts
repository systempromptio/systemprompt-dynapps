// The admin route map, in one place.
//
// Specs and page objects never write a literal admin URL: they name a route
// here, so a move is one edit rather than a sweep. The values below are agent
// A4's flat table; the old nested paths still answer with a 308 for one
// release, which is why the pre-existing specs continue to pass unchanged.
export const PATHS = {
  // /admin renders the overview itself for a console user, and redirects a
  // non-console user to their profile — it is no longer a hop to /admin/analytics.
  overview: '/admin',
  analytics: '/admin/analytics',

  users: '/admin/users',
  user: (id: string) => `/admin/users/${id}`,
  userUsage: (id: string) => `/admin/users/${id}?tab=usage`,
  // User detail also answers on the query form, which some older specs use.
  userByQuery: (id: string) => `/admin/user?id=${id}`,
  // Unassigned is a filter on the roster now, not a page of its own.
  unassigned: '/admin/users?filter=unassigned',
  groups: '/admin/groups',
  group: (id: string) => `/admin/groups/${id}`,
  projects: '/admin/projects',
  project: (id: string) => `/admin/projects/${id}`,
  accessControl: '/admin/access-control',
  roles: '/admin/roles',
  devices: '/admin/devices',
  devicesTab: (tab: string) => `/admin/devices?tab=${tab}`,

  requests: '/admin/requests',
  request: (id: string) => `/admin/requests/${id}`,
  sessions: '/admin/sessions',
  session: (id: string) => `/admin/sessions/${id}`,
  traces: '/admin/traces',
  trace: (id: string) => `/admin/traces/${id}`,
  contexts: '/admin/contexts',
  context: (id: string) => `/admin/contexts/${id}`,
  history: '/admin/history',
  conversation: (id: string) => `/admin/history/conversations/${id}`,

  // Skill and tool ACTIVITY are analytics tabs; the catalog entries below are
  // the definitions. They are different pages and the names keep them apart.
  skillActivity: '/admin/analytics?tab=skills',
  toolActivity: '/admin/analytics?tab=tools',
  costs: '/admin/analytics?tab=cost',

  governance: '/admin/governance',
  governanceDecisions: '/admin/governance?tab=decisions',
  governanceSafety: '/admin/governance?tab=safety',
  governanceHooks: '/admin/governance?tab=hooks',
  governanceDecision: (id: string) => `/admin/governance/decisions/${id}`,
  approvals: '/admin/governance/approvals',
  secretsAudit: '/admin/governance/secrets',

  marketplaces: '/admin/marketplaces',
  marketplace: (id: string) => `/admin/marketplaces/${id}`,
  plugins: '/admin/plugins',
  plugin: (id: string) => `/admin/plugins/${id}`,
  skills: '/admin/skills',
  skill: (id: string) => `/admin/skills/${id}`,
  mcp: '/admin/mcp',
  mcpServer: (id: string) => `/admin/mcp/${id}`,
  gateway: '/admin/gateway',

  settings: '/admin/settings',
  setup: '/admin/setup',
  profile: '/admin/profile',
  login: '/admin/login',
} as const;

// In the sidebar but not mounted yet: their page authors own them, and they
// 404 until those land. Nothing may assert on one before its author says so.
export const UNMOUNTED_PATHS = {} as const;
