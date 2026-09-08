#!/usr/bin/env node
/**
 * Render drawio-diagrams goldens and saved eval candidates to PNG for VISUAL review.
 *
 * The eval harness only judges the authored ` ```drawio ` YAML text — it never renders an
 * image. This helper closes that gap: it wraps the skill's own scripts/generate.mjs so a human
 * can eyeball what each golden and each generated candidate actually looks like. It is NOT part
 * of the harness (it never touches run.mjs / lib/**), so it is safe to change freely.
 *
 * Every case asks for a diagram, so a candidate / golden markdown must carry one
 * ` ```drawio:<type>:<id> ` block. Per file we classify the outcome so a failed render is itself
 * signal:
 *   - rendered : a valid block produced a .drawio + .png
 *   - invalid  : no block at all, or a block that failed spec validation / render — both are
 *                real defects (a prose-only answer did not do the task)
 *
 * Usage:
 *   node render-candidates.mjs <path> [--out-dir <dir>] [--scale 1.5] [--background #ffffff]
 *
 *   <path>  a single .md file (a golden or one run-NN-candidate.md), OR a directory. For a
 *           directory we render every run-*-candidate.md in it (a saved .results run dir) and,
 *           if present, a top-level golden `diagram.md`.
 *   --out-dir  where PNGs go. Default: <dir-of-md>/rendered/<md-basename>/ per file, so a
 *              results dir gets a self-contained rendered/ tree. For a golden, pass the case's
 *              assets dir explicitly (e.g. --out-dir <case>/assets) so `./assets/<id>.png`
 *              references resolve next to the golden.
 */
import { readdirSync, statSync, existsSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { dirname, join, resolve, basename } from 'node:path'
import { execFileSync } from 'node:child_process'

const here = dirname(fileURLToPath(import.meta.url))
// here = .project/eval/evalsuite/skills/drawio-diagrams -> up 5 to repo root, then into the skill.
const GENERATE = resolve(here, '../../../../../skills/plan/drawio-diagrams/scripts/generate.mjs')

function parseArgs(argv) {
  const flags = {}
  const positional = []
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i]
    if (!a.startsWith('--')) { positional.push(a); continue }
    const key = a.slice(2)
    const next = argv[i + 1]
    if (next != null && !next.startsWith('--')) { flags[key] = next; i++ } else { flags[key] = true }
  }
  return { flags, positional }
}

/** Collect the markdown files to render for a given input path. */
function collectTargets(input) {
  const st = existsSync(input) ? statSync(input) : null
  if (!st) { console.error(`Path not found: ${input}`); process.exit(2) }
  if (st.isFile()) return [resolve(input)]
  const files = readdirSync(input)
  const targets = []
  const golden = join(input, 'diagram.md')
  if (existsSync(golden)) targets.push(resolve(golden))
  for (const f of files.sort()) {
    if (/^run-.*candidate\.md$/.test(f)) targets.push(resolve(join(input, f)))
  }
  if (!targets.length) {
    console.error(`No golden or run-*-candidate.md files under ${input}`)
    process.exit(2)
  }
  return targets
}

/** Render one markdown file; classify the outcome without throwing. */
function renderOne(md, { outDir, scale, background }) {
  const out = outDir ?? join(dirname(md), 'rendered', basename(md).replace(/\.md$/, ''))
  const args = [GENERATE, '--md', md, '--out-dir', out]
  if (scale) args.push('--scale', String(scale))
  if (background) args.push('--background', String(background))
  try {
    const stdout = execFileSync('node', args, { encoding: 'utf8', cwd: here, stdio: ['ignore', 'pipe', 'pipe'] })
    const parsed = JSON.parse(stdout)
    const pngs = (parsed.diagrams ?? []).map((d) => d.png).filter(Boolean)
    return { md, status: 'rendered', pngs }
  } catch (err) {
    const errText = String(err.stderr ?? err.stdout ?? err.message ?? '')
    if (/blocks found/.test(errText)) {
      return { md, status: 'invalid', pngs: [], detail: 'no drawio block in the answer — nothing to render' }
    }
    let detail = errText.trim()
    try { detail = JSON.stringify(JSON.parse(errText).error) } catch { /* keep raw */ }
    return { md, status: 'invalid', pngs: [], detail }
  }
}

const { flags, positional } = parseArgs(process.argv.slice(2))
if (!positional.length) {
  console.error('Usage: node render-candidates.mjs <path> [--out-dir <dir>] [--scale 1.5] [--background #ffffff]')
  process.exit(2)
}

const targets = collectTargets(positional[0])
const opts = { outDir: flags['out-dir'], scale: flags.scale, background: flags.background }
const results = targets.map((md) => renderOne(md, opts))

const counts = { rendered: 0, invalid: 0 }
console.log(`\nRendered ${targets.length} file(s) via ${GENERATE}:\n`)
for (const r of results) {
  counts[r.status]++
  const rel = r.md.replace(`${process.cwd()}/`, '')
  if (r.status === 'rendered') console.log(`  [rendered] ${rel}\n             ${r.pngs.join('\n             ')}`)
  else console.log(`  [INVALID ] ${rel}\n             ${r.detail}`)
}
console.log(`\nSummary: ${counts.rendered} rendered, ${counts.invalid} invalid.`)
// An INVALID candidate is a real defect worth flagging loudly; exit non-zero so a batch script
// can notice.
process.exit(counts.invalid > 0 ? 1 : 0)
