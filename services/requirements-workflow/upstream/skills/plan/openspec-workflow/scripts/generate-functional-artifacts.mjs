#!/usr/bin/env node
/**
 * generate-functional-artifacts.mjs — deterministically project the FUNCTIONAL
 * flow's downstream artifacts from a change's already-approved BA/SA artifacts, in
 * ONE run that keeps them in sync:
 *   1. the client-facing FSD's read-only sub-sections (`#### User Story` +
 *      `#### Acceptance Criteria` verbatim from the covering proposal slice;
 *      `#### Key behaviours` as a TABLE from the covering spec requirement(s)); and
 *   2. `stories.md` (one story per slice: the slice Story, an acceptance-criteria
 *      reference to the FSD block, and design-derived implementation notes).
 *
 * The join key is the clean WBS code shared by the proposal slice, the spec
 * requirement heading (Option-H), and the FSD block heading. Nothing here decides
 * scope or behaviour — every field is copied or joined from an existing artifact.
 * To change a Story/AC, edit the proposal slice; a Key behaviour, edit the spec;
 * then re-run (never hand-edit the generated files).
 *
 * FSD: fills the OpenSpec fsd.md template directly (bootstrap when absent; surgical
 * read-only-body replacement when present). Stories: full re-render, but any
 * backfilled Jira key (heading id) + `- Jira:` link is PRESERVED so a re-sync never
 * clobbers it; the acceptance-criteria reference is a live Confluence anchor when the
 * FSD is published, else a plain local reference (no hard block).
 *
 * Validation runs FIRST: every violation is collected and reported once with an
 * actionable, upstream-routing message (WHERE / WHAT / FIX / DOCS). A critical
 * blocks BOTH artifacts; a warning prints but lets generation proceed. No
 * --strict / --force.
 *
 * Usage:
 *   node generate-functional-artifacts.mjs <change-dir> [--dry] [--out-dir <dir>] [--template <path>]
 *
 * Integration (ISD) projection is a future sibling reusing lib/ — out of scope here.
 */

