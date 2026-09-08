#!/usr/bin/env node
/**
 * .project/scripts/init.mjs — project initialization
 *
 * Accepts project values as --flags so an LLM can pass them without interactive
 * stdin. With no flags, falls back to readline for human use.
 *
 * In flag mode, missing optional values become empty strings and missing required
 * values fail fast instead of prompting.
 *
 * Flags:
 *   --project-name=<name>
 *   --atlassian-base-url=<url>
 *   --jira-project-key=<key>
 *   --confluence-space-key=<key>
 *   --github-repo=<url>
 *   --project-path=<path>       Active project code path relative to project root (default: ".")
 *   --legacy-repo=<url>         Legacy repo URL — cloned into legacy_code/ at the project root
 *   --legacy-branch=<branch>    Branch to check out in the legacy repo (default: remote HEAD)
 *   --sfcc-site-id=<id>
 *   --sfcc-site-prefix=</prefix>
 *   --status
 *   --sync-openspec             Force-refresh tooling-owned openspec files from library templates
 *
 * Flows:
 *   NEW project  (.env.example absent) — Q&A (or flags) → .env.example + .env
 *   EXISTING project (.env.example present) — copy .env.example → .env if missing, then repair
 *     (clone missing repos, update ignore files, install deps) — idempotent.
 *
 * Code repo layout:
 *   PROJECT_CODE_PATH="."       — openspec/ and the git repo are at the project root (CODE_REPO)
 *   PROJECT_CODE_PATH="sf-next" — git repo lives in CODE_REPO/sf-next/; openspec/ is inside it.
 *     When the subfolder is missing and GITHUB_REPO is set, it is cloned automatically (SSH first,
 *     HTTPS fallback).
 *
 * Legacy repo: when LEGACY_REPO is set, clones the legacy codebase into legacy_code/ at CODE_REPO.
 *   It is gitignored in the storefront repo (carries its own .git from Bitbucket/GitHub).
 *   SSH first, HTTPS fallback.
 *
 * Usage:
 *   # human (interactive)
 *   node .cursor/.project/scripts/init.mjs
 *
 *   # LLM / CI (non-interactive)
 *   node .cursor/.project/scripts/init.mjs \
 *     --project-name=my-project \
 *     --atlassian-base-url=https://your-company.atlassian.net \
 *     --jira-project-key=PROJ \
 *     --confluence-space-key=PROJ \
 *     [--github-repo=https://github.com/org/repo] \
 *     [--project-path=sf-next] \
 *     [--legacy-repo=git@bitbucket.org:org/legacy.git] [--legacy-branch=main] \
 *     [--sfcc-site-id=MySite] [--sfcc-site-prefix=/uk]
 */

import { createInterface }                                from 'node:readline'
import { readFileSync, writeFileSync, existsSync,
         copyFileSync, cpSync, rmSync }                   from 'node:fs'
import { resolve, dirname, join }                        from 'node:path'
import { fileURLToPath }                                  from 'node:url'
import { execSync }                                       from 'node:child_process'

const __dirname   = dirname(fileURLToPath(import.meta.url))
const PROJECT_DIR = resolve(__dirname, '..')
const CURSOR_DIR  = resolve(__dirname, '../..')
const CODE_REPO   = resolve(__dirname, '../../..')

const ENV_EXAMPLE       = resolve(PROJECT_DIR, '.env.example')
const ENV_FILE          = resolve(PROJECT_DIR, '.env')
const TEMPLATE          = resolve(PROJECT_DIR, 'templates', 'env.example.tmpl')
const CURSORIGNORE_TMPL        = resolve(PROJECT_DIR, 'templates', 'cursorignore.tmpl')
const CURSORINDEXINGIGNORE_TMPL = resolve(PROJECT_DIR, 'templates', 'cursorindexingignore.tmpl')
const README_TARGET     = resolve(CURSOR_DIR, 'README.md')
const README_TMPL       = resolve(PROJECT_DIR, 'templates', 'README.post-init.md.tmpl')

// ─── Parse --flags ────────────────────────────────────────────────────────────

const FLAGS = parseFlags(process.argv.slice(2))

if (FLAGS.status || FLAGS.check) {
  printStatus()
  process.exit(0)
}

// Refresh the tooling-owned OpenSpec IP (config.yaml + schemas/) in the code repo from the library
// templates. These paths are gitignored (local-only), so /project-sync force-overwrites them from the
// shared library so every teammate's local schema/templates track our latest changes. Never touches
// openspec/specs or openspec/changes (client deliverables).
if (FLAGS['sync-openspec']) {
  syncOpenSpecSchemas()
  process.exit(0)
}

// ─── Existing project ─────────────────────────────────────────────────────────

