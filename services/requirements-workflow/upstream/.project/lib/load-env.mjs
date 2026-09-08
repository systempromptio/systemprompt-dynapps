/**
 * Shared env loader — reads `.cursor/.project/.env` and injects KEY=VALUE
 * pairs into process.env. Shell env always wins: keys already in process.env
 * are never overwritten.
 *
 * The .env file lives inside .project/ which is listed in .cursorignore so
 * the LLM never sees credentials. It is also listed in .gitignore so secrets
 * are never committed. Use .cursor/.project/.env.example (committed, no secrets) as
 * the reference for which keys are required.
 *
 * Usage (import side-effect only):
 *   import '../../../../../.project/lib/load-env.mjs'
 *   // path relative to caller (5 levels up = .cursor/ root → .project/.env)
 */

import { readFileSync } from 'node:fs'
import { resolve, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

const __dirname = dirname(fileURLToPath(import.meta.url))

// lib/ → .project/ → .env
const PROJECT_DIR = resolve(__dirname, '..')
const ENV_FILE = resolve(PROJECT_DIR, '.env')

function loadEnv() {
  let content
  try {
    content = readFileSync(ENV_FILE, 'utf8')
  } catch {
    // No .env file — rely on shell env. If required keys are missing,
    // auth.mjs will print a specific error below.
    return
  }

  for (const line of content.split('\n')) {
    const trimmed = line.trim()
    if (!trimmed || trimmed.startsWith('#')) continue
    const eqIdx = trimmed.indexOf('=')
    if (eqIdx === -1) continue
    const key = trimmed.slice(0, eqIdx).trim()
    const value = trimmed.slice(eqIdx + 1).trim()
    // Shell env wins — only set if not already present
    if (!(key in process.env)) process.env[key] = value
  }
}

loadEnv()
