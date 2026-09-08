// Config-driven "requirement-block section" model shared by the FSD (functional)
// and ISD (integration) projections. A requirement-block document has a section
// ("## In Scope Functional Requirements") of `### <WBS> — <name>` blocks, each
// carrying script-owned read-only sub-sections (User Story, Acceptance Criteria,
// Key behaviours) plus optional LLM-owned sub-sections (FSD: Visualization).
// A block's `groups` are its covering spec requirements (`{ label, scenarios }`),
// which the Key behaviours table renders as named row groups.
//
// `makeReqBlocks(config)` returns the parse + template-split + bootstrap +
// surgical-reconcile primitives bound to one document shape. The FSD wrapper
// (fsd.mjs) binds the functional config (read-only trio + Visualization, whole-doc
// bootstrap); the ISD sibling binds the integration config (read-only trio, no
// Visualization, section inserted before a configured heading). The template is
// the single shape source (no templating engine).
//
// config:
//   readOnly        [names]  script-owned sub-sections (replaced verbatim from the projection)
//   canonicalOrder  [names]  full sub-section order within a block (may include LLM-owned)
//   sectionRe       RegExp   matches the section H2 heading line exactly
//   sectionMatchRe  RegExp   loose test flagging the in-scope section while parsing a doc
//   titleSuffixRe   RegExp   the document-type suffix stripped from the H1 (e.g. / FSD$/)
//   insertBefore    string?  H2 heading before which to INSERT the section into an
//                            existing doc that lacks it (ISD); null when the section
//                            always exists in the bootstrap template (FSD)
//   legendLabel     string   bold label opening the script-owned legend block that
//                            sits in the section preamble, before the first block

import { splitLines, splitFirstDash, DASH, WBS_RE, trimBlank } from './markdown.mjs'
import { renderKeyBehavioursTable } from './specs.mjs'

