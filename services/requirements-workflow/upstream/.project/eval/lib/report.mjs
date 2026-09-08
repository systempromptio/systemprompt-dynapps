// Verdict parsing and human-readable reporting (single verdict + N-run aggregate).

/**
 * Extract the JSON verdict from the judge's raw text. The judge is instructed to
 * return a bare JSON object, but we defensively strip code fences and locate the
 * outermost object in case a model wraps it.
 * @param {string} raw
 * @returns {object} parsed verdict
 */
/**
 * Normalize a parsed verdict's `rubric_scores` so every dimension is an object
 * `{ score, reason }`. The judge occasionally emits a dimension as a bare number
 * (e.g. `"scope_fit": 0`) instead of `{ "score": 0, ... }`. Downstream consumers
 * (per-dimension aggregation, the single-verdict report) read `dim.score`, so a
 * bare-number dimension was silently dropped from the stats (undercounting `n`).
 * This coerces it without changing any score. Non-numeric/other shapes are left
 * untouched so genuinely malformed dimensions still surface as missing.
 * @param {object} verdict parsed verdict (mutated copy returned)
 * @returns {object}
 */
export function normalizeVerdictShape(verdict) {
  if (!verdict || typeof verdict !== 'object') return verdict;
  const rubric = verdict.rubric_scores;
  if (!rubric || typeof rubric !== 'object') return verdict;
  const normalized = {};
  for (const [dim, entry] of Object.entries(rubric)) {
    normalized[dim] = typeof entry === 'number' ? { score: entry } : entry;
  }
  return { ...verdict, rubric_scores: normalized };
}

export function parseVerdict(raw) {
  return normalizeVerdictShape(parseVerdictInner(raw));
}

function parseVerdictInner(raw) {
  const text = raw.trim();

  // The judge may emit a brief <reasoning> scratchpad before the verdict, and is
  // asked to wrap the verdict in a ```json fence. Prefer the LAST fenced block so
  // any reasoning prose (or an example fence inside it) never shadows the verdict.
  const fences = [...text.matchAll(/```(?:json)?\s*([\s\S]*?)```/gi)];
  if (fences.length) {
    const inner = fences[fences.length - 1][1].trim();
    try {
      return JSON.parse(inner);
    } catch {
      /* fall through to the reasoning-aware fallbacks below */
    }
  }

  // No usable fence: drop a leading reasoning block, then take what remains.
  const body = text.replace(/^[\s\S]*?<\/reasoning>\s*/i, '').trim() || text;
  try {
    return JSON.parse(body);
  } catch {
    // Last resort: the outermost {...} span of the post-reasoning body.
    const start = body.indexOf('{');
    const end = body.lastIndexOf('}');
    if (start !== -1 && end > start) {
      return JSON.parse(body.slice(start, end + 1));
    }
    throw new Error(`Judge did not return valid JSON. Raw output:\n${raw}`);
  }
}

/**
 * Recompute the pass flag from the judge's own §8 invariant instead of trusting
 * the raw `pass` field. The judge (temp 0) occasionally emits `pass = false` with
 * `score >= threshold` and empty `critical_findings`, which violates its own
 * consistency rule. The harness is the source of truth for pass/fail.
 * @param {object} verdict
 * @param {number} threshold
 * @returns {object} a shallow copy with a recomputed `pass`
 */
export function normalizeVerdict(verdict, threshold) {
  const v = { ...verdict };
  const score = typeof v.score === 'number' ? v.score : NaN;
  const crit = Array.isArray(v.critical_findings) ? v.critical_findings : [];
  v.pass = Number.isFinite(score) && score >= threshold && crit.length === 0;
  // Enforce the rubric's own cap ("Overall score and pass": any critical ⇒
  // score ≤ 0.5). The judge (temp 0) occasionally emits a high score alongside
  // a critical, contradicting its own rule; capping keeps the reported score
  // honest. This can only LOWER a score — `pass` is already gated on an empty
  // critical set, so the pass rate is unaffected.
  if (crit.length > 0 && Number.isFinite(score) && score > 0.5) v.score = 0.5;
  return v;
}

