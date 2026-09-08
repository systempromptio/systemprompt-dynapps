// Shared markdown primitives for the functional-artifacts projection (FSD + stories)
// and the future integration-artifacts (ISD) sibling. Pure, no IO.

// The em-dash-with-spaces separator EMITTED in every generated heading (the proposal
// slice `S# — <WBS> — <name>`, the spec Option-H `<WBS> — <name>`, and the FSD/ISD
// block `<WBS> — <name>`). Emission is always canonical; only PARSING is lenient.
export const DASH = ' — '
export const WBS_RE = /^[A-Za-z0-9]+(?:\.[A-Za-z0-9]+)+$/

// Lenient PARSE separator: authors (and LLMs) routinely type a plain hyphen or an
// en-dash instead of the em-dash. Accept any of em-dash / en-dash / hyphen when it is
// surrounded by spaces, so a malformed heading still splits correctly instead of
// collapsing the name to `null`. Requiring the surrounding spaces keeps a hyphenated
// name token (e.g. `Consent-Manager`) intact.
const HEAD_SEP_RE = / [—–-] /

export const splitLines = (md) => md.split(/\r?\n/)

// Split on the FIRST heading separator only, so a name that itself contains a spaced
// dash stays intact. Returns [before, after] or [text, null] when absent.
export function splitFirstDash(text) {
  const m = HEAD_SEP_RE.exec(text)
  if (!m) return [text.trim(), null]
  return [text.slice(0, m.index).trim(), text.slice(m.index + m[0].length).trim()]
}

// A heading at level 2-4 ends a level-4 sub-section body; a level-5 (`#####`) marker
// group heading does NOT (it is content inside the Acceptance Criteria body).
export const isSubsectionBoundary = (line) => /^#{2,4}\s/.test(line)

export function trimBlank(lines) {
  const out = [...lines]
  while (out.length && out[0].trim() === '') out.shift()
  while (out.length && out[out.length - 1].trim() === '') out.pop()
  return out
}

// Escape a pipe so a value stays inside one markdown table cell.
export const escCell = (s) => (s || '').replace(/\|/g, '\\|')

// ─── GitHub-flavoured pipe-table parsing (used for design cross-cutting rows) ──

// Parse every pipe table into { header: string[], rows: string[][] }.
export function parseTables(md) {
  const splitRow = (line) =>
    line
      .replace(/^\s*\|/, '')
      .replace(/\|\s*$/, '')
      .split('|')
      .map((c) => c.trim())
  const isDivider = (line) => /^\s*\|?[\s:|-]+\|?\s*$/.test(line) && line.includes('-')

  const lines = splitLines(md)
  const tables = []
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]
    if (!/^\s*\|/.test(line)) continue
    if (i + 1 >= lines.length || !isDivider(lines[i + 1])) continue
    const header = splitRow(line)
    const rows = []
    let j = i + 2
    for (; j < lines.length && /^\s*\|/.test(lines[j]); j++) {
      if (isDivider(lines[j])) continue
      rows.push(splitRow(lines[j]))
    }
    tables.push({ header, rows })
    i = j - 1
  }
  return tables
}

export function findTable(tables, required) {
  return tables.find((t) => {
    const joined = t.header.join(' | ').toLowerCase()
    return required.every((r) => joined.includes(r.toLowerCase()))
  })
}

export function columnIndex(header, name) {
  return header.findIndex((h) => h.toLowerCase().includes(name.toLowerCase()))
}
