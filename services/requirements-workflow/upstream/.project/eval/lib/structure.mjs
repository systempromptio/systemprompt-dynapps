// Deterministic structural gate for generated artifacts.
//
// The LLM judge scores content fidelity but is unreliable at exact section
// presence, ordering, and table columns (see the opsx-eval sensitivity probes,
// group B: a whole missing section only dented one rubric dimension, and a
// reorder / extra section went unnoticed). This module adds a code-level gate
// derived from the GOLDEN fixture — the single canon in this eval — so the
// candidate's required sections, their order, and each section's table columns
// must match the golden's.
//
// The golden is deliberately the source of truth (not the schema template): the
// golden+judge loop is "change the golden to define what we want, tune the
// judge, then drag the sources up to the golden". Editing the golden fixture
// therefore re-defines the enforced structure automatically, with no second
// list to keep in sync.
//
// The gate is independent of the judge and fully deterministic: the same
// candidate always yields the same result, with no model call. It answers only
// "is the structure right?" — content correctness stays the judge's job.

/**
 * Normalize a heading/cell for identity comparison: fold dash-like Unicode
 * codepoints to an ASCII hyphen (models often emit U+2011 non-breaking hyphen,
 * en/em dashes, or the minus sign in headings like "Cross‑cutting"), drop
 * whitespace around separators, collapse remaining whitespace, then lowercase.
 * Without the dash fold, "Cross‑cutting" (U+2011) and "Cross-cutting" (U+002D)
 * compare unequal and the gate false-fails. Without the separator fold, the
 * template's "RQ · WBS" and a candidate's "RQ·WBS" compare unequal for the same
 * reason — the tokens and their order are identical, only the spacing around the
 * separator differs, which is not a structural difference.
 */
function norm(s) {
  return String(s ?? '')
    .replace(/[\u2010\u2011\u2012\u2013\u2014\u2015\u2212]/g, '-')
    .replace(/\s*([\u00b7\u2022/-])\s*/g, '$1')
    .replace(/\s+/g, ' ')
    .trim()
    .toLowerCase();
}

/** Split a markdown table row into trimmed cells, dropping the edge pipes. */
function splitRow(line) {
  let s = line.trim();
  if (s.startsWith('|')) s = s.slice(1);
  if (s.endsWith('|')) s = s.slice(0, -1);
  return s.split('|').map((c) => c.trim());
}

/** True for a GFM table separator row like `|---|:--:|` (after the header row). */
function isSeparatorRow(line) {
  if (!/\|/.test(line)) return false;
  const cells = splitRow(line);
  return cells.length > 0 && cells.every((c) => /^:?-+:?$/.test(c));
}

/**
 * Parse a markdown doc into its ordered level-2 (`##`) sections. For each section
 * capture the header cells of the FIRST markdown table it contains (or `null`
 * when the section has no table, e.g. a prose handoff or a checkbox gate).
 * Fenced code blocks are skipped so a `##` or `|` inside a fence is never
 * mistaken for structure.
 * @param {string} md
 * @returns {Array<{ title: string, columns: string[]|null }>}
 */
