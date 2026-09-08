#!/usr/bin/env node
/**
 * Deterministic test for generate-functional-artifacts.mjs (merged FSD + stories).
 *
 * Materializes a temp change-dir from the store-locator `ba` eval fixture
 * (proposal.ba.md -> proposal.md, specs.ba/ -> specs/, design.ba.md -> design.md),
 * then asserts:
 *   - clean validation on the golden inputs; 9-block single FSD target;
 *   - FSD read-only projection: Story + AC verbatim, a flat table for a
 *     single-requirement block, one bold group row per requirement name (with
 *     continuous numbering) for a shared-WBS block, GH.UTL cross-capability
 *     aggregation, AND-collapse, bootstrap->surgical no-op;
 *   - the REQUIREMENT PARTITION checks: GROUP_LABEL_DUPLICATE and
 *     MULTIPLE_MANAGEMENT_GROUPS as errors, THIN_GROUP as a warning;
 *   - the committed fsd-scaffold.ba.md drift guard (run with --update to refresh);
 *   - broken inputs raise the expected error codes (SPEC_HEADING_NOT_OPTION_H,
 *     SLICE_NO_SPEC) and the CLI exits non-zero;
 *   - the CLI writes BOTH the FSD and stories.md in one run; a clean re-run is noop;
 *   - stories.md is projected from the block proposal (D7 fix): one story per slice,
 *     with the slice Story and an acceptance-criteria reference;
 *   - a re-run PRESERVES a backfilled Jira key + `- Jira:` link + the created line.
 *
 * Run: node skills/plan/openspec-workflow/scripts/generate-functional-artifacts.test.mjs [--update]
 */