if (existsSync(ENV_EXAMPLE)) {
  console.log('✓  .env.example found — project already initialized.')

  if (existsSync(ENV_FILE)) {
    console.log('✓  .env exists — running repair to restore any missing repos/scaffold.')
    console.log('')
    console.log('Tip: to re-initialize, delete .cursor/.project/.env.example and re-run.')
  } else {
    copyFileSync(ENV_EXAMPLE, ENV_FILE)
    console.log('')
    console.log('Copied  .env.example  →  .env')
    console.log('')
    printCredentialSteps()
  }

  console.log('')
  setupCodeRepo()
  setupLegacyRepo()
  setupBaseSfNext()
  setupProjectContext()
  process.exit(0)
}

// ─── New project ──────────────────────────────────────────────────────────────

hr()
console.log('  Project initialization — new project setup')
hr()
console.log('')

// In flags mode (at least one flag provided): skip interactive prompts entirely.
// Missing optional fields default to ''; missing required fields cause an error.
const FLAGS_MODE = Object.keys(FLAGS).length > 0

if (FLAGS_MODE) {
  console.log('Running non-interactively (flags provided).')
} else {
  console.log('Press Enter to skip optional fields.')
}
console.log('')

// Lazy readline — only opened when running interactively (no flags provided)
let rl = null
function getRL() {
  if (!rl) rl = createInterface({ input: process.stdin, output: process.stdout })
  return rl
}

function ask(flag, label, required = false) {
  if (FLAGS[flag] !== undefined) return Promise.resolve(FLAGS[flag])
  if (FLAGS_MODE) {
    if (required) {
      console.error(`Error: required flag --${flag} is missing.`)
      process.exit(1)
    }
    return Promise.resolve('')
  }
  const hint = required ? '' : ' (optional)'
  return new Promise((res) =>
    getRL().question(`  ${label}${hint}: `, (a) => res(a.trim()))
  )
}

const PROJECT_NAME       = await ask('project-name',          'Project name',                        true)
const ATLASSIAN_BASE_URL = await ask('atlassian-base-url',    'Atlassian base URL',                  true)
const JIRA_PROJECT_KEY   = await ask('jira-project-key',      'Jira project key',                    true)
const JIRA_BOARD_ID      = await ask('jira-board-id',         'Jira board ID (for sprint features)')
const CONFLUENCE_SPACE_KEY = await ask('confluence-space-key','Confluence space key',                true)
const GITHUB_REPO          = await ask('github-repo',         'GitHub repo URL')
const PROJECT_CODE_PATH    = normalizeProjectPath(
  await ask('project-path', 'Active project code path (relative to project root; blank = repo root)'))
const LEGACY_REPO          = await ask('legacy-repo',         'Legacy repo URL (cloned to legacy_code/)')
const LEGACY_BRANCH        = await ask('legacy-branch',       'Legacy repo branch')
const SFCC_SITE_ID         = await ask('sfcc-site-id',        'SFCC site ID')
const SFCC_SITE_PREFIX     = await ask('sfcc-site-prefix',    'SFCC site prefix (e.g. /uk)')
const SCAPI_CLIENT_ID      = await ask('scapi-client-id',     'SCAPI client ID (PUBLIC__app__commerce__api__clientId)')
const SCAPI_ORG_ID         = await ask('scapi-org-id',        'SCAPI organization ID (e.g. f_ecom_xxx_001)')
const SCAPI_SHORT_CODE     = await ask('scapi-short-code',    'SCAPI short code (e.g. kv7kzm78)')
const MRT_PROJECT          = await ask('mrt-project',         'MRT project slug (for pnpm push in base-sf-next/)')
const MRT_TARGET           = await ask('mrt-target',          'MRT target environment (default: development)')

if (rl) rl.close()

// ─── Generate files ───────────────────────────────────────────────────────────

let tmpl = readFileSync(TEMPLATE, 'utf8')
const values = {
  PROJECT_NAME, ATLASSIAN_BASE_URL, JIRA_PROJECT_KEY, JIRA_BOARD_ID,
  CONFLUENCE_SPACE_KEY, GITHUB_REPO, PROJECT_CODE_PATH, SFCC_SITE_ID, SFCC_SITE_PREFIX,
  LEGACY_REPO, LEGACY_BRANCH,
  SCAPI_CLIENT_ID, SCAPI_ORG_ID, SCAPI_SHORT_CODE, MRT_PROJECT, MRT_TARGET,
}
for (const [key, val] of Object.entries(values)) {
  tmpl = tmpl.replaceAll(`{{${key}}}`, val)
}

writeFileSync(ENV_EXAMPLE, tmpl, 'utf8')
copyFileSync(ENV_EXAMPLE, ENV_FILE)

const readmeWritten = writeProjectReadme(PROJECT_NAME)

