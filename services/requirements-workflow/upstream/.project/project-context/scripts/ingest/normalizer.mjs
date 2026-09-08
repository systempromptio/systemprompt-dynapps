// Materialize every configured source into inbox/<type>/ as indexable text.
//
// Each source has an origin `path` (default inbox/<type>). Files are pulled
// from the origin into inbox/<type> so the indexer only ever reads inbox and
// external origins (e.g. an openspec/ tree) stay untouched:
//
//   .md / .txt (native)  -> copied verbatim into inbox/<type>/<relpath>
//   .pdf / .docx (binary) -> extracted to inbox/<type>/<relpath>_normalized.txt
//
//   original.pdf  -> original.pdf_normalized.txt
//   original.docx -> original.docx_normalized.txt
//
// When the origin already IS inbox/<type> (the default), files are left in
// place and only binaries get a normalized sidecar next to the original.
// include/exclude globs decide which files are taken from the origin.

import { stat, writeFile, copyFile, mkdir, readdir, rm } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import { extname, relative, resolve, join, dirname } from 'node:path';

import { parse, isBinarySuffix } from './parsers.mjs';
import { log, warn } from '../core/log.mjs';
import { walk } from '../core/walk.mjs';

export const NORMALIZED_SUFFIX = '_normalized.txt';

const NATIVE_TEXT = new Set(['.txt', '.md']);

function isNativeText(path) {
  return NATIVE_TEXT.has(extname(path).toLowerCase());
}

function toPosix(p) {
  return p.split('\\').join('/');
}

// ---------------------------------------------------------------------------
// Glob filtering (include / exclude)
// ---------------------------------------------------------------------------

function globToRegExp(glob) {
  const specials = /[.+^${}()|[\]\\]/g;
  let re = '^';
  let i = 0;
  while (i < glob.length) {
    const c = glob[i];
    if (c === '*') {
      if (glob[i + 1] === '*') {
        if (glob[i + 2] === '/') {
          re += '(?:.*/)?';
          i += 3;
        } else {
          re += '.*';
          i += 2;
        }
      } else {
        re += '[^/]*';
        i += 1;
      }
    } else if (c === '?') {
      re += '[^/]';
      i += 1;
    } else {
      re += c.replace(specials, '\\$&');
      i += 1;
    }
  }
  return new RegExp(`${re}$`);
}

const globCache = new Map();
function matchesAny(rel, patterns) {
  return patterns.some((p) => {
    let re = globCache.get(p);
    if (!re) {
      re = globToRegExp(p);
      globCache.set(p, re);
    }
    return re.test(rel);
  });
}

/** Whether a file (relative posix path) passes a source's include/exclude filter. */
export function matchFilters(rel, { include = null, exclude = null } = {}) {
  if (exclude && matchesAny(rel, exclude)) return false;
  if (include && !matchesAny(rel, include)) return false;
  return true;
}

// A transcript speaker line ends with a timestamp, e.g. "Emrah Turkmen   12:34".
const SPEAKER_RE = /^.{1,60}\s\d{1,2}:\d{2}\s*$/;
// A buffered speech fragment ends a sentence when it closes with . ! ? (+ quote/paren).
const SENTENCE_END_RE = /[.!?]["')\]]?$/;

function isSpeakerLine(line) {
  return !/^\d/.test(line) && SPEAKER_RE.test(line);
}

/**
 * Repair PDF line-wrap artifacts in meeting transcripts: put each speaker header
 * on its own line and reflow speech to one sentence per line. Wording is
 * preserved verbatim; only whitespace/newlines change.
 */
export function dewrapTranscript(text) {
  const out = [];
  let buffer = '';
  const flush = () => {
    if (buffer.trim()) out.push(buffer.trim());
    buffer = '';
  };

  for (const raw of text.split('\n')) {
    const stripped = raw.trim();
    if (!stripped) continue;
    if (isSpeakerLine(stripped)) {
      flush();
      if (out.length && out[out.length - 1] !== '') out.push('');
      out.push(stripped);
      continue;
    }
    buffer = buffer ? `${buffer} ${stripped}` : stripped;
    if (SENTENCE_END_RE.test(buffer)) flush();
  }
  flush();
  return out.join('\n');
}

export function normalizedPath(source) {
  return source + NORMALIZED_SUFFIX;
}

export function isNormalizedFile(path) {
  return path.endsWith(NORMALIZED_SUFFIX);
}

export function isBinarySource(path) {
  return isBinarySuffix(extname(path)) && !isNormalizedFile(path);
}

/** Whether the normalized/target file is missing or older than its source. */
async function isStale(target, source) {
  if (!existsSync(target)) return true;
  const [ts, ss] = await Promise.all([stat(target), stat(source)]);
  return ts.mtimeMs < ss.mtimeMs;
}

/** Extract + (for transcripts) dewrap `source` and write the text to `dest`. */
async function extractTo(source, dest, sourceType = '') {
  try {
    let text = await parse(source);
    if (!text.trim()) {
      warn(`empty extraction, skipping ${source}`);
      return false;
    }
    if (sourceType === 'meeting_notes') text = dewrapTranscript(text);
    await mkdir(dirname(dest), { recursive: true });
    await writeFile(dest, text, 'utf8');
    return true;
  } catch (err) {
    warn(`normalization failed for ${source}: ${err?.message ?? err}`);
    return false;
  }
}

/** Recursively collect every file path under `dir` (absolute). */
async function listFiles(dir) {
  const out = [];
  for await (const p of walk(dir)) out.push(p);
  return out;
}

/**
 * Remove files under `target` that are not in the `expected` set, then prune
 * now-empty directories. Only used for external origins (inbox stays owned).
 */
async function pruneTarget(target, expected) {
  for (const path of await listFiles(target)) {
    if (!expected.has(path)) {
      log(`  [prune] ${relative(target, path)}`);
      await rm(path, { force: true });
    }
  }
}

/**
 * Materialize every configured source into inbox/<type>/.
 * @param {object} config loaded config (needs inboxPath + sources)
 * @returns {Promise<{created:number, skipped:number, failed:number}>}
 */
export async function materialize(config) {
  let created = 0;
  let skipped = 0;
  let failed = 0;

  for (const [name, source] of Object.entries(config.sources)) {
    const origin = source.path;
    const target = join(config.inboxPath, name);
    const inPlace = resolve(origin) === resolve(target);
    await mkdir(target, { recursive: true });
    const expected = new Set();

    for await (const path of walk(origin)) {
      if (isNormalizedFile(path)) continue; // never re-materialize our own sidecars
      const rel = toPosix(relative(origin, path));
      if (!matchFilters(rel, source)) continue;

      if (isBinarySource(path)) {
        const dest = inPlace ? path : join(target, relative(origin, path));
        const norm = normalizedPath(dest);
        expected.add(norm);
        if (await isStale(norm, path)) {
          log(`  [normalize] ${name}/${rel}`);
          if (await extractTo(path, norm, name)) created++;
          else failed++;
        } else {
          skipped++;
        }
      } else if (isNativeText(path)) {
        if (inPlace) {
          skipped++; // already an indexable file living in inbox
          continue;
        }
        const dest = join(target, relative(origin, path));
        expected.add(dest);
        if (await isStale(dest, path)) {
          log(`  [copy] ${name}/${rel}`);
          await mkdir(dirname(dest), { recursive: true });
          await copyFile(path, dest);
          created++;
        } else {
          skipped++;
        }
      }
    }

    if (!inPlace) await pruneTarget(target, expected);
  }

  return { created, skipped, failed };
}