import assert from 'node:assert/strict'
import { cpSync, mkdtempSync, mkdirSync, writeFileSync, readFileSync, rmSync, existsSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { spawnSync } from 'node:child_process'

import { buildModel, validate } from './generate-functional-artifacts.mjs'
import { bootstrap, surgicalUpdate, parseFsdDoc, splitFsdTemplate } from './lib/fsd.mjs'
import { parseDeltaSpecs, renderKeyBehavioursTable } from './lib/specs.mjs'

const HERE = dirname(fileURLToPath(import.meta.url))
const ROOT = resolve(HERE, '../../../..')
const FIX = join(ROOT, '.project/eval/evalsuite/openspec-feature/fixtures/store-locator')
const TMPL = join(ROOT, '.project/templates/openspec/schemas/feature/templates/fsd.md')
const SCRIPT = join(HERE, 'generate-functional-artifacts.mjs')

let passed = 0
const check = (label, fn) => {
  fn()
  passed += 1
  console.log(`  ok  ${label}`)
}

function makeChange() {
  const dir = mkdtempSync(join(tmpdir(), 'opsx-fa-'))
  cpSync(join(FIX, 'proposal/final.ba.md'), join(dir, 'proposal.md'))
  mkdirSync(join(dir, 'specs'), { recursive: true })
  cpSync(join(FIX, 'specs.ba'), join(dir, 'specs'), { recursive: true })
  if (existsSync(join(FIX, 'design/final.ba.md'))) cpSync(join(FIX, 'design/final.ba.md'), join(dir, 'design.md'))
  return dir
}
const runCli = (dir) => spawnSync('node', [SCRIPT, dir, '--template', TMPL], { encoding: 'utf8' })

const cleanups = []
const track = (d) => (cleanups.push(d), d)

try {
  const change = track(makeChange())
  const model = buildModel(change)

  check('validation has no errors on the golden ba inputs', () => {
    const errs = validate(model).filter((f) => f.severity === 'error')
    assert.equal(errs.length, 0, 'unexpected errors: ' + JSON.stringify(errs, null, 2))
  })
  check('single FSD target with 9 blocks resolved from slices', () => {
    assert.equal(model.targets.length, 1)
    assert.equal(model.targets[0].blocks.length, 9)
  })

  const template = readFileSync(TMPL, 'utf8')
  const md = bootstrap(template, model.targets[0].blocks, model.legend)

  check('FSD chrome preserved + Story/AC/table projected', () => {
    assert.match(md, /# <WBS> - <Feature Name> FSD/)
    assert.ok(md.includes('As a shopper, I SHALL be able to search for stores by entering a postal code or a city, or by using my current location.'))
    assert.ok(md.includes('Business users SHALL be able to configure the default search radius and its units in Business Manager.'))
  })
  const tableOf = (wbs) => md.slice(md.indexOf(`### ${wbs}`)).match(/\| # \| Scenario \|[\s\S]*?\n\n/)[0]

  check('a single-requirement block projects a flat table (no group rows)', () => {
    const table = tableOf('SLS.AC')
    assert.ok(!/^\| \*\*/m.test(table), 'a group row appeared in a single-requirement block')
    assert.deepEqual([...table.matchAll(/^\| (\d+) \|/gm)].map((m) => m[1]), ['1', '2', '3'])
  })
  check('a shared-WBS block opens each requirement with a bold group row, numbering continuous', () => {
    const table = tableOf('SLS.MAP')
    assert.deepEqual(
      [...table.matchAll(/^\| \*\*(.+?)\*\* \|/gm)].map((m) => m[1]),
      [
        'Synchronized map and list',
        'Side-by-side map and list on desktop',
        'List-first layout with map toggle on mobile and tablet',
        'Management',
      ],
    )
    assert.deepEqual(
      [...table.matchAll(/^\| (\d+) \|/gm)].map((m) => m[1]),
      ['1', '2', '3', '4', '5', '6', '7', '8'],
    )
    // the group row precedes its own requirement's scenarios
    assert.ok(table.indexOf('| **Management** |') < table.indexOf('Results per page follow the Business Manager configuration'))
    assert.ok(
      table.indexOf('Map is shown side by side with the list on desktop') >
        table.indexOf('| **Side-by-side map and list on desktop** |'),
    )
  })
  check('a Management requirement authored mid-file still closes the block table', () => {
    const reqs = parseDeltaSpecs(join(FIX, 'specs.ba'))
      .filter((r) => r.wbs === 'SLS.MAP')
      .map((r) => ({ label: r.name, scenarios: r.scenarios }))
    const moved = [reqs[reqs.length - 1], ...reqs.slice(0, -1)] // author puts Management first
    const labels = [...renderKeyBehavioursTable(moved).matchAll(/^\| \*\*(.+?)\*\* \|/gm)].map((m) => m[1])
    assert.equal(labels[labels.length - 1], 'Management')
    assert.equal(labels[0], 'Synchronized map and list')
  })
  check('GH.UTL block aggregates two capabilities under one WBS; AND collapses via <br>', () => {
    const table = tableOf('GH.UTL')
    assert.deepEqual(
      [...table.matchAll(/^\| \*\*(.+?)\*\* \|/gm)].map((m) => m[1]),
      ['Footer utility links', 'Retired store directory URL redirects to the locator'],
    )
    assert.ok(table.includes('Footer exposes the Our stores link') && table.includes('Opening the retired URL lands on the locator'))
    assert.ok(md.includes("the shopper's coordinates are not persisted<br>the shopper's coordinates are not sent to analytics"))
  })
  check('bootstrap -> surgical update is a byte-identical no-op', () => {
    assert.equal(surgicalUpdate(md, model.targets[0].blocks, null, null, model.legend), md)
  })
  check('the proposal legend is projected once, ahead of the first block, and re-projected over drift', () => {
    assert.equal((md.match(/^\*\*Re-usability Legend\*\*$/gm) || []).length, 1)
    assert.ok(md.includes('- `[SF Next OOTB]` — base SF Next functionality, used as delivered'))
    assert.ok(md.indexOf('**Re-usability Legend**') < md.indexOf('### SLS.SRCH'))
    assert.ok(md.indexOf('## In Scope Functional Requirements') < md.indexOf('**Re-usability Legend**'))
    const tampered = md.replace('- `[New]` — brand-new functionality, built from scratch', '- `[New]` — hand-edited gloss')
    const restored = surgicalUpdate(tampered, model.targets[0].blocks, null, null, model.legend)
    assert.equal(restored, md, 'a hand-edited legend was not re-projected from the proposal')
    assert.ok(validate({ ...model, targets: [{ ...model.targets[0], existing: parseFsdDoc(tampered) }] })
      .some((f) => f.code === 'READ_ONLY_DRIFT' && /Re-usability Legend/.test(f.where)),
      'a hand-edited legend raised no READ_ONLY_DRIFT warning')
  })

  // Feedback round-trip: CHANGE / ADD / DELETE reconcile on an ENRICHED FSD
  check('surgical reconcile handles CHANGE + ADD + DELETE, preserving enrichment', () => {
    const enriched = readFileSync(join(FIX, 'fsd.ba/store-locator.md'), 'utf8') // has real Visualization/chrome
    const blockTemplate = splitFsdTemplate(readFileSync(TMPL, 'utf8')).blockTemplate
    const kept = parseFsdDoc(enriched).blocks
      .filter((b) => b.inScope && b.wbs && b.wbs !== 'SLS.DIR') // DELETE SLS.DIR
      .map((b) => ({ wbs: b.wbs, name: b.name, userStory: `Story ${b.wbs}`, acceptance: `1. AC ${b.wbs}`, groups: [] }))
    kept.push({ wbs: 'SLS.NEW', name: 'Brand new requirement', userStory: 'New story', acceptance: '1. New AC', groups: [] }) // ADD
    const out = surgicalUpdate(enriched, kept, blockTemplate)
    assert.ok(!/^### SLS\.DIR /m.test(out), 'DELETE: retired SLS.DIR block still present')
    assert.ok(/^### SLS\.NEW — Brand new requirement/m.test(out), 'ADD: new SLS.NEW block not inserted')
    assert.ok(out.includes('Story SLS.SRCH'), 'CHANGE: SLS.SRCH User Story not updated')
    assert.ok(out.includes('interactive map beside the synchronized result list'), 'enrichment: SLS.MAP Visualization not preserved')
    assert.ok(out.includes('## Deferred Requirements') && out.includes('### SLS.INV'), 'out-of-scope Deferred block not preserved')
  })

  // EXTEND (additive / --extend): prior in-scope blocks are preserved, not stripped
  check('additive (--extend) keeps prior in-scope blocks, updates matched, appends new', () => {
    const enriched = readFileSync(join(FIX, 'fsd.ba/store-locator.md'), 'utf8')
    const blockTemplate = splitFsdTemplate(readFileSync(TMPL, 'utf8')).blockTemplate
    const priorWbs = parseFsdDoc(enriched).blocks.filter((b) => b.inScope && b.wbs).map((b) => b.wbs)
    const keep = priorWbs[0]
    const blocks = [
      { wbs: keep, name: 'ignored on update', userStory: 'EXTEND-UPDATED-STORY', acceptance: '1. EXTEND-UPDATED-AC', groups: [] },
      { wbs: 'SLS.EXT', name: 'Extend-only new requirement', userStory: 'new', acceptance: '1. new', groups: [] },
    ]
    const out = surgicalUpdate(enriched, blocks, blockTemplate, null, model.legend, { additive: true })
    for (const w of priorWbs) assert.match(out, new RegExp('^### ' + w.replace(/\./g, '\\.') + ' ', 'm'), `prior block ${w} dropped on extend`)
    assert.match(out, /^### SLS\.EXT — Extend-only new requirement/m)
    assert.ok(out.includes('EXTEND-UPDATED-STORY'), 'matched block read-only body not updated on extend')
    assert.ok(out.includes('interactive map beside the synchronized result list'), 'enrichment: SLS.MAP Visualization not preserved on extend')
  })
  check('default mode WARNS an uncovered prior in-scope block will be removed; --extend flips it to preserved', () => {
    const OLD = '### SLS.OLD — Retired requirement\n\n#### User Story\n\nold\n\n#### Acceptance Criteria\n\n1. old\n\n#### Key behaviours\n\n(none)\n\n'
    const existing = parseFsdDoc(md.replace('## Deferred Requirements', OLD + '## Deferred Requirements'))
    const tmodel = { ...model, targets: [{ ...model.targets[0], existing }] }
    const def = validate(tmodel, false)
    assert.ok(def.some((f) => f.code === 'UNMATCHED_BLOCK_WILL_BE_REMOVED' && /SLS\.OLD/.test(f.where)), 'default mode should warn removal')
    assert.equal(def.filter((f) => f.severity === 'error').length, 0, 'the removal warning must be non-blocking')
    const ext = validate(tmodel, true)
    assert.ok(ext.some((f) => f.code === 'PRIOR_BLOCK_PRESERVED' && /SLS\.OLD/.test(f.where)), 'extend mode should report preserved')
    assert.ok(!ext.some((f) => f.code === 'UNMATCHED_BLOCK_WILL_BE_REMOVED'), 'extend mode must not warn removal')
  })

  // eval fixture drift guard
  const SCAFFOLD_FIXTURE = join(FIX, 'fsd-scaffold.ba.md')
  if (process.argv.includes('--update')) {
    writeFileSync(SCAFFOLD_FIXTURE, md)
    console.log('  ..  regenerated fsd-scaffold.ba.md (--update)')
  }
  check('committed fsd-scaffold.ba.md matches the current bootstrap output (drift guard)', () => {
    assert.ok(existsSync(SCAFFOLD_FIXTURE), 'fsd-scaffold.ba.md missing — run with --update')
    assert.equal(readFileSync(SCAFFOLD_FIXTURE, 'utf8'), md,
      'fsd-scaffold.ba.md is stale — run `node generate-functional-artifacts.test.mjs --update`')
  })

  check('a spec heading without a WBS raises SPEC_HEADING_NOT_OPTION_H', () => {
    const bad = track(makeChange())
    const specPath = join(bad, 'specs/store-locator/spec.md')
    writeFileSync(specPath, readFileSync(specPath, 'utf8').replace('### Requirement: SLS.DIR — Get directions', '### Requirement: Get directions'))
    assert.ok(validate(buildModel(bad)).map((f) => f.code).includes('SPEC_HEADING_NOT_OPTION_H'))
  })
  // REQUIREMENT PARTITION checks (the group labels a block projects)
  const withSpec = (edit) => {
    const bad = track(makeChange())
    const specPath = join(bad, 'specs/store-locator/spec.md')
    writeFileSync(specPath, edit(readFileSync(specPath, 'utf8')))
    return validate(buildModel(bad))
  }
  check('two requirements of one WBS with the same name raise GROUP_LABEL_DUPLICATE', () => {
    const findings = withSpec((s) =>
      s.replace(
        '### Requirement: SLS.MAP — List-first layout with map toggle on mobile and tablet',
        '### Requirement: SLS.MAP — Side-by-side map and list on desktop',
      ))
    assert.ok(findings.some((f) => f.code === 'GROUP_LABEL_DUPLICATE' && f.severity === 'error'))
  })
  check('a second Management requirement on one WBS raises MULTIPLE_MANAGEMENT_GROUPS', () => {
    const findings = withSpec((s) =>
      s.replace(
        '### Requirement: SLS.MAP — List-first layout with map toggle on mobile and tablet',
        '### Requirement: SLS.MAP — Management',
      ))
    assert.ok(findings.some((f) => f.code === 'MULTIPLE_MANAGEMENT_GROUPS' && f.severity === 'error'),
      'expected MULTIPLE_MANAGEMENT_GROUPS; got ' + findings.map((f) => f.code).join(', '))
  })
  check('a one-scenario observer group in a split block raises THIN_GROUP (warning only)', () => {
    const findings = withSpec((s) =>
      s.replace(`#### Scenario: Map stays visible while the shopper browses the list
- **GIVEN** a shopper on a desktop viewport with search results shown
- **WHEN** they browse and select stores in the result list
- **THEN** the map stays visible next to the list

`, ''))
    const thin = findings.filter((f) => f.code === 'THIN_GROUP')
    assert.equal(thin.length, 1, 'expected exactly one THIN_GROUP warning')
    assert.equal(thin[0].severity, 'warning')
    assert.equal(findings.filter((f) => f.severity === 'error').length, 0, 'THIN_GROUP must not block generation')
  })
  check('the golden ba specs raise no partition finding at all', () => {
    assert.equal(validate(model).filter((f) => ['GROUP_LABEL_DUPLICATE', 'MULTIPLE_MANAGEMENT_GROUPS', 'THIN_GROUP'].includes(f.code)).length, 0)
  })

  check('an in-scope slice with no covering spec raises SLICE_NO_SPEC', () => {
    const bad = track(makeChange())
    const specPath = join(bad, 'specs/store-locator/spec.md')
    let spec = readFileSync(specPath, 'utf8')
    const from = spec.indexOf('### Requirement: SLS.I18N')
    const nextIdx = spec.indexOf('### Requirement:', from + 10)
    spec = spec.slice(0, from) + (nextIdx >= 0 ? spec.slice(nextIdx) : '')
    writeFileSync(specPath, spec)
    const findings = validate(buildModel(bad))
    assert.ok(findings.map((f) => f.code).includes('SLICE_NO_SPEC'))
    for (const f of findings) assert.ok(f.where && f.problem && f.fix && f.docs, 'finding missing fields: ' + f.code)
  })

  // CLI end-to-end: BOTH artifacts in one run, then noop
  check('CLI writes FSD + stories.md in one run, then reports noop on a clean re-run', () => {
    const c = track(makeChange())
    const r1 = runCli(c)
    assert.equal(r1.status, 0, 'first run should exit 0; stderr: ' + r1.stderr)
    assert.ok(existsSync(join(c, 'fsd/store-locator.md')), 'FSD not created')
    assert.ok(existsSync(join(c, 'stories.md')), 'stories.md not created')
    assert.match(r1.stderr, /created FSD store-locator\.md \(9\)/)
    assert.match(r1.stderr, /stories\.md \(9\)/)
    const r2 = runCli(c)
    assert.equal(r2.status, 0)
    assert.match(r2.stderr, /noop FSD/)
    assert.match(r2.stderr, /noop stories\.md/)
  })
  check('CLI exits non-zero on a broken change (missing WBS heading)', () => {
    const bad = track(makeChange())
    const specPath = join(bad, 'specs/store-locator/spec.md')
    writeFileSync(specPath, readFileSync(specPath, 'utf8').replace('### Requirement: SLS.DIR — Get directions', '### Requirement: Get directions'))
    const r = runCli(bad)
    assert.equal(r.status, 1)
    assert.match(r.stderr, /SPEC_HEADING_NOT_OPTION_H/)
  })

  // stories projection (D7 fix: block proposal) + Jira preservation
  check('stories.md is projected from the block proposal (one story per slice)', () => {
    const c = track(makeChange())
    runCli(c)
    const st = readFileSync(join(c, 'stories.md'), 'utf8')
    assert.match(st, /### TBD-1 — SLS\.SRCH — Store search by postal code, city, or location/)
    assert.ok(st.includes('**User story:** As a shopper, I SHALL be able to search for stores'))
    assert.ok(st.includes('**Acceptance criteria:**'))
    assert.equal((st.match(/^### /gm) || []).length, 9, 'expected 9 story blocks')
  })
  check('re-run PRESERVES a backfilled Jira key + link + created line', () => {
    const c = track(makeChange())
    runCli(c)
    const p = join(c, 'stories.md')
    let st = readFileSync(p, 'utf8')
    // simulate jira.mjs backfill for the first story
    st = st.replace('## Stories', 'Jira tickets created via /opsx:propose-approved: PROJ-1\n\n## Stories')
    st = st.replace('### TBD-1 — SLS.SRCH', '### PROJ-1 — SLS.SRCH')
    st = st.replace(/(### PROJ-1 — SLS\.SRCH[^\n]*\n)/, '$1\n- Jira: [PROJ-1](https://jira/PROJ-1)\n')
    writeFileSync(p, st)
    runCli(c)
    const after = readFileSync(p, 'utf8')
    assert.match(after, /### PROJ-1 — SLS\.SRCH/, 'real Jira key not preserved on re-run')
    assert.ok(after.includes('- Jira: [PROJ-1](https://jira/PROJ-1)'), 'Jira ref line not preserved')
    assert.ok(after.includes('Jira tickets created via /opsx:propose-approved: PROJ-1'), 'created line not preserved')
    assert.ok(!/### TBD-1 — SLS\.SRCH/.test(after), 'TBD id resurfaced (clobbered the key)')
  })

  console.log(`\ngenerate-functional-artifacts.test: ${passed} checks passed\n`)
} finally {
  for (const d of cleanups) {
    try {
      rmSync(d, { recursive: true, force: true })
    } catch {}
  }
}
