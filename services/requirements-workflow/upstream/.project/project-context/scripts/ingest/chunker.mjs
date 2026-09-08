// Document chunker.
//
// Three strategies, fixed per source type:
//   llm    - LLM line-number chunking for meeting transcripts (semantic
//            boundaries). Falls back to the script transcript chunker when the
//            text does not look like a transcript (e.g. dewrap failed) or no LLM
//            is available.
//   script - heading-aware splitter for structured markdown (confluence, specs,
//            adrs) with breadcrumb prefixes; speaker-accumulation for transcripts.
//   none   - jira: body and comments split into their own chunks.
//
// Robustness is built in rather than bolted on. Whatever a strategy produces,
// `enforceCeiling` is the final backstop that guarantees no chunk exceeds
// max_chars + split_tolerance, splitting on sentence -> whitespace window ->
// hard boundary, so a pathological mega-line or mega table row can never leak an
// oversized chunk into the index. Word tokens are preserved (no mid-token cuts
// unless a single token itself exceeds the window).

import { strategyFor } from '../core/config.mjs';
import { warn } from '../core/log.mjs';

const LINES_PER_SECTION = 300;
const JIRA_SPLIT_THRESHOLD = 5000;
const JIRA_MAX_GROUP = 3000;
const TRANSCRIPT_SPEAKER_MIN = 5;

