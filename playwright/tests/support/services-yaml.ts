// Just enough YAML to read the `include:` lists the marketplace and plugin
// descriptors carry, so the expected-entitlements table can be checked against
// services/ without a YAML dependency the suite has no other use for.
//
// Marketplaces that reach the instance as a fetched services bundle rather
// than from services/ (the Europe development workspace, published by
// Astound-Digital/sfnext-claude-dev-kit) are declared under `_bundled` in
// setup/seed/entitlements.json and merged in here, so the table can still be
// derived and the manifest test still proves the served set.
//
// It understands one shape and refuses everything else loudly: a mapping key
// at a known indent, followed by indented or YAML's indentless `- item` lines.
import { readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';

export const SERVICES = join(__dirname, '..', '..', '..', 'services');

interface BundledDeclarations {
  marketplaces?: Record<string, { plugins: string[]; mcp_servers: string[] }>;
  plugins?: Record<string, { skills: string[]; mcp_servers: string[] }>;
}

function bundled(): BundledDeclarations {
  const table = JSON.parse(
    readFileSync(join(__dirname, '..', '..', 'setup', 'seed', 'entitlements.json'), 'utf8'),
  ) as { _bundled?: BundledDeclarations };
  return table._bundled ?? {};
}

function stripComment(line: string): string {
  const hash = line.indexOf('#');
  return (hash >= 0 ? line.slice(0, hash) : line).trimEnd();
}

/** The `- item` list under `<section>:\n  include:` in a descriptor. */
export function includeList(file: string, section: string): string[] {
  const lines = readFileSync(file, 'utf8').split('\n').map(stripComment);
  const sectionAt = lines.findIndex((l) => l.trim() === `${section}:`);
  if (sectionAt < 0) throw new Error(`${file}: no '${section}:' section`);
  const sectionIndent = lines[sectionAt].search(/\S/);
  let includeAt = -1;
  for (let i = sectionAt + 1; i < lines.length; i += 1) {
    const l = lines[i];
    if (l.trim() === '') continue;
    const indent = l.search(/\S/);
    if (indent <= sectionIndent) break;
    if (l.trim() === 'include:' || l.trim() === 'include: []') {
      includeAt = i;
      break;
    }
  }
  if (includeAt < 0) throw new Error(`${file}: '${section}:' has no include list`);
  if (lines[includeAt].trim() === 'include: []') return [];
  const listIndent = lines[includeAt].search(/\S/);
  const items: string[] = [];
  for (let i = includeAt + 1; i < lines.length; i += 1) {
    const l = lines[i];
    if (l.trim() === '') continue;
    const indent = l.search(/\S/);
    const m = l.trim().match(/^-\s+(\S+)$/);
    if (indent < listIndent || (indent === listIndent && !m)) break;
    if (!m) throw new Error(`${file}:${i + 1}: expected a '- item' line, got '${l.trim()}'`);
    items.push(m[1]);
  }
  return items;
}

function descriptors(dir: string): Map<string, string> {
  const root = join(SERVICES, dir);
  return new Map(readdirSync(root).map((id) => [id, join(root, id, 'config.yaml')]));
}

/** id -> {plugins, mcp_servers} for every marketplace under services/. */
export function marketplaceIncludes(): Map<string, { plugins: string[]; mcpServers: string[] }> {
  const fromTree = new Map(
    [...descriptors('marketplaces')].map(([id, file]) => [
      id,
      { plugins: includeList(file, 'plugins'), mcpServers: includeList(file, 'mcp_servers') },
    ]),
  );
  for (const [id, m] of Object.entries(bundled().marketplaces ?? {})) {
    if (fromTree.has(id)) throw new Error(`${id} is both bundled and under services/`);
    fromTree.set(id, { plugins: m.plugins, mcpServers: m.mcp_servers });
  }
  return fromTree;
}

/** id -> {skills, mcp_servers} for every plugin under services/. */
export function pluginIncludes(): Map<string, { skills: string[]; mcpServers: string[] }> {
  const fromTree = new Map(
    [...descriptors('plugins')].map(([id, file]) => [
      id,
      { skills: includeList(file, 'skills'), mcpServers: includeList(file, 'mcp_servers') },
    ]),
  );
  for (const [id, p] of Object.entries(bundled().plugins ?? {})) {
    if (fromTree.has(id)) throw new Error(`${id} is both bundled and under services/`);
    fromTree.set(id, { skills: p.skills, mcpServers: p.mcp_servers });
  }
  return fromTree;
}

export interface DeclaredAccessRule {
  ruleType: string;
  values: string[];
  access: 'allow' | 'deny';
}

export interface DeclaredAccess {
  defaultIncluded: boolean;
  roles: string[];
  rules: DeclaredAccessRule[];
}

// `roles: [a, b]` / `values: [a]` / `roles: []` — an inline flow list.
function flowList(file: string, lineNo: number, value: string): string[] {
  const m = value.trim().match(/^\[(.*)\]$/);
  if (!m) throw new Error(`${file}:${lineNo}: expected an inline '[a, b]' list, got '${value.trim()}'`);
  return m[1]
    .split(',')
    .map((s) => s.trim())
    .filter((s) => s.length > 0);
}

// Why: serde_yaml accepts a flow list and a block sequence interchangeably, so
// a descriptor the server loads happily must not be rejected here. A block
// sequence may sit at the key's own indent, which is legal YAML, so items are
// taken at `>=` the key indent and only when they start with `- `; a sibling
// key (`access:`) or the outer `rules:` entries (shallower) end the run.
// Returns the last line consumed so the caller can skip past the items, which
// otherwise fail the `key: value` match.
function listAt(
  file: string,
  lines: string[],
  idx: number,
  value: string,
  indent: number,
): { values: string[]; last: number } {
  if (value.trim() !== '') return { values: flowList(file, idx + 1, value), last: idx };
  const values: string[] = [];
  let last = idx;
  for (let i = idx + 1; i < lines.length; i += 1) {
    const l = lines[i];
    if (l.trim() === '') continue;
    if (l.search(/\S/) < indent) break;
    const m = l.trim().match(/^-\s+(.*)$/);
    if (!m) break;
    values.push(m[1].trim());
    last = i;
  }
  return { values, last };
}

/**
 * The `access:` block of one marketplace descriptor: what core's
 * `ingest_marketplace_access` projects into access_control_rules at boot —
 * one `role` row per entry in `roles`, and one row per value of every entry
 * in `rules`, in the subject dimension that entry names.
 */
export function accessBlock(file: string): DeclaredAccess {
  const lines = readFileSync(file, 'utf8').split('\n').map(stripComment);
  const accessAt = lines.findIndex((l) => l.trim() === 'access:');
  if (accessAt < 0) throw new Error(`${file}: no 'access:' block`);
  const accessIndent = lines[accessAt].search(/\S/);
  const block: DeclaredAccess = { defaultIncluded: true, roles: [], rules: [] };
  let rule: DeclaredAccessRule | null = null;
  let ruleIndent = -1;
  for (let i = accessAt + 1; i < lines.length; i += 1) {
    const l = lines[i];
    if (l.trim() === '') continue;
    const indent = l.search(/\S/);
    if (indent <= accessIndent) break;
    const text = l.trim();
    const kv = text.match(/^(-\s+)?([a-z_]+):\s*(.*)$/);
    if (!kv) throw new Error(`${file}:${i + 1}: unexpected line in access block: '${text}'`);
    const [, dash, key, value] = kv;
    if (dash) {
      rule = { ruleType: '', values: [], access: 'allow' };
      ruleIndent = indent;
      block.rules.push(rule);
    } else if (rule && indent <= ruleIndent) {
      rule = null;
    }
    if (rule) {
      if (key === 'rule_type') rule.ruleType = value;
      else if (key === 'values') {
        const list = listAt(file, lines, i, value, indent);
        rule.values = list.values;
        i = list.last;
      }
      else if (key === 'access') {
        if (value !== 'allow' && value !== 'deny') throw new Error(`${file}:${i + 1}: access must be allow|deny`);
        rule.access = value;
      } else if (key !== 'justification') throw new Error(`${file}:${i + 1}: unknown rule key '${key}'`);
      continue;
    }
    if (key === 'default_included') block.defaultIncluded = value === 'true';
    else if (key === 'roles') {
      const list = listAt(file, lines, i, value, indent);
      block.roles = list.values;
      i = list.last;
    }
    else if (key === 'rules') {
      if (value !== '' && value !== '[]') throw new Error(`${file}:${i + 1}: 'rules:' must be a block sequence or []`);
    } else if (key !== 'justification') throw new Error(`${file}:${i + 1}: unknown access key '${key}'`);
  }
  for (const r of block.rules) {
    if (!r.ruleType || r.values.length === 0) throw new Error(`${file}: an access rule lacks rule_type or values`);
  }
  return block;
}

/** id -> declared access block for every marketplace under services/. */
export function marketplaceAccess(): Map<string, DeclaredAccess> {
  return new Map([...descriptors('marketplaces')].map(([id, file]) => [id, accessBlock(file)]));
}
