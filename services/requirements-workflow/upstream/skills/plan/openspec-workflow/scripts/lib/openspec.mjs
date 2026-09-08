// IO helpers: resolve an OpenSpec artifact template via the CLI (with a dev/test
// override) and write a file only when its content changed (idempotent no-op).

import { readFileSync, writeFileSync, existsSync, mkdirSync } from 'node:fs'
import { basename, dirname, join, resolve } from 'node:path'
import { execFileSync } from 'node:child_process'

// Find the OpenSpec project root (the directory that contains an `openspec/` folder)
// by walking UP from the change dir. The `openspec` CLI resolves the project from its
// cwd, so we must invoke it there — not from wherever the script happened to be run.
// Returns undefined when no root is found (execFileSync then falls back to process cwd).
function openspecCwd(changeDir) {
  let dir = resolve(changeDir)
  for (let i = 0; i < 8; i++) {
    if (existsSync(join(dir, 'openspec'))) return dir
    const parent = dirname(dir)
    if (parent === dir) break
    dir = parent
  }
  return undefined
}

// Resolve an artifact template. Prefer the live, project-correct template
// (`openspec instructions <artifact> --json` -> .template); a path override is used
// in dev/tests. Only invoke when a bootstrap actually needs the template.
export function resolveTemplate(artifact, changeDir, templateOverride) {
  if (templateOverride) return readFileSync(templateOverride, 'utf8')
  const changeName = basename(resolve(changeDir))
  try {
    const json = execFileSync('openspec', ['instructions', artifact, '--change', changeName, '--json'], {
      encoding: 'utf8',
      cwd: openspecCwd(changeDir),
    })
    const parsed = JSON.parse(json)
    if (!parsed.template) throw new Error(`no \`template\` field in \`openspec instructions ${artifact} --json\``)
    return parsed.template
  } catch (e) {
    throw new Error(
      `Could not resolve the ${artifact}.md template. Pass --template <path>, or install the ` +
        `OpenSpec CLI (npm install -g @fission-ai/openspec@latest) and run inside an initialized project.\n` +
        `Underlying error: ${e.message}`,
    )
  }
}

export function writeIfChanged(path, md, dry) {
  if (dry) return { status: 'dry', md }
  const prev = existsSync(path) ? readFileSync(path, 'utf8') : null
  if (prev === md) return { status: 'noop' }
  mkdirSync(dirname(path), { recursive: true })
  writeFileSync(path, md, 'utf8')
  return { status: prev === null ? 'created' : 'updated' }
}
