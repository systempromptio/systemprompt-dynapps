// Resolution for the evalsuite layout.
//
// Layout (no mapping registry — paths are composed by convention):
//   <suite>/<schema>/judge/<artifact>.md        judge rubric (contract + rubric + declared context)
//   <suite>/<schema>/judge/reasoning.md         judge routing-pass procedure
//   <suite>/<schema>/generator/<artifact>.md    generation manifest + task
//   <suite>/<schema>/generator/contract.md      generation harness contract
//   <suite>/<schema>/generator/reasoning.md     generation planning-pass procedure
//   <suite>/<schema>/fixtures/<case>/           golden case in OpenSpec change-folder convention
//
// A judge rubric's frontmatter declares `context:` — the grounding it needs,
// resolved from the case folder (discovery -> .context/initial-context; fsd -> discovery+specs).
// The same declaration drives generation inputs. Both judge rubric and generator
// may declare `sources:` (rules/templates inlined verbatim), relative to the file.
//
// Fixture states: an artifact may be stored as a directory of states rather than
// one file (`discovery/initial.ba.md` + `discovery/final.ba.md`), because a single
// file cannot hold both the draft as first emitted and the artifact as it settles
// after the review gate. A golden resolves to the draft — that is what generation
// must produce — and a context resolves to the settled state, which is what a
// downstream artifact actually receives. State names come from config (`states:`).

import { readFileSync, existsSync, readdirSync, statSync } from 'node:fs';
import { resolve, dirname, join, relative, isAbsolute } from 'node:path';
import { parse } from 'yaml';
import {
  loadMappings,
  loadMappedSource,
  resolveSkill,
  loadOpenSpecInstructions,
  collectSubstitutions,
} from './references.mjs';

/** Split a markdown file into { frontmatter, body }. Frontmatter is optional. */
export function splitFrontmatter(text) {
  if (!text.startsWith('---')) return { frontmatter: {}, body: text };
  const end = text.indexOf('\n---', 3);
  if (end === -1) return { frontmatter: {}, body: text };
  const raw = text.slice(3, end).replace(/^\n/, '');
  const rest = text.slice(end + 4).replace(/^\r?\n/, '');
  let frontmatter = {};
  try {
    frontmatter = parse(raw) ?? {};
  } catch {
    frontmatter = {};
  }
  return { frontmatter, body: rest };
}

function readDoc(path) {
  const { frontmatter, body } = splitFrontmatter(readFileSync(path, 'utf8'));
  return { path, frontmatter, body };
}

/** List `<artifact>` names under <suite>/<schema>/<kind> for helpful errors. */
function listArtifacts(suite, schema, kind) {
  const dir = join(suite, schema, kind);
  if (!existsSync(dir)) return [];
  const reserved = new Set(['contract.md', 'reasoning.md']);
  return readdirSync(dir)
    .filter((f) => f.endsWith('.md') && !reserved.has(f))
    .map((f) => f.replace(/\.md$/, ''));
}

/** Inline declared sources (rules/templates), paths relative to baseDir. */
function inlineSources(baseDir, refs) {
  const list = Array.isArray(refs) ? refs : [];
  return list.map((ref) => {
    const abs = resolve(baseDir, ref);
    const exists = existsSync(abs) && statSync(abs).isFile();
    return { ref, path: abs, exists, content: exists ? readFileSync(abs, 'utf8') : null };
  });
}

/** Recursively collect .md files under an absolute path (file or dir). */
function collectMarkdown(absPath) {
  if (!existsSync(absPath)) return { found: false, kind: 'missing', files: [] };
  const st = statSync(absPath);
  if (st.isFile()) {
    return { found: true, kind: 'file', files: [{ rel: absPath, content: readFileSync(absPath, 'utf8') }] };
  }
  const files = [];
  const walk = (dir) => {
    for (const entry of readdirSync(dir).sort()) {
      const p = join(dir, entry);
      const s = statSync(p);
      if (s.isDirectory()) walk(p);
      // `baseline.md` files are baseline capability specs consumed only via an
      // explicit context id (e.g. `specs/<cap>/baseline`); exclude them from
      // generic directory globs so they do not leak into delta-spec context.
      else if (entry.endsWith('.md') && entry !== 'baseline.md')
        files.push({ rel: relative(absPath, p), content: readFileSync(p, 'utf8') });
    }
  };
  walk(absPath);
  return { found: files.length > 0, kind: 'dir', files };
}

