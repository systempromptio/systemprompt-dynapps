#!/usr/bin/env node
/**
 * Deterministic test for generate-integration-artifacts.mjs (ISD In-Scope section + stories).
 *
 * Materializes a temp change-dir from the store-locator `sa` eval fixture
 * (proposal.sa.md -> proposal.md, specs.sa/ -> specs/, design.sa.md -> design.md),
 * then asserts:
 *   - clean validation on the golden sa inputs; single ISD target with 2 blocks;
 *   - ISD bootstrap: chrome preserved, In-Scope Story + AC verbatim, 3PI.MAP.GEO
 *     aggregates geocoding + privacy scenarios (shared WBS) under one bold group
 *     row per requirement name with continuous numbering, a single-requirement
 *     block staying flat, AND-collapse, and NO Visualization sub-section anywhere;
 *   - the REQUIREMENT PARTITION checks: a duplicate name is GROUP_LABEL_DUPLICATE,
 *     a one-scenario cross-cutting group raises no THIN_GROUP;
 *   - bootstrap -> surgical update is a byte-identical no-op;
 *   - surgical reconcile handles CHANGE + ADD + DELETE, preserving LLM enrichment
 *     outside the In-Scope section;
 *   - inserting the In-Scope section into a legacy ISD that lacks it (insertBefore);
 *   - the committed isd-scaffold.sa.md drift guard (run with --update to refresh);
 *   - broken inputs raise the expected error codes (SPEC_HEADING_NOT_OPTION_H,
 *     SLICE_NO_SPEC) and the CLI exits non-zero;
 *   - the CLI writes BOTH the ISD and stories.md in one run; a clean re-run is noop;
 *   - stories.md is projected (one story per slice) with the shopper Story verbatim
 *     and an ISD acceptance-criteria reference; a re-run PRESERVES a backfilled Jira
 *     key + `- Jira:` link + the created line.
 *
 * Run: node skills/plan/openspec-workflow/scripts/generate-integration-artifacts.test.mjs [--update]
 */