export function parseSections(md) {
  const lines = String(md ?? '').split(/\r?\n/);
  const sections = [];
  let current = null;
  let inFence = false;
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    if (/^\s*(```|~~~)/.test(line)) {
      inFence = !inFence;
      continue;
    }
    if (inFence) continue;

    const heading = /^##\s+(.+?)\s*#*\s*$/.exec(line); // level-2 only (`##`, not `#`/`###`)
    if (heading) {
      current = { title: heading[1].trim(), columns: null, none: false };
      sections.push(current);
      continue;
    }

    // First table of the section: a pipe header row followed by a separator row.
    if (current && current.columns === null && /^\s*\|.*\|\s*$/.test(line) && isSeparatorRow(lines[i + 1] ?? '')) {
      current.columns = splitRow(line);
    } else if (current && current.columns === null && /^\s*none\.?\s*$/i.test(line)) {
      // A section whose only body is the template-sanctioned "None." marker (no
      // table) — legitimate for table-optional sections (Open Questions, Change
      // Request Candidates, Dependencies). Record it so the column gate accepts it
      // instead of hard-failing a correctly-empty section.
      current.none = true;
    }
  }
  return sections;
}

/**
 * Parse the Document Consumption Guide row set from a design/design-reconcile doc:
 * the section names in the guide table's FIRST column (its data rows). The guide
 * is the table whose header is `Section | What it settles… | Key data`; we key off
 * a first column normalising to `section` plus a `key data` column so another
 * table is never mistaken for it. Fenced blocks are skipped. Returns the ordered
 * list of first-cell values (verbatim), or `[]` when the doc carries no guide.
 * @param {string} md
 * @returns {string[]}
 */
export function parseDcgRows(md) {
  const lines = String(md ?? '').split(/\r?\n/);
  let inFence = false;
  let inGuide = false;
  const rows = [];
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    if (/^\s*(```|~~~)/.test(line)) {
      inFence = !inFence;
      continue;
    }
    if (inFence) continue;
    if (!inGuide) {
      // Locate the guide header: a pipe row `Section | … | Key data` followed by a
      // separator row.
      if (/^\s*\|.*\|\s*$/.test(line) && isSeparatorRow(lines[i + 1] ?? '')) {
        const cells = splitRow(line);
        if (cells.length >= 2 && norm(cells[0]) === 'section' && cells.some((c) => norm(c) === 'key data')) {
          inGuide = true;
          i += 1; // skip the separator row on the next iteration boundary
        }
      }
      continue;
    }
    // In the guide table: collect data rows until the table ends.
    if (/^\s*\|.*\|\s*$/.test(line)) {
      if (isSeparatorRow(line)) continue;
      const first = splitRow(line)[0] ?? '';
      // Strip inline emphasis/backticks so `**Risks**` and `Risks` compare equal.
      const name = first.replace(/[`*_]/g, '').trim();
      if (name) rows.push(name);
    } else if (line.trim() === '') {
      continue; // a blank line inside/after the table — keep scanning briefly
    } else {
      break; // a non-table, non-blank line ends the guide
    }
  }
  return rows;
}

/**
 * Parse the `### ` subsection titles under a `## Integration Specification`
 * section, in document order, EXCLUDING the `### Overview` (the signpost block
 * itself). This is the authoritative set the signpost must name — it is
 * machine-parseable from the body, unlike the signpost's prose. Stops at the next
 * `##`. Fenced blocks are skipped. Returns `[]` when the doc has no Integration
 * Specification (a functional design).
 * @param {string} md
 * @returns {string[]}
 */
export function parseIntegrationSubsections(md) {
  const lines = String(md ?? '').split(/\r?\n/);
  let inFence = false;
  let inSection = false;
  const subs = [];
  for (const line of lines) {
    if (/^\s*(```|~~~)/.test(line)) {
      inFence = !inFence;
      continue;
    }
    if (inFence) continue;
    const h2 = /^##\s+(.+?)\s*#*\s*$/.exec(line);
    if (h2) {
      inSection = /integration specification/.test(norm(h2[1]));
      continue;
    }
    if (!inSection) continue;
    const h3 = /^###\s+(.+?)\s*#*\s*$/.exec(line);
    if (h3) {
      const title = h3[1].replace(/[`*_]/g, '').trim();
      if (norm(title) !== 'overview') subs.push(title);
    }
  }
  return subs;
}

/**
 * Extract the candidate's `### Overview` signpost TEXT — the `Read in order:`
 * sentence, which wraps across lines until a blank line or the next heading/table.
 * Returned verbatim (joined) for substring presence checks; the signpost is prose
 * (arrows + parenthetical subsection names), not a parseable list, so we test
 * whether each authoritative subsection NAME appears in it rather than splitting
 * it. Returns '' when the doc carries no signpost.
 * @param {string} md
 * @returns {string}
 */