/**
 * Fold K judge samples of the SAME candidate into one verdict, cancelling the
 * judge's temp-0 nondeterminism. The overall score is the (lower) median across
 * samples. `critical_findings` is a majority vote on whether a critical is
 * PRESENT, not on issue wording (see the inline note below): once a majority of
 * samples flag anything, the representative sample's whole critical list is
 * kept, so a critical raised by a minority of samples can survive the fold. The
 * rubric reasons come from the representative (median) sample so they stay
 * internally consistent. `pass` is recomputed from the folded score + criticals.
 * @param {Array<object>} samples parsed verdicts
 * @param {number} threshold
 * @returns {object} folded, normalized verdict with a `judge_samples` summary
 */
export function aggregateJudgeSamples(samples, threshold) {
  const list = Array.isArray(samples) ? samples : [];
  if (list.length <= 1) {
    const v = normalizeVerdict(list[0] ?? {}, threshold);
    v.judge_samples = list.map((s) => ({
      score: typeof s.score === 'number' ? s.score : NaN,
      criticalCount: (s.critical_findings ?? []).length,
    }));
    return v;
  }

  const scored = list.map((s) => ({ s, score: typeof s.score === 'number' ? s.score : NaN }));
  const sorted = [...scored].sort((a, b) => a.score - b.score);
  const repIndex = Math.floor((sorted.length - 1) / 2); // lower median
  const rep = { ...sorted[repIndex].s };
  const medianScore = sorted[repIndex].score;

  // Majority vote on the PRESENCE of a critical, not on issue wording: the judge
  // rephrases the same defect each sample, so text matching would wrongly drop a
  // consensus critical. If a majority of samples flag any critical, keep the
  // representative's criticals (or, if the rep has none, the lowest sample's).
  const majorityNeeded = Math.floor(list.length / 2) + 1;
  const samplesWithCritical = list.filter((s) => (s.critical_findings ?? []).length > 0).length;
  const criticalMajority = samplesWithCritical >= majorityNeeded;
  let foldedCriticals = [];
  if (criticalMajority) {
    foldedCriticals =
      (rep.critical_findings ?? []).length > 0
        ? rep.critical_findings
        : sorted.find((x) => (x.s.critical_findings ?? []).length > 0)?.s.critical_findings ?? [];
  }

  rep.score = medianScore;
  rep.critical_findings = foldedCriticals;
  rep.pass = Number.isFinite(medianScore) && medianScore >= threshold && !criticalMajority;
  // Same rubric cap as normalizeVerdict, applied to the folded verdict: a
  // majority-voted critical caps the median score at 0.5. Pass is already gated
  // on !criticalMajority, so this only makes the reported score honest.
  if (foldedCriticals.length > 0 && Number.isFinite(medianScore) && medianScore > 0.5) rep.score = 0.5;
  rep.judge_samples = scored.map((w) => ({
    score: w.score,
    criticalCount: (w.s.critical_findings ?? []).length,
  }));
  return rep;
}

/**
 * Remove the generator's inline planning pass from a raw candidate response.
 * The prompt asks for `<plan>...</plan>` followed by the finished artifact. If a
 * model skips the plan, keep the raw output so legacy behaviour still works.
 *
 * Cut at the LAST `</plan>`, not the first: the model sometimes mentions the
 * literal token inside the plan body (e.g. a self-check like "output only Markdown
 * after the plan block"), and matching the first occurrence would slice mid-plan
 * and leak plan prose into the candidate. The real artifact always follows the
 * final close tag.
 * @param {string} raw
 * @returns {string} candidate artifact markdown
 */
export function stripPlan(raw) {
  const text = String(raw ?? '').trim();
  const matches = [...text.matchAll(/<\/plan>\s*/gi)];
  if (!matches.length) return text;
  const close = matches[matches.length - 1];
  return text.slice(close.index + close[0].length).trim();
}

/**
 * Split an assembled prompt into its `===== TITLE =====` blocks and estimate the
 * token cost (~chars/4) of each. Lets `--prompt-only` show where the prompt
 * weight is per block without an ad hoc script. Nested blocks (e.g. a SKILL
 * inside the SKILLS wrapper) surface as their own rows, which is what you want
 * when deciding what to trim.
 * @param {string} text assembled system or user prompt
 * @returns {Array<{ title: string, chars: number, tokens: number }>}
 */
