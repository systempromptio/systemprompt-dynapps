// Judge core — the single reusable primitive both subcommands run over.
//
//   generateArtifact(...) -> candidate markdown (eval only)
//   judgeArtifact(...)    -> { verdict, raw }   (eval + judge)
//
// The CLI-only discovery guard lives in run.mjs, NOT here: the core can judge
// any artifact as long as context is supplied (eval feeds it fixtures), while
// the `judge` CLI forbids discovery until live primary-context sourcing exists.

import { pathToFileURL } from 'node:url';
import { buildGenerationPrompt, buildJudgePrompt } from './prompt.mjs';
import { parseVerdict, stripPlan, aggregateJudgeSamples } from './report.mjs';
import { checkStructure, deriveStructureFacts } from './structure.mjs';

/**
 * Deterministic presence facts, computed once from the candidate + golden and
 * injected into the judge prompt when the resolved artifact opts in (judge
 * frontmatter `derived_structure_facts: true`). Empty string when not opted in or
 * when the golden anchors no guide/signpost, so the prompt stays byte-identical
 * for every other artifact.
 */
function resolveStructureFacts(resolved, golden, candidate, contextBlocks) {
  if (!resolved?.derivedStructureFacts) return '';
  if (!golden || !golden.found || !golden.content) return '';
  // The body is the authoritative structure for the integration signpost. It is
  // the `design-in-review` context block (id ends with `design-in-review`); the
  // guide diff needs only candidate + golden, so a missing body just drops the
  // signpost fact.
  const body = (Array.isArray(contextBlocks) ? contextBlocks : []).find(
    (c) => c?.found && /design-in-review$/.test(String(c.id ?? '')),
  );
  return deriveStructureFacts(candidate, golden.content, body?.content ?? '');
}

/**
 * Deterministic structural gate, applied to the folded verdict after judging.
 * Independent of the LLM judge: it checks the candidate's section presence,
 * order, and table columns against the GOLDEN fixture (the structural canon of
 * the golden+judge loop). A structural failure is a hard gate — it forces
 * `pass = false` — but it deliberately leaves the content `score` untouched so
 * the rubric signal stays readable ("content was fine, structure was not").
 * Skipped (ok) when the case ships no golden to anchor structure.
 * @param {object} verdict folded verdict (mutated in place)
 * @param {{ found: boolean, content: string }} golden resolved golden fixture
 * @param {string} candidate candidate markdown the judge scored
 */
function applyStructureGate(verdict, golden, candidate) {
  if (!golden || !golden.found || !golden.content) {
    verdict.structure = { ok: true, findings: [], skipped: true };
    return verdict;
  }
  const result = checkStructure(candidate, golden.content);
  verdict.structure = result;
  if (!result.ok) verdict.pass = false;
  return verdict;
}

/**
 * Deterministic validity gate: a schema-agnostic sibling of the structure gate
 * for artifacts whose "renderability" is objective and code-checkable (e.g. a
 * drawio diagram's call stack), which the LLM judge cannot be trusted to
 * simulate. When the resolved artifact declares a `validator` module (judge
 * frontmatter `validator: <path>`), import it and run its `validateArtifact` on
 * the candidate; a failure is a hard gate (forces `pass = false`) but leaves the
 * content `score` untouched, exactly like the structure gate. Skipped (ok) when
 * the schema declares no validator, so lib/ stays schema-agnostic.
 * @param {object} verdict folded verdict (mutated in place)
 * @param {object} resolved resolved artifact (may carry `validator.module`)
 * @param {string} candidate candidate markdown the judge scored
 */
async function applyValidityGate(verdict, resolved, candidate) {
  const cfg = resolved?.validator;
  if (!cfg || !cfg.module) {
    verdict.validity = { ok: true, findings: [], skipped: true };
    return verdict;
  }
  let result;
  try {
    const mod = await import(pathToFileURL(cfg.module).href);
    if (typeof mod.validateArtifact !== 'function') {
      verdict.validity = { ok: true, findings: [], skipped: true };
      return verdict;
    }
    result = mod.validateArtifact(candidate) ?? { ok: true, findings: [] };
  } catch (err) {
    // A broken/unloadable validator must not silently pass a bad candidate, but
    // it also must not crash the batch: record the error and leave the verdict
    // untouched (skipped), so the run is judged on the LLM signal alone.
    verdict.validity = { ok: true, findings: [], skipped: true, error: String(err?.message ?? err) };
    return verdict;
  }
  verdict.validity = result;
  if (!result.ok && !result.skipped) verdict.pass = false;
  return verdict;
}

/**
 * Generate one candidate artifact in memory (temperature > 0).
 * Returns both the raw model output (with the `<plan>` pass) and the stripped
 * candidate, so callers can judge the candidate while still saving the raw
 * generation for inspection.
 * @returns {Promise<{ raw: string, candidate: string }>}
 */