hr()
console.log('  Generated')
hr()
console.log('  .cursor/.project/.env.example  ← commit this (no secrets)')
console.log('  .cursor/.project/.env          ← yours only (gitignored)')
if (readmeWritten) console.log('  .cursor/README.md              ← project-specific (commit to project branch)')
console.log('')
setupCodeRepo()
setupLegacyRepo()
setupBaseSfNext()
setupSkillScriptDeps()
setupProjectContext()
console.log('')
hr()
console.log('  Next steps')
hr()
console.log('')
printCredentialSteps()
console.log('')
console.log('  2. Commit .env.example to the library repo:')
console.log('       git -C .cursor add .project/.env.example')
console.log('       git -C .cursor commit -m "chore: init project config"')
console.log('')

// ─── Code repo setup ──────────────────────────────────────────────────────────

function setupCodeRepo() {
  const projectPath = getProjectPath()
  const op = projectPath === '.' ? '/openspec' : `/${projectPath}/openspec`

  // When PROJECT_CODE_PATH is a subfolder, the git repo lives there — not at CODE_REPO root.
  // Try to clone it from GITHUB_REPO if it's missing.
  if (projectPath !== '.') {
    const subfolderGit = join(CODE_REPO, projectPath, '.git')
    if (!existsSync(subfolderGit)) {
      if (!connectSubfolderRepo(projectPath)) {
        console.log('')
        hr()
        console.log(`  ⚠  CODE REPO SKIPPED`)
        hr()
        console.log(`  ${join(CODE_REPO, projectPath)}`)
        console.log('  is not a git repository and GITHUB_REPO is not set (or clone failed).')
        console.log(`  Clone your project into ${projectPath}/ manually, then re-run init.`)
        return
      }
    }
  } else if (!existsSync(join(CODE_REPO, '.git'))) {
    console.log('')
    hr()
    console.log('  ⚠  CODE REPO SKIPPED')
    hr()
    console.log(`  ${CODE_REPO}`)
    console.log('  is not a git repository, so the code-repo half of setup was skipped:')
    console.log('  no .gitignore / .cursorignore / openspec/ scaffold, and no code-repo commit.')
    console.log('')
    console.log('  Make the code repo a git repository first (init it or clone your project repo')
    console.log('  into this folder), then re-run the initializer. Manual fallback entries:')
    console.log('     .gitignore             → /.cursor/')
    console.log(`                               ${op}/changes/**/fsd/*.baseline.md`)
    console.log(`                               ${op}/changes/**/isd/*.baseline.md`)
    console.log(`                               ${op}/config.yaml`)
    console.log(`                               ${op}/schemas.yaml`)
    console.log(`                               ${op}/schemas/`)
    console.log('     .cursorignore          → .cursor/.project/.env')
    console.log('                               .cursor/.project/project-context/.index/')
    console.log('                               .cursor/.project/project-context/node_modules/')
    console.log('                               .cursor/node_modules/')
    console.log('     .cursorindexingignore  → .cursor/.project/')
    console.log('                               .cursor/package.json')
    console.log('                               .cursor/package-lock.json')
    return
  }

  // Determine the git root for writing gitignore. When project code is in a subfolder, that
  // subfolder is its own git repo — gitignore entries go there (not at CODE_REPO root).
  const isNestedLayout = projectPath !== '.'
  const gitRoot        = isNestedLayout ? join(CODE_REPO, projectPath) : CODE_REPO

  if (isNestedLayout) {
    console.log(`\nStorefront Next: ${gitRoot}`)
    setupWorkspaceIgnores()
  } else {
    console.log(`Code repo: ${CODE_REPO}`)
  }

  // openspec gitignore entries go into the actual git repo root (.gitignore of the project code).
  const gitignorePath    = join(gitRoot, '.gitignore')
  const gitignoreEntries = [
    // /.cursor/ only when .cursor/ is inside the git repo (root layout).
    ...(!isNestedLayout ? [ensureEntry(gitignorePath, '/.cursor/')] : []),
    ensureEntry(gitignorePath, `/openspec/changes/**/fsd/*.baseline.md`),
    ensureEntry(gitignorePath, `/openspec/changes/**/isd/*.baseline.md`),
    // Tooling IP — the OpenSpec schema, artifact templates, and config are our prompts. Keep them
    // out of the client's code repo; they live locally and are refreshed from the library by
    // /project-sync (node .cursor/.project/scripts/init.mjs --sync-openspec).
    ensureEntry(gitignorePath, `/openspec/config.yaml`),
    ensureEntry(gitignorePath, `/openspec/schemas.yaml`),
    ensureEntry(gitignorePath, `/openspec/schemas/`),
  ]
  const gitignoreChanged = gitignoreEntries.some(Boolean)

  const cursorignoreChanged         = isNestedLayout
    ? false
    : ensureEntriesFromTemplate(join(CODE_REPO, '.cursorignore'), CURSORIGNORE_TMPL)
  const cursorindexingignoreChanged = isNestedLayout
    ? false
    : ensureEntriesFromTemplate(join(CODE_REPO, '.cursorindexingignore'), CURSORINDEXINGIGNORE_TMPL)

  if (gitignoreChanged)                console.log('  ✓  Updated .gitignore')
  else                                 console.log('  ✓  .gitignore already up to date')

  if (!isNestedLayout) {
    if (cursorignoreChanged)           console.log('  ✓  Wrote .cursorignore from template')
    else                               console.log('  ✓  .cursorignore already up to date')
    if (cursorindexingignoreChanged)   console.log('  ✓  Wrote .cursorindexingignore from template')
    else                               console.log('  ✓  .cursorindexingignore already up to date')
  }

  copyOpenSpecScaffold()
  updateProjectContextPaths(projectPath)

  if (!isNestedLayout) {
    // No auto-commit. init only sets up files; committing/pushing is a manual step the lead does
    // explicitly. openspec/config.yaml + openspec/schemas/ are gitignored (tooling IP), so a manual
    // `git add openspec` picks up only the client-facing scaffold (specs/, changes/).
    console.log('')
    console.log('  ℹ  openspec/config.yaml + openspec/schemas/ are gitignored (tooling IP, local-only).')
    console.log('  ℹ  Review and commit the code-repo config yourself, e.g.:')
    console.log(`       cd "${CODE_REPO}"`)
    console.log('       git add .gitignore .cursorignore .cursorindexingignore openspec')
    console.log('       git commit -m "chore: add Cursor AI tooling config"')
  }
}