import assert from 'node:assert/strict'
import { cpSync, mkdtempSync, mkdirSync, writeFileSync, readFileSync, rmSync, existsSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { spawnSync } from 'node:child_process'

import { buildModel, validate } from './generate-integration-artifacts.mjs'
import { bootstrap, surgicalUpdate, parseIsdDoc, splitIsdTemplate } from './lib/isd.mjs'

const HERE = dirname(fileURLToPath(import.meta.url))
const ROOT = resolve(HERE, '../../../..')
const FIX = join(ROOT, '.project/eval/evalsuite/openspec-feature/fixtures/store-locator')
const TMPL = join(ROOT, '.project/templates/openspec/schemas/feature/templates/isd.md')
const SCRIPT = join(HERE, 'generate-integration-artifacts.mjs')

let passed = 0
const check = (label, fn) => {
  fn()
  passed += 1
  console.log(`  ok  ${label}`)
}

function makeChange() {
  const dir = mkdtempSync(join(tmpdir(), 'opsx-ia-'))
  cpSync(join(FIX, 'proposal/final.sa.md'), join(dir, 'proposal.md'))
  mkdirSync(join(dir, 'specs'), { recursive: true })
  cpSync(join(FIX, 'specs.sa'), join(dir, 'specs'), { recursive: true })
  if (existsSync(join(FIX, 'design/final.sa.md'))) cpSync(join(FIX, 'design/final.sa.md'), join(dir, 'design.md'))
  return dir
}
const runCli = (dir) => spawnSync('node', [SCRIPT, dir, '--template', TMPL], { encoding: 'utf8' })

const cleanups = []
const track = (d) => (cleanups.push(d), d)

try {
  const change = track(makeChange())
  const model = buildModel(change)

  check('validation has no errors on the golden sa inputs', () => {
    const errs = validate(model).filter((f) => f.severity === 'error')
    assert.equal(errs.length, 0, 'unexpected errors: ' + JSON.stringify(errs, null, 2))
  })
  check('single ISD target with 4 blocks resolved from slices', () => {
    assert.equal(model.targets.length, 1)
    assert.equal(model.targets[0].blocks.length, 4)
  })
  check('lenient dash: hyphen-separated slice headings still parse the block name', () => {
    const c = track(makeChange())
    const pp = join(c, 'proposal.md')
    // simulate an author/LLM that typed a plain hyphen instead of the canonical em-dash
    const mutated = readFileSync(pp, 'utf8')
      .split('\n')
      .map((l) => (/^### S\d+/.test(l) ? l.replaceAll(' — ', ' - ') : l))
      .join('\n')
    writeFileSync(pp, mutated)
    const m = buildModel(c)
    assert.equal(
      validate(m).filter((f) => f.severity === 'error').length,
      0,
      'hyphen-separated slice headings should still validate clean',
    )
    for (const b of m.targets[0].blocks) {
      assert.ok(b.name && b.name !== 'null', `block ${b.wbs} name not parsed from a hyphen heading: ${b.name}`)
    }
  })

  const template = readFileSync(TMPL, 'utf8')
  const md = bootstrap(template, model.targets[0].blocks, model.legend)

  check('ISD chrome preserved + Story/AC/table projected, no Visualization', () => {
    assert.match(md, /# <Service \/ Capability Name> ISD/)
    assert.ok(md.includes('## General ISD Information'), 'ISD chrome (General ISD Information) missing')
    assert.ok(md.includes('## Integration Specification'), 'ISD Integration Specification chrome missing')
    assert.ok(md.includes('As a shopper, I want to search for stores by postal code, city, or my current location, so that I see the nearest stores first.'))
    assert.ok(md.includes('The capability SHALL expose a stable interface that resolves a postal code or city to coordinates'))
    assert.ok(!md.includes('#### Visualization'), 'ISD blocks must NOT carry a Visualization sub-section')
  })
  check('3PI.MAP.GEO block aggregates shared-WBS (geocoding + privacy) scenarios; AND collapses via <br>', () => {
    const block = md.slice(md.indexOf('### 3PI.MAP.GEO'), md.indexOf('### 3PI.MAP.RND'))
    assert.ok(block.includes('Valid postcode resolves to proximity-ranked results'), 'geocoding scenario missing')
    assert.ok(block.includes('Coordinates are not persisted or sent to analytics'), 'shared-WBS privacy scenario not aggregated')
    assert.ok(md.includes("the shopper's coordinates are not persisted by the capability<br>the shopper's coordinates are not sent to analytics"), 'AND not collapsed via <br>')
  })
  check('each shared-WBS requirement opens a bold group row; numbering continuous; single-requirement blocks stay flat', () => {
    const geo = md.slice(md.indexOf('### 3PI.MAP.GEO'), md.indexOf('### 3PI.MAP.RND'))
    assert.deepEqual(
      [...geo.matchAll(/^\| \*\*(.+?)\*\* \|/gm)].map((m) => m[1]),
      ['Postcode and city geocoding with proximity ranking', 'Shopper coordinate privacy'],
    )
    assert.deepEqual([...geo.matchAll(/^\| (\d+) \|/gm)].map((m) => m[1]), ['1', '2', '3', '4', '5'])
    assert.ok(geo.indexOf('| **Shopper coordinate privacy** |') < geo.indexOf('Coordinates are not persisted or sent to analytics'))
    const rnd = md.slice(md.indexOf('### 3PI.MAP.RND'), md.indexOf('### 3PI.MAP.STY'))
    assert.ok(!/^\| \*\*/m.test(rnd), 'a group row appeared in a single-requirement block')
  })
  check('a duplicate requirement name on one WBS raises GROUP_LABEL_DUPLICATE; a one-scenario privacy group raises no THIN_GROUP', () => {
    assert.equal(validate(model).filter((f) => ['GROUP_LABEL_DUPLICATE', 'MULTIPLE_MANAGEMENT_GROUPS', 'THIN_GROUP'].includes(f.code)).length, 0)
    const bad = track(makeChange())
    const specPath = join(bad, 'specs/cartora-maps-integration/spec.md')
    writeFileSync(specPath, readFileSync(specPath, 'utf8')
      .replace('### Requirement: 3PI.MAP.GEO — Shopper coordinate privacy',
        '### Requirement: 3PI.MAP.GEO — Postcode and city geocoding with proximity ranking'))
    assert.ok(validate(buildModel(bad)).some((f) => f.code === 'GROUP_LABEL_DUPLICATE' && f.severity === 'error'))
  })
  check('In-Scope section sits directly after Integration Specification, before Out Of Scope', () => {
    assert.ok(md.indexOf('## Integration Specification') < md.indexOf('## In Scope Functional Requirements'))
    assert.ok(md.indexOf('## In Scope Functional Requirements') < md.indexOf('## Out Of Scope / Limitations'))
  })
  check('bootstrap -> surgical update is a byte-identical no-op', () => {
    assert.equal(surgicalUpdate(md, model.targets[0].blocks, null, null, model.legend), md)
  })
  check('the proposal legend is projected once, ahead of the first block, and re-projected over drift', () => {
    assert.equal((md.match(/^\*\*Re-usability Legend\*\*$/gm) || []).length, 1)
    assert.ok(md.includes('- `[3rd party OOTB]` — delivered by the third-party service as licensed, consumed as published'))
    assert.ok(md.indexOf('**Re-usability Legend**') < md.indexOf('### 3PI.MAP.GEO'))
    assert.ok(md.indexOf('## In Scope Functional Requirements') < md.indexOf('**Re-usability Legend**'))
    const tampered = md.replace('- `[New]` — brand-new functionality, built from scratch', '- `[New]` — hand-edited gloss')
    const restored = surgicalUpdate(tampered, model.targets[0].blocks, null, null, model.legend)
    assert.equal(restored, md, 'a hand-edited legend was not re-projected from the proposal')
    assert.ok(validate({ ...model, targets: [{ ...model.targets[0], existing: parseIsdDoc(tampered) }] })
      .some((f) => f.code === 'READ_ONLY_DRIFT' && /Re-usability Legend/.test(f.where)),
      'a hand-edited legend raised no READ_ONLY_DRIFT warning')
  })

  // Feedback round-trip: CHANGE / ADD / DELETE reconcile on an ENRICHED ISD
  check('surgical reconcile handles CHANGE + ADD + DELETE, preserving enrichment', () => {
    // enrich the bootstrap output with a sentinel in a non-owned (LLM) section
    const enriched = md.replace('# <Service / Capability Name> ISD', '# Cartora Maps Integration ISD')
      .replace('<One-paragraph intro: what this section specifies.>', 'ENRICHED-INTRO-SENTINEL')
    const blockTemplate = splitIsdTemplate(readFileSync(TMPL, 'utf8')).blockTemplate
    const kept = parseIsdDoc(enriched).blocks
      .filter((b) => b.inScope && b.wbs && b.wbs !== '3PI.MAP.RND') // DELETE 3PI.MAP.RND
      .map((b) => ({ wbs: b.wbs, name: b.name, userStory: `Story ${b.wbs}`, acceptance: `1. AC ${b.wbs}`, groups: [] }))
    kept.push({ wbs: '3PI.MAP.NEW', name: 'Brand new integration point', userStory: 'New story', acceptance: '1. New AC', groups: [] }) // ADD
    const out = surgicalUpdate(enriched, kept, blockTemplate)
    assert.ok(!/^### 3PI\.MAP\.RND /m.test(out), 'DELETE: 3PI.MAP.RND block still present')
    assert.ok(/^### 3PI\.MAP\.NEW — Brand new integration point/m.test(out), 'ADD: new block not inserted')
    assert.ok(out.includes('Story 3PI.MAP.GEO'), 'CHANGE: 3PI.MAP.GEO User Story not updated')
    assert.ok(out.includes('ENRICHED-INTRO-SENTINEL'), 'enrichment: Integration Specification intro not preserved')
    assert.ok(out.includes('## Integration Specification') && out.includes('## Open Questions'), 'LLM-authored ISD sections not preserved')
  })

  // Insert the In-Scope section into a LEGACY ISD that lacks it (insertBefore branch)
  check('inserts the In-Scope section into a legacy ISD that has none (before Out Of Scope)', () => {
    // derive a section-less legacy ISD from the golden (strip the In-Scope section if present)
    const golden = readFileSync(join(FIX, 'isd.sa/cartora-maps-integration.md'), 'utf8')
    const legacy = golden.replace(/\n## In Scope Functional Requirements[\s\S]*?(?=\n## Out Of Scope \/ Limitations)/, '\n')
    assert.ok(!legacy.includes('## In Scope Functional Requirements'), 'failed to derive a section-less legacy ISD')
    const { blockTemplate, sectionPreamble } = splitIsdTemplate(readFileSync(TMPL, 'utf8'))
    const out = surgicalUpdate(legacy, model.targets[0].blocks, blockTemplate, sectionPreamble)
    assert.ok(out.includes('## In Scope Functional Requirements'), 'section not inserted')
    assert.ok(out.indexOf('## In Scope Functional Requirements') < out.indexOf('## Out Of Scope / Limitations'), 'section not placed before Out Of Scope / Limitations')
    assert.ok(/^### 3PI\.MAP\.GEO — /m.test(out) && /^### 3PI\.MAP\.RND — /m.test(out), 'blocks not inserted')
    assert.ok(out.includes('## Integration Specification'), 'legacy body not preserved')
  })

  // EXTEND (additive / --extend): prior in-scope blocks are preserved, not stripped
  check('additive (--extend) keeps prior in-scope blocks, updates matched, appends new', () => {
    const blockTemplate = splitIsdTemplate(readFileSync(TMPL, 'utf8')).blockTemplate
    const enriched = md.replace('<One-paragraph intro: what this section specifies.>', 'ENRICHED-EXTEND-SENTINEL')
    const priorWbs = parseIsdDoc(enriched).blocks.filter((b) => b.inScope && b.wbs).map((b) => b.wbs)
    const keep = priorWbs[0]
    // this change covers ONE prior WBS (an update) + one brand-new WBS; the other priors are untouched
    const blocks = [
      { wbs: keep, name: 'ignored on update', userStory: 'EXTEND-UPDATED-STORY', acceptance: '1. EXTEND-UPDATED-AC', groups: [] },
      { wbs: '3PI.MAP.EXT', name: 'Extend-only new point', userStory: 'new', acceptance: '1. new', groups: [] },
    ]
    const out = surgicalUpdate(enriched, blocks, blockTemplate, null, model.legend, { additive: true })
    for (const w of priorWbs) assert.match(out, new RegExp('^### ' + w.replace(/\./g, '\\.') + ' ', 'm'), `prior block ${w} dropped on extend`)
    assert.match(out, /^### 3PI\.MAP\.EXT — Extend-only new point/m)
    assert.ok(out.includes('EXTEND-UPDATED-STORY'), 'matched block read-only body not updated on extend')
    assert.ok(out.includes('ENRICHED-EXTEND-SENTINEL'), 'LLM enrichment not preserved on extend')
  })
  check('default mode WARNS an uncovered prior in-scope block will be removed; --extend flips it to preserved', () => {
    const OLD = '### 3PI.MAP.OLD — Retired point\n\n#### User Story\n\nold\n\n#### Acceptance Criteria\n\n1. old\n\n#### Key behaviours\n\n(none)\n\n'
    const existing = parseIsdDoc(md.replace('## Out Of Scope / Limitations', OLD + '## Out Of Scope / Limitations'))
    const tmodel = { ...model, targets: [{ ...model.targets[0], existing }] }
    const def = validate(tmodel, false)
    assert.ok(def.some((f) => f.code === 'UNMATCHED_BLOCK_WILL_BE_REMOVED' && /3PI\.MAP\.OLD/.test(f.where)), 'default mode should warn removal')
    assert.equal(def.filter((f) => f.severity === 'error').length, 0, 'the removal warning must be non-blocking')
    const ext = validate(tmodel, true)
    assert.ok(ext.some((f) => f.code === 'PRIOR_BLOCK_PRESERVED' && /3PI\.MAP\.OLD/.test(f.where)), 'extend mode should report preserved')
    assert.ok(!ext.some((f) => f.code === 'UNMATCHED_BLOCK_WILL_BE_REMOVED'), 'extend mode must not warn removal')
  })

  // eval fixture drift guard
  const SCAFFOLD_FIXTURE = join(FIX, 'isd-scaffold.sa.md')
  if (process.argv.includes('--update')) {
    writeFileSync(SCAFFOLD_FIXTURE, md)
    console.log('  ..  regenerated isd-scaffold.sa.md (--update)')
  }
  check('committed isd-scaffold.sa.md matches the current bootstrap output (drift guard)', () => {
    assert.ok(existsSync(SCAFFOLD_FIXTURE), 'isd-scaffold.sa.md missing — run with --update')
    assert.equal(readFileSync(SCAFFOLD_FIXTURE, 'utf8'), md,
      'isd-scaffold.sa.md is stale — run `node generate-integration-artifacts.test.mjs --update`')
  })

  check('a spec heading without a WBS raises SPEC_HEADING_NOT_OPTION_H', () => {
    const bad = track(makeChange())
    const specPath = join(bad, 'specs/cartora-maps-integration/spec.md')
    writeFileSync(specPath, readFileSync(specPath, 'utf8').replace('### Requirement: 3PI.MAP.RND — Governed map rendering entrypoint', '### Requirement: Governed map rendering entrypoint'))
    assert.ok(validate(buildModel(bad)).map((f) => f.code).includes('SPEC_HEADING_NOT_OPTION_H'))
  })
  check('an in-scope slice with no covering spec raises SLICE_NO_SPEC', () => {
    const bad = track(makeChange())
    const specPath = join(bad, 'specs/cartora-maps-integration/spec.md')
    let spec = readFileSync(specPath, 'utf8')
    // drop both 3PI.MAP.RND-covering requirement(s) so its slice loses coverage
    const from = spec.indexOf('### Requirement: 3PI.MAP.RND')
    const nextIdx = spec.indexOf('### Requirement:', from + 10)
    spec = spec.slice(0, from) + (nextIdx >= 0 ? spec.slice(nextIdx) : '')
    writeFileSync(specPath, spec)
    const findings = validate(buildModel(bad))
    assert.ok(findings.map((f) => f.code).includes('SLICE_NO_SPEC'))
    for (const f of findings) assert.ok(f.where && f.problem && f.fix && f.docs, 'finding missing fields: ' + f.code)
  })

  // CLI end-to-end: BOTH artifacts in one run, then noop
  check('CLI writes ISD + stories.md in one run, then reports noop on a clean re-run', () => {
    const c = track(makeChange())
    const r1 = runCli(c)
    assert.equal(r1.status, 0, 'first run should exit 0; stderr: ' + r1.stderr)
    assert.ok(existsSync(join(c, 'isd/cartora-maps-integration.md')), 'ISD not created')
    assert.ok(existsSync(join(c, 'stories.md')), 'stories.md not created')
    assert.match(r1.stderr, /created ISD cartora-maps-integration\.md \(4\)/)
    assert.match(r1.stderr, /stories\.md \(4\)/)
    const r2 = runCli(c)
    assert.equal(r2.status, 0)
    assert.match(r2.stderr, /noop ISD/)
    assert.match(r2.stderr, /noop stories\.md/)
  })
  check('CLI exits non-zero on a broken change (missing WBS heading)', () => {
    const bad = track(makeChange())
    const specPath = join(bad, 'specs/cartora-maps-integration/spec.md')
    writeFileSync(specPath, readFileSync(specPath, 'utf8').replace('### Requirement: 3PI.MAP.RND — Governed map rendering entrypoint', '### Requirement: Governed map rendering entrypoint'))
    const r = runCli(bad)
    assert.equal(r.status, 1)
    assert.match(r.stderr, /SPEC_HEADING_NOT_OPTION_H/)
  })

  // stories projection (shopper Story verbatim, ISD reference) + Jira preservation
  check('stories.md is projected (one story per slice) with the shopper Story and an ISD reference', () => {
    const c = track(makeChange())
    runCli(c)
    const st = readFileSync(join(c, 'stories.md'), 'utf8')
    assert.match(st, /### TBD-1 — 3PI\.MAP\.GEO — Geocoding with proximity ranking/)
    assert.match(st, /### TBD-2 — 3PI\.MAP\.RND — Governed map rendering/)
    assert.match(st, /### TBD-3 — 3PI\.MAP\.STY — Consumed map presentation \(style and map type\)/)
    assert.match(st, /### TBD-4 — 3PI\.MAP\.MRK — Consumed store markers/)
    assert.ok(st.includes('**User story:** As a shopper, I want to search for stores by postal code, city, or my current location'))
    assert.ok(st.includes('**Acceptance criteria:**'))
    assert.ok(/-\s*ISD:/.test(st), 'stories References should label the covering doc as ISD')
    assert.equal((st.match(/^### /gm) || []).length, 4, 'expected 4 story blocks')
  })
  check('re-run PRESERVES a backfilled Jira key + link + created line', () => {
    const c = track(makeChange())
    runCli(c)
    const p = join(c, 'stories.md')
    let st = readFileSync(p, 'utf8')
    st = st.replace('## Stories', 'Jira tickets created via /opsx:propose-approved: PROJ-1\n\n## Stories')
    st = st.replace('### TBD-1 — 3PI.MAP.GEO', '### PROJ-1 — 3PI.MAP.GEO')
    st = st.replace(/(### PROJ-1 — 3PI\.MAP\.GEO[^\n]*\n)/, '$1\n- Jira: [PROJ-1](https://jira/PROJ-1)\n')
    writeFileSync(p, st)
    runCli(c)
    const after = readFileSync(p, 'utf8')
    assert.match(after, /### PROJ-1 — 3PI\.MAP\.GEO/, 'real Jira key not preserved on re-run')
    assert.ok(after.includes('- Jira: [PROJ-1](https://jira/PROJ-1)'), 'Jira ref line not preserved')
    assert.ok(after.includes('Jira tickets created via /opsx:propose-approved: PROJ-1'), 'created line not preserved')
    assert.ok(!/### TBD-1 — 3PI\.MAP\.GEO/.test(after), 'TBD id resurfaced (clobbered the key)')
  })

  console.log(`\ngenerate-integration-artifacts.test: ${passed} checks passed\n`)
} finally {
  for (const d of cleanups) {
    try {
      rmSync(d, { recursive: true, force: true })
    } catch {}
  }
}
