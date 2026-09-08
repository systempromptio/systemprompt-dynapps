// Delta-spec parser + Key-behaviours TABLE renderer. Reads the change's delta
// specs directly from raw markdown (the openspec CLI JSON does not expose the
// `#### Scenario:` heading text). Reusable by the FSD and ISD projections.

import { readFileSync, existsSync, readdirSync, statSync } from 'node:fs'
import { join } from 'node:path'
import { splitLines, splitFirstDash, WBS_RE, escCell } from './markdown.mjs'
import { finding, DOCS } from './findings.mjs'

export function listSpecFiles(dir) {
  if (!existsSync(dir)) return []
  const out = []
  for (const cap of readdirSync(dir)) {
    const p = join(dir, cap)
    if (statSync(p).isDirectory()) {
      const f = join(p, 'spec.md')
      if (existsSync(f)) out.push({ cap, file: f })
    }
  }
  return out
}

// Parse one `#### Scenario: <name>` and its GIVEN/WHEN/THEN/AND bullets. The whole
// heading text is the scenario name — no part of it is interpreted.
// AND lines append to the current keyword's list.
export function parseScenario(lines, startIdx) {
  const name = lines[startIdx].replace(/^####\s+Scenario:\s*/, '').trim()
  const given = []
  const when = []
  const then = []
  let cur = null
  for (let i = startIdx + 1; i < lines.length; i++) {
    if (/^#{2,4}\s/.test(lines[i])) break
    const m = /^-\s+\*\*(GIVEN|WHEN|THEN|AND)\*\*\s*(.*)$/.exec(lines[i].trim())
    if (!m) continue
    const [, kw, txt] = m
    if (kw === 'GIVEN') cur = given
    else if (kw === 'WHEN') cur = when
    else if (kw === 'THEN') cur = then
    ;(cur || then).push(txt.trim())
  }
  return { name, given, when, then }
}

export function parseSpecFile(cap, file) {
  const lines = splitLines(readFileSync(file, 'utf8'))
  const reqs = []
  let section = null
  let cur = null
  for (let i = 0; i < lines.length; i++) {
    const secM = /^##\s+(ADDED|MODIFIED|REMOVED|RENAMED)\s+Requirements/.exec(lines[i])
    if (secM) {
      section = secM[1]
      continue
    }
    const reqM = /^###\s+Requirement:\s*(.+)$/.exec(lines[i])
    if (reqM) {
      const headingText = reqM[1].trim()
      const [maybeWbs, maybeName] = splitFirstDash(headingText)
      const hasWbs = maybeName !== null && WBS_RE.test(maybeWbs)
      cur = {
        cap,
        file,
        section,
        headingText,
        wbs: hasWbs ? maybeWbs : null,
        name: hasWbs ? maybeName : headingText,
        scenarios: [],
      }
      reqs.push(cur)
      continue
    }
    if (cur && /^####\s+Scenario:/.test(lines[i])) cur.scenarios.push(parseScenario(lines, i))
  }
  return reqs
}

export function parseDeltaSpecs(dir) {
  return listSpecFiles(dir).flatMap(({ cap, file }) => parseSpecFile(cap, file))
}

const MANAGEMENT_RE = /^management$/i
// A cross-cutting guarantee is a seat of its own (REQUIREMENT PARTITION), so — like
// Management — it legitimately carries a single scenario.
const CROSS_CUTTING_RE = /privacy|accessibilit|a11y|localizat|localisat|internationalizat|i18n/i

// Deterministic REQUIREMENT PARTITION checks over the requirements aggregated per
// WBS (the shape each becomes one FSD/ISD block): the names are the block's group
// labels and part of the archive merge key, so they must be unique and carry at
// most one Management seat. A one-scenario group in a split block is a warning —
// the ">=2 scenarios" bar for splitting an observer seat by context, with the
// Management and cross-cutting seats exempt.
export function partitionFindings(byWbs) {
  const out = []
  for (const [wbs, reqs] of byWbs) {
    const seen = new Set()
    for (const r of reqs) {
      const key = r.name.trim().toLowerCase()
      if (seen.has(key)) {
        out.push(finding('GROUP_LABEL_DUPLICATE', 'error', `${r.file} "### Requirement: ${r.headingText}"`,
          `WBS ${wbs} carries two requirements named "${r.name}"; the name is this requirement's group label in the projected Key behaviours table and part of its archive merge key, so a duplicate is ambiguous in both.`,
          'Rename one of them for the behaviour it holds — requirement names are unique within a WBS — then re-run.', DOCS.specs))
      } else seen.add(key)
    }
    const management = reqs.filter((r) => MANAGEMENT_RE.test(r.name.trim()))
    if (management.length > 1) {
      out.push(finding('MULTIPLE_MANAGEMENT_GROUPS', 'error',
        `${management.map((r) => r.file).join(', ')} "### Requirement: ${wbs} — Management"`,
        `WBS ${wbs} carries ${management.length} Management requirements; the administering business user is ONE seat per WBS.`,
        'Merge them into the single trailing Management requirement for this WBS, then re-run.', DOCS.specs))
    }
    if (reqs.length < 2) continue
    // The two-scenario bar governs splitting ONE observer seat, so it applies only
    // among sibling observer requirements of the SAME capability; requirements this
    // WBS collects from different capabilities are separate seats by construction.
    const observersByCap = new Map()
    for (const r of reqs) {
      const name = r.name.trim()
      if (MANAGEMENT_RE.test(name) || CROSS_CUTTING_RE.test(name)) continue
      if (!observersByCap.has(r.cap)) observersByCap.set(r.cap, [])
      observersByCap.get(r.cap).push(r)
    }
    for (const siblings of observersByCap.values()) {
      if (siblings.length < 2) continue
      for (const r of siblings) {
        if (r.scenarios.length !== 1) continue
        out.push(finding('THIN_GROUP', 'warning', `${r.file} "### Requirement: ${r.headingText}"`,
          `WBS ${wbs} splits this capability's observer seat into ${siblings.length} requirements and "${r.name.trim()}" holds a single scenario, so it projects a group header over one row.`,
          'Split an observer seat by context only when each part keeps >=2 scenarios — otherwise fold this behaviour back into the sibling requirement and let the context sit inside its scenario (a Management or cross-cutting requirement is exempt).', DOCS.specs))
      }
    }
  }
  return out
}

// Spec order, except the Management group always closes the block: the reader
// meets the shopper-facing behaviour first and the administering seat last, no
// matter where the author placed it in the spec file (REQUIREMENT PARTITION).
export function orderGroups(groups) {
  const management = groups.filter((g) => MANAGEMENT_RE.test(String(g.label).trim()))
  if (!management.length) return groups
  return [...groups.filter((g) => !MANAGEMENT_RE.test(String(g.label).trim())), ...management]
}

// Build the Key-behaviours TABLE for a requirement block from its GROUPS — one
// group per covering spec requirement, `{ label, scenarios }` in spec order.
// One table per block, scenarios in source order, numbering continuous across the
// requirements that share the block's WBS. A block covering 2+ requirements opens
// each group with a bold sub-header row carrying that requirement's NAME (the
// group label the FSD/ISD reader sees); a single-requirement block stays flat.
// Multiple GIVEN/WHEN/THEN + AND lines collapse into their cell via <br>.
export function renderKeyBehavioursTable(groups) {
  const rows = ['| # | Scenario | GIVEN | WHEN | THEN |', '| --- | --- | --- | --- | --- |']
  const grouped = groups.length > 1
  let n = 0
  for (const g of orderGroups(groups)) {
    if (grouped) rows.push(`| **${escCell(g.label)}** |  |  |  |  |`)
    for (const sc of g.scenarios) {
      n += 1
      rows.push(
        `| ${n} | ${escCell(sc.name)} | ${escCell(sc.given.join('<br>'))} | ${escCell(
          sc.when.join('<br>'),
        )} | ${escCell(sc.then.join('<br>'))} |`,
      )
    }
  }
  return rows.join('\n')
}
