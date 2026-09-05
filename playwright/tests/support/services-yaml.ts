// Just enough YAML to read the `include:` lists the marketplace and plugin
// descriptors carry, so the expected-entitlements table can be checked against
// services/ without a YAML dependency the suite has no other use for.
//
// It understands one shape and refuses everything else loudly: a mapping key
// at a known indent, followed by `- item` lines at a deeper indent.
import { readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';

export const SERVICES = join(__dirname, '..', '..', '..', 'services');

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
    if (indent <= listIndent) break;
    const m = l.trim().match(/^-\s+(\S+)$/);
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
  return new Map(
    [...descriptors('marketplaces')].map(([id, file]) => [
      id,
      { plugins: includeList(file, 'plugins'), mcpServers: includeList(file, 'mcp_servers') },
    ]),
  );
}

/** id -> {skills, mcp_servers} for every plugin under services/. */
export function pluginIncludes(): Map<string, { skills: string[]; mcpServers: string[] }> {
  return new Map(
    [...descriptors('plugins')].map(([id, file]) => [
      id,
      { skills: includeList(file, 'skills'), mcpServers: includeList(file, 'mcp_servers') },
    ]),
  );
}