// ─── Workspace ignore files (nested layout only) ──────────────────────────────

// When PROJECT_CODE_PATH is a subfolder, the workspace root is not a git repo.
// Only .cursorignore and .cursorindexingignore are meaningful there.
function setupWorkspaceIgnores() {
  console.log(`Workspace: ${CODE_REPO}`)
  const cursorignoreChanged = ensureEntriesFromTemplate(
    join(CODE_REPO, '.cursorignore'), CURSORIGNORE_TMPL)
  const cursorindexingignoreChanged = ensureEntriesFromTemplate(
    join(CODE_REPO, '.cursorindexingignore'), CURSORINDEXINGIGNORE_TMPL)
  if (cursorignoreChanged)           console.log('  ✓  Wrote .cursorignore from template')
  else                               console.log('  ✓  .cursorignore already up to date')
  if (cursorindexingignoreChanged)   console.log('  ✓  Wrote .cursorindexingignore from template')
  else                               console.log('  ✓  .cursorindexingignore already up to date')
}

// ─── Clone the project code repo into the subfolder ───────────────────────────

// Used only when PROJECT_CODE_PATH !== '.' and the subfolder is missing.
// Tries SSH first, falls back to HTTPS. Returns true when the subfolder ends up with a HEAD commit.
function connectSubfolderRepo(projectPath) {
  const repoUrl = readEnvValue('GITHUB_REPO')
  if (!repoUrl) return false

  const destDir  = join(CODE_REPO, projectPath)
  const sshUrl   = toSshUrl(repoUrl)
  const httpsUrl = toHttpsUrl(repoUrl)

  console.log('')
  console.log(`Storefront Next — cloning GITHUB_REPO into ${projectPath}/:`)
  console.log(`  ${sshUrl}`)

  if (!cloneRemoteRepo(sshUrl, httpsUrl, projectPath)) return false

  try {
    execSync('git rev-parse HEAD', { cwd: destDir, stdio: 'pipe' })
    return true
  } catch {
    return false
  }
}

// ─── Clone Storefront Next template into base-sf-next/ ────────────────────────

// Always clones the public SF Next template (branch: latest) into base-sf-next/ at CODE_REPO
// so every developer has the canonical upstream for diffing and reference, without modifying it.
// Carries its own .git — for reference only. Skipped when base-sf-next/.git already exists.
function setupBaseSfNext() {
  const SF_NEXT_TEMPLATE_URL = 'https://github.com/SalesforceCommerceCloud/storefront-next-template'
  const SF_NEXT_BRANCH       = 'latest'
  const destDir              = join(CODE_REPO, 'base-sf-next')

  console.log('')
  console.log('Storefront Next template (reference):')

  if (existsSync(join(destDir, '.git'))) {
    console.log('  ✓  base-sf-next/ already present — skipping clone')
    generateBaseSfNextEnv()
    applyBaseSfNextTemplates()
    return
  }

  if (existsSync(destDir)) {
    console.log('  ⚠  base-sf-next/ exists but is not a git repository — skipping clone')
    console.log('       Remove or rename the folder, then re-run init.')
    return
  }

  const sshUrl   = toSshUrl(SF_NEXT_TEMPLATE_URL)
  const httpsUrl = toHttpsUrl(SF_NEXT_TEMPLATE_URL)

  console.log('  Cloning storefront-next-template into base-sf-next/:')
  console.log(`  ${httpsUrl}  (branch: ${SF_NEXT_BRANCH})`)

  if (cloneRemoteRepo(sshUrl, httpsUrl, 'base-sf-next', SF_NEXT_BRANCH)) {
    console.log('  ✓  Cloned base-sf-next/ from storefront-next-template')
    generateBaseSfNextEnv()
    applyBaseSfNextTemplates()
  } else {
    console.log('  ⚠  Could not clone storefront-next-template.')
    console.log(`       cd "${CODE_REPO}"`)
    console.log(`       git clone --branch ${SF_NEXT_BRANCH} ${httpsUrl} base-sf-next`)
  }
}