export async function generateArtifact({ client, resolved, contextBlocks }) {
  const { system, user } = buildGenerationPrompt({
    schema: resolved.schema,
    artifact: resolved.artifact,
    generationContract: resolved.generationContract,
    genReasoning: resolved.genReasoning,
    scenarioBody: resolved.scenario?.body ?? '',
    scenarioSources: resolved.scenarioSources,
    command: resolved.command,
    skills: resolved.skills,
    rules: resolved.rules,
    schemaArtifact: resolved.schemaArtifact,
    contextBlocks,
  });
  const raw = await client.generate(system, user);
  return { raw, candidate: stripPlan(raw) };
}

/**
 * Detect a candidate that carries no artifact: empty/whitespace, or a
 * structureless blob (a refusal sentence, stray prose after a bare planning
 * pass). Templated artifacts carry `#` headings, but a DELIVERY SCOPE GATE stop
 * is a valid free-form artifact (the schema mandates "output ONLY the
 * decomposition" — often bold pseudo-headings plus a table / list, no `#` at
 * all), so any markdown structure signal (heading, table row, list item, bold
 * title line) routes the candidate to the judge, which grades the stop's logic.
 * Used to short-circuit judging so a blank candidate can never be scored.
 * @param {string} candidate
 * @returns {boolean}
 */
function isNoArtifact(candidate) {
  const text = String(candidate ?? '').trim();
  if (!text) return true;
  if (/^#{1,6}\s/m.test(text)) return false;
  return !/^(\||[-*]\s|\d+[.)]\s|\*\*[^\n]+\*\*\s*$)/m.test(text);
}

/**
 * Deterministic verdict for a no-artifact candidate: all-zero, one critical, no
 * model call. Without this the LLM judge can hallucinate a perfect score on an
 * empty candidate (observed: 1.0 on a bare `<plan>` output). The structure gate
 * would still fail such a run, but the rubric score must not lie.
 * @returns {object} verdict
 */
function noArtifactVerdict(candidate) {
  const snippet = String(candidate ?? '').trim().slice(0, 200);
  return {
    pass: false,
    score: 0,
    rubric_scores: {},
    critical_findings: [
      {
        issue:
          'No artifact produced: the candidate is empty or contains no artifact content ' +
          '(e.g. a refusal or a bare planning pass), so nothing can be scored.',
        evidence: snippet ? `Candidate begins: ${snippet}` : 'Candidate is empty.',
      },
    ],
    warnings: [],
    missing_evidence: [],
    judge_samples: [],
  };
}

/**
 * Judge one candidate against the rubric + context (+ optional golden).
 *
 * With `samples > 1` the same candidate is judged K times and the verdicts are
 * folded (median score + majority-vote criticals) to cancel the judge's temp-0
 * nondeterminism. The returned `raw` is the representative (median) sample; every
 * raw is returned in `raws` so a save trail can keep the full spread.
 * @returns {Promise<{ verdict: object, raw: string, raws: string[] }>}
 */
/**
 * True when a verdict carries no evidence that the STANDARD's closed-rule gates
 * were actually run. The prompt requires one `gates` entry per gate, each listing
 * the items its enumeration step produced; a verdict with no populated
 * enumeration was scored by pattern-matching the rubric's examples instead of by
 * running its procedures (measured: 120/120 samples skipped the routing pass
 * entirely, which tracked with firing on rubric example values and missing values
 * of the same class beside them).
 * @param {object} verdict
 * @returns {boolean}
 */
function gatesNotRun(verdict) {
  const gates = verdict?.gates;
  if (!Array.isArray(gates) || gates.length === 0) return true;
  return !gates.some((g) => Array.isArray(g?.enumerated) && g.enumerated.length > 0);
}

const DISPOSITIONS = new Set(['critical', 'warning', 'missing_evidence', 'no_breach']);

/**
 * True when a gate lists an item as having reached the firing step but does not
 * say what it is. Survived-without-disposition is the half of the gate contract
 * the traceability rule does not cover: criticals always traced back to a
 * survived item, while survived items routinely produced nothing (measured: one
 * verdict carried 37 surviving items and zero findings), which is where the
 * unfired consumer-contract family lived.
 * @param {object} verdict
 * @returns {boolean}
 */
function survivedNotDispositioned(verdict) {
  const gates = Array.isArray(verdict?.gates) ? verdict.gates : [];
  return gates.some((g) =>
    (Array.isArray(g?.survived) ? g.survived : []).some(
      (s) => !s || typeof s !== 'object' || !DISPOSITIONS.has(s.disposition),
    ),
  );
}