export function parseSignpostText(md) {
  const lines = String(md ?? '').split(/\r?\n/);
  let inFence = false;
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    if (/^\s*(```|~~~)/.test(line)) {
      inFence = !inFence;
      continue;
    }
    if (inFence) continue;
    const m = /read in order\s*:\s*(.*)$/i.exec(line);
    if (!m) continue;
    const buf = [m[1]];
    for (let j = i + 1; j < lines.length; j++) {
      const next = lines[j];
      if (next.trim() === '' || /^\s*(#{1,6}\s|\||```|~~~)/.test(next)) break;
      buf.push(next);
    }
    return buf.join(' ').replace(/\s+/g, ' ').trim();
  }
  return '';
}

/**
 * Deterministic presence facts for the design-reconcile judge. The LLM judge
 * hallucinates that a Document Consumption Guide row or an Integration
 * Specification subsection the candidate DOES carry is "missing"/"dropped";
 * feeding it harness-parsed presence removes the eyeballing.
 *
 * - DCG row set: the candidate's guide rows diffed against the golden's (both are
 *   clean table first-columns, so an exact set diff is reliable).
 * - Integration signpost: the authoritative subsection set is the body's
 *   `### ` headings under `## Integration Specification` (machine-parseable),
 *   tested for NAME presence inside the candidate's prose `Read in order:` signpost
 *   (substring, dash/space-folded) — robust to the signpost's arrow/parenthetical
 *   phrasing where a comma-split would mangle it.
 *
 * Returns '' when the golden anchors no guide and the body no integration section.
 * @param {string} candidateMd the candidate under judgement
 * @param {string} goldenMd the golden projection (guide + signpost)
 * @param {string} bodyMd the design-in-review body (authoritative structure)
 * @returns {string}
 */
export function deriveStructureFacts(candidateMd, goldenMd, bodyMd = '') {
  const goldRows = parseDcgRows(goldenMd);
  const candRows = parseDcgRows(candidateMd);
  const bodySubs = parseIntegrationSubsections(bodyMd);
  const signpost = parseSignpostText(candidateMd);
  if (!goldRows.length && !bodySubs.length) return '';

  const fmt = (list) => (list.length ? list.join(', ') : '(none)');
  const parts = [];
  parts.push(
    'The lists below were computed DETERMINISTICALLY by the harness by parsing the ' +
      'CANDIDATE, the GOLDEN, and the body. They are GROUND TRUTH for PRESENCE: never ' +
      'record a row or subsection this analysis marks PRESENT-IN-CANDIDATE as ' +
      '"missing"/"dropped", and never claim one it marks ABSENT is present. Judge ' +
      'staleness/handoff on top of these facts, do not re-derive presence by eye.',
  );

  if (goldRows.length) {
    const candNorm = new Set(candRows.map(norm));
    const goldNorm = new Set(goldRows.map(norm));
    const missing = goldRows.filter((g) => !candNorm.has(norm(g)));
    const extra = candRows.filter((c) => !goldNorm.has(norm(c)));
    parts.push(
      'Document Consumption Guide row set (guide table, first column):\n' +
        `- GOLDEN rows (${goldRows.length}): ${fmt(goldRows)}\n` +
        `- CANDIDATE rows (${candRows.length}): ${fmt(candRows)}\n` +
        `- Rows in GOLDEN but NOT in candidate (genuinely dropped): ${fmt(missing)}\n` +
        `- Rows in candidate but NOT in GOLDEN (extra): ${fmt(extra)}`,
    );
  }
  if (bodySubs.length) {
    const sign = norm(signpost);
    const named = bodySubs.filter((s) => sign.includes(norm(s)));
    const unnamed = bodySubs.filter((s) => !sign.includes(norm(s)));
    parts.push(
      'Integration Specification `### Overview` signpost — the body carries these ' +
        '`###` subsections (authoritative set the signpost must name):\n' +
        `- Body subsections (${bodySubs.length}): ${fmt(bodySubs)}\n` +
        `- Named in the candidate's "Read in order:" signpost (PRESENT): ${fmt(named)}\n` +
        `- NOT named in the candidate's signpost (genuinely omitted): ${fmt(unnamed)}` +
        (signpost ? '' : '\n- (no "Read in order:" signpost found in the candidate)'),
    );
  }
  return parts.join('\n\n');
}

