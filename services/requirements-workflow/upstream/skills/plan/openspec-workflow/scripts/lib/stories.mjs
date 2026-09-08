// Stories projection: build stories.md from the approved proposal slices + the
// generated FSD blocks (+ design cross-cutting notes), rendered through Nunjucks.
// Decides nothing — a pure projection of the slicing/behaviour.
//
// Publish-optional AC reference: when the covered FSD carries a `- Confluence:`
// URL the acceptance-criteria is a live anchor link; otherwise it is a plain local
// reference (no hard block) that upgrades to the anchor on the next run after
// publish. Jira preservation: on re-generation, an existing story's backfilled key
// (heading id) and `- Jira:` link are preserved so a re-sync never clobbers them.

import { readFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { createRequire } from 'node:module'
import { splitLines, splitFirstDash } from './markdown.mjs'
import { normalizeUserStory } from './proposal.mjs'
import { parseTables, findTable, columnIndex } from './markdown.mjs'

const require = createRequire(import.meta.url)
const nunjucks = require('nunjucks')
const TEMPLATES_DIR = resolve(dirname(fileURLToPath(import.meta.url)), '..', 'templates')

// Confluence auto-anchors the first in-scope `#### Acceptance Criteria` as
// `Acceptance-Criteria`, then `.1`, `.2`, … per later in-scope block.
const AC_ANCHOR_BASE = 'Acceptance-Criteria'
export const acAnchor = (ordinal) => (ordinal === 0 ? AC_ANCHOR_BASE : `${AC_ANCHOR_BASE}.${ordinal}`)

// Design Base-Configuration & Cross-Cutting concerns that become story
// Implementation notes, in emit order, with their canonical labels.
const NOTE_CONCERNS = [
  ['Rendering', 'Rendering & hydration'],
  ['Caching', 'Caching'],
  ['Security', 'Security'],
  ['Performance', 'Performance'],
]

// Split a design constraint cell into its intent text and trailing skill link.
function splitSkill(constraint) {
  const skills = [...constraint.matchAll(/`([^`]+)`/g)].map((m) => m[1])
  const skill = skills.length ? skills[skills.length - 1] : ''
  let text = constraint
    .replace(/\s*\(`[^`]+`\)\s*$/, '')
    .replace(/\s*`[^`]+`\s*$/, '')
    .trim()
  text = text.replace(/[;,\s]+$/, '').trim()
  return { text, skill }
}

// Parse the design Base Configuration & Cross-Cutting rows that feed Implementation
// notes: { concern, label, appliesTo: 'all' | Set<WBS>, text, skill }.
export function parseDesignNotes(designMd) {
  if (!designMd) return []
  const t = findTable(parseTables(designMd), ['Concern', 'Applies to'])
  if (!t) return []
  const concernCol = columnIndex(t.header, 'Concern')
  const appliesCol = columnIndex(t.header, 'Applies to')
  const constraintCol = columnIndex(t.header, 'Constraint')
  const notes = []
  for (const row of t.rows) {
    const concern = (row[concernCol] || '').trim()
    const match = NOTE_CONCERNS.find(([name]) => name.toLowerCase() === concern.toLowerCase())
    if (!match) continue
    const constraint = (row[constraintCol] || '').trim()
    if (!constraint) continue
    const { text, skill } = splitSkill(constraint)
    if (!skill) continue
    const appliesRaw = (row[appliesCol] || '').trim()
    const appliesTo = /^all$/i.test(appliesRaw)
      ? 'all'
      : new Set([...appliesRaw.matchAll(/[A-Za-z0-9._-]+/g)].map((m) => m[0]))
    notes.push({ concern, label: match[1], appliesTo, text, skill })
  }
  return notes
}

// Parse an existing stories.md for backfilled Jira metadata to PRESERVE on re-run:
// the global "Jira tickets created …" line and, per covered WBS, the heading id
// (real key when not a TBD placeholder) and the `- Jira: …` reference line.
export function parseStoriesJira(md) {
  const byWbs = new Map()
  let createdLine = null
  if (!md) return { createdLine, byWbs }
  const lines = splitLines(md)
  let curWbs = null
  for (const raw of lines) {
    const line = raw.trim()
    if (/^Jira tickets created/i.test(line)) createdLine = line
    const h3 = /^###\s+(\S+)\s+—\s+(.+)$/.exec(raw)
    if (h3) {
      const id = h3[1].trim()
      const [wbs] = splitFirstDash(h3[2].trim())
      curWbs = wbs
      if (!byWbs.has(curWbs)) byWbs.set(curWbs, { id: /^TBD-/i.test(id) ? null : id, jiraRef: null })
      else byWbs.get(curWbs).id = /^TBD-/i.test(id) ? byWbs.get(curWbs).id : id
      continue
    }
    const jm = /^-\s*Jira:\s*(.+)$/i.exec(line)
    if (jm && curWbs && byWbs.has(curWbs)) byWbs.get(curWbs).jiraRef = line
  }
  return { createdLine, byWbs }
}

// Build the stories model. `fsdByWbs` maps a covered WBS to { block, fsd } where
// `fsd` is a parsed FSD doc ({ url, featureName, ... }) and `block` has { name,
// ordinal }. `existingStoriesMd` (optional) supplies preserved Jira metadata.
// `docLabel` is the covering client document's type ('FSD' for the functional
// flow, 'ISD' for the integration flow): it labels the acceptance-criteria
// fallback reference and the References row. `fsdByWbs` maps a covered WBS to
// { block, fsd } regardless of the document type (the entry's `fsd` is the parsed
// covering doc — an FSD or an ISD).
export function buildStories({ slices, fsdByWbs, designNotes = [], changeName, code, featureName, existingStoriesMd, docLabel = 'FSD' }) {
  const jira = parseStoriesJira(existingStoriesMd)
  const codeName = code && featureName && code !== featureName ? `${code} - ${featureName}` : featureName || code
  const stories = slices
    .filter((s) => s.wbs && fsdByWbs.has(s.wbs))
    .map((s, i) => {
      const { block, fsd } = fsdByWbs.get(s.wbs)
      const wbsSet = new Set(s.coversWbs.length ? s.coversWbs : [s.wbs])
      const preserved = jira.byWbs.get(s.wbs)

      const acLink = fsd.url
        ? { label: `${s.wbs} — Acceptance Criteria`, target: `${fsd.url}#${acAnchor(block.ordinal)}` }
        : { label: `${s.wbs} — Acceptance Criteria (see the ${fsd.featureName} ${docLabel})`, target: '' }

      const implementationNotes = designNotes
        .filter((n) => n.appliesTo === 'all' || [...n.appliesTo].some((w) => wbsSet.has(w)))
        .map((n) => ({ label: n.label, text: n.text, skill: n.skill }))

      const references = [{ label: docLabel, title: fsd.featureName, target: fsd.url || '' }]

      return {
        id: preserved?.id || `TBD-${i + 1}`,
        title: `${s.wbs} — ${block.name}`,
        userStory: normalizeUserStory(s.userStory),
        acLink,
        implementationNotes,
        references,
        jiraRef: preserved?.jiraRef || '',
      }
    })
  return { code, featureName, codeName, changeName, jiraCreatedLine: jira.createdLine || '', stories }
}

export function renderStories(model) {
  const env = new nunjucks.Environment(new nunjucks.FileSystemLoader(TEMPLATES_DIR, { noCache: true }), {
    autoescape: false,
    trimBlocks: true,
    lstripBlocks: true,
    throwOnUndefined: false,
  })
  const out = env.render('stories.md.njk', model)
  return (
    out
      .replace(/\n(### )/g, '\n\n$1')
      .replace(/\n(\*\*Implementation notes:\*\*)/g, '\n\n$1')
      .replace(/\n(\*\*References:\*\*)/g, '\n\n$1')
      .replace(/\n{3,}/g, '\n\n')
      .trimEnd() + '\n'
  )
}
