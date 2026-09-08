// Atlassian source: fetch Confluence pages and Jira issues from the REST API
// and write them as Markdown into inbox/. The document builders are pure and
// exercised by tests (scripts/tests/atlassian.test.mjs); sync does the I/O.
//
// Settings come from .env: ATLASSIAN_SITE, ATLASSIAN_EMAIL, ATLASSIAN_API_TOKEN,
// CONFLUENCE_SPACES, JIRA_PROJECTS, JIRA_JQL, ATLASSIAN_INCLUDE_COMMENTS.

import { writeFileSync, readFileSync, existsSync, mkdirSync, readdirSync, unlinkSync } from 'node:fs';
import { join } from 'node:path';

import * as cheerio from 'cheerio';
import TurndownService from 'turndown';

import { log } from '../core/log.mjs';

// ---------------------------------------------------------------------------
// HTML -> Markdown
// ---------------------------------------------------------------------------

const turndown = new TurndownService({
  headingStyle: 'atx',
  bulletListMarker: '-',
  codeBlockStyle: 'fenced',
});

/** Convert Atlassian HTML to Markdown, stripping scripts and Connect widgets. */
export function htmlToMd(html) {
  if (!html) return '';
  const $ = cheerio.load(html, null, false);
  $('script, style, iframe').remove();
  $('.ap-container, .ap-iframe-container, .ap-content-container').remove();
  let result = turndown.turndown($.html()).trim();
  result = result.replace(/\/\/\s*<!\[CDATA\[[\s\S]*?\/\/\s*\]\]>/g, '');
  return result;
}

// ---------------------------------------------------------------------------
// Document builders (pure - exercised by the byte-identical test)
// ---------------------------------------------------------------------------

/** Build a Confluence markdown document (frontmatter + title + body + comments). */
export function buildConfluenceDoc(page, bodyHtml, commentsMd, { site }) {
  const title = page.title ?? 'Untitled';
  const pageId = page.id ?? '';
  const spaceKey = page.spaceId ?? '';
  const updated = (page.version?.createdAt ?? '').slice(0, 10);
  const url = `https://${site}/wiki/spaces/${spaceKey}/pages/${pageId}`;

  const lines = [
    '---',
    `title: ${JSON.stringify(title)}`,
    'source_type: confluence',
    `page_id: ${pageId}`,
    `space: ${spaceKey}`,
    `updated: ${updated}`,
    `url: ${url}`,
    '---',
    '',
    `# ${title}`,
    '',
    htmlToMd(bodyHtml),
  ];
  if (commentsMd) lines.push('', '## Comments', '', commentsMd);
  return `${lines.join('\n')}\n`;
}

/** Build a Jira markdown document (frontmatter + header + description + comments). */
export function buildJiraDoc(issue, { site }) {
  const key = issue.key;
  const fields = issue.fields ?? {};
  const rendered = issue.renderedFields ?? {};

  const summary = fields.summary ?? '';
  const status = fields.status?.name ?? '';
  const issueType = fields.issuetype?.name ?? '';
  const priority = fields.priority?.name ?? '';
  const assignee = fields.assignee?.displayName ?? 'Unassigned';
  const reporter = fields.reporter?.displayName ?? '';
  const created = (fields.created ?? '').slice(0, 10);
  const updated = (fields.updated ?? '').slice(0, 10);
  const labels = (fields.labels ?? []).join(', ');
  const components = (fields.components ?? []).map((c) => c.name ?? '').join(', ');
  const fixVersions = (fields.fixVersions ?? []).map((v) => v.name ?? '').join(', ');

  let descMd;
  if (rendered.description) descMd = htmlToMd(rendered.description);
  else descMd = typeof fields.description === 'string' ? fields.description : '';

  const url = `https://${site}/browse/${key}`;

  const lines = [
    '---',
    `title: ${JSON.stringify(`[${key}] ${summary}`)}`,
    'source_type: jira',
    `key: ${key}`,
    `status: ${status}`,
    `type: ${issueType}`,
    `priority: ${priority}`,
    `assignee: ${assignee}`,
    `reporter: ${reporter}`,
    `created: ${created}`,
    `updated: ${updated}`,
    ...(labels ? [`labels: ${labels}`] : []),
    ...(components ? [`components: ${components}`] : []),
    ...(fixVersions ? [`fix_versions: ${fixVersions}`] : []),
    `url: ${url}`,
    '---',
    '',
    `# [${key}] ${summary}`,
    '',
    `**Status:** ${status}  |  **Type:** ${issueType}  |  **Priority:** ${priority}`,
    `**Assignee:** ${assignee}  |  **Reporter:** ${reporter}`,
    `**Created:** ${created}  |  **Updated:** ${updated}`,
    '',
  ];

  if (descMd) lines.push('## Description', '', descMd, '');

  const comments = fields.comment?.comments ?? [];
  const renderedComments = rendered.comment?.comments ?? [];
  if (comments.length) {
    lines.push('## Comments', '');
    comments.forEach((c, i) => {
      const author = c.author?.displayName ?? 'unknown';
      const createdC = (c.created ?? '').slice(0, 10);
      const renderedBody = renderedComments[i]?.renderedBody || renderedComments[i]?.body || '';
      const bodyMd = renderedBody ? htmlToMd(renderedBody) : String(c.body ?? '');
      lines.push(`**${author}** (${createdC}):`, bodyMd, '');
    });
  }

  return `${lines.join('\n')}\n`;
}