/** Exact column match: same count, same names, same order (whitespace/case-insensitive). */
function columnsMatch(expected, got) {
  if (!Array.isArray(got) || expected.length !== got.length) return false;
  return expected.every((c, i) => norm(c) === norm(got[i]));
}

/**
 * Split a multi-file bundle (marker lines `--- <path> ---`, e.g. a specs delta
 * bundle) into a Map keyed by capability path — the marker path with its first
 * segment dropped, so a golden `specs.ba/store-locator/spec.md` and a candidate
 * `specs/store-locator/spec.md` match on `store-locator/spec.md` (the flow-suffixed
 * prefix is a harness artifact, not structure). Returns null when there are no
 * markers, so single-file artifacts fall through to the plain section check.
 * @param {string} md
 * @returns {Map<string,string>|null}
 */
function splitBundle(md) {
  const lines = String(md ?? '').split(/\r?\n/);
  const marker = /^---\s+(\S.*?)\s+---\s*$/;
  const files = new Map();
  let key = null;
  let buf = [];
  let seen = false;
  const flush = () => {
    if (key === null) return;
    // Append (not overwrite) when a capability path repeats: models sometimes
    // re-emit a file's marker (e.g. a trailing empty duplicate, or the same
    // capability split across two marker chunks). Concatenating preserves the
    // real content instead of letting an empty trailing duplicate clobber it.
    const body = buf.join('\n');
    files.set(key, files.has(key) ? `${files.get(key)}\n${body}` : body);
  };
  for (const line of lines) {
    const m = marker.exec(line);
    if (m) {
      seen = true;
      flush();
      const parts = m[1].trim().split('/');
      key = parts.length > 1 ? parts.slice(1).join('/') : parts[0];
      buf = [];
    } else if (key !== null) {
      buf.push(line);
    }
  }
  flush();
  return seen ? files : null;
}

/**
 * Check one document's `##` sections against a golden document: every golden
 * section present, in the golden's order, with matching table columns, and no
 * unexpected extra sections. When the golden has no parseable sections the check
 * is skipped (ok), so it never blocks a case that ships no golden to anchor structure.
 * @param {string} candidateMd
 * @param {string} goldenMd
 * @returns {{ ok: boolean, findings: Array<{ issue: string, evidence?: string }>, skipped?: boolean }}
 */
