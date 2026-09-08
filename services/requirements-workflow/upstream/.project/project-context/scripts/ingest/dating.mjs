// Document dating + per-source date manifest.
//
// Every document gets one "effective" date, resolved by priority:
//   1. frontmatter  (updated / created / date - present for Atlassian sources)
//   2. LLM          (extracted from the text - for PDFs, meetings, specs)
//   3. mtime        (file modification time - last-resort fallback)
//
// The resolved date is cached in a per-source manifest at inbox/<type>/.dates.json
// keyed by the document's path within the source, with its md5. The manifest is
// the source of truth for dates: it lets teammates share identical dates for
// non-deterministic sources (LLM-dated meeting notes) via git, and it avoids
// re-running the LLM when a file's content is unchanged (md5 match).
//
// The date is metadata only - it is never written into chunk text.

import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import { join, dirname } from 'node:path';

export const MANIFEST_FILE = '.dates.json';

const ISO_DATE = /\b(\d{4}-\d{2}-\d{2})\b/;

/** Parse a leading YAML frontmatter block into a flat { key: value } map. */
export function parseFrontmatter(content) {
  if (!content.startsWith('---')) return {};
  const end = content.indexOf('\n---', 3);
  if (end === -1) return {};
  const block = content.slice(3, end);
  const out = {};
  for (const line of block.split('\n')) {
    const m = line.match(/^([A-Za-z0-9_]+):\s*(.*)$/);
    if (m) out[m[1]] = m[2].trim();
  }
  return out;
}

/** Effective date from frontmatter: prefer `updated`, then `created`, then `date`. */
function frontmatterDate(content) {
  const fm = parseFrontmatter(content);
  const raw = fm.updated || fm.created || fm.date;
  const m = raw ? String(raw).match(ISO_DATE) : null;
  return m ? m[1] : null;
}

/** File mtime (seconds) as a YYYY-MM-DD string. */
function mtimeDate(mtimeSeconds) {
  return new Date(mtimeSeconds * 1000).toISOString().slice(0, 10);
}

const DATE_SYSTEM_PROMPT =
  'You extract the single most representative date of a document: the date it was created ' +
  'or the date it describes (e.g. the meeting date for meeting notes, the version date for a ' +
  'spec). Reply with ONLY that date as YYYY-MM-DD, or the exact word NONE if there is no clear ' +
  'date in the text.';

/** Ask the LLM for the document date. Returns YYYY-MM-DD or null. */
async function llmDate(content, llm) {
  try {
    const out = await llm(DATE_SYSTEM_PROMPT, content.slice(0, 2000));
    const m = String(out).match(ISO_DATE);
    return m ? m[1] : null;
  } catch {
    return null;
  }
}

/**
 * Resolve a document's effective date.
 * @returns {Promise<{date: string, source: 'frontmatter'|'llm'|'mtime'}>}
 */
export async function resolveDate({ content, mtime, llm = null, useLLM = true }) {
  const fm = frontmatterDate(content);
  if (fm) return { date: fm, source: 'frontmatter' };
  if (useLLM && llm) {
    const d = await llmDate(content, llm);
    if (d) return { date: d, source: 'llm' };
  }
  return { date: mtimeDate(mtime), source: 'mtime' };
}

// ---------------------------------------------------------------------------
// Per-source manifest (inbox/<type>/.dates.json)
// ---------------------------------------------------------------------------

export function manifestPath(sourceDir) {
  return join(sourceDir, MANIFEST_FILE);
}

/** Load a source's date manifest ({ [relPath]: { date, date_source, md5 } }). */
export async function loadManifest(sourceDir) {
  const p = manifestPath(sourceDir);
  if (!existsSync(p)) return {};
  try {
    const data = JSON.parse(await readFile(p, 'utf8'));
    return data && typeof data === 'object' ? data : {};
  } catch {
    return {};
  }
}

/** Write a source's date manifest with sorted keys for stable git diffs. */
export async function saveManifest(sourceDir, manifest) {
  const ordered = {};
  for (const key of Object.keys(manifest).sort()) ordered[key] = manifest[key];
  const p = manifestPath(sourceDir);
  await mkdir(dirname(p), { recursive: true });
  await writeFile(p, `${JSON.stringify(ordered, null, 2)}\n`, 'utf8');
}

/**
 * Resolve a document date using the manifest as a cache: reuse the cached entry
 * when the md5 matches (no LLM re-run), otherwise resolve fresh and update the
 * manifest in place. Returns { date, source }.
 */
export async function resolveWithManifest(manifest, key, { content, mtime, md5, llm, useLLM }) {
  const cached = manifest[key];
  if (cached && cached.md5 === md5 && cached.date) {
    return { date: cached.date, source: cached.date_source ?? 'mtime' };
  }
  const { date, source } = await resolveDate({ content, mtime, llm, useLLM });
  manifest[key] = { date, date_source: source, md5 };
  return { date, source };
}