export function profileBlocks(text) {
  const body = String(text ?? '');
  const re = /^===== (.+?) =====$/gm;
  const marks = [...body.matchAll(re)];
  if (!marks.length) {
    return [{ title: '(whole prompt)', chars: body.length, tokens: Math.round(body.length / 4) }];
  }
  const out = [];
  const pre = body.slice(0, marks[0].index).trim();
  if (pre) out.push({ title: '(preamble)', chars: pre.length, tokens: Math.round(pre.length / 4) });
  for (let i = 0; i < marks.length; i++) {
    const start = marks[i].index + marks[i][0].length;
    const end = i + 1 < marks.length ? marks[i + 1].index : body.length;
    const content = body.slice(start, end);
    out.push({ title: marks[i][1].trim(), chars: content.length, tokens: Math.round(content.length / 4) });
  }
  return out;
}

/**
 * Format a per-block token profile of the assembled prompt, sorted heaviest
 * first, for `--prompt-only`.
 * @param {string} system
 * @param {string} user
 * @returns {string}
 */
export function formatPromptProfile(system, user) {
  const rows = [
    ...profileBlocks(system).map((b) => ({ ...b, side: 'system' })),
    ...profileBlocks(user).map((b) => ({ ...b, side: 'user' })),
  ];
  const total = rows.reduce((a, b) => a + b.tokens, 0);
  const lines = ['', 'Prompt profile (est. tokens per block, ~chars/4, heaviest first):'];
  for (const r of [...rows].sort((a, b) => b.tokens - a.tokens)) {
    const share = total ? Math.round((r.tokens / total) * 100) : 0;
    lines.push(`  ${String(r.tokens).padStart(6)}  ${String(share).padStart(3)}%  [${r.side}] ${r.title}`);
  }
  lines.push(`  ${String(total).padStart(6)}  100%  TOTAL`);
  lines.push('');
  return lines.join('\n');
}

function pct(n) {
  return typeof n === 'number' ? `${Math.round(n * 100)}%` : 'n/a';
}

function num(n, d = 3) {
  return typeof n === 'number' && Number.isFinite(n) ? n.toFixed(d) : 'n/a';
}

/**
 * Format a single verdict for stdout.
 * @param {object} verdict
 * @param {object} meta { schema, artifact, model, candidatePath, passThreshold }
 * @returns {string}
 */
export function formatReport(verdict, meta) {
  const lines = [];
  const status = verdict.pass ? 'PASS' : 'FAIL';
  lines.push('');
  lines.push(`## Eval: ${meta.schema}/${meta.artifact} — ${status}`);
  if (meta.candidatePath) lines.push(`Candidate: ${meta.candidatePath}`);
  lines.push(`Judge: ${meta.model}`);
  lines.push(`Score: ${pct(verdict.score)} (threshold ${pct(meta.passThreshold)})`);

  const rubric = verdict.rubric_scores ?? {};
  const dims = Object.keys(rubric);
  if (dims.length) {
    lines.push('');
    lines.push('Rubric:');
    for (const dim of dims) {
      const r = rubric[dim] ?? {};
      lines.push(`  - ${dim}: ${pct(r.score)} — ${r.reason ?? ''}`.trimEnd());
    }
  }

  const structure = verdict.structure;
  if (structure && !structure.skipped) {
    lines.push('');
    lines.push(`Structure gate: ${structure.ok ? 'PASS' : 'FAIL'}`);
    for (const f of structure.findings ?? []) {
      lines.push(`  - ${f.issue}`);
      if (f.evidence) lines.push(`    evidence: ${f.evidence}`);
    }
  }

  const validity = verdict.validity;
  if (validity && !validity.skipped) {
    lines.push('');
    lines.push(`Validity gate: ${validity.ok ? 'PASS' : 'FAIL'}`);
    for (const f of validity.findings ?? []) {
      lines.push(`  - ${f.issue}`);
      if (f.evidence) lines.push(`    evidence: ${f.evidence}`);
    }
  }

  const crit = verdict.critical_findings ?? [];
  if (crit.length) {
    lines.push('');
    lines.push('Critical findings:');
    for (const f of crit) {
      lines.push(`  - ${f.issue}`);
      if (f.evidence) lines.push(`    evidence: ${f.evidence}`);
    }
  }

  const warn = verdict.warnings ?? [];
  if (warn.length) {
    lines.push('');
    lines.push('Warnings:');
    for (const f of warn) {
      lines.push(`  - ${f.issue}`);
      if (f.evidence) lines.push(`    evidence: ${f.evidence}`);
    }
  }

  const missing = verdict.missing_evidence ?? [];
  if (missing.length) {
    lines.push('');
    lines.push('Missing evidence:');
    for (const m of missing) lines.push(`  - ${m}`);
  }

  if (verdict.recommendation) {
    lines.push('');
    lines.push(`Recommendation: ${verdict.recommendation}`);
  }
  lines.push('');
  return lines.join('\n');
}