import { readFileSync, existsSync } from 'node:fs'
import { basename, dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

import { parseProposalSlices, parseProposalLegend } from './lib/proposal.mjs'
import { listSpecFiles, parseSpecFile, partitionFindings } from './lib/specs.mjs'
import {
  parseFsdDoc,
  splitFsdTemplate,
  bootstrap,
  surgicalUpdate,
  projectedBodies,
  READ_ONLY,
} from './lib/fsd.mjs'
import { parseDesignNotes, buildStories, renderStories } from './lib/stories.mjs'
import { finding, renderFindings, DOCS } from './lib/findings.mjs'
import { resolveTemplate, writeIfChanged } from './lib/openspec.mjs'

const TOOL = 'generate-functional-artifacts'

// ─── build model ─────────────────────────────────────────────────────────────

function buildModel(changeDir) {
  const proposalPath = join(changeDir, 'proposal.md')
  const specsDir = join(changeDir, 'specs')
  const designPath = join(changeDir, 'design.md')
  const storiesPath = join(changeDir, 'stories.md')

  const proposalOk = existsSync(proposalPath)
  const proposalMd = proposalOk ? readFileSync(proposalPath, 'utf8') : ''
  const parsed = proposalOk
    ? parseProposalSlices(proposalMd)
    : { slices: [], hasDeliveryBreakdown: false }
  const slices = parsed.slices
  const legend = proposalOk ? parseProposalLegend(proposalMd) : ''

  const specFiles = listSpecFiles(specsDir)
  const requirements = specFiles.flatMap(({ cap, file }) => parseSpecFile(cap, file))

  const byWbs = new Map()
  for (const r of requirements) {
    if (!r.wbs || r.section === 'REMOVED') continue
    if (!byWbs.has(r.wbs)) byWbs.set(r.wbs, [])
    byWbs.get(r.wbs).push(r)
  }

  const designNotes = existsSync(designPath) ? parseDesignNotes(readFileSync(designPath, 'utf8')) : []

  // group slices by their FSD target file
  const targetMap = new Map()
  for (const s of slices) {
    const rel = s.fsd || s.isd
    if (!rel || !s.wbs) continue
    const path = resolve(changeDir, rel)
    if (!targetMap.has(path)) targetMap.set(path, { path, rel, blocks: [] })
    const reqs = byWbs.get(s.wbs) || []
    targetMap.get(path).blocks.push({
      wbs: s.wbs,
      name: s.name,
      userStory: s.userStory,
      acceptance: s.acceptance,
      groups: reqs.map((r) => ({ label: r.name, scenarios: r.scenarios, req: r })),
    })
  }
  const targets = [...targetMap.values()]
  for (const t of targets) {
    t.existing = existsSync(t.path) ? parseFsdDoc(readFileSync(t.path, 'utf8')) : null
  }

  return {
    changeDir,
    proposalPath,
    specsDir,
    storiesPath,
    proposalOk,
    hasDeliveryBreakdown: parsed.hasDeliveryBreakdown,
    specsExist: specFiles.length > 0,
    slices,
    legend,
    requirements,
    byWbs,
    designNotes,
    targets,
    existingStoriesMd: existsSync(storiesPath) ? readFileSync(storiesPath, 'utf8') : null,
  }
}

// ─── validate ────────────────────────────────────────────────────────────────

function validate(model, extend = false) {
  const out = []
  const { proposalOk, hasDeliveryBreakdown, slices, requirements, byWbs, targets } = model

  if (!proposalOk) {
    out.push(finding('PROPOSAL_NOT_FOUND', 'error', model.proposalPath,
      'proposal.md was not found or could not be read.',
      'Create the proposal first (/opsx:propose), then re-run.', DOCS.proposal))
    return out
  }
  if (!hasDeliveryBreakdown) {
    out.push(finding('PROPOSAL_NO_DELIVERY_BREAKDOWN', 'error', model.proposalPath,
      'no "## Delivery Breakdown" section with slice blocks (### S# — <WBS> — <name>).',
      'Author the Delivery Breakdown as per-slice blocks, then re-run.', DOCS.proposal))
    return out
  }

  // per-slice attribute checks
  const seenWbs = new Map()
  for (const s of slices) {
    const where = `${model.proposalPath} slice "${[s.s, s.headingWbs, s.name].filter(Boolean).join(' — ')}"`
    if (s.coversWbs.length === 0) {
      out.push(finding('SLICE_NO_COVERS_WBS', 'error', where,
        'the slice has no clean WBS in "- **Covers (WBS):**".',
        'Add a single clean WBS code (e.g. SLS.SRCH) to Covers (WBS), then re-run.', DOCS.proposal))
    } else if (s.coversWbs.length > 1) {
      out.push(finding('SLICE_MULTI_COVERS_WBS', 'error', `${where} (Covers: ${s.coversWbs.join(', ')})`,
        'a slice must cover EXACTLY ONE clean WBS.',
        'Split the slice so each covers one WBS, then re-run.', DOCS.proposal))
    }
    if (s.wbs) {
      if (seenWbs.has(s.wbs)) {
        out.push(finding('DUPLICATE_SLICE_WBS', 'error', `${where} (WBS ${s.wbs})`,
          `WBS ${s.wbs} is covered by more than one slice (${seenWbs.get(s.wbs)} and ${s.s}); the slice→FSD→story join must be 1:1.`,
          'Merge or re-slice so each in-scope WBS is covered by exactly one slice, then re-run.', DOCS.proposal))
      } else seenWbs.set(s.wbs, s.s)
    }
    if (!s.fsd && !s.isd) {
      out.push(finding('SLICE_NO_FSD_POINTER', 'error', where,
        'the slice has no "- **FSD:**" routing pointer.',
        'Add "- **FSD:** fsd/<feature>.md" to the slice metadata, then re-run.', DOCS.proposal))
    }
    if (!s.userStory.trim()) {
      out.push(finding('SLICE_NO_USER_STORY', 'error', where,
        'the slice has no "#### User Story" body.',
        'Author the slice User Story (it becomes the FSD block story + the story sentence verbatim), then re-run.', DOCS.proposal))
    }
    if (!s.acceptance.trim()) {
      out.push(finding('SLICE_NO_AC', 'error', where,
        'the slice has no "#### Acceptance Criteria" body.',
        'Author the slice Acceptance Criteria (they become the FSD block AC verbatim), then re-run.', DOCS.proposal))
    }
  }

  if (!model.specsExist) {
    out.push(finding('SPECS_NOT_FOUND', 'error', model.specsDir,
      'no delta specs found (expected specs/<capability>/spec.md).',
      'Author the change specs (/opsx:propose specs step), then re-run.', DOCS.specs))
  }

  for (const r of requirements) {
    if (r.section === 'REMOVED') continue
    if (!r.wbs) {
      out.push(finding('SPEC_HEADING_NOT_OPTION_H', 'error', `${r.file} "### Requirement: ${r.headingText}"`,
        'an ADDED/MODIFIED requirement heading carries no WBS code (Option-H is the FSD join key).',
        `Rewrite the heading as "### Requirement: <WBS> — ${r.headingText}", then re-run.`, DOCS.specs))
      continue
    }
    for (const sc of r.scenarios) {
      if (sc.then.length === 0) {
        out.push(finding('SCENARIO_NO_THEN', 'error', `${r.file} ${r.wbs} scenario "${sc.name}"`,
          'the scenario has no THEN outcome.',
          'Add a "- **THEN** <observable outcome>" bullet to the scenario, then re-run.', DOCS.specs))
      }
    }
  }

  // join integrity
  const sliceWbs = new Set(slices.map((s) => s.wbs).filter(Boolean))
  for (const s of slices) {
    if (!s.wbs) continue
    if (!byWbs.has(s.wbs)) {
      out.push(finding('SLICE_NO_SPEC', 'error',
        `${model.proposalPath} slice "${[s.s, s.wbs, s.name].filter(Boolean).join(' — ')}" (WBS ${s.wbs})`,
        'this in-scope slice has no covering spec requirement, so its Key behaviours table would be empty.',
        `Add a "### Requirement: ${s.wbs} — …" block with its scenario set to a specs/<capability>/spec.md, then re-run.`, DOCS.specs))
    }
  }
  for (const [wbs, reqs] of byWbs) {
    if (!sliceWbs.has(wbs)) {
      out.push(finding('ORPHAN_SPEC_WBS', 'error',
        `${reqs.map((r) => r.file).join(', ')} "### Requirement: ${wbs} — …"`,
        `WBS ${wbs} matches no proposal Delivery Breakdown slice.`,
        `Add a slice covering ${wbs} to the proposal, or correct the spec WBS to a locked one, then re-run.`, DOCS.proposal))
    }
  }

  // requirement partition (the group labels each block projects)
  out.push(...partitionFindings(byWbs))

  // per existing target: reused-WBS guard + read-only drift
  for (const t of targets) {
    if (!t.existing) continue
    const exLegend = t.existing.legend
    if (exLegend && exLegend.body && exLegend.body !== model.legend.trim()) {
      if (extend) {
        out.push(finding('LEGEND_PRESERVED_ON_EXTEND', 'warning', `${t.path} "Re-usability Legend"`,
          'the published legend differs from this change\'s proposal legend; --extend LEAVES it as published.',
          'Reconcile the legend manually on the published doc if the new blocks need new markers.', DOCS.proposal))
      } else {
        out.push(finding('READ_ONLY_DRIFT', 'warning', `${t.path} "Re-usability Legend"`,
          'the legend is script-owned but its current content differs from the proposal projection; it will be OVERWRITTEN.',
          'To change it, edit the proposal\'s "## Re-usability Legend", then re-run.', DOCS.proposal))
      }
    }
    for (const b of t.existing.blocks) {
      if (b.wbs && !b.inScope && sliceWbs.has(b.wbs)) {
        out.push(finding('WBS_REUSED_OUT_OF_SCOPE', 'error', `${t.path} "### ${b.headingText}"`,
          `WBS ${b.wbs} is reused by an out-of-scope block (Deferred / Change Requests); a duplicate WBS breaks the FSD/story join.`,
          'Give the out-of-scope block its own distinct WBS, then re-run.', DOCS.fsd))
      }
    }
    // Prior in-scope block not covered by this change: PRESERVED on --extend, else REMOVED.
    for (const b of t.existing.blocks) {
      if (!b.inScope || !b.wbs || sliceWbs.has(b.wbs)) continue
      if (extend) {
        out.push(finding('PRIOR_BLOCK_PRESERVED', 'warning', `${t.path} "### ${b.headingText}"`,
          `in-scope block ${b.wbs} is not covered by this change; --extend PRESERVES it verbatim (prior-change scope).`,
          'No action needed for an EXTEND; to drop a prior requirement, edit the published doc deliberately.', DOCS.fsd))
      } else {
        out.push(finding('UNMATCHED_BLOCK_WILL_BE_REMOVED', 'warning', `${t.path} "### ${b.headingText}"`,
          `in-scope block ${b.wbs} is not covered by this change's proposal and WILL be REMOVED.`,
          'If extending an already-published document, re-run with --extend to preserve prior blocks.', DOCS.fsd))
      }
    }
    for (const block of t.blocks) {
      const ex = t.existing.blocks.find((eb) => eb.wbs === block.wbs && eb.inScope)
      if (!ex) continue
      const bodies = projectedBodies(block)
      for (const name of READ_ONLY) {
        const sec = ex.sections.find((s) => s.name.toLowerCase() === name.toLowerCase())
        if (!sec) continue
        const currentBody = t.existing.lines.slice(sec.bodyStart, sec.bodyEnd).join('\n').trim()
        if (currentBody && currentBody !== bodies[name].trim()) {
          out.push(finding('READ_ONLY_DRIFT', 'warning', `${t.path} block ${block.wbs} "#### ${name}"`,
            'this section is script-owned but its current content differs from the upstream projection; it will be OVERWRITTEN.',
            name === 'Key behaviours' ? 'To change it, edit the covering spec requirement, then re-run.'
              : 'To change it, edit the covering proposal slice, then re-run.',
            name === 'Key behaviours' ? DOCS.specs : DOCS.proposal))
        }
      }
    }
  }

  return out
}

// ─── projection ──────────────────────────────────────────────────────────────

// Project one FSD file's markdown: bootstrap when the file is absent; otherwise
// reconcile in place (update matched blocks' read-only bodies, ADD blocks for newly
// covered WBS, REMOVE blocks whose WBS is no longer covered), preserving enrichment.
// The block template is resolved only when an ADD is actually needed.
function projectFsd(target, changeDir, templateState, legend, additive = false) {
  const needTemplate = () => {
    if (templateState.md === null) templateState.md = resolveTemplate('fsd', changeDir, templateState.override)
    return templateState.md
  }
  if (target.existing) {
    const existingWbs = new Set(target.existing.blocks.filter((b) => b.inScope).map((b) => b.wbs))
    const needsAdd = target.blocks.some((b) => !existingWbs.has(b.wbs))
    const blockTemplate = needsAdd ? splitFsdTemplate(needTemplate()).blockTemplate : null
    return surgicalUpdate(readFileSync(target.path, 'utf8'), target.blocks, blockTemplate, null, legend, { additive })
  }
  return bootstrap(needTemplate(), target.blocks, legend)
}

// After the FSD md is known, index its in-scope blocks by WBS for the stories join.
function indexFsd(md) {
  const fsd = parseFsdDoc(md)
  const byWbs = new Map()
  for (const b of fsd.blocks) if (b.inScope && b.wbs) byWbs.set(b.wbs, { block: b, fsd })
  return { fsd, byWbs }
}

// ─── CLI ─────────────────────────────────────────────────────────────────────

function main(argv) {
  const args = argv.slice(2)
  const dry = args.includes('--dry')
  const extend = args.includes('--extend')
  const flagVal = (name) => {
    const i = args.indexOf(name)
    return i >= 0 ? args[i + 1] : null
  }
  const outDir = flagVal('--out-dir')
  const templateOverride = flagVal('--template')
  const positional = args.filter(
    (a, i) => !a.startsWith('--') && args[i - 1] !== '--out-dir' && args[i - 1] !== '--template',
  )
  const changeDir = positional[0]

  if (!changeDir) {
    console.error('Usage: node generate-functional-artifacts.mjs <change-dir> [--dry] [--extend] [--out-dir <dir>] [--template <path>]')
    process.exit(2)
  }
  if (!existsSync(changeDir)) {
    console.error(`Change directory not found: ${changeDir}`)
    process.exit(2)
  }

  const model = buildModel(changeDir)
  const findings = validate(model, extend)
  const errors = findings.filter((f) => f.severity === 'error')
  if (findings.length) console.error(renderFindings(findings, TOOL))
  if (errors.length) process.exit(1)

  if (!model.targets.length) {
    console.error(`${TOOL}: no FSD target resolved from the proposal slices (no "- **FSD:**" pointer).`)
    process.exit(1)
  }

  const results = []
  const templateState = { md: null, override: templateOverride }
  const fsdByWbs = new Map()
  let primaryFsdMd = null

  // 1) FSD(s)
  for (const t of model.targets) {
    const md = projectFsd(t, changeDir, templateState, model.legend, extend)
    if (primaryFsdMd === null) primaryFsdMd = md
    const { byWbs } = indexFsd(md)
    for (const [wbs, entry] of byWbs) if (!fsdByWbs.has(wbs)) fsdByWbs.set(wbs, entry)
    const fsdPath = outDir ? join(outDir, basename(t.path)) : t.path
    if (dry) process.stdout.write(md + '\n')
    else {
      const r = writeIfChanged(fsdPath, md, false)
      results.push({ label: `FSD ${basename(fsdPath)}`, status: r.status, n: t.blocks.length })
    }
  }

  // 2) stories.md — projected from the slices + the FSD blocks just produced
  const primary = indexFsd(primaryFsdMd).fsd
  const storiesModel = buildStories({
    slices: model.slices,
    fsdByWbs,
    designNotes: model.designNotes,
    changeName: basename(resolve(changeDir)),
    code: primary.code,
    featureName: primary.featureName,
    existingStoriesMd: model.existingStoriesMd,
  })
  const storiesMd = renderStories(storiesModel)
  const storiesPath = outDir ? join(outDir, 'stories.md') : model.storiesPath
  if (dry) process.stdout.write(storiesMd)
  else {
    const r = writeIfChanged(storiesPath, storiesMd, false)
    results.push({ label: 'stories.md', status: r.status, n: storiesModel.stories.length })
  }

  if (!dry) {
    for (const r of results) console.error(`${TOOL}: ${r.status} ${r.label} (${r.n})`)
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main(process.argv)
}

export { buildModel, validate, projectFsd, indexFsd }