// ─── Apply project-specific overrides to base-sf-next/ ────────────────────────

// Copies any files from .project/templates/base-sf-next/ into base-sf-next/, overwriting
// the vanilla cloned versions. Called after clone and on every repair run so project-specific
// customisations (config.server.ts, etc.) always stay applied.
function applyBaseSfNextTemplates() {
  const templatesDir = resolve(PROJECT_DIR, 'templates', 'base-sf-next')
  const destDir      = join(CODE_REPO, 'base-sf-next')

  if (!existsSync(templatesDir)) return

  cpSync(templatesDir, destDir, { recursive: true, force: true })
  console.log('  ✓  Applied project-specific overrides to base-sf-next/ (config.server.ts, ...)')
}

// ─── Generate base-sf-next/.env from .env.default + project credentials ───────

// Copies .env.default → .env in base-sf-next/, then overwrites SCAPI and MRT variables
// with project-specific values from the project .env.example/.env when they are non-empty.
// Called after a fresh clone and on every repair run so the file stays current.
// No-op when .env.default is absent (non-standard clone state).
function generateBaseSfNextEnv() {
  const destDir      = join(CODE_REPO, 'base-sf-next')
  const envDefault   = join(destDir, '.env.default')
  const envTarget    = join(destDir, '.env')

  if (!existsSync(envDefault)) {
    console.log('  ⚠  base-sf-next/.env.default not found — skipping .env generation')
    return
  }

  const SCAPI_MAP = {
    'PUBLIC__app__commerce__api__clientId':       'SCAPI_CLIENT_ID',
    'PUBLIC__app__commerce__api__organizationId': 'SCAPI_ORG_ID',
    'PUBLIC__app__commerce__api__shortCode':      'SCAPI_SHORT_CODE',
    'PUBLIC__app__defaultSiteId':                 'SFCC_SITE_ID',
    'MRT_PROJECT':                                'MRT_PROJECT',
    'MRT_TARGET':                                 'MRT_TARGET',
  }

  let content = readFileSync(envDefault, 'utf8')

  for (const [sfNextKey, projectKey] of Object.entries(SCAPI_MAP)) {
    const projectVal = readEnvValue(projectKey)
    if (!projectVal) continue
    const regex = new RegExp(`^(${sfNextKey}=).*$`, 'm')
    if (regex.test(content)) {
      content = content.replace(regex, `$1${projectVal}`)
    } else {
      content = content.trimEnd() + `\n${sfNextKey}=${projectVal}\n`
    }
  }

  writeFileSync(envTarget, content, 'utf8')
  console.log('  ✓  Generated base-sf-next/.env from .env.default + project credentials')
}

// ─── Clone legacy repository into legacy_code/ ────────────────────────────────

// When LEGACY_REPO is set, git-clones the legacy codebase into legacy_code/ at CODE_REPO.
// The folder is gitignored in the storefront repo — it carries its own .git from Bitbucket/GitHub.
// Skipped when legacy_code/.git already exists.
function setupLegacyRepo() {
  const repoUrl = readEnvValue('LEGACY_REPO')
  if (!repoUrl) return

  const legacyDir = join(CODE_REPO, 'legacy_code')

  console.log('')
  console.log('Legacy code (reference):')

  if (existsSync(join(legacyDir, '.git'))) {
    console.log('  ✓  legacy_code/ already present — skipping clone')
    return
  }

  if (existsSync(legacyDir)) {
    console.log('  ⚠  legacy_code/ exists but is not a git repository — skipping clone')
    console.log('       Remove or rename the folder, then re-run init.')
    return
  }

  const sshUrl   = toSshUrl(repoUrl)
  const httpsUrl = toHttpsUrl(repoUrl)
  const branch   = readEnvValue('LEGACY_BRANCH')

  console.log('  Cloning LEGACY_REPO into legacy_code/:')
  console.log(`  ${sshUrl}`)
  if (branch) console.log(`  branch: ${branch}`)

  if (cloneRemoteRepo(sshUrl, httpsUrl, 'legacy_code', branch)) {
    console.log('  ✓  Cloned legacy_code/ from LEGACY_REPO')
  } else {
    console.log('  ⚠  Could not clone legacy repo (auth / host key / wrong URL / branch).')
    console.log(`       cd "${CODE_REPO}"`)
    const manualUrl = httpsUrl || sshUrl
    if (branch) {
      console.log(`       git clone --branch ${branch} ${manualUrl} legacy_code`)
    } else {
      console.log(`       git clone ${manualUrl} legacy_code`)
    }
  }
}