export const DEFAULT_STATES = {
  draft: 'initial',
  settled: 'final',
  reviewSession: '.context/review-session.{artifact}.{flow}.md',
};

/**
 * True when `<caseDir>/<id>` is a directory of artifact STATES — it holds at
 * least one `initial*.md` / `final*.md`. Bundle directories (a specs delta, an
 * fsd folder) hold neither and are left alone, so they keep resolving as globs.
 */
function isStateDir(caseDir, id, states) {
  const dir = join(caseDir, id);
  if (!existsSync(dir) || !statSync(dir).isDirectory()) return false;
  const known = new Set([states.draft, states.settled]);
  return readdirSync(dir).some((f) => f.endsWith('.md') && known.has(f.split('.')[0]));
}

/**
 * Expand an id into the ordered state ids to try.
 *
 * `want` is 'draft' for a golden (what this run must produce) and 'settled' for a
 * context (what this run receives). A bare id that names a state directory expands
 * to the wanted state, and the draft falls back to the settled state so a flow
 * without a review session ships one file instead of two identical ones.
 *
 * An id that already names a state (`discovery/initial`) is STRICT — no fallback.
 * A composition that asks for the draft is asking for a document that genuinely
 * differs from the settled one; silently handing it the settled state would turn
 * a review-gate trap into a no-op that still passes.
 */
function stateCandidates(caseDir, id, states, want) {
  const known = [states.draft, states.settled];
  if (known.includes(id.split('/').pop())) return [id];
  if (!isStateDir(caseDir, id, states)) return [id];
  return want === 'draft'
    ? [`${id}/${states.draft}`, `${id}/${states.settled}`]
    : [`${id}/${states.settled}`];
}

/**
 * Refuse a flow that was changed by a review gate but ships no draft state.
 *
 * Review edits land inside table cells and prose, spread across most of the
 * artifact, so the draft cannot be recovered from the settled document by cutting
 * sections — it has to exist as its own file. Without it, the draft falls back to
 * the settled state and generation is judged against decisions only a human could
 * have made: every run fails, and for the wrong reason. The session file is the
 * one mechanical signal that the two states diverged, so this fails loudly at
 * resolution time rather than showing up later as an unexplained score drop.
 */
function assertDraftForReviewedFlow(caseDir, artifact, flow, states, flowsConfig) {
  if (!flow || !states?.reviewSession || !isStateDir(caseDir, artifact, states)) return;
  const session = join(caseDir, fillPattern(states.reviewSession, { artifact, flow }));
  if (!existsSync(session)) return;
  const draftId = `${artifact}/${states.draft}`;
  const draftName = flowsConfig
    ? fillPattern(flowsConfig.golden, { artifact: draftId, flow })
    : `${draftId}.${flow}.md`;
  if (existsSync(join(caseDir, draftName))) return;
  throw new Error(
    `Flow "${flow}" has a review session (${fillPattern(states.reviewSession, { artifact, flow })}) but no draft state (${draftName}).\n` +
      `A reviewed flow must ship both states: the draft is what generation is judged against, ` +
      `and the settled artifact carries decisions the generator could not have known.`,
  );
}

/**
 * Split markdown into a preamble plus its level-2 (`##`) sections, keeping the
 * text verbatim so a section can be moved between documents unchanged. Fenced
 * blocks are skipped so a `##` inside a fence never starts a section.
 */