function mean(xs) {
  return xs.length ? xs.reduce((a, b) => a + b, 0) / xs.length : NaN;
}

function stddev(xs) {
  if (xs.length < 2) return 0;
  const m = mean(xs);
  return Math.sqrt(mean(xs.map((x) => (x - m) ** 2)));
}

/**
 * Tally the items of one verdict list across all runs, keeping which run numbers
 * (1-based) each distinct finding came from so a recurring defect can be traced
 * back to specific candidates. Sorted by frequency, then earliest run.
 * @param {Array<object>} verdicts
 * @param {string} listKey verdict field to read (e.g. 'critical_findings')
 * @param {(item: any) => string|undefined} keyFn
 * @returns {Array<[string, { count: number, runs: number[] }]>}
 */
function tallyWithRuns(verdicts, listKey, keyFn) {
  const map = new Map();
  verdicts.forEach((v, i) => {
    for (const it of v[listKey] ?? []) {
      const key = keyFn(it);
      if (!key) continue;
      if (!map.has(key)) map.set(key, { count: 0, runs: new Set() });
      const entry = map.get(key);
      entry.count += 1;
      entry.runs.add(i + 1);
    }
  });
  return [...map.entries()]
    .map(([key, e]) => [key, { count: e.count, runs: [...e.runs].sort((a, b) => a - b) }])
    .sort((a, b) => b[1].count - a[1].count || a[1].runs[0] - b[1].runs[0]);
}

/**
 * Aggregate N verdicts into a variability report.
 * @param {Array<object>} verdicts
 * @param {object} meta { schema, artifact, model, passThreshold, caseDir, runs }
 * @returns {string}
 */