// ─── Skill-script shared dependencies ─────────────────────────────────────────

// Installs the shared Node deps used by skill scripts (Jira/Confluence CLI, FSD/ISD
// templating, markdown round-trip) from the .cursor/ root package.json. Node resolves
// node_modules by walking up the tree, so a single root install serves every
// skills/**/scripts/ file. node_modules is gitignored (whole .cursor/ is), so each clone
// needs its own install. Non-fatal: print the manual fallback and continue on failure.
function setupSkillScriptDeps() {
  if (!existsSync(resolve(CURSOR_DIR, 'package.json'))) return

  console.log('')
  console.log('Skill scripts (shared Node dependencies):')
  try {
    execSync('npm install --no-audit --no-fund', { cwd: CURSOR_DIR, stdio: 'inherit' })
    console.log('  ✓  Installed skill-script dependencies at .cursor/')
  } catch {
    console.log('  ⚠  npm install failed — install skill-script deps manually:')
    console.log('       cd .cursor && npm install')
  }
}

// ─── Project Context setup ────────────────────────────────────────────────────

// Installs the project-context engine's dependencies. node_modules is gitignored, so every
// clone (lead or dev-join) needs its own install before the MCP server / sync can run.
// Safe no-op if the engine folder isn't present. Failure is non-fatal — we print the manual
// fallback and continue, so a missing network never blocks the rest of init.
function setupProjectContext() {
  const engineDir = resolve(PROJECT_DIR, 'project-context')
  if (!existsSync(engineDir)) return

  console.log('')
  console.log('Project Context (local project memory):')
  try {
    execSync('npm install', { cwd: engineDir, stdio: 'inherit' })
    console.log('  ✓  Installed project-context engine dependencies')
  } catch {
    console.log('  ⚠  npm install failed — install engine deps manually:')
    console.log('       cd .cursor/.project/project-context && npm install')
  }
  console.log('  → Once credentials are set, run /project-context:sync to build the index.')
}

// ─── OpenSpec setup ───────────────────────────────────────────────────────────

// Copies the openspec/ scaffold into the code repo. No git guard — always called from within
// setupCodeRepo (which has already confirmed the code repo is a git repository).
function copyOpenSpecScaffold() {
  console.log('')
  console.log('OpenSpec:')

  // The runtime `openspec` CLI (opsx list/validate/archive) is still needed — warn if missing,
  // but do not skip the scaffold copy: the files are useful and the CLI can be installed later.
  try {
    execSync('openspec --version', { stdio: 'pipe' })
  } catch {
    console.log('  ⚠  openspec CLI not found — install for runtime commands:')
    console.log('     npm install -g @fission-ai/openspec@latest')
  }

  // Pure folder copy — the template mirrors a real openspec/ tree (config.yaml, schemas/feature,
  // empty specs + changes/archive). No `openspec init`: OpenSpec is file-based, and init would
  // otherwise clobber the toolkit's own curated .cursor commands/skills.
  // force:false + errorOnExist:false = fill only missing files, never overwrite an existing project.
  const openspecTemplateSrc = join(PROJECT_DIR, 'templates', 'openspec')
  const openspecDest        = openspecDirFor(getProjectPath())

  if (existsSync(openspecTemplateSrc)) {
    cpSync(openspecTemplateSrc, openspecDest, { recursive: true, force: false, errorOnExist: false })
    console.log(`  ✓  openspec/ scaffold ready at ${relFromCodeRepo(openspecDest)} (config, schemas/feature, specs, changes)`)
  }
}

// ─── OpenSpec schema refresh (--sync-openspec) ────────────────────────────────