export function makeReqBlocks(config) {
  const {
    readOnly,
    canonicalOrder,
    sectionRe,
    sectionMatchRe,
    titleSuffixRe,
    insertBefore = null,
    legendLabel = 'Re-usability Legend',
    label = 'template',
  } = config

  // The legend block opens with its bold label and runs to the next heading. It is
  // script-owned: authored once in the proposal, projected here, never edited here.
  const legendRe = new RegExp(`^\\*\\*${legendLabel}\\*\\*\\s*$`)
  const renderLegend = (legend) => `**${legendLabel}**\n\n${legend.trim()}`

  const READ_ONLY = readOnly
  const canonRank = (name) => {
    const i = canonicalOrder.findIndex((n) => n.toLowerCase() === name.toLowerCase())
    return i < 0 ? canonicalOrder.length : i
  }
  const canonName = (name) => READ_ONLY.find((n) => n.toLowerCase() === name.toLowerCase()) || name

  // The projected read-only body content (trimmed) for each sub-section of a block.
  const projectedBodies = (block) => ({
    'User Story': block.userStory.trim(),
    'Acceptance Criteria': block.acceptance.trim(),
    'Key behaviours': renderKeyBehavioursTable(block.groups),
  })

  // Parse a document into requirement blocks (with sub-section body line spans for
  // surgical replacement) plus document metadata. In-scope blocks carry a 0-based
  // `ordinal`; out-of-scope blocks (Deferred / Change Requests, or an ISD's
  // non-in-scope H3s) carry inScope=false.
  function parseDoc(md) {
    const lines = splitLines(md)
    const blocks = []
    let inScope = false
    let ordinal = 0
    let cur = null
    let curSection = null

    const titleLine = lines.find((l) => /^#\s+/.test(l)) || ''
    const title = titleLine.replace(/^#\s+/, '').replace(titleSuffixRe, '').trim()
    const dash = title.search(/\s[—-]\s/)
    const code = dash >= 0 ? title.slice(0, dash).trim() : title
    const featureName = dash >= 0 ? title.slice(dash).replace(/^\s[—-]\s/, '').trim() : title
    const confLine = lines.find((l) => /^-\s*Confluence:/i.test(l.trim()))
    const url = confLine ? confLine.replace(/^-\s*Confluence:\s*/i, '').trim() : ''

    const closeSection = (sec, endIdx) => {
      if (sec) sec.bodyEnd = endIdx
    }
    const finishBlock = (endIdx) => {
      if (cur) {
        closeSection(curSection, endIdx)
        curSection = null
        cur.endIdx = endIdx
        blocks.push(cur)
        cur = null
      }
    }

    for (let i = 0; i < lines.length; i++) {
      const h2 = /^##\s+(.*)$/.exec(lines[i])
      if (h2) {
        finishBlock(i)
        inScope = sectionMatchRe.test(h2[1])
        continue
      }
      const h3 = /^###\s+(.+)$/.exec(lines[i])
      if (h3) {
        finishBlock(i)
        const [maybeWbs, maybeName] = splitFirstDash(h3[1].trim())
        cur = {
          headingIdx: i,
          headingText: h3[1].trim(),
          wbs: maybeName !== null && WBS_RE.test(maybeWbs) ? maybeWbs : null,
          name: maybeName !== null ? maybeName : h3[1].trim(),
          inScope,
          ordinal: inScope ? ordinal++ : null,
          preambleEnd: -1,
          sections: [],
        }
        continue
      }
      if (cur) {
        const h4 = /^####\s+(.+?)\s*$/.exec(lines[i])
        if (h4) {
          if (cur.preambleEnd < 0) cur.preambleEnd = i
          closeSection(curSection, i)
          curSection = { name: h4[1].trim(), headingIdx: i, bodyStart: i + 1, bodyEnd: -1 }
          cur.sections.push(curSection)
        }
      }
    }
    finishBlock(lines.length)
    return { lines, blocks, url, code, featureName, legend: legendSpan(lines) }
  }

  // Locate the legend block in a line array: its label line, the exclusive end
  // (the next heading of any level), and its current body. null when absent.
  function legendSpan(lines) {
    const start = lines.findIndex((l) => legendRe.test(l.trim()))
    if (start < 0) return null
    let end = lines.length
    for (let i = start + 1; i < lines.length; i++) {
      if (/^#{1,6}\s/.test(lines[i])) {
        end = i
        break
      }
    }
    return { start, end, body: trimBlank(lines.slice(start + 1, end)).join('\n').trim() }
  }

  // Replace (or insert, or drop) the legend block inside the text that precedes the
  // first requirement block. An empty projection removes the block rather than
  // leaving a stub in a client-facing document.
  function applyLegend(headText, legend) {
    const body = (legend || '').trim()
    const lines = splitLines(headText.replace(/\s+$/, ''))
    const span = legendSpan(lines)
    if (!body) {
      if (span) lines.splice(span.start, span.end - span.start)
      return trimBlank(lines).join('\n')
    }
    const rendered = renderLegend(body).split('\n')
    if (span) lines.splice(span.start, span.end - span.start, ...rendered, '')
    else lines.push('', ...rendered)
    return trimBlank(lines).join('\n')
  }

  // Split the template into { head, sectionPreamble, blockTemplate, tail }: the
  // chrome before the first requirement block, the section heading + its comment
  // preamble (heading line to the first block), the single requirement-block
  // sub-template, and the chrome from the next H2 onward. The `<!-- Repeat ... -->`
  // marker between blocks is dropped.
  function splitTemplate(md) {
    const lines = splitLines(md)
    const sectionIdx = lines.findIndex((l) => sectionRe.test(l))
    if (sectionIdx < 0) throw new Error(`${label} template: no in-scope requirements section heading`)
    let blockStart = -1
    for (let i = sectionIdx + 1; i < lines.length; i++) {
      if (/^###\s+/.test(lines[i])) {
        blockStart = i
        break
      }
    }
    if (blockStart < 0) throw new Error(`${label} template: no \`### <WBS>\` requirement block to use as a sub-template`)
    let tailStart = lines.length
    let repeatIdx = -1
    for (let i = blockStart + 1; i < lines.length; i++) {
      if (repeatIdx < 0 && /^<!--\s*Repeat/i.test(lines[i].trim())) repeatIdx = i
      if (/^##\s+/.test(lines[i])) {
        tailStart = i
        break
      }
    }
    const blockEnd = repeatIdx >= 0 ? repeatIdx : tailStart
    return {
      head: lines.slice(0, blockStart).join('\n').replace(/\s+$/, '') + '\n',
      sectionPreamble: trimBlank(lines.slice(sectionIdx, blockStart)),
      blockTemplate: trimBlank(lines.slice(blockStart, blockEnd)),
      tail: lines.slice(tailStart).join('\n').trim() + '\n',
    }
  }

  // Fill a template block's line array: substitute the heading and replace the
  // read-only sub-section bodies, inserting a missing read-only sub-section at its
  // canonical position. LLM-owned sub-sections and the heading preamble stay stubs.
  function fillTemplateBlock(templateLines, block) {
    const bodies = projectedBodies(block)
    const heading = `### ${block.wbs}${DASH}${block.name}`
    let preambleEnd = templateLines.length
    const subs = []
    let cur = null
    for (let i = 1; i < templateLines.length; i++) {
      const h4 = /^####\s+(.+?)\s*$/.exec(templateLines[i])
      if (h4) {
        if (subs.length === 0) preambleEnd = i
        cur = { name: h4[1].trim(), body: [] }
        subs.push(cur)
      } else if (cur) {
        cur.body.push(templateLines[i])
      }
    }
    const preamble = templateLines.slice(1, preambleEnd)

    const setBody = (sub) => {
      if (READ_ONLY.some((n) => n.toLowerCase() === sub.name.toLowerCase())) {
        sub.body = [''].concat(bodies[canonName(sub.name)].split('\n')).concat([''])
      } else {
        sub.body = trimBlank(sub.body)
        sub.body = sub.body.length ? [''].concat(sub.body).concat(['']) : ['']
      }
    }
    subs.forEach(setBody)

    for (const name of READ_ONLY) {
      if (!subs.some((s) => s.name.toLowerCase() === name.toLowerCase())) {
        const rank = canonRank(name)
        let at = subs.findIndex((s) => canonRank(s.name) > rank)
        if (at < 0) at = subs.length
        subs.splice(at, 0, { name, body: [''].concat(bodies[name].split('\n')).concat(['']) })
      }
    }

    const out = [heading, ...preamble]
    for (const s of subs) out.push(`#### ${s.name}`, ...s.body)
    return out
  }

  function bootstrap(templateMd, blocks, legend = '') {
    const { head, blockTemplate, tail } = splitTemplate(templateMd)
    const rendered = blocks.map((b) => fillTemplateBlock(blockTemplate, b).join('\n'))
    return (
      (applyLegend(head, legend) + '\n\n' + rendered.join('\n\n') + '\n\n' + tail.trimStart())
        .replace(/\n{3,}/g, '\n\n')
        .trimEnd() + '\n'
    )
  }

  // Return a matched in-scope block's own line slice with ONLY the read-only
  // sub-section bodies replaced by the projection; a missing read-only sub-section
  // is inserted at its canonical position. All other lines (heading, preamble
  // comment, LLM enrichment) stay verbatim. Trailing blank lines are trimmed.
  function updateMatchedBlock(docLines, eb, block) {
    const base = eb.headingIdx
    const slice = docLines.slice(base, eb.endIdx)
    const bodies = projectedBodies(block)
    const edits = []
    for (const name of READ_ONLY) {
      const sec = eb.sections.find((s) => s.name.toLowerCase() === name.toLowerCase())
      const replacement = [''].concat(bodies[name].split('\n')).concat([''])
      if (sec) {
        edits.push({ start: sec.bodyStart - base, end: sec.bodyEnd - base, replacement })
      } else {
        const rank = canonRank(name)
        let after = null
        for (const s of eb.sections) if (canonRank(s.name) < rank) after = s
        const at = after ? after.bodyEnd - base : eb.preambleEnd >= 0 ? eb.preambleEnd - base : 1
        edits.push({ start: at, end: at, replacement: [`#### ${name}`, ...replacement] })
      }
    }
    edits.sort((a, b) => b.start - a.start)
    for (const e of edits) slice.splice(e.start, e.end - e.start, ...e.replacement)
    return slice.join('\n').trim()
  }

  // RECONCILE an existing document to the projected in-scope block set — the
  // feedback round-trip after a change/add/remove (fixed UPSTREAM, script re-run):
  //   - UPDATES a matched block's read-only bodies, preserving enrichment verbatim
  //     (byte-identical no-op when unchanged);
  //   - ADDS a block for a newly-covered WBS (from the template block) — requires
  //     `blockTemplate`;
  //   - REMOVES an in-scope block whose WBS is no longer covered (default mode only);
  //   - emits the blocks in the projected (slice) order.
  // `additive` (EXTEND) mode instead PRESERVES every existing in-scope block: it
  // updates matched blocks, keeps unmatched prior-change blocks verbatim (never
  // removes them), appends newly-covered blocks after, and leaves the published
  // legend untouched — so extending an already-published FSD/ISD never strips a
  // prior change's requirements. Everything outside the in-scope section is left
  // verbatim. When the doc has NO in-scope section yet, insert it: after the section
  // heading when present, else — when `insertBefore` is configured — the whole
  // section (its template preamble + blocks) before that heading; requires
  // `blockTemplate` + `sectionPreamble`.
  function surgicalUpdate(existingMd, blocks, blockTemplate = null, sectionPreamble = null, legend = '', { additive = false } = {}) {
    const doc = parseDoc(existingMd)
    const lines = doc.lines
    const inScope = doc.blocks.filter((b) => b.inScope)
    const existingByWbs = new Map(inScope.filter((b) => b.wbs).map((b) => [b.wbs, b]))
    const projectedByWbs = new Map(blocks.filter((b) => b.wbs).map((b) => [b.wbs, b]))

    const added = (block) => {
      if (!blockTemplate) {
        throw new Error(
          `surgicalUpdate: block ${block.wbs} is new (not in the existing document) and needs the ` +
            `block template to be added — pass blockTemplate (splitTemplate().blockTemplate).`,
        )
      }
      return fillTemplateBlock(blockTemplate, block).join('\n')
    }
    const verbatim = (eb) => lines.slice(eb.headingIdx, eb.endIdx).join('\n').trim()

    // Preserve prior blocks only when we actually have an existing section to keep.
    const preserveExisting = additive && inScope.length > 0

    let regionText
    if (preserveExisting) {
      // EXTEND: update matched in place, keep other prior blocks verbatim, append the new ones.
      const existingUnits = inScope.map((eb) => {
        const block = eb.wbs != null ? projectedByWbs.get(eb.wbs) : undefined
        return block ? updateMatchedBlock(lines, eb, block) : verbatim(eb)
      })
      const newUnits = blocks.filter((b) => !existingByWbs.has(b.wbs)).map(added)
      regionText = [...existingUnits, ...newUnits].join('\n\n')
    } else {
      const unit = (block) => {
        const eb = existingByWbs.get(block.wbs)
        return eb ? updateMatchedBlock(lines, eb, block) : added(block)
      }
      regionText = blocks.map(unit).join('\n\n')
    }

    let head, tail
    if (inScope.length) {
      const regionStart = inScope[0].headingIdx
      const regionEnd = inScope[inScope.length - 1].endIdx
      head = lines.slice(0, regionStart).join('\n')
      tail = lines.slice(regionEnd).join('\n')
    } else {
      const h2 = lines.findIndex((l) => sectionRe.test(l))
      if (h2 >= 0) {
        // Section heading present but empty: insert the region right after it (+ its comment preamble).
        let next = lines.length
        for (let i = h2 + 1; i < lines.length; i++) if (/^##\s+/.test(lines[i])) { next = i; break }
        head = lines.slice(0, next).join('\n')
        tail = lines.slice(next).join('\n')
      } else if (insertBefore) {
        // Section absent entirely (e.g. extending a legacy ISD): insert the whole
        // section (its template preamble + blocks) before the configured heading.
        if (!sectionPreamble) {
          throw new Error(
            `surgicalUpdate: the document has no in-scope section and none can be inserted without ` +
              `sectionPreamble (splitTemplate().sectionPreamble).`,
          )
        }
        const at = lines.findIndex((l) => l.trim() === insertBefore)
        const anchor = at >= 0 ? at : lines.length
        head = lines.slice(0, anchor).join('\n') + '\n\n' + sectionPreamble.join('\n')
        tail = lines.slice(anchor).join('\n')
      } else {
        throw new Error(`surgicalUpdate: existing document has no in-scope requirements section`)
      }
    }

    // EXTEND keeps the published legend as-is; a fresh/legacy insert projects the proposal's.
    const headText = preserveExisting ? head : applyLegend(head, legend)
    return [headText, regionText, tail]
      .join('\n\n')
      .replace(/\n{3,}/g, '\n\n')
      .trimEnd() + '\n'
  }

  return {
    READ_ONLY,
    canonRank,
    canonName,
    projectedBodies,
    parseDoc,
    splitTemplate,
    fillTemplateBlock,
    bootstrap,
    surgicalUpdate,
  }
}