export function formatAggregate(verdicts, meta) {
  const lines = [];
  const n = verdicts.length;
  const scores = verdicts.map((v) => v.score).filter((s) => typeof s === 'number' && Number.isFinite(s));
  const passCount = verdicts.filter((v) => v.pass).length;

  lines.push('');
  lines.push(`## Eval aggregate: ${meta.schema}/${meta.artifact} — ${n} runs`);
  if (meta.genModel) lines.push(`Generator: ${meta.genModel}`);
  lines.push(`Judge: ${meta.model}`);
  if (meta.caseDir) lines.push(`Case: ${meta.caseDir}`);
  lines.push(`Pass rate: ${passCount}/${n} (${pct(n ? passCount / n : NaN)}), threshold ${pct(meta.passThreshold)}`);
  lines.push(
    `Score: mean ${num(mean(scores))} | min ${num(Math.min(...scores))} | max ${num(Math.max(...scores))} | stddev ${num(stddev(scores))}`,
  );
  const structRuns = verdicts.filter((v) => v.structure && !v.structure.skipped);
  if (structRuns.length) {
    const structOk = structRuns.filter((v) => v.structure.ok).length;
    lines.push(`Structure gate: ${structOk}/${structRuns.length} ok`);
  }
  const validRuns = verdicts.filter((v) => v.validity && !v.validity.skipped);
  if (validRuns.length) {
    const validOk = validRuns.filter((v) => v.validity.ok).length;
    lines.push(`Validity gate: ${validOk}/${validRuns.length} ok`);
  }

  // Per-run one-liners.
  lines.push('');
  lines.push('Per run:');
  verdicts.forEach((v, i) => {
    const crit = (v.critical_findings ?? []).length;
    const st = v.structure && !v.structure.skipped ? (v.structure.ok ? 'ok' : 'FAIL') : '-';
    const val = v.validity && !v.validity.skipped ? (v.validity.ok ? 'ok' : 'FAIL') : '-';
    lines.push(
      `  ${String(i + 1).padStart(2)}: ${v.pass ? 'PASS' : 'FAIL'}  score ${num(v.score, 2)}  critical ${crit}  struct ${st}  valid ${val}`,
    );
  });

  // Per-dimension means (spot weak/variable rubric dimensions).
  const dimScores = new Map();
  for (const v of verdicts) {
    const rubric = v.rubric_scores ?? {};
    for (const [dim, r] of Object.entries(rubric)) {
      const s = r?.score;
      if (typeof s !== 'number' || !Number.isFinite(s)) continue;
      if (!dimScores.has(dim)) dimScores.set(dim, []);
      dimScores.get(dim).push(s);
    }
  }
  if (dimScores.size) {
    lines.push('');
    lines.push('Rubric dimensions (mean [min-max] stddev, n):');
    const rows = [...dimScores.entries()].sort((a, b) => mean(a[1]) - mean(b[1]));
    for (const [dim, xs] of rows) {
      lines.push(
        `  - ${dim}: ${num(mean(xs), 2)} [${num(Math.min(...xs), 2)}-${num(Math.max(...xs), 2)}] sd ${num(stddev(xs), 2)} (n=${xs.length})`,
      );
    }
  }

  // Recurring findings across runs, each traceable to the run numbers that
  // produced it (inspect those saved candidates with --save).
  const fmtRuns = (runs) => `[runs ${runs.join(',')}]`;

  const critFreq = tallyWithRuns(verdicts, 'critical_findings', (f) => f.issue);
  if (critFreq.length) {
    lines.push('');
    lines.push('Recurring critical findings (count x issue):');
    for (const [issue, e] of critFreq.slice(0, 10)) lines.push(`  - ${e.count}x  ${fmtRuns(e.runs)}  ${issue}`);
  }

  const missFreq = tallyWithRuns(verdicts, 'missing_evidence', (m) => (typeof m === 'string' ? m : m?.issue));
  if (missFreq.length) {
    lines.push('');
    lines.push('Recurring missing evidence (count x item):');
    for (const [item, e] of missFreq.slice(0, 10)) lines.push(`  - ${e.count}x  ${fmtRuns(e.runs)}  ${item}`);
  }

  const warnFreq = tallyWithRuns(verdicts, 'warnings', (f) => f.issue);
  if (warnFreq.length) {
    lines.push('');
    lines.push('Recurring warnings (count x issue):');
    for (const [issue, e] of warnFreq.slice(0, 10)) lines.push(`  - ${e.count}x  ${fmtRuns(e.runs)}  ${issue}`);
  }

  // Deterministic structure-gate findings, traceable to the runs that produced
  // them (same shape as the finding tallies above).
  const structFindings = verdicts.map((v) => ({ structure_findings: v.structure?.findings ?? [] }));
  const structFreq = tallyWithRuns(structFindings, 'structure_findings', (f) => f.issue);
  if (structFreq.length) {
    lines.push('');
    lines.push('Recurring structure findings (count x issue):');
    for (const [issue, e] of structFreq.slice(0, 10)) lines.push(`  - ${e.count}x  ${fmtRuns(e.runs)}  ${issue}`);
  }

  // Deterministic validity-gate findings (unrenderable specs), same shape.
  const validFindings = verdicts.map((v) => ({ validity_findings: v.validity?.findings ?? [] }));
  const validFreq = tallyWithRuns(validFindings, 'validity_findings', (f) => f.issue);
  if (validFreq.length) {
    lines.push('');
    lines.push('Recurring validity findings (count x issue):');
    for (const [issue, e] of validFreq.slice(0, 10)) lines.push(`  - ${e.count}x  ${fmtRuns(e.runs)}  ${issue}`);
  }

  lines.push('');
  return lines.join('\n');
}