/**
 * One judge call that survives a transient non-JSON response, and that re-asks
 * when the judge skipped the gate enumerations. The judge model occasionally
 * emits a truncated/garbled body that `parseVerdict` cannot read; a single such
 * response used to abort the whole batch (Promise.all rejection). Retry the call
 * a few times before giving up.
 *
 * A verdict that parses but carries no gate working, or that lists survived items
 * without saying what they are, is retried on the same grounds. The last attempt
 * is accepted rather than thrown so one stubborn sample cannot abort a batch — it
 * is marked `gates_not_run` / `survived_not_dispositioned` so the share of samples
 * still skipping each half of the contract stays measurable. This is a
 * reliability and observability guard only: a returned verdict's scores are used
 * exactly as given.
 * @returns {Promise<{ raw: string, verdict: object }>}
 */
async function judgeOnce(client, system, user, attempts = 3) {
  let lastErr;
  let lastUngated;
  for (let a = 0; a < attempts; a++) {
    const raw = await client.judge(system, user);
    let verdict;
    try {
      verdict = parseVerdict(raw);
    } catch (err) {
      lastErr = err;
      continue;
    }
    const noGates = gatesNotRun(verdict);
    const noDispositions = survivedNotDispositioned(verdict);
    if (!noGates && !noDispositions) return { raw, verdict };
    lastUngated = { raw, verdict, noGates, noDispositions };
  }
  if (lastUngated) {
    if (lastUngated.noGates) lastUngated.verdict.gates_not_run = true;
    if (lastUngated.noDispositions) lastUngated.verdict.survived_not_dispositioned = true;
    return { raw: lastUngated.raw, verdict: lastUngated.verdict };
  }
  throw lastErr;
}

export async function judgeArtifact({ client, resolved, contextBlocks, golden, candidate, samples = 1 }) {
  // Deterministic short-circuit BEFORE the model call: a blank / no-artifact
  // candidate can never earn a score, and the judge model would otherwise
  // hallucinate one. The structure gate still runs and can only tighten this.
  if (isNoArtifact(candidate)) {
    const verdict = noArtifactVerdict(candidate);
    applyStructureGate(verdict, golden, candidate);
    const raw = JSON.stringify(verdict, null, 2);
    return { verdict, raw, raws: [raw] };
  }

  const { system, user } = buildJudgePrompt({
    schema: resolved.schema,
    artifact: resolved.artifact,
    judgeBody: resolved.judge.body,
    judgeSources: resolved.judgeSources,
    judgeReasoning: resolved.judgeReasoning,
    contextBlocks,
    golden,
    candidate,
    passThreshold: resolved.passThreshold,
    structureFacts: resolveStructureFacts(resolved, golden, candidate, contextBlocks),
  });

  const k = Math.max(1, samples | 0);
  const raws = [];
  const verdicts = [];
  for (let i = 0; i < k; i++) {
    const { raw, verdict } = await judgeOnce(client, system, user);
    raws.push(raw);
    verdicts.push(verdict);
  }

  const verdict = aggregateJudgeSamples(verdicts, resolved.passThreshold);
  // Deterministic gates run once on the candidate (neither structure nor spec
  // validity varies across judge samples) and can only tighten the verdict to a
  // fail. Structure checks section/table shape against the golden; validity runs
  // the schema's declared renderability check (drawio: the skill's own spec
  // validator) so an unrenderable spec cannot pass on LLM signal alone.
  applyStructureGate(verdict, golden, candidate);
  await applyValidityGate(verdict, resolved, candidate);
  // Representative raw = the sample at the (lower) median overall score, matching
  // the folding in aggregateJudgeSamples so the saved raw explains the verdict.
  const order = verdicts
    .map((v, i) => ({ i, score: typeof v.score === 'number' ? v.score : NaN }))
    .sort((a, b) => a.score - b.score);
  const repIndex = order[Math.floor((order.length - 1) / 2)].i;
  return { verdict, raw: raws[repIndex], raws };
}

/** Build the generation prompt without calling the model (for --prompt-only). */
export function previewGenerationPrompt({ resolved, contextBlocks }) {
  return buildGenerationPrompt({
    schema: resolved.schema,
    artifact: resolved.artifact,
    generationContract: resolved.generationContract,
    genReasoning: resolved.genReasoning,
    scenarioBody: resolved.scenario?.body ?? '',
    scenarioSources: resolved.scenarioSources,
    command: resolved.command,
    skills: resolved.skills,
    rules: resolved.rules,
    schemaArtifact: resolved.schemaArtifact,
    contextBlocks,
  });
}

/** Build the judge prompt without calling the model (for --prompt-only). */
export function previewJudgePrompt({ resolved, contextBlocks, golden, candidate }) {
  const cand = candidate ?? '(candidate will be generated at eval time)';
  return buildJudgePrompt({
    schema: resolved.schema,
    artifact: resolved.artifact,
    judgeBody: resolved.judge.body,
    judgeSources: resolved.judgeSources,
    judgeReasoning: resolved.judgeReasoning,
    contextBlocks,
    golden,
    candidate: cand,
    passThreshold: resolved.passThreshold,
    structureFacts: resolveStructureFacts(resolved, golden, cand, contextBlocks),
  });
}