// ---------------------------------------------------------------------------
// Atlassian client
// ---------------------------------------------------------------------------

function csv(value) {
  return (value ?? '').split(',').map((s) => s.trim()).filter(Boolean);
}

/** Strip scheme/path from a base URL, leaving the bare host (e.g. foo.atlassian.net). */
function hostOf(value) {
  return (value ?? '')
    .trim()
    .replace(/^https?:\/\//i, '')
    .replace(/\/.*$/, '')
    .trim();
}

export function settings() {
  // Our variable names take precedence; the toolkit's names are accepted as a
  // fallback so a deploy project can reuse its existing Atlassian credentials.
  const site = process.env.ATLASSIAN_SITE?.trim() || hostOf(process.env.ATLASSIAN_BASE_URL);
  const email = process.env.ATLASSIAN_EMAIL ?? '';
  const token = process.env.ATLASSIAN_API_TOKEN ?? '';
  const spaces = csv(process.env.CONFLUENCE_SPACES ?? process.env.CONFLUENCE_SPACE_KEY);
  const projects = csv(process.env.JIRA_PROJECTS ?? process.env.JIRA_PROJECT_KEY);
  return {
    site,
    email,
    token,
    spaces,
    projects,
    jql: (process.env.JIRA_JQL ?? '').trim(),
    includeComments: (process.env.ATLASSIAN_INCLUDE_COMMENTS ?? 'true').trim().toLowerCase() !== 'false',
  };
}

function authHeader({ email, token }) {
  return `Basic ${Buffer.from(`${email}:${token}`).toString('base64')}`;
}

const RETRYABLE = new Set([429, 500, 502, 503, 504]);
const MAX_ATTEMPTS = 5;
const BACKOFF_CAP_MS = 30_000;

function wait(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

// Backoff before the next retry: honor Retry-After (seconds) on 429, else exponential.
function backoffMs(attempt, res) {
  const raw = res?.headers?.get?.('retry-after');
  const secs = raw != null ? Number(raw) : NaN;
  if (Number.isFinite(secs) && secs >= 0) return Math.min(secs * 1000, BACKOFF_CAP_MS);
  return Math.min(500 * 2 ** attempt, BACKOFF_CAP_MS);
}

// Fetch with bounded retries on 429/5xx and network errors; 4xx (except 429) throw
// at once. Injectable fetch/sleep keep it unit-testable. Returns the ok Response.
export async function requestWithRetry(url, init = {}, { fetchImpl = fetch, sleep = wait } = {}) {
  const method = init.method ?? 'GET';
  let lastErr;
  for (let attempt = 0; attempt < MAX_ATTEMPTS; attempt++) {
    let res;
    try {
      res = await fetchImpl(url, init);
    } catch (err) {
      lastErr = err;
      if (attempt === MAX_ATTEMPTS - 1) break;
      const ms = backoffMs(attempt, null);
      log(`[retry] ${method} ${url} network error (${err?.message ?? err}); retrying in ${ms}ms`);
      await sleep(ms);
      continue;
    }
    if (res.ok) return res;
    if (!RETRYABLE.has(res.status) || attempt === MAX_ATTEMPTS - 1) {
      const body = await res.text().catch(() => '');
      throw new Error(`${method} ${url} -> ${res.status}${body ? `: ${body.slice(0, 300)}` : ''}`);
    }
    const ms = backoffMs(attempt, res);
    log(`[retry] ${method} ${url} -> ${res.status}; retrying in ${ms}ms`);
    await sleep(ms);
  }
  throw new Error(`${method} ${url} failed after ${MAX_ATTEMPTS} attempts: ${lastErr?.message ?? lastErr}`);
}

async function apiGet(cfg, url) {
  const res = await requestWithRetry(url, {
    headers: { Authorization: authHeader(cfg), Accept: 'application/json' },
  });
  return res.json();
}

async function apiPost(cfg, url, body) {
  const res = await requestWithRetry(url, {
    method: 'POST',
    headers: { Authorization: authHeader(cfg), Accept: 'application/json', 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  });
  return res.json();
}

export function buildJiraJql({ projects, jql }) {
  if (!projects.length) return jql;
  const projectClause = `project IN (${projects.join(', ')})`;
  if (!jql) return projectClause;
  const orderMatch = jql.match(/\s+ORDER\s+BY\s+.+$/i);
  const filterPart = orderMatch ? jql.slice(0, orderMatch.index).trim() : jql.trim();
  const orderPart = orderMatch ? orderMatch[0].trim() : '';
  const combined = filterPart ? `${projectClause} AND (${filterPart})` : projectClause;
  return orderPart ? `${combined} ${orderPart}` : combined;
}

function extractCursor(nextLink) {
  const m = nextLink.match(/cursor=([^&]+)/);
  return m ? decodeURIComponent(m[1]) : null;
}

// Rewriting a byte-identical doc bumps mtime for nothing and the indexer would
// re-chunk and re-embed it. Touch the file only when the content really changed.
export function writeIfChanged(path, content) {
  if (!existsSync(path)) {
    writeFileSync(path, content, 'utf8');
    return 'added';
  }
  if (readFileSync(path, 'utf8') === content) return 'unchanged';
  writeFileSync(path, content, 'utf8');
  return 'updated';
}

export async function syncConfluence(cfg, home, state) {
  const inbox = join(home, 'inbox', 'confluence');
  mkdirSync(inbox, { recursive: true });
  if (!cfg.spaces.length) {
    log('[confluence] no spaces configured - skipping.');
    return { added: 0, updated: 0, deleted: 0 };
  }

  const base = `https://${cfg.site}/wiki/api/v2`;
  const startedAt = new Date().toISOString();
  const lastSync = state.confluence?.last_sync ?? '';
  const counts = { added: 0, updated: 0, unchanged: 0, deleted: 0 };
  const allIds = new Set();

  for (const spaceKey of cfg.spaces) {
    log(`[confluence] fetching space ${spaceKey}...`);
    const spaceResp = await apiGet(cfg, `${base}/spaces?keys=${encodeURIComponent(spaceKey)}&limit=1`);
    const spaceId = spaceResp.results?.[0]?.id;
    if (!spaceId) {
      log(`[confluence] space ${spaceKey} not found - skipping.`);
      continue;
    }

    let cursor = null;
    const pages = [];
    for (;;) {
      const params = new URLSearchParams({ limit: '50', status: 'current' });
      if (cursor) params.set('cursor', cursor);
      const data = await apiGet(cfg, `${base}/spaces/${spaceId}/pages?${params}`);
      for (const page of data.results ?? []) {
        allIds.add(String(page.id));
        const updated = page.version?.createdAt ?? '';
        if (lastSync && updated && updated <= lastSync) continue;
        pages.push(page);
      }
      const next = data._links?.next;
      if (!next) break;
      cursor = extractCursor(next);
      if (!cursor) break;
    }
    log(`[confluence]   ${pages.length} page(s) to process`);

    for (const page of pages) {
      const pageId = String(page.id);
      const full = await apiGet(cfg, `${base}/pages/${pageId}?body-format=view`);
      const bodyHtml = full.body?.view?.value ?? '';
      // spaceId on the v2 page is a numeric id; keep it for building the page url.
      page.spaceId = page.spaceId ?? full.spaceId ?? '';
      let commentsMd = '';
      if (cfg.includeComments) commentsMd = await fetchConfluenceComments(cfg, base, pageId);
      const content = buildConfluenceDoc(page, bodyHtml, commentsMd, { site: cfg.site });
      const path = join(inbox, `${pageId}.md`);
      counts[writeIfChanged(path, content)]++;
    }
  }

  for (const file of readdirSync(inbox).filter((f) => f.endsWith('.md'))) {
    // Keep hand-authored `*gitkeep*` sources: they have no Confluence page and
    // are the documented escape hatch (see inbox .gitignore), so never prune them.
    if (file.includes('gitkeep')) continue;
    if (!allIds.has(file.replace(/\.md$/, ''))) {
      unlinkSync(join(inbox, file));
      counts.deleted++;
      log(`[confluence]   pruned ${file}`);
    }
  }

  state.confluence = { last_sync: startedAt };
  return counts;
}

async function fetchConfluenceComments(cfg, base, pageId) {
  let data;
  try {
    data = await apiGet(cfg, `${base}/pages/${pageId}/footer-comments?limit=50`);
  } catch {
    return '';
  }
  const parts = [];
  for (const c of data.results ?? []) {
    const author = c.version?.authorId ?? 'unknown';
    const created = c.version?.createdAt ?? '';
    const bodyValue = c.body?.view?.value || c.body?.storage?.value || '';
    const bodyMd = bodyValue ? htmlToMd(bodyValue) : '';
    if (bodyMd) parts.push(`**${author}** (${created.slice(0, 10)}):\n${bodyMd}`);
  }
  return parts.join('\n\n---\n\n');
}

export async function syncJira(cfg, home, state) {
  const inbox = join(home, 'inbox', 'jira');
  mkdirSync(inbox, { recursive: true });

  let jql = buildJiraJql(cfg);
  if (!jql) {
    log('[jira] no projects/JQL configured - skipping.');
    return { added: 0, updated: 0, deleted: 0 };
  }

  const base = `https://${cfg.site}/rest/api/3`;
  const startedAt = new Date().toISOString();
  const lastSync = state.jira?.last_sync ?? '';
  if (lastSync) {
    const orderMatch = jql.match(/\s+ORDER\s+BY\s+.+$/i);
    const baseJql = orderMatch ? jql.slice(0, orderMatch.index).trim() : jql;
    const orderPart = orderMatch ? orderMatch[0].trim() : '';
    jql = `(${baseJql}) AND updated >= '${lastSync.slice(0, 10)}'`;
    if (orderPart) jql = `${jql} ${orderPart}`;
  }

  const counts = { added: 0, updated: 0, unchanged: 0, deleted: 0 };
  const seen = new Set();
  const fields = 'summary,status,issuetype,priority,assignee,reporter,created,updated,description,labels,components,fixVersions,comment';

  let token = null;
  const issues = [];
  for (;;) {
    const body = { jql, maxResults: 50, expand: 'renderedFields', fields: fields.split(',') };
    if (token) body.nextPageToken = token;
    const data = await apiPost(cfg, `${base}/search/jql`, body);
    const batch = data.issues ?? [];
    issues.push(...batch);
    token = data.nextPageToken;
    if (!token || !batch.length) break;
  }
  log(`[jira] ${issues.length} issue(s) to process`);

  for (const issue of issues) {
    seen.add(issue.key);
    let full = issue;
    if (cfg.includeComments) {
      full = await apiGet(cfg, `${base}/issue/${issue.key}?expand=renderedFields&fields=${fields}`);
    }
    const content = buildJiraDoc(full, { site: cfg.site });
    const path = join(inbox, `${issue.key}.md`);
    counts[writeIfChanged(path, content)]++;
  }

  if (!lastSync) {
    for (const file of readdirSync(inbox).filter((f) => f.endsWith('.md'))) {
      // Keep hand-authored `*gitkeep*` sources (documented escape hatch); never prune them.
      if (file.includes('gitkeep')) continue;
      if (!seen.has(file.replace(/\.md$/, ''))) {
        unlinkSync(join(inbox, file));
        counts.deleted++;
        log(`[jira]   pruned ${file}`);
      }
    }
  }

  state.jira = { last_sync: startedAt };
  return counts;
}
