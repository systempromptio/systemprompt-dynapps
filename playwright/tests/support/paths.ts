// The admin route map, in one place.
//
// Specs and page objects never write a literal admin URL: they name a route
// here. The console's information architecture is being flattened (agent A4
// owns the mapping), and a rename must be one edit in this file rather than a
// sweep through every spec and every demo chapter.
export const PATHS = {
  overview: '/admin',

  users: '/admin/users',
  user: (id: string) => `/admin/users/${id}`,
  groups: '/admin/groups',
  group: (id: string) => `/admin/groups/${id}`,
  projects: '/admin/projects',
  project: (id: string) => `/admin/projects/${id}`,
  unassigned: '/admin/unassigned',
  invites: '/admin/invites',

  requests: '/admin/requests',
  request: (id: string) => `/admin/requests/${id}`,
  sessions: '/admin/sessions',
  session: (id: string) => `/admin/sessions/${id}`,
  skills: '/admin/skills',
  tools: '/admin/tools',
  costs: '/admin/costs',

  governance: '/admin/governance',
  safety: '/admin/safety',
  approvals: '/admin/approvals',
  anomalies: '/admin/anomalies',
  access: '/admin/access',

  marketplace: '/admin/marketplace',
  plugins: '/admin/plugins',
  mcp: '/admin/mcp',
  devices: '/admin/devices',
  bridges: '/admin/bridges',
  config: '/admin/config',

  profile: '/admin/profile',
  keys: '/admin/keys',
  login: '/admin/login',
} as const;
