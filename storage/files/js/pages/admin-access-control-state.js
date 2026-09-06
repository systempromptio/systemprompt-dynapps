const EMPTY_CATALOGUE = {
  gateway_routes: [],
  mcp_servers: [],
  plugins: [],
  agents: [],
  marketplaces: [],
  skills: [],
};

export const UNASSIGNED_GROUP = 'unassigned';

export const DB_SOURCE_LABEL = 'database (this instance)';

let catalogue = EMPTY_CATALOGUE;
let groups = [];
let projects = [];
let canWrite = false;

const readEmbedded = (key, fallback) => {
  const el = document.querySelector(`script[data-ac="${key}"]`);
  if (!el) return fallback;
  try {
    return JSON.parse(el.textContent) ?? fallback;
  } catch {
    return fallback;
  }
};

export const loadState = () => {
  catalogue = readEmbedded('entity-catalogue', EMPTY_CATALOGUE);
  groups = readEmbedded('groups', []);
  projects = readEmbedded('projects', []);
  canWrite = readEmbedded('can-write', false) === true;
};

export const entitySections = () => [
  { type: 'marketplace', label: 'Marketplaces', items: catalogue.marketplaces ?? [] },
  { type: 'plugin', label: 'Plugins', items: catalogue.plugins ?? [] },
  { type: 'skill', label: 'Skills', items: catalogue.skills ?? [] },
  { type: 'mcp_server', label: 'MCP servers', items: catalogue.mcp_servers ?? [] },
  { type: 'gateway_route', label: 'Gateway routes', items: catalogue.gateway_routes ?? [] },
  { type: 'agent', label: 'Agents', items: catalogue.agents ?? [] },
];

export const groupList = () => groups;

export const projectList = () => projects;

export const isWritable = () => canWrite;

export const cloneTemplate = (name) =>
  document.querySelector(`template[data-tpl="${name}"]`).content.firstElementChild.cloneNode(true);

export const slot = (root, name) => root.querySelector(`[data-slot="${name}"]`);

export const setText = (root, name, value) => {
  const el = slot(root, name);
  if (el) el.textContent = value == null ? '' : String(value);
};

export const showPane = (pane) => {
  for (const el of document.querySelectorAll('[data-pane]')) {
    el.hidden = el.dataset.pane !== pane;
  }
};