// Force-overwrite the tooling-owned OpenSpec files (config.yaml, .gitignore, schemas/) in the code
// repo from the library templates. Called by /project-sync so a schema/template change we make in the
// library propagates to every teammate's gitignored local copy. schemas/ is wiped-then-copied so
// removed template files don't linger; specs/ and changes/ (client deliverables) are never touched.
function syncOpenSpecSchemas() {
  const templatesOpenspec = join(PROJECT_DIR, 'templates', 'openspec')
  const destOpenspec      = openspecDirFor(getProjectPath())

  if (!existsSync(destOpenspec)) {
    console.error(`✗  openspec/ not found at ${relFromCodeRepo(destOpenspec)} — run /project-init first.`)
    process.exit(1)
  }

  console.log(`OpenSpec schema refresh (tooling IP → ${relFromCodeRepo(destOpenspec)}):`)

  const srcConfig = join(templatesOpenspec, 'config.yaml')
  const destConfig = join(destOpenspec, 'config.yaml')
  if (existsSync(srcConfig)) {
    copyFileSync(srcConfig, destConfig)
    console.log('  ✓  Updated openspec/config.yaml')
  }

  // Tooling-owned too: the ignore rules follow from how our workflow writes into changes/.
  const srcIgnore  = join(templatesOpenspec, '.gitignore')
  const destIgnore = join(destOpenspec, '.gitignore')
  if (existsSync(srcIgnore)) {
    copyFileSync(srcIgnore, destIgnore)
    console.log('  ✓  Updated openspec/.gitignore')
  }

  const srcSchemas  = join(templatesOpenspec, 'schemas')
  const destSchemas = join(destOpenspec, 'schemas')
  if (existsSync(srcSchemas)) {
    rmSync(destSchemas, { recursive: true, force: true })
    cpSync(srcSchemas, destSchemas, { recursive: true, force: true })
    console.log('  ✓  Updated openspec/schemas/ (schema.yaml + artifact templates)')
  }

  console.log('  ℹ  config.yaml + schemas/ are gitignored (local-only) — refreshed from the tooling library.')
}

// ─── Project Context path update ─────────────────────────────────────────────

