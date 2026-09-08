// Live-reference resolution for generation manifests.
//
// A generator manifest points at REAL project assets (a command, skills,
// a schema artifact) instead of hand-written prose. This module loads those
// assets and neutralizes the live references they contain (project-context MCP,
// Jira, the openspec CLI, interactive tools) via evalsuite/<schema>/mappings.yaml.
//
// Substitutions are NOT surgical rewrites: for each matched mapping we attach a
// note to the asset's block so the model treats the live dependency as an
// eval-safe fixture/stub. See the opsx-eval skill, "Architecture".

import { readFileSync, existsSync, readdirSync, statSync, mkdtempSync, cpSync, rmSync, realpathSync } from 'node:fs';
import { resolve, join } from 'node:path';
import { tmpdir } from 'node:os';
import { execFileSync } from 'node:child_process';
import { parse } from 'yaml';

/** Load the mapping registry for a schema. Missing file => no mappings. */
export function loadMappings(config, schema) {
  const path = join(config.paths.suite, schema, 'mappings.yaml');
  if (!existsSync(path)) return [];
  try {
    const raw = parse(readFileSync(path, 'utf8')) ?? {};
    return Array.isArray(raw.references) ? raw.references : [];
  } catch {
    return [];
  }
}

/** Scan text for each mapping's `match` substrings; return the matched notes. */
export function applyMappings(text, mappings) {
  const substitutions = [];
  const body = String(text ?? '');
  for (const m of mappings) {
    const patterns = Array.isArray(m.match) ? m.match : [];
    if (patterns.some((p) => body.includes(p))) {
      substitutions.push({
        id: m.id,
        kind: m.redirect_context ? 'redirect' : 'stub',
        note: m.note ?? m.stub ?? '',
        redirectContext: m.redirect_context ?? null,
      });
    }
  }
  return { substitutions };
}

/** Load a source file and apply mappings to its content. baseDir-relative or absolute. */
export function loadMappedSource(ref, baseDir, mappings) {
  const path = resolve(baseDir, ref);
  const exists = existsSync(path) && statSync(path).isFile();
  const content = exists ? readFileSync(path, 'utf8') : null;
  const { substitutions } = exists ? applyMappings(content, mappings) : { substitutions: [] };
  return { ref, path, exists, content, substitutions };
}

/** Recursively find `<name>/SKILL.md` under skillsRoot; null if not found. */
function findSkillByName(skillsRoot, name) {
  if (!existsSync(skillsRoot)) return null;
  const stack = [skillsRoot];
  while (stack.length) {
    const dir = stack.pop();
    for (const entry of readdirSync(dir)) {
      const p = join(dir, entry);
      if (!statSync(p).isDirectory()) continue;
      if (entry === name && existsSync(join(p, 'SKILL.md'))) return join(p, 'SKILL.md');
      stack.push(p);
    }
  }
  return null;
}

/**
 * Resolve a `skills:` entry (a bare skill name, or a path relative to baseDir)
 * to a mapped source block.
 */
export function resolveSkill(entry, { baseDir, skillsRoot, mappings }) {
  const looksLikePath = entry.includes('/') || entry.endsWith('.md');
  if (looksLikePath) return loadMappedSource(entry, baseDir, mappings);

  const path = findSkillByName(skillsRoot, entry);
  if (!path) return { ref: entry, path: null, exists: false, content: null, substitutions: [] };
  const content = readFileSync(path, 'utf8');
  const { substitutions } = applyMappings(content, mappings);
  return { ref: entry, path, exists: true, content, substitutions };
}

/**
 * Load the OpenSpec instructions for a `schema_artifact:` ref of the form
 * "<schema>/<artifact>" straight from the real `openspec` CLI, so the generation
 * prompt sees exactly what an agent gets from `openspec instructions <artifact>`
 * (no re-implementation to drift out of sync).
 *
 * The CLI needs a scaffolded change inside a real `openspec/` root, which does
 * not exist in this source repo, so we make a throwaway temp root: copy the
 * templates `openspec/` into it, `openspec new change <name>`, then
 * `openspec instructions <artifact> --change <name> --schema <schema>`. The temp
 * absolute prefix is stripped from the rendered text so paths read
 * `openspec/changes/<name>/...`, and `mappings.yaml` is applied to the result so
 * live references (project-context, jira, the CLI itself, interactive tools) are
 * neutralized and reported, same as every other block.
 *
 * This hard-depends on the `openspec` CLI being on PATH; if it is missing or
 * fails, the returned block is marked `exists: false` with a clear error/hint,
 * which surfaces in the manifest sourcing report.
 */
export function loadOpenSpecInstructions(ref, { openspecRoot, mappings, changeName = 'change' }) {
  if (!ref) return null;
  const [schemaName, artifactId] = String(ref).split('/');
  if (!schemaName || !artifactId) {
    return { ref, exists: false, error: `expected "<schema>/<artifact>", got "${ref}"` };
  }

  const tmp = mkdtempSync(join(tmpdir(), 'opsx-instr-'));
  const root = join(tmp, 'openspec');
  try {
    cpSync(openspecRoot, root, { recursive: true });
    execFileSync('openspec', ['new', 'change', changeName, '--schema', schemaName, '--json'], {
      cwd: tmp,
      stdio: 'ignore',
    });
    const rawText = execFileSync(
      'openspec',
      ['instructions', artifactId, '--change', changeName, '--schema', schemaName],
      { cwd: tmp, encoding: 'utf8' },
    );
    // The CLI prints absolute paths (resolved through symlinks); strip the temp
    // prefix so the prompt shows clean repo-relative `openspec/changes/...` paths.
    const realTmp = realpathSync(tmp);
    const rendered = rawText.split(`${realTmp}/`).join('').split(`${tmp}/`).join('').trimEnd();
    const { substitutions } = applyMappings(rendered, mappings);
    return { ref, exists: true, schemaName, artifactId, changeName, rendered, substitutions };
  } catch (err) {
    const missing = /ENOENT/.test(String(err?.message)) || /not found/i.test(String(err?.message));
    const hint = missing
      ? ' — is the `openspec` CLI on PATH? Install it: npm i -g @fission-ai/openspec'
      : '';
    return { ref, exists: false, schemaName, artifactId, error: `openspec CLI failed: ${err?.message}${hint}` };
  } finally {
    rmSync(tmp, { recursive: true, force: true });
  }
}

/** Flatten all substitutions across resolved blocks into a de-duplicated report. */
export function collectSubstitutions(blocks) {
  const seen = new Map();
  for (const b of blocks) {
    for (const s of b?.substitutions ?? []) {
      if (!seen.has(s.id)) seen.set(s.id, s);
    }
  }
  return [...seen.values()];
}