function checkSections(candidateMd, goldenMd) {
  const expected = parseSections(goldenMd);
  if (!expected.length) return { ok: true, findings: [], skipped: true };

  const actual = parseSections(candidateMd);

  // A candidate with NO parseable top-level sections (an empty artifact, or a
  // refusal that reached the gate) is a single structural FAIL — collapse it to
  // one finding instead of re-listing every required section as "missing", which
  // only spams the aggregate recurring-findings without adding signal. A PARTIAL
  // candidate (some sections present) still gets the full per-section diff below,
  // so every approved section variant is checked and reported.
  if (!actual.length) {
    return {
      ok: false,
      findings: [{
        issue: 'No artifact structure: none of the required sections are present',
        evidence: `expected ${expected.length} sections starting with "## ${expected[0].title}"`,
      }],
    };
  }

  const actualByTitle = new Map();
  actual.forEach((s, index) => {
    const key = norm(s.title);
    if (!actualByTitle.has(key)) actualByTitle.set(key, { ...s, index });
  });

  const findings = [];
  const matchedOrder = [];

  // 1. Presence + columns.
  for (const exp of expected) {
    const got = actualByTitle.get(norm(exp.title));
    if (!got) {
      findings.push({ issue: `Missing required section "## ${exp.title}"`, evidence: 'section not found in candidate' });
      continue;
    }
    matchedOrder.push({ title: exp.title, index: got.index });
    if (exp.columns) {
      if (!got.columns) {
        if (!got.none) {
          findings.push({
            issue: `Section "## ${exp.title}" is missing its table`,
            evidence: `expected columns: ${exp.columns.join(' | ')}`,
          });
        }
        // else: a table-optional section legitimately empty, written as the
        // template-sanctioned "None." marker — structurally satisfied.
      } else if (!columnsMatch(exp.columns, got.columns)) {
        findings.push({
          issue: `Section "## ${exp.title}" has wrong table columns`,
          evidence: `expected: ${exp.columns.join(' | ')} — got: ${got.columns.join(' | ')}`,
        });
      }
    }
  }

  // 2. Order: matched sections must appear in the candidate in the template order.
  for (let i = 1; i < matchedOrder.length; i++) {
    if (matchedOrder[i].index < matchedOrder[i - 1].index) {
      findings.push({
        issue: `Section "## ${matchedOrder[i].title}" is out of order`,
        evidence: `expected to appear after "## ${matchedOrder[i - 1].title}"`,
      });
    }
  }

  // 3. No unexpected extra top-level sections (strict structure).
  const expectedTitles = new Set(expected.map((e) => norm(e.title)));
  for (const s of actual) {
    if (!expectedTitles.has(norm(s.title))) {
      findings.push({ issue: `Unexpected section "## ${s.title}"`, evidence: 'not part of the template structure' });
    }
  }

  return { ok: findings.length === 0, findings };
}

/**
 * Check a candidate's structure against the GOLDEN fixture's structure.
 *
 * Single-file artifacts (discovery, design, proposal, …): the candidate's `##`
 * sections must match the golden's — presence, order, table columns, no extras.
 *
 * Multi-file bundles (a specs delta bundle, emitted with `--- <path> ---` file
 * markers): each capability file is checked INDEPENDENTLY against its golden
 * counterpart (matched by capability path). Cross-file section ORDER is NOT a
 * contract — OpenSpec archives each capability file on its own, merging by
 * capability + requirement heading, so the order in which the model emits the
 * files carries no meaning. Comparing a concatenation would (and did) raise
 * false "## ADDED Requirements is out of order" fails whenever the model emitted
 * the capability files in a different order than the golden. File inventory
 * (missing / extra capability files) stays the judge's `file_inventory` law, so
 * unmatched files are ignored here rather than double-flagged.
 * @param {string} candidateMd
 * @param {string} goldenMd the golden fixture markdown (the structural canon)
 * @returns {{ ok: boolean, findings: Array<{ issue: string, evidence?: string }>, skipped?: boolean }}
 */
export function checkStructure(candidateMd, goldenMd) {
  const goldenFiles = splitBundle(goldenMd);
  if (!goldenFiles) return checkSections(candidateMd, goldenMd);

  const candidateFiles = splitBundle(candidateMd) ?? new Map();
  const findings = [];
  let checkedAny = false;
  for (const [key, goldenBody] of goldenFiles) {
    if (!candidateFiles.has(key)) continue; // inventory is the judge's file_inventory law
    checkedAny = true;
    const res = checkSections(candidateFiles.get(key), goldenBody);
    for (const f of res.findings) findings.push({ ...f, issue: `${key}: ${f.issue}` });
  }
  return checkedAny ? { ok: findings.length === 0, findings } : { ok: true, findings: [], skipped: true };
}