// Updates project-context/config.yaml source paths to reflect PROJECT_CODE_PATH.
// When the project code is in a subfolder (e.g. sf-next), specs and archive paths must
// include that prefix. No-op when config.yaml is absent or paths are already correct.
function updateProjectContextPaths(projectPath) {
  const configPath = resolve(PROJECT_DIR, 'project-context', 'config.yaml')
  if (!existsSync(configPath)) return

  const prefix  = projectPath === '.' ? '' : `${projectPath}/`
  const specsPath   = `../../../${prefix}openspec/specs`
  const archivePath = `../../../${prefix}openspec/changes/archive`

  let content = readFileSync(configPath, 'utf8')
  const original = content

  // Replace existing path: values for specs and openspec_archive sources.
  content = content.replace(
    /^(\s+path:\s+)\.\.\/\.\.\/\.\.\/.*openspec\/specs\s*$/m,
    `$1${specsPath}`,
  )
  content = content.replace(
    /^(\s+path:\s+)\.\.\/\.\.\/\.\.\/.*openspec\/changes\/archive\s*$/m,
    `$1${archivePath}`,
  )

  if (content !== original) {
    writeFileSync(configPath, content, 'utf8')
    console.log('  ✓  Updated project-context/config.yaml source paths')
  }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

// Normalize a user-supplied project code path (relative to the project root, where .cursor/ lives).
// Blank, "." and "./" all mean the repo root. Strips a leading "./" and any trailing slashes so the
// value is safe to use both as a path segment and as a gitignore prefix.
function normalizeProjectPath(p) {
  const v = (p || '').trim().replace(/^\.\//, '').replace(/\/+$/, '')
  return v === '' || v === '.' ? '.' : v
}

// Resolve the active project code path: an explicit --project-path flag wins, otherwise fall back to
// PROJECT_CODE_PATH persisted in .env.example/.env (so dev-join and --sync-openspec find the right
// openspec/ without re-asking). Defaults to the repo root.
function getProjectPath() {
  if (FLAGS['project-path'] !== undefined) return normalizeProjectPath(FLAGS['project-path'])
  return normalizeProjectPath(readEnvValue('PROJECT_CODE_PATH', '.'))
}

// Absolute path to the openspec/ tree for a given normalized project path.
function openspecDirFor(projectPath) {
  return join(CODE_REPO, projectPath, 'openspec')
}

// Display a path relative to the code repo root (e.g. "openspec" or "sf-next/openspec").
function relFromCodeRepo(absPath) {
  const rel = absPath.slice(CODE_REPO.length).replace(/^[/\\]+/, '')
  return rel || '.'
}

// Normalize an https(s) GitHub/Bitbucket URL to its SSH form; pass through git@… URLs unchanged.
function toSshUrl(url) {
  const trimmed = url.trim().replace(/\/+$/, '')
  if (trimmed.startsWith('git@')) return trimmed
  const m = trimmed.match(/^https?:\/\/([^/]+)\/(.+?)(?:\.git)?$/)
  if (!m) return trimmed
  return `git@${m[1]}:${m[2]}.git`
}

// Normalize git@… or https URLs to HTTPS; used as automatic fallback when SSH fails.
function toHttpsUrl(url) {
  const trimmed = url.trim().replace(/\/+$/, '')
  if (/^https?:\/\//.test(trimmed)) {
    return trimmed.endsWith('.git') ? trimmed : `${trimmed}.git`
  }
  const m = trimmed.match(/^git@([^:]+):(.+?)(?:\.git)?$/)
  if (m) return `https://${m[1]}/${m[2]}.git`
  return trimmed
}

// git clone with SSH-first, HTTPS-fallback. Returns true when the target directory was created.
function cloneRemoteRepo(preferredUrl, fallbackUrl, destName, branch) {
  const urls = preferredUrl === fallbackUrl ? [preferredUrl] : [preferredUrl, fallbackUrl]

  for (let i = 0; i < urls.length; i++) {
    const url = urls[i]
    try {
      const cloneCmd = branch
        ? `clone --branch ${branch} ${url} ${destName}`
        : `clone ${url} ${destName}`
      execSync(`git ${cloneCmd}`, { cwd: CODE_REPO, stdio: 'inherit' })
      if (i > 0) console.log(`  ✓  Cloned via HTTPS fallback (${url})`)
      return true
    } catch {
      if (i === 0 && urls.length > 1) {
        console.log('  ⚠  SSH clone failed — retrying with HTTPS:')
        console.log(`  ${urls[1]}`)
      }
    }
  }

  return false
}

// Read a KEY=value from .env.example first (committed, always present after init), then .env.
function readEnvValue(key, fallback = '') {
  for (const f of [ENV_EXAMPLE, ENV_FILE]) {
    if (!existsSync(f)) continue
    for (const line of readFileSync(f, 'utf8').split('\n')) {
      const t = line.trim()
      if (!t || t.startsWith('#')) continue
      const eq = t.indexOf('=')
      if (eq === -1) continue
      if (t.slice(0, eq).trim() === key) return t.slice(eq + 1).trim()
    }
  }
  return fallback
}

function parseFlags(args) {
  const result = {}
  for (const arg of args) {
    if (!arg.startsWith('--')) continue
    const eq = arg.indexOf('=')
    if (eq === -1) {
      result[arg.slice(2)] = true
    } else {
      result[arg.slice(2, eq)] = arg.slice(eq + 1)
    }
  }
  return result
}

function ensureEntry(filePath, entry) {
  const content = existsSync(filePath) ? readFileSync(filePath, 'utf8') : ''
  const bare    = entry.replace(/^\//, '')
  const lines   = content.split('\n').map((l) => l.trim())
  const exists  = lines.some((l) => l === entry || l === bare || l === `/${bare}`)
  if (exists) return false
  const sep = content.length && !content.endsWith('\n') ? '\n' : ''
  writeFileSync(filePath, `${content}${sep}${entry}\n`, 'utf8')
  return true
}

function ensureEntriesFromTemplate(targetPath, templatePath) {
  const tmpl    = readFileSync(templatePath, 'utf8')
  const entries = tmpl.split('\n').map((l) => l.trim()).filter((l) => l && !l.startsWith('#'))

  if (!existsSync(targetPath)) {
    writeFileSync(targetPath, tmpl, 'utf8')
    return true
  }

  let changed = false
  for (const entry of entries) {
    if (ensureEntry(targetPath, entry)) changed = true
  }
  return changed
}

function writeProjectReadme(projectName) {
  if (!existsSync(README_TMPL)) return false
  const branch = `project/${projectName}`
  let readme = readFileSync(README_TMPL, 'utf8')
  readme = readme
    .replaceAll('{{PROJECT_NAME}}', projectName)
    .replaceAll('{{PROJECT_BRANCH}}', branch)
  writeFileSync(README_TARGET, readme, 'utf8')
  return true
}

function printStatus() {
  const hasEnvExample = existsSync(ENV_EXAMPLE)
  const hasEnv = existsSync(ENV_FILE)

  if (!hasEnvExample) {
    console.log('Status: new project')
    console.log('Action: collect project details, then run init.mjs with flags.')
    return
  }

  if (!hasEnv) {
    console.log('Status: project initialized, local .env missing')
    console.log('Action: run init.mjs without flags to copy .env.example to .env.')
    return
  }

  console.log('Status: project initialized, local .env exists')
  console.log('Action: do not ask initialization questions.')
}

function hr() {
  console.log('──────────────────────────────────────────────────────────────')
}

function printCredentialSteps() {
  console.log('  1. Open .cursor/.project/.env and fill in credentials:')
  console.log('')
  console.log('       ATLASSIAN_EMAIL=your@email.com')
  console.log('       ATLASSIAN_API_TOKEN=<token>')
  console.log('       AWS_BEARER_TOKEN_BEDROCK=ABSK...   (project-context: AWS console → Bedrock → API keys)')
  console.log('')
  console.log('       Get token: https://id.atlassian.com/manage-profile/security/api-tokens')
}