const SPEAKER_HEADER_RE = /^[A-Za-z][\w ,.\[\]()\-]+\s{2,}\d+:\d+/;
const HEADING_RE = /^(#{1,4})\s+(.*)/;
const TABLE_SEP_RE = /^\s*\|[\s\-|:]+\|\s*$/;
const JIRA_COMMENT_RE = /^\*\*[^*]+\*\*\s+\(\d{4}-\d{2}-\d{2}\):/gm;
const SENTENCE_SPLIT_RE = /(?<=[.!?])\s+/;

// ---------------------------------------------------------------------------
// Size primitives
// ---------------------------------------------------------------------------

function splitSentences(text) {
  return text.split(SENTENCE_SPLIT_RE).filter((s) => s.length > 0);
}

/** Break a long unit on whitespace near targetChars, never exceeding maxChars. */
function windowSplit(text, targetChars, maxChars) {
  const out = [];
  let i = 0;
  while (i < text.length) {
    let end = Math.min(i + targetChars, text.length);
    if (end < text.length) {
      const ws = text.lastIndexOf(' ', end);
      if (ws > i) end = ws;
    }
    if (end - i > maxChars) end = i + maxChars; // absolute last resort
    out.push(text.slice(i, end));
    i = end;
    while (text[i] === ' ') i++;
  }
  return out.filter((s) => s.length > 0);
}

/** Split text so every piece is <= maxChars, preferring sentence boundaries. */
function hardSplit(text, { targetChars, maxChars }) {
  if (text.length <= maxChars) return [text];
  const pieces = [];
  let buf = '';
  const flush = () => {
    if (buf.trim()) pieces.push(buf.trim());
    buf = '';
  };
  for (const sentence of splitSentences(text)) {
    const units = sentence.length <= maxChars ? [sentence] : windowSplit(sentence, targetChars, maxChars);
    for (const unit of units) {
      if (!buf) buf = unit;
      else if ((buf + ' ' + unit).length <= targetChars) buf = `${buf} ${unit}`;
      else {
        flush();
        buf = unit;
      }
    }
  }
  flush();
  return pieces.length ? pieces : [text];
}

/** Final backstop: split anything above the ceiling (max_chars + tolerance). */
function enforceCeiling(chunks, { targetChars, maxChars, splitTolerance }) {
  const ceiling = maxChars + splitTolerance;
  const out = [];
  for (const chunk of chunks) {
    if (chunk.length <= ceiling) out.push(chunk);
    else out.push(...hardSplit(chunk, { targetChars, maxChars }));
  }
  return out;
}

/** Merge chunks below min_chars into their neighbour. */
function mergeSmall(chunks, minChars) {
  const result = [];
  let pending = '';
  for (const raw of chunks) {
    const chunk = raw.trim();
    if (!chunk) continue;
    const candidate = pending ? `${pending}\n\n${chunk}` : chunk;
    if (candidate.length < minChars) {
      pending = candidate;
    } else {
      result.push(candidate);
      pending = '';
    }
  }
  if (pending) {
    if (result.length) result[result.length - 1] += `\n\n${pending}`;
    else result.push(pending);
  }
  return result;
}

// ---------------------------------------------------------------------------
// Jira (none strategy): body + comments
// ---------------------------------------------------------------------------

function splitJira(text, { minChars, title = '' }) {
  const match = text.match(/^## Comments\s*$/m);
  if (!match) return [text];

  const idx = match.index;
  const body = text.slice(0, idx).trim();
  const commentsRaw = text.slice(idx).trim();
  if (body.length < minChars || !commentsRaw) return [text];

  const ticketHeader = title ? `${title}\n\n` : '';
  if (commentsRaw.length <= JIRA_SPLIT_THRESHOLD) {
    return [body, ticketHeader + commentsRaw];
  }

  const boundaries = [...commentsRaw.matchAll(JIRA_COMMENT_RE)].map((m) => m.index);
  if (boundaries.length === 0) return [body, ticketHeader + commentsRaw];

  const individual = boundaries.map((start, i) =>
    commentsRaw.slice(start, boundaries[i + 1] ?? commentsRaw.length).trim(),
  );

  const sectionHeader = `${ticketHeader}## Comments\n\n`;
  const groups = [];
  let acc = [];
  let accLen = 0;
  for (const comment of individual) {
    if (acc.length && accLen + comment.length + 2 > JIRA_MAX_GROUP) {
      groups.push(sectionHeader + acc.join('\n\n'));
      acc = [comment];
      accLen = comment.length;
    } else {
      acc.push(comment);
      accLen += comment.length + 2;
    }
  }
  if (acc.length) groups.push(sectionHeader + acc.join('\n\n'));

  return [body, ...groups];
}

// ---------------------------------------------------------------------------
// Script markdown (confluence, specs, adrs)
// ---------------------------------------------------------------------------

function parseBlocks(bodyText) {
  const blocks = [];
  for (const para of bodyText.split(/\n{2,}/)) {
    const trimmed = para.trim();
    if (!trimmed) continue;
    const isTable = trimmed.split('\n').some((ln) => TABLE_SEP_RE.test(ln.trim()));
    blocks.push({ kind: isTable ? 'table' : 'para', text: trimmed });
  }
  return blocks;
}

function splitTableRows(tableText, prefix, maxChars) {
  const lines = tableText.split('\n').filter((ln) => ln.trim());
  const sepIdx = lines.findIndex((ln) => TABLE_SEP_RE.test(ln.trim()));
  if (sepIdx === -1 || sepIdx + 1 >= lines.length) return [(prefix + tableText).trim()];

  const headerLines = lines.slice(0, sepIdx + 1);
  let dataRows;
  if (lines[sepIdx + 1].includes('**')) {
    headerLines.push(lines[sepIdx + 1]);
    dataRows = lines.slice(sepIdx + 2);
  } else {
    dataRows = lines.slice(sepIdx + 1);
  }
  if (dataRows.length === 0) return [(prefix + tableText).trim()];

  const tableHeader = headerLines.join('\n');
  const result = [];
  let acc = [];
  for (const row of dataRows) {
    const candidate = `${tableHeader}\n${[...acc, row].join('\n')}`;
    if ((prefix + candidate).length <= maxChars || acc.length === 0) {
      acc.push(row);
    } else {
      result.push((prefix + tableHeader + '\n' + acc.join('\n')).trim());
      acc = [row];
    }
  }
  if (acc.length) result.push((prefix + tableHeader + '\n' + acc.join('\n')).trim());
  return result.length ? result : [(prefix + tableText).trim()];
}

function splitBySentences(text, target) {
  const chunks = [];
  let current = '';
  for (const sent of splitSentences(text)) {
    const candidate = current ? `${current} ${sent}` : sent;
    if (candidate.length <= target) current = candidate;
    else {
      if (current) chunks.push(current);
      current = sent;
    }
  }
  if (current) chunks.push(current);
  return chunks.length ? chunks : [text];
}

function splitMarkdown(text, { targetChars, maxChars }) {
  const sections = [];
  let headingStack = [];
  let body = [];
  const flush = () => sections.push({ crumbs: headingStack.map((h) => h.text), body: [...body] });

  for (const line of text.split('\n')) {
    const m = line.match(HEADING_RE);
    if (m) {
      flush();
      body = [];
      const level = m[1].length;
      headingStack = headingStack.filter((h) => h.level < level);
      headingStack.push({ level, text: m[2].trim() });
      body.push(line);
    } else {
      body.push(line);
    }
  }
  flush();

  const rawChunks = [];
  for (const { crumbs, body: bodyLines } of sections) {
    const bodyText = bodyLines.join('\n').trim();
    if (!bodyText) continue;

    const breadcrumb = crumbs.length > 1 ? crumbs.join(' > ') : '';
    const prefix = breadcrumb ? `${breadcrumb}\n\n` : '';
    const full = (prefix + bodyText).trim();

    if (full.length <= maxChars) {
      rawChunks.push(full);
      continue;
    }

    let acc = '';
    for (const { kind, text: block } of parseBlocks(bodyText)) {
      if (kind === 'table') {
        if (acc) {
          rawChunks.push((prefix + acc).trim());
          acc = '';
        }
        const blockFull = (prefix + block).trim();
        if (blockFull.length <= maxChars) rawChunks.push(blockFull);
        else rawChunks.push(...splitTableRows(block, prefix, maxChars));
      } else {
        const candidate = acc ? `${acc}\n\n${block}` : block;
        if ((prefix + candidate).length <= targetChars) {
          acc = candidate;
        } else {
          if (acc) rawChunks.push((prefix + acc).trim());
          if ((prefix + block).length > maxChars) {
            for (const sentChunk of splitBySentences(block, targetChars)) {
              rawChunks.push((prefix + sentChunk).trim());
            }
            acc = '';
          } else {
            acc = block;
          }
        }
      }
    }
    if (acc) rawChunks.push((prefix + acc).trim());
  }
  return rawChunks;
}

// ---------------------------------------------------------------------------
// Transcripts (meeting_notes)
// ---------------------------------------------------------------------------

function countSpeakerLines(text) {
  let n = 0;
  for (const ln of text.split('\n')) if (SPEAKER_HEADER_RE.test(ln.trim())) n++;
  return n;
}

/** A transcript has recognizable speaker/timestamp headers. A dewrap failure
 * (a few mega-lines) does not, and is routed to the document splitter instead. */
function looksLikeTranscript(text) {
  return countSpeakerLines(text) >= TRANSCRIPT_SPEAKER_MIN;
}

function safeSplitPoints(lines) {
  const safe = new Set([0]);
  for (let i = 1; i < lines.length; i++) {
    const ln = lines[i].trim();
    if (SPEAKER_HEADER_RE.test(ln)) safe.add(i);
    else if (!ln && i + 1 < lines.length && lines[i + 1].trim()) safe.add(i + 1);
  }
  return [...safe].sort((a, b) => a - b);
}

/** Speaker-boundary accumulation chunker (script strategy / LLM fallback). */
function splitTranscriptScript(text, { targetChars }) {
  const lines = text.split('\n');
  const points = safeSplitPoints(lines);

  const rawChunks = [];
  let chunkStart = 0;
  let accLen = 0;
  for (let idx = 0; idx < points.length; idx++) {
    const sp = points[idx];
    if (sp === 0) continue;
    const prevSp = points[idx - 1];
    const turnLen = lines.slice(prevSp, sp).join('\n').length;
    if (accLen > 0 && accLen + turnLen > targetChars) {
      rawChunks.push(lines.slice(chunkStart, prevSp).join('\n').trim());
      chunkStart = prevSp;
      accLen = turnLen;
    } else {
      accLen += turnLen;
    }
  }
  rawChunks.push(lines.slice(chunkStart).join('\n').trim());
  return rawChunks.filter((c) => c);
}

function meetingHeader(title, text) {
  const dateMatch = text.slice(0, 500).match(/\b(\d{1,2}\s+\w+\s+\d{4})\b/);
  const parts = [];
  if (title) parts.push(`Meeting: ${title}`);
  if (dateMatch) parts.push(`Date: ${dateMatch[1]}`);
  return parts.length ? `${parts.join(' | ')}\n\n` : '';
}

// ---------------------------------------------------------------------------
// LLM line-number chunking
// ---------------------------------------------------------------------------

const PROMPT_MEETING_NOTES = `You are segmenting a meeting transcript for a retrieval index.

The input has stable line numbers. Return the line numbers where each new chunk should START. The caller splits the original text at those positions - never rewrite or reproduce the text.

Output contract:
- Return only valid JSON: {"starts": [1, 12, 34, 67]}
- "starts" must be sorted, unique integers that exist in the input.
- Always include the first input line. Do not include an end marker or a line after the final input line.

A good chunk should answer one future retrieval question: a specific requirement, question and answer, decision with rationale, process, metric/report, issue/risk, or action-item cluster. Keep a question with its answer, a decision with its rationale, and an action item with owner/context. Prefer chunks around 800-1800 characters; split broad topics before they mix unrelated questions, but do not split a coherent exchange just to hit the target size.

Good boundaries are speaker/timestamp lines that open a substantive new topic, or clear transition phrases. Never choose a blank line, a PDF line-wrap continuation fragment, or a short acknowledgement ("Yeah.", "Okay.").`;

const PROMPT_STRUCTURED = `You are segmenting a structured project document for a retrieval index.

The input is a Confluence page, spec, ADR, or archived design/proposal with numbered lines. Return the line numbers where each new chunk should START. The caller splits the original text at those positions - never rewrite or reproduce the text.

Output contract:
- Return only valid JSON: {"starts": [1, 18, 45]}
- "starts" must be sorted, unique integers that exist in the input.
- Always include the first input line. Do not include an end marker or a line after the final input line.

Prefer H2/H3 headings when they begin a substantive section. Keep each heading attached to the content that follows it. Keep tiny heading-only sections with adjacent content. Never split inside a list, table, or fenced code block. If a long section contains multiple unrelated requirements, decisions, procedures, or acceptance criteria, split at a safe paragraph or lower-level heading boundary. Avoid one-line fragments.`;

const PROMPT_DEFAULT = `You are segmenting text for a retrieval index.

The input has numbered lines. Return the line numbers where each new chunk should START. The caller splits the original text at those positions - never rewrite or reproduce the text.

Output contract:
- Return only valid JSON: {"starts": [1, 30, 75]}
- "starts" must be sorted, unique integers that exist in the input.
- Always include the first input line. Do not include an end marker or a line after the final input line.

Split at natural topic or section boundaries, never mid-sentence. Prefer chunks that answer one future retrieval question and keep directly related context together. Avoid one-line fragments.`;

function systemPromptFor(sourceType) {
  if (sourceType === 'meeting_notes') return PROMPT_MEETING_NOTES;
  if (
    sourceType === 'confluence' ||
    sourceType === 'specs' ||
    sourceType === 'adrs' ||
    sourceType === 'openspec_archive'
  ) {
    return PROMPT_STRUCTURED;
  }
  return PROMPT_DEFAULT;
}

function parseJsonLineNumbers(raw) {
  const candidates = [raw.trim()];
  const m = raw.match(/(\{[\s\S]*\}|\[[\s\S]*\])/);
  if (m) candidates.push(m[1]);
  for (const candidate of candidates) {
    let data;
    try {
      data = JSON.parse(candidate);
    } catch {
      continue;
    }
    const values = Array.isArray(data) ? data : data.starts ?? data.line_numbers ?? data.boundaries;
    if (!Array.isArray(values)) continue;
    const numbers = [];
    for (const v of values) {
      if (Number.isInteger(v)) numbers.push(v);
      else if (typeof v === 'string' && /^\d+$/.test(v.trim())) numbers.push(Number(v.trim()));
    }
    if (numbers.length) return numbers;
  }
  return [];
}

function parseLineNumbers(raw, minLine, maxLine) {
  let found = parseJsonLineNumbers(raw);
  if (!found.length) found = [...raw.matchAll(/\b(\d+)\b/g)].map((m) => Number(m[1]));
  const valid = [...new Set(found.filter((n) => n >= minLine && n <= maxLine))].sort((a, b) => a - b);
  if (!valid.length || valid[0] !== minLine) valid.unshift(minLine);
  return valid;
}

async function askLineNumbers(sectionLines, offset, sourceType, llm) {
  const numbered = sectionLines.map((ln, i) => `${offset + i + 1}: ${ln}`).join('\n');
  try {
    const raw = await llm(systemPromptFor(sourceType), numbered);
    return parseLineNumbers(raw, offset + 1, offset + sectionLines.length).map((n) => n - 1);
  } catch (err) {
    warn(`LLM line-number request failed (${err?.message ?? err}); falling back to script chunker`);
    return [];
  }
}

function splitAtLines(lines, starts) {
  const sorted = [...new Set(starts)].sort((a, b) => a - b);
  if (sorted[0] !== 0) sorted.unshift(0);
  const boundaries = [...sorted, lines.length];
  return sorted.map((_, i) => lines.slice(boundaries[i], boundaries[i + 1]).join('\n'));
}

async function splitTranscriptLlm(text, { chunking, llm, sourceType }) {
  const lines = text.split('\n');
  const total = lines.length;

  let starts;
  if (total <= LINES_PER_SECTION) {
    starts = await askLineNumbers(lines, 0, sourceType, llm);
  } else {
    const all = [];
    let secStart = 0;
    while (secStart < total) {
      const secLines = lines.slice(secStart, secStart + LINES_PER_SECTION);
      const secEnd = secStart + secLines.length;
      const s = await askLineNumbers(secLines, secStart, sourceType, llm);
      all.push(...s);
      if (secEnd >= total) break;
      const last = s.length ? Math.max(...s) : secStart;
      secStart = last > secStart ? last : secStart + LINES_PER_SECTION;
    }
    starts = all;
  }

  if (!starts || starts.length === 0) return splitTranscriptScript(text, chunking);
  return splitAtLines(lines, starts);
}

// ---------------------------------------------------------------------------
// Strategy dispatch
// ---------------------------------------------------------------------------

const TRANSCRIPT_TYPES = new Set(['meeting_notes']);

/**
 * Split a document into an array of chunk texts.
 * @param {string} content
 * @param {{ sourceType: string, config: object, title?: string,
 *           llm?: (system: string, user: string) => Promise<string> }} opts
 * @returns {Promise<string[]>}
 */
export async function splitDocument(content, { sourceType, config, title = '', llm = null } = {}) {
  const { chunking } = config;
  const strategy = strategyFor(config, sourceType);
  const isTranscriptType = TRANSCRIPT_TYPES.has(sourceType);

  let raw;
  if (strategy === 'none') {
    raw = sourceType === 'jira' ? splitJira(content, { minChars: chunking.minChars, title }) : [content];
  } else if (strategy === 'llm') {
    if (isTranscriptType && !looksLikeTranscript(content)) {
      raw = splitMarkdown(content, chunking); // dewrap failed - treat as a document
    } else if (llm) {
      raw = await splitTranscriptLlm(content, { chunking, llm, sourceType });
      raw = raw.map((c) => meetingHeader(title, content) + c);
    } else {
      raw = splitTranscriptScript(content, chunking);
      raw = raw.map((c) => meetingHeader(title, content) + c);
    }
  } else {
    raw = isTranscriptType ? splitTranscriptScript(content, chunking) : splitMarkdown(content, chunking);
  }

  const merged = mergeSmall(raw, chunking.minChars);
  const ceiled = enforceCeiling(merged, chunking);
  return ceiled.map((c) => c.trim()).filter((c) => c.length > 0);
}

// ---------------------------------------------------------------------------
// Contextual enrichment (index-time; structured docs only)
// ---------------------------------------------------------------------------

const ENRICH_SYSTEM =
  'You write factual retrieval context that will be prepended to an indexed chunk. ' +
  'Given document context (title, outline, opening text) and the exact chunk, write ' +
  '1-2 short sentences that make the chunk self-contained and easier to find. Use ' +
  'only facts visible in the document context or chunk. Do not infer, generalize, ' +
  'recommend, or add new requirements, dates, statuses, systems, or decisions. Mention ' +
  'the document title/section and useful project terms, feature names, Jira keys, ' +
  'Salesforce objects, or acronyms when they are present. Do not repeat full sentences ' +
  'from the chunk verbatim. Output plain text only, no labels or markdown. If no useful ' +
  'context can be added, return an empty string.';

const ENRICH_MAX_DOC_CHARS = 15_000;

function buildDocContext(title, content) {
  const toc = content
    .split('\n')
    .filter((ln) => /^#{1,4} /.test(ln))
    .slice(0, 40)
    .map((ln) => ln.trimEnd())
    .join('\n');
  const parts = [`# ${title}`];
  if (toc) parts.push(`## Outline\n${toc}`);
  parts.push(`## Document (opening)\n${content.slice(0, ENRICH_MAX_DOC_CHARS)}`);
  return parts.join('\n\n');
}

async function enrichChunk(chunkText, docContext, llm) {
  const user =
    `<document>\n${docContext}\n</document>\n\n` +
    `<chunk>\n${chunkText}\n</chunk>\n\n` +
    'Write 1-2 factual retrieval-context sentences for this chunk:';
  try {
    const context = (await llm(ENRICH_SYSTEM, user)).trim();
    if (context) return `${context}\n\n${chunkText}`;
  } catch (err) {
    warn(`enrichment failed: ${err?.message ?? err}`);
  }
  return chunkText;
}

// ---------------------------------------------------------------------------
// Metadata
// ---------------------------------------------------------------------------

function titleCase(s) {
  return s.replace(/[-_]/g, ' ').replace(/\s+/g, ' ').trim().replace(/\b\w/g, (c) => c.toUpperCase());
}

export function inferTitle(content, filePath) {
  const parts = String(filePath).split('/').filter(Boolean);
  const name = parts[parts.length - 1] ?? '';
  // openspec capability specs are all named spec.md; the capability folder is
  // the meaningful identity, so prefer it over the generic first heading.
  if (/^spec\.md$/i.test(name) && parts.length >= 2) {
    return titleCase(parts[parts.length - 2]);
  }
  for (const line of content.split('\n')) {
    const m = line.trim().match(/^#{1,6} (.+)/);
    if (m) return m[1].trim();
  }
  const stem = name.replace(/_normalized\.txt$/i, '').replace(/\.(pdf|docx|txt|md)$/i, '');
  return titleCase(stem);
}

function findLineRange(original, chunk, searchFrom = 0) {
  const probe = chunk.slice(0, 80).trim();
  const pos = probe ? original.indexOf(probe, searchFrom) : -1;
  if (pos === -1) {
    const startLine = original.slice(0, searchFrom).split('\n').length;
    const endLine = startLine + chunk.split('\n').length;
    return { startLine, endLine, cursor: searchFrom + chunk.length };
  }
  const endPos = pos + chunk.length;
  const startLine = original.slice(0, pos).split('\n').length;
  const endLine = original.slice(0, endPos).split('\n').length;
  return { startLine, endLine, cursor: endPos };
}

// ---------------------------------------------------------------------------
// Public entry point (used by the indexer)
// ---------------------------------------------------------------------------

/**
 * Chunk a document into records with full metadata.
 * @returns {Promise<object[]>} chunk records (camelCase fields)
 */
export async function chunkDocument({ filePath, sourceType, docId, content, mtime, contentHash, docDate = '', dateSource = '', archived = false, config, llm = null }) {
  const title = inferTitle(content, filePath);
  const indexedAt = new Date().toISOString();

  const sourceTexts = await splitDocument(content, { sourceType, config, title, llm });
  let indexedTexts = sourceTexts;

  // Enrich every structured-markdown source (script strategy): confluence,
  // specs, adrs, openspec_archive, and any future script-based type.
  const shouldEnrich =
    config.chunking.enrich && llm && sourceTexts.length > 1 && strategyFor(config, sourceType) === 'script';
  if (shouldEnrich) {
    const docContext = buildDocContext(title, content);
    indexedTexts = await Promise.all(
      sourceTexts.map((t) =>
        t.length >= config.chunking.minChars ? enrichChunk(t, docContext, llm) : Promise.resolve(t),
      ),
    );
  }

  const chunks = [];
  let cursor = 0;
  indexedTexts.forEach((text, i) => {
    const trimmed = text.trim();
    if (!trimmed) return;
    const sourceText = (sourceTexts[i] ?? text).trim();
    const { startLine, endLine, cursor: next } = findLineRange(content, sourceText, cursor);
    cursor = next;
    chunks.push({
      chunkId: `${docId}::${i}`,
      docId,
      text: trimmed,
      file: String(filePath),
      sourceType,
      title,
      lines: `${startLine}-${endLine}`,
      chunkIndex: i,
      chunkTotal: 0,
      mtime,
      contentHash,
      docDate,
      dateSource,
      archived,
      indexedAt,
    });
  });
  for (const c of chunks) c.chunkTotal = chunks.length;
  return chunks;
}
