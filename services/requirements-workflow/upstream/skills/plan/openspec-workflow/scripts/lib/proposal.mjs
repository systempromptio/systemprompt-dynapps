// Proposal Delivery Breakdown parser (Phase-A slice-block shape). Single source of
// truth for the FSD, the stories, and the future ISD — every downstream artifact
// joins on the slice's clean WBS. Replaces the retired `S#|Story|Covers` table
// parser (fixes the stale generate-stories join).

import { splitLines, splitFirstDash, isSubsectionBoundary } from './markdown.mjs'

// Body of a top-level `## <name>` section, trimmed ('' when the section is absent).
function sectionBody(lines, heading) {
  const re = new RegExp(`^##\\s+${heading}\\s*$`)
  const start = lines.findIndex((l) => re.test(l))
  if (start < 0) return ''
  let end = lines.length
  for (let i = start + 1; i < lines.length; i++) {
    if (/^##\s+/.test(lines[i])) {
      end = i
      break
    }
  }
  return lines.slice(start + 1, end).join('\n').trim()
}

// The proposal's `## Re-usability Legend` bullets, verbatim. The proposal is the
// only place the legend is authored; the FSD/ISD carry it as a projection.
export function parseProposalLegend(md) {
  return sectionBody(splitLines(md), 'Re-usability Legend')
}

// Parse the Delivery Breakdown into slice blocks:
//   ### S# — <WBS> — <name>
//   - **Covers (WBS):** <WBS>
//   - **FSD:** fsd/<feature>.md   (or **ISD:** on an integration change)
//   #### User Story
//   <verbatim body>
//   #### Acceptance Criteria
//   <verbatim body, incl. any ##### group sub-headings>
export function parseProposalSlices(md) {
  const lines = splitLines(md)
  let start = -1
  let end = lines.length
  for (let i = 0; i < lines.length; i++) {
    if (/^##\s+Delivery Breakdown\s*$/.test(lines[i])) start = i
    else if (start >= 0 && /^##\s+/.test(lines[i])) {
      end = i
      break
    }
  }
  if (start < 0) return { slices: [], hasDeliveryBreakdown: false }

  const headIdx = []
  for (let i = start + 1; i < end; i++) if (/^###\s+S\d+\b/.test(lines[i])) headIdx.push(i)

  const slices = []
  for (let h = 0; h < headIdx.length; h++) {
    const from = headIdx[h]
    const to = h + 1 < headIdx.length ? headIdx[h + 1] : end
    const block = lines.slice(from, to)
    const headingText = block[0].replace(/^###\s+/, '').trim()
    const [s, rest] = splitFirstDash(headingText)
    const [headingWbs, name] = rest ? splitFirstDash(rest) : [null, null]

    const meta = {}
    for (const l of block) {
      const m = /^-\s+\*\*(.+?):\*\*\s*(.*)$/.exec(l.trim())
      if (m) meta[m[1].trim()] = m[2].trim()
    }
    const coversRaw = meta['Covers (WBS)'] || ''
    const coversWbs = [...coversRaw.matchAll(/[A-Za-z0-9]+(?:\.[A-Za-z0-9]+)+/g)].map((m) => m[0])

    const sec = {}
    for (let i = 0; i < block.length; i++) {
      const hm = /^####\s+(.+?)\s*$/.exec(block[i])
      if (!hm) continue
      const body = []
      for (let j = i + 1; j < block.length; j++) {
        if (isSubsectionBoundary(block[j])) break
        body.push(block[j])
      }
      sec[hm[1].trim()] = body.join('\n').trim()
    }

    slices.push({
      s,
      name,
      headingWbs,
      coversWbs,
      wbs: coversWbs[0] || headingWbs,
      fsd: meta['FSD'] || '',
      isd: meta['ISD'] || '',
      capability: meta['Capability'] || '',
      userStory: sec['User Story'] || '',
      acceptance: sec['Acceptance Criteria'] || '',
    })
  }
  return { slices, hasDeliveryBreakdown: true }
}

// Ensure a story sentence ends with terminal punctuation (stories projection).
export function normalizeUserStory(text) {
  const s = (text || '').trim()
  if (!s) return s
  return /[.!?]$/.test(s) ? s : `${s}.`
}