function splitTopSections(md) {
  const preamble = [];
  const sections = [];
  let current = null;
  let inFence = false;
  for (const line of String(md ?? '').split(/\r?\n/)) {
    if (/^\s*(```|~~~)/.test(line)) inFence = !inFence;
    const heading = inFence ? null : /^##\s+(.+?)\s*#*\s*$/.exec(line);
    if (heading) {
      current = { title: heading[1].trim(), lines: [line] };
      sections.push(current);
      continue;
    }
    (current ? current.lines : preamble).push(line);
  }
  return { preamble, sections };
}

/**
 * Split one `##` section's lines into a lead (the `##` heading plus any text
 * before the first `###`) and its level-3 (`###`) subsections. Lets a composition
 * address a subsection — e.g. the Integration Specification's `### Overview` reader
 * map — the same way splitTopSections addresses a `##` section. Fence-aware, so a
 * `###` inside a code block never starts a subsection.
 */
function splitSubSections(sectionLines) {
  const lead = [];
  const subs = [];
  let current = null;
  let inFence = false;
  for (const line of sectionLines) {
    if (/^\s*(```|~~~)/.test(line)) inFence = !inFence;
    const heading = inFence ? null : /^###\s+(.+?)\s*#*\s*$/.exec(line);
    if (heading) {
      current = { title: heading[1].trim(), lines: [line] };
      subs.push(current);
      continue;
    }
    (current ? current.lines : lead).push(line);
  }
  return { lead, subs };
}

const normTitle = (s) => String(s ?? '').replace(/\s+/g, ' ').trim().toLowerCase();

/**
 * A section selector is either a top-level title (`Solution Overview`) or a nested
 * one addressing a `###` inside a `##`, written `Parent > Sub`
 * (`Integration Specification > Overview`). The ` > ` delimiter is used rather than
 * `/` because real titles carry `/` (e.g. `Risks / Mitigations`). A trailing `?`
 * marks the selector OPTIONAL: it resolves when the block is present and is skipped
 * (not an error) when it is absent, so one refresh manifest can serve flows that
 * differ in shape — the Integration Specification `### Overview` exists only in an
 * integration-flow design, so `Integration Specification > Overview?` reconciles it
 * there and is a no-op for a functional design.
 */
const parseSel = (t) => {
  let raw = String(t ?? '').trim();
  const optional = raw.endsWith('?');
  if (optional) raw = raw.slice(0, -1).trim();
  const at = raw.indexOf(' > ');
  return at === -1
    ? { parent: raw, sub: null, optional }
    : { parent: raw.slice(0, at).trim(), sub: raw.slice(at + 3).trim(), optional };
};

function findSection(sections, title, where) {
  const hit = sections.find((s) => normTitle(s.title) === normTitle(title));
  if (!hit) {
    throw new Error(
      `Composed fixture: section "## ${title}" not found in ${where}.\n` +
        `Sections present: ${sections.map((s) => s.title).join(', ') || '(none)'}`,
    );
  }
  return hit;
}

function findSub(subs, sub, parent, where) {
  const hit = subs.find((s) => normTitle(s.title) === normTitle(sub));
  if (!hit) {
    throw new Error(
      `Composed fixture: subsection "### ${sub}" not found under "## ${parent}" in ${where}.\n` +
        `Subsections present: ${subs.map((s) => s.title).join(', ') || '(none)'}`,
    );
  }
  return hit;
}

/** Join a section's parts back into a single trailing-newline-terminated document. */
const joinSections = (parts) =>
  parts
    .filter((part) => part.trim())
    .join('\n')
    .replace(/\s+$/, '')
    .concat('\n');

/**
 * Keep only the named sections/subsections, dropping the preamble and emitting what
 * remains in document order. A nested selector (`Parent > Sub`) contributes just
 * that `### Sub` block; an optional selector that is absent contributes nothing.
 */
function cutSections(md, titles, where) {
  const { sections } = splitTopSections(md);
  const out = [];
  for (const sel of titles.map(parseSel)) {
    const sec = sections.find((s) => normTitle(s.title) === normTitle(sel.parent));
    if (!sec) {
      if (sel.optional) continue;
      findSection(sections, sel.parent, where);
    }
    const pos = sections.indexOf(sec);
    if (sel.sub === null) {
      out.push({ pos, text: sec.lines.join('\n').replace(/\s+$/, '') });
      continue;
    }
    const { subs } = splitSubSections(sec.lines);
    const sub = subs.find((s) => normTitle(s.title) === normTitle(sel.sub));
    if (!sub) {
      if (sel.optional) continue;
      findSub(subs, sel.sub, sel.parent, where);
    }
    out.push({ pos, text: sub.lines.join('\n').replace(/\s+$/, '') });
  }
  out.sort((a, b) => a.pos - b.pos);
  return `${out.map((o) => o.text).join('\n\n')}\n`;
}

/**
 * Drop the named sections/subsections, keeping the preamble and everything else in
 * order. A nested selector removes only the `### Sub` block from within its parent;
 * an optional selector that is absent removes nothing.
 */
function dropSections(md, titles, where) {
  const { preamble, sections } = splitTopSections(md);
  const sels = titles.map(parseSel);
  const wholeDrop = new Set();
  for (const sel of sels.filter((s) => s.sub === null)) {
    const sec = sections.find((s) => normTitle(s.title) === normTitle(sel.parent));
    if (!sec) {
      if (sel.optional) continue;
      findSection(sections, sel.parent, where);
    }
    wholeDrop.add(normTitle(sec.title));
  }
  const parts = [preamble.join('\n')];
  for (const s of sections) {
    const tn = normTitle(s.title);
    if (wholeDrop.has(tn)) continue;
    const nested = sels.filter((sel) => sel.sub !== null && normTitle(sel.parent) === tn);
    if (nested.length) {
      const { lead, subs } = splitSubSections(s.lines);
      const cut = new Set();
      for (const sel of nested) {
        const sub = subs.find((x) => normTitle(x.title) === normTitle(sel.sub));
        if (!sub) {
          if (sel.optional) continue;
          findSub(subs, sel.sub, sel.parent, where);
        }
        cut.add(normTitle(sub.title));
      }
      if (cut.size) {
        s.lines = joinSections([lead.join('\n'), ...subs.filter((x) => !cut.has(normTitle(x.title))).map((x) => x.lines.join('\n'))])
          .replace(/\n$/, '')
          .split('\n');
      }
    }
    parts.push(s.lines.join('\n'));
  }
  for (const sel of sels.filter((s) => s.sub !== null && !s.optional)) {
    if (!sections.some((s) => normTitle(s.title) === normTitle(sel.parent))) findSection(sections, sel.parent, where);
  }
  return joinSections(parts);
}

/**
 * Replace the named sections/subsections of `md` with the same-named ones of
 * `donorMd`. A nested selector swaps only the `### Sub` block inside its parent; an
 * optional selector absent from either side is skipped.
 */
function spliceSections(md, donorMd, titles, where, donorWhere) {
  const target = splitTopSections(md);
  const donor = splitTopSections(donorMd);
  for (const sel of titles.map(parseSel)) {
    const into = target.sections.find((s) => normTitle(s.title) === normTitle(sel.parent));
    const from = donor.sections.find((s) => normTitle(s.title) === normTitle(sel.parent));
    if (!into || !from) {
      if (sel.optional) continue;
      findSection(target.sections, sel.parent, where);
      findSection(donor.sections, sel.parent, donorWhere);
    }
    if (sel.sub === null) {
      into.lines = from.lines;
      continue;
    }
    const intoSub = splitSubSections(into.lines);
    const fromSub = splitSubSections(from.lines);
    const intoHit = intoSub.subs.find((x) => normTitle(x.title) === normTitle(sel.sub));
    const fromHit = fromSub.subs.find((x) => normTitle(x.title) === normTitle(sel.sub));
    if (!intoHit || !fromHit) {
      if (sel.optional) continue;
      findSub(intoSub.subs, sel.sub, sel.parent, where);
      findSub(fromSub.subs, sel.sub, sel.parent, donorWhere);
    }
    intoHit.lines = fromHit.lines;
    into.lines = joinSections([intoSub.lead.join('\n'), ...intoSub.subs.map((x) => x.lines.join('\n'))])
      .replace(/\n$/, '')
      .split('\n');
  }
  return joinSections([target.preamble.join('\n'), ...target.sections.map((s) => s.lines.join('\n'))]);
}

/**
 * Build a fixture that is a projection of stored states rather than a file of its
 * own. Declared in the eval stage's frontmatter:
 *
 *   from:     the state to start from
 *   sections: keep only these (a cut — e.g. the blocks a refresh re-emits)
 *   override: replace these sections from another state (a splice — e.g. the
 *             settled body still carrying the draft's derived blocks)
 *   drop:     remove these sections entirely (e.g. hide a derived block from one
 *             role so it cannot be mistaken for the candidate's, or so the
 *             candidate must add it)
 *
 * Any selector may address a nested `###` subsection as `Parent > Sub` (e.g.
 * `Integration Specification > Overview`), so a refresh can splice/drop/cut the
 * Integration Specification's derived `### Overview` reader map on its own, exactly
 * as it does the top-level `## Solution Overview` / `## Document Consumption Guide`.
 *
 * Order of operations is from -> override -> drop -> cut, so a role can, in one
 * spec, take a settled body, swap one block for the stale draft's, and drop
 * another block the candidate must (re-)emit.
 *
 * Nothing derived is committed, so the hand-edited states stay the only truth and
 * a composed fixture cannot drift away from the states it sits between.
 */
function composeFixture(caseDir, spec, opts) {
  const read = (id) => {
    const got = readState(caseDir, id, opts);
    if (!got.found) {
      throw new Error(`Composed fixture: state "${id}" not found in ${caseDir}${opts.flow ? ` for flow ${opts.flow}` : ''}.`);
    }
    return got.content;
  };
  let content = read(spec.from);
  if (spec.override) {
    content = spliceSections(content, read(spec.override.from), spec.override.sections ?? [], spec.from, spec.override.from);
  }
  if (Array.isArray(spec.drop) && spec.drop.length) {
    content = dropSections(content, spec.drop, spec.from);
  }
  if (Array.isArray(spec.sections) && spec.sections.length) {
    content = cutSections(content, spec.sections, spec.from);
  }
  return content;
}

/** True when a composition's every source state resolves for this flow. */
function composable(caseDir, spec, opts) {
  const ids = [spec.from, ...(spec.override ? [spec.override.from] : [])];
  return ids.every((id) => readState(caseDir, id, opts).found);
}

/**
 * Read one state as a single document: the per-flow file if there is one, else
 * the flowless file, else a directory glob. Used by context resolution and by
 * fixture composition, so both agree on what a state id means.
 */
function readState(caseDir, id, { flow = null, flowsConfig = null, states = DEFAULT_STATES, want = 'settled' } = {}) {
  for (const candidate of stateCandidates(caseDir, id, states, want)) {
    const names = [];
    if (flow && flowsConfig) names.push(fillPattern(flowsConfig.golden, { artifact: candidate, flow }).replace(/\.md$/, ''));
    names.push(candidate);
    for (const name of names) {
      const asFile = join(caseDir, `${name}.md`);
      const collected = existsSync(asFile) ? collectMarkdown(asFile) : collectMarkdown(join(caseDir, name));
      if (!collected.found) continue;
      const content = collected.files
        .map((f) => (collected.kind === 'dir' ? `--- ${name}/${f.rel} ---\n${f.content}` : f.content))
        .join('\n\n');
      return { found: true, content };
    }
  }
  return { found: false, content: '' };
}

/**
 * Resolve the judge rubric + generator manifest + declared context ids for schema/artifact.
 * @returns {object}
 */
export function resolveArtifact(config, schema, artifact, { changeName, withInstructions = true } = {}) {
  if (!schema || !artifact) {
    throw new Error('Both --schema and --artifact are required.');
  }
  const suite = config.paths.suite;
  const judgePath = join(suite, schema, 'judge', `${artifact}.md`);
  const judgeReasoningPath = join(suite, schema, 'judge', 'reasoning.md');
  const generatorPath = join(suite, schema, 'generator', `${artifact}.md`);
  const generationContractPath = join(suite, schema, 'generator', 'contract.md');
  const genReasoningPath = join(suite, schema, 'generator', 'reasoning.md');

  if (!existsSync(judgePath)) {
    const available = listArtifacts(suite, schema, 'judge');
    throw new Error(
      `No judge rubric at ${judgePath}.\n` +
        `Available judge rubrics for "${schema}": ${available.length ? available.join(', ') : '(none)'}`,
    );
  }

  const judge = readDoc(judgePath);
  const judgeSources = inlineSources(dirname(judgePath), judge.frontmatter.sources);
  // A context entry is either a plain state id or a composition (see composeFixture).
  const contextIds = (Array.isArray(judge.frontmatter.context) ? judge.frontmatter.context : []).map((entry) =>
    typeof entry === 'string' ? { id: entry } : { ...entry, id: entry.id ?? entry.from },
  );
  // Optional: a golden that is a projection of stored states rather than a file.
  const goldenSpec = judge.frontmatter?.golden ?? null;
  const passThreshold = judge.frontmatter?.scoring?.pass_threshold ?? config.scoring.passThreshold;

  // Optional per-schema deterministic validity gate: the judge frontmatter may
  // declare `validator: <path>` (relative to the judge file) to a module that
  // exports `validateArtifact(candidateMd) -> { ok, findings }`. lib/ stays
  // schema-agnostic — it just runs whatever the schema declared (see judge.mjs
  // applyValidityGate). Absent field => no gate (skipped ok).
  const validator = judge.frontmatter?.validator
    ? { module: resolve(dirname(judgePath), judge.frontmatter.validator) }
    : null;

  // Per-schema, injectable reasoning/output procedure. Missing file is non-fatal:
  // judging still works, just without the procedure.
  const judgeReasoning = existsSync(judgeReasoningPath) ? readFileSync(judgeReasoningPath, 'utf8') : '';

  // Per-schema generation harness contract and planning procedure. Missing files
  // are non-fatal so legacy schemas can still run while they are being migrated.
  const generationContract = existsSync(generationContractPath)
    ? readFileSync(generationContractPath, 'utf8')
    : '';
  const genReasoning = existsSync(genReasoningPath) ? readFileSync(genReasoningPath, 'utf8') : '';

  let scenario = null;
  let scenarioSources = [];
  let command = null;
  let skills = [];
  let rules = [];
  let schemaArtifact = null;
  let substitutions = [];
  // Optional generator-owned context: when the generator manifest declares its
  // own `context`, the candidate is generated against a DIFFERENT projection of
  // the stored states than the judge grounds against (e.g. a refresh where the
  // generator sees the stale summary and must add the handoff, while the judge
  // sees neither derived block). Absent => the generator falls back to the
  // judge's context, so every other artifact is unchanged.
  let generatorContextIds = null;
  if (existsSync(generatorPath)) {
    scenario = readDoc(generatorPath);
    const baseDir = dirname(generatorPath);
    const fm = scenario.frontmatter ?? {};
    if (Array.isArray(fm.context)) {
      generatorContextIds = fm.context.map((entry) =>
        typeof entry === 'string' ? { id: entry } : { ...entry, id: entry.id ?? entry.from },
      );
    }

    // Manifest sourcing: real command + skills + rules + schema artifact, with
    // live references neutralized via the mapping registry.
    const mappings = loadMappings(config, schema);
    scenarioSources = (Array.isArray(fm.sources) ? fm.sources : []).map((ref) =>
      loadMappedSource(ref, baseDir, mappings),
    );
    if (fm.command) command = loadMappedSource(fm.command, baseDir, mappings);
    if (Array.isArray(fm.skills)) {
      skills = fm.skills.map((entry) =>
        resolveSkill(entry, { baseDir, skillsRoot: config.paths.skillsRoot, mappings }),
      );
    }
    // Rules the real subagent gets by Cursor attachment (alwaysApply / globs).
    // We curate the process-affecting set here (see the opsx-eval skill,
    // "Reference & attachment model").
    if (Array.isArray(fm.rules)) {
      rules = fm.rules.map((ref) => loadMappedSource(ref, baseDir, mappings));
    }
    // The OpenSpec block is compiled by the real `openspec` CLI (spawns a
    // process + temp root), so only do it when the block is actually used — the
    // generation prompt. The judge path does not use it.
    if (fm.schema_artifact && withInstructions) {
      schemaArtifact = loadOpenSpecInstructions(fm.schema_artifact, {
        openspecRoot: config.paths.openspecRoot,
        mappings,
        changeName: changeName ?? `${artifact}-eval`,
      });
    }
    substitutions = collectSubstitutions([command, ...skills, ...rules, schemaArtifact, ...scenarioSources]);
  }

  return {
    schema,
    artifact,
    judgePath,
    judge,
    judgeSources,
    judgeReasoningPath,
    judgeReasoning,
    scenarioPath: generatorPath,
    scenario,
    generationContractPath,
    generationContract,
    genReasoningPath,
    genReasoning,
    scenarioSources,
    command,
    skills,
    rules,
    schemaArtifact,
    substitutions,
    contextIds,
    generatorContextIds,
    goldenSpec,
    passThreshold,
    validator,
    // Opt-in: inject deterministic, harness-computed presence facts (DCG row set +
    // integration signpost, diffed against the golden) into the judge prompt. Only
    // the artifact whose judge sets this gets the block, so every other judge
    // prompt stays byte-identical.
    derivedStructureFacts: judge.frontmatter?.derived_structure_facts === true,
  };
}

/** Resolve a --dir/--context case dir: absolute, cwd-relative, or under fixtures. */
export function resolveCaseDir(config, schema, dirArg) {
  if (!dirArg) return null;
  const candidates = [
    isAbsolute(dirArg) ? dirArg : resolve(process.cwd(), dirArg),
    join(config.paths.suite, schema, 'fixtures', dirArg),
  ];
  for (const c of candidates) {
    if (existsSync(c) && statSync(c).isDirectory()) return c;
  }
  throw new Error(
    `Case dir not found: ${dirArg}\nTried:\n  - ${candidates.join('\n  - ')}`,
  );
}

/** Fill a `{token}` pattern (e.g. "{artifact}.{flow}.md") from a values map. */
function fillPattern(pattern, values) {
  return pattern.replace(/\{(\w+)\}/g, (m, k) => (k in values ? String(values[k]) : m));
}

/**
 * Resolve declared context ids to concatenated markdown from a case dir.
 * Each id maps to `<caseDir>/<id>.md` (file) or `<caseDir>/<id>/` (dir glob).
 *
 * When `flow` is set, a per-flow overlay `<id>.<flow>.md` is composed with the
 * base: if the base exists the overlay is appended (e.g. .context/initial-context.md +
 * .context/initial-context.ba.md); if the base is absent the overlay is used standalone
 * (e.g. a downstream artifact whose only flow-specific golden is discovery.ba.md).
 * @param {object} [opts] { flow, flowsConfig } — enables overlay composition
 * @returns {Array<{ id, found, content }>}
 */
export function resolveContext(caseDir, contextIds, { flow = null, flowsConfig = null, states = DEFAULT_STATES } = {}) {
  const out = [];
  for (const entry of contextIds) {
    const { id } = typeof entry === 'string' ? { id: entry } : entry;

    // A composed context is a projection of stored states (see composeFixture),
    // so it has no file of its own and no overlay to append.
    if (entry?.from) {
      const opts = { flow, flowsConfig, states };
      out.push(
        composable(caseDir, entry, opts)
          ? { id, found: true, content: composeFixture(caseDir, entry, opts) }
          : { id, found: false, content: '' },
      );
      continue;
    }

    // A context is what this run RECEIVES, so a state directory resolves to the
    // settled state — what a downstream artifact is actually handed.
    let baseContent = '';
    let resolvedId = id;
    for (const candidate of stateCandidates(caseDir, id, states, 'settled')) {
      const asFile = join(caseDir, `${candidate}.md`);
      const base = existsSync(asFile) ? collectMarkdown(asFile) : collectMarkdown(join(caseDir, candidate));
      if (!base.found) {
        resolvedId = candidate;
        continue;
      }
      resolvedId = candidate;
      baseContent = base.files
        .map((f) => (base.kind === 'dir' ? `--- ${candidate}/${f.rel} ---\n${f.content}` : f.content))
        .join('\n\n');
      break;
    }

    let overlayContent = '';
    if (flow && flowsConfig) {
      const overlayName = fillPattern(flowsConfig.contextOverlay, { context: resolvedId, flow });
      const overlayPath = join(caseDir, overlayName);
      if (existsSync(overlayPath) && statSync(overlayPath).isFile()) {
        overlayContent = readFileSync(overlayPath, 'utf8');
      } else {
        // Per-flow directory overlay (e.g. specs.ba/ next to the flowless specs/
        // baselines), mirroring resolveGolden's dir support for bundle artifacts.
        const overlayDirName = overlayName.replace(/\.md$/, '');
        const overlayDir = join(caseDir, overlayDirName);
        if (existsSync(overlayDir) && statSync(overlayDir).isDirectory()) {
          const collected = collectMarkdown(overlayDir);
          if (collected.found) {
            overlayContent = collected.files
              .map((f) => `--- ${overlayDirName}/${f.rel} ---\n${f.content}`)
              .join('\n\n');
          }
        }
      }
    }

    if (!baseContent && !overlayContent) {
      out.push({ id, found: false, content: '' });
      continue;
    }
    const content = [baseContent, overlayContent].filter(Boolean).join('\n\n');
    out.push({ id, found: true, content });
  }
  return out;
}

/**
 * Parse the capability names from a discovery artifact's `## Affected
 * Capabilities` table (first column, backticks stripped). Returns a Set, or
 * null when no such section is present.
 */
function parseAffectedCapabilities(discoveryMd) {
  const start = discoveryMd.indexOf('## Affected Capabilities');
  if (start === -1) return null;
  const rest = discoveryMd.slice(start + '## Affected Capabilities'.length);
  const end = rest.indexOf('\n## ');
  const section = end === -1 ? rest : rest.slice(0, end);
  const caps = new Set();
  for (const line of section.split('\n')) {
    const t = line.trim();
    if (!t.startsWith('|')) continue;
    const first = (t.split('|')[1] ?? '').replace(/`/g, '').trim();
    if (!first || first.toLowerCase() === 'capability' || /^:?-+:?$/.test(first)) continue;
    caps.add(first);
  }
  return caps;
}

/**
 * Scope baseline capability specs (context ids shaped `specs/<cap>/spec`) to the
 * capabilities THIS change's discovery marks affected. Mirrors reality: a change
 * pours in only the baselines for the capabilities it touches, so a flow whose
 * discovery has no baseline-bearing capability (e.g. a new-integration ADD)
 * carries no baseline at all. Blocks that are not baseline ids, and every block
 * when discovery is absent or has no Affected Capabilities table, pass through
 * unchanged.
 * @param {Array<{ id, found, content }>} contextBlocks
 * @returns {Array<{ id, found, content }>}
 */
export function scopeBaselinesToDiscovery(contextBlocks) {
  // The id may name a state (`discovery/final`) once discovery's fixtures are
  // stored as states, so match the artifact rather than the exact id — otherwise
  // this quietly stops scoping instead of failing.
  const isDiscovery = (id) => id === 'discovery' || String(id).startsWith('discovery/');
  const discovery = contextBlocks.find((b) => isDiscovery(b.id) && b.found && b.content);
  if (!discovery) return contextBlocks;
  const affected = parseAffectedCapabilities(discovery.content);
  if (!affected) return contextBlocks;
  return contextBlocks.filter((b) => {
    const m = /^specs\/(.+)\/spec$/.exec(b.id);
    if (!m) return true;
    return affected.has(m[1]);
  });
}

/**
 * Resolve the golden expected artifact from a case dir.
 * `<caseDir>/<artifact>.md` (e.g. discovery.md) or `<caseDir>/<artifact>/` (e.g. fsd/, specs/).
 *
 * When `flow` is set, the per-flow golden `<artifact>.<flow>.md` (or dir) is
 * preferred; if it is absent the flowless `<artifact>.md` is used as fallback.
 * @param {object} [opts] { flow, flowsConfig }
 * @returns {{ found, content }}
 */
export function resolveGolden(caseDir, artifact, { flow = null, flowsConfig = null, states = DEFAULT_STATES, spec = null } = {}) {
  // A golden may be declared as a projection of stored states instead of a file.
  if (spec?.from) {
    const opts = { flow, flowsConfig, states };
    return composable(caseDir, spec, opts)
      ? { found: true, content: composeFixture(caseDir, spec, opts) }
      : { found: false, content: '' };
  }

  // A golden is what this run must PRODUCE, so a state directory resolves to the
  // draft — the artifact as first emitted, before any review gate changed it.
  assertDraftForReviewedFlow(caseDir, artifact, flow, states, flowsConfig);
  const candidates = stateCandidates(caseDir, artifact, states, 'draft');
  const names = [];
  if (flow && flowsConfig) {
    for (const c of candidates) names.push(fillPattern(flowsConfig.golden, { artifact: c, flow }).replace(/\.md$/, ''));
  }
  names.push(...candidates);

  for (const name of names) {
    const asFile = join(caseDir, `${name}.md`);
    const asDir = join(caseDir, name);
    const collected = existsSync(asFile) ? collectMarkdown(asFile) : collectMarkdown(asDir);
    if (!collected.found) continue;
    const content = collected.files
      .map((f) => (collected.kind === 'dir' ? `--- ${name}/${f.rel} ---\n${f.content}` : f.content))
      .join('\n\n');
    return { found: true, content };
  }
  return { found: false, content: '' };
}

/**
 * Determine which delivery-scope flows a case exercises for an artifact: the
 * flows in `config.flows.order` that ship a per-flow golden `<artifact>.<flow>.md`
 * (file or dir) in the case dir. Returns `[null]` when the case is flowless, so
 * callers can treat the flowed and single-run paths uniformly.
 * @returns {Array<string|null>}
 */
export function resolveFlows(config, caseDir, artifact, { specs = [] } = {}) {
  const order = config.flows?.order ?? [];
  if (!caseDir || !order.length) return [null];
  const states = config.states ?? DEFAULT_STATES;
  const composed = specs.filter((s) => s?.from);
  const active = order.filter((flow) => {
    const opts = { flow, flowsConfig: config.flows, states };
    // A composed fixture has no file to look for, so a flow is exercised when every
    // state its compositions read exists for it. Compositions are strict, which is
    // what keeps the review-gate stage on the one flow that ships a draft as well
    // as a settled artifact instead of degenerating into a no-op on the others.
    if (composed.length) return composed.every((spec) => composable(caseDir, spec, opts));
    assertDraftForReviewedFlow(caseDir, artifact, flow, states, config.flows);
    return stateCandidates(caseDir, artifact, states, 'draft').some((c) => {
      const name = fillPattern(config.flows.golden, { artifact: c, flow }).replace(/\.md$/, '');
      return existsSync(join(caseDir, `${name}.md`)) || existsSync(join(caseDir, name));
    });
  });
  return active.length ? active : [null];
}
