// Reads a comment body as a small, safe Markdown subset and returns a tree of
// plain objects, never markup: paragraphs, bullet and numbered lists, fenced
// code, and inline bold, italic, code, https links and mentions. Anything
// else, raw HTML included, stays literal text. The tree is drawn by
// markdown-dom.js with text nodes only, so a body can never inject HTML.
import { TOKEN } from './mention-token.js';

// Nesting stops here so a hostile body cannot make the reader recurse deeply.
const MAX_DEPTH = 4;

const TOKEN_AT = new RegExp(TOKEN.source, 'uy');
const EMAIL = /[\w.+-]+@[A-Za-z0-9-]+(?:\.[A-Za-z0-9-]+)+/y;
const LINK = /\[([^[\]\n]{1,300})\]\(([^()\s]{1,2000})\)/y;
const BARE_URL = /https:\/\/[^\s<>]+/y;
const FENCE = /^ {0,3}(`{3,}|~{3,})/;
const ITEM = /^ {0,3}([-*+]|\d{1,9}[.)])[ \t]+(.*)$/;

const isAlnum = (ch) => ch !== undefined && /[\p{L}\p{N}]/u.test(ch);
const isSpace = (ch) => ch === undefined || /\s/.test(ch);
const isClosingFence = (line, fence) => {
  const bare = line.trim();
  return bare.length >= fence.length && bare === fence[0].repeat(bare.length);
};

// The body line the DynWork desktop posts as a reply to say it took an @Agent
// mention: the web shows it as a small system line, not as a comment.
export const isSystemLine = (body) => body.trim() === 'Dynwork picked this up.';

export const parseBody = (body) => {
  const lines = body.replace(/\r\n?/g, '\n').split('\n');
  const blocks = [];
  let paragraph = [];
  const flush = () => {
    if (paragraph.length) blocks.push({ t: 'p', c: parseInline(paragraph.join('\n'), 0, false) });
    paragraph = [];
  };
  let i = 0;
  while (i < lines.length) {
    const line = lines[i];
    const fence = FENCE.exec(line);
    const item = ITEM.exec(line);
    if (fence) {
      flush();
      const close = lines.findIndex((l, at) => at > i && isClosingFence(l, fence[1]));
      const end = close === -1 ? lines.length : close;
      blocks.push({ t: 'pre', v: lines.slice(i + 1, end).join('\n') });
      i = end + 1;
    } else if (item) {
      flush();
      i = readList(lines, i, blocks);
    } else {
      if (line.trim()) paragraph.push(line);
      else flush();
      i += 1;
    }
  }
  flush();
  return blocks;
};

// Consecutive items of one kind make one list; an indented line after an item
// continues it. Lists do not nest: a deeper marker is part of its item's text.
const readList = (lines, from, blocks) => {
  const ordered = /\d/.test(ITEM.exec(lines[from])[1]);
  const items = [];
  let i = from;
  while (i < lines.length) {
    const match = ITEM.exec(lines[i]);
    if (match && /\d/.test(match[1]) === ordered) {
      items.push(match[2]);
    } else if (items.length && /^ {2,}\S/.test(lines[i])) {
      items[items.length - 1] += `\n${lines[i].trim()}`;
    } else {
      break;
    }
    i += 1;
  }
  blocks.push({ t: ordered ? 'ol' : 'ul', c: items.map((text) => ({ t: 'li', c: parseInline(text, 0, false) })) });
  return i;
};

export const parseInline = (text, depth, inLink) => {
  const out = [];
  const push = (node) => {
    const last = out.at(-1);
    if (node.t === 'text' && last?.t === 'text') last.v += node.v;
    else out.push(node);
  };
  let i = 0;
  while (i < text.length) {
    const found = scan(text, i, depth, inLink);
    if (found) {
      push(found.node);
      i = found.end;
    } else {
      push({ t: 'text', v: text[i] });
      i += 1;
    }
  }
  return out;
};

// One construct at `i`, or null when the character is plain text. A mention
// token and an email are matched before any emphasis, so the dots, underscores
// and the @ inside them never split the name.
const scan = (text, i, depth, inLink) => {
  const ch = text[i];
  const before = text[i - 1];
  if (ch === '@') return mentionAt(text, i);
  if (ch === '`') return codeAt(text, i);
  if (isAlnum(ch) && !isAlnum(before)) {
    const email = match(EMAIL, text, i);
    if (email) return { node: { t: 'text', v: email[0] }, end: i + email[0].length };
  }
  if (depth >= MAX_DEPTH) return null;
  if (ch === '[' && !inLink) return linkAt(text, i, depth);
  if (ch === 'h' && !inLink && !isAlnum(before)) return bareUrlAt(text, i);
  if (ch === '*' || ch === '_') return emphasisAt(text, i, depth, inLink);
  return null;
};

const match = (pattern, text, at) => {
  pattern.lastIndex = at;
  return pattern.exec(text);
};

const mentionAt = (text, i) => {
  const token = match(TOKEN_AT, text, i);
  if (token) return { node: { t: 'mention', label: token[1], id: token[2] }, end: i + token[0].length };
  // A person written as an email, `@ana.ruiz@dynapps.es`, is one mention.
  const email = isAlnum(text[i - 1]) ? null : match(EMAIL, text, i + 1);
  if (email) return { node: { t: 'mention', label: email[0], id: null }, end: i + 1 + email[0].length };
  return null;
};

const codeAt = (text, i) => {
  const size = /^`+/.exec(text.slice(i))[0].length;
  const ticks = '`'.repeat(size);
  let close = text.indexOf(ticks, i + size);
  while (close !== -1 && (text[close - 1] === '`' || text[close + size] === '`')) close = text.indexOf(ticks, close + 1);
  if (close === -1) return { node: { t: 'text', v: ticks }, end: i + size };
  const inner = text.slice(i + size, close);
  const code = /^ .*\S.* $/s.test(inner) ? inner.slice(1, -1) : inner;
  return { node: { t: 'code', v: code }, end: close + size };
};

// Only https leaves the page; every other scheme stays text.
const httpsHref = (url) => (URL.canParse(url) && new URL(url).protocol === 'https:' ? new URL(url).href : null);

const linkAt = (text, i, depth) => {
  const link = match(LINK, text, i);
  const href = link && httpsHref(link[2]);
  if (!href) return null;
  return { node: { t: 'a', href, c: parseInline(link[1], depth + 1, true) }, end: i + link[0].length };
};

const bareUrlAt = (text, i) => {
  const raw = match(BARE_URL, text, i)?.[0].replace(/[.,;:!?'"*_)\]]+$/, '');
  const href = raw && httpsHref(raw);
  if (!href) return null;
  return { node: { t: 'a', href, c: [{ t: 'text', v: raw }] }, end: i + raw.length };
};

// `**x**` and `__x__` are bold, `*x*` and `_x_` italic. A marker needs text
// right inside it, and an underscore never opens or closes inside a word, so
// `snake_case_name` stays as typed.
const emphasisAt = (text, i, depth, inLink) => {
  const mark = text[i];
  const double = text[i + 1] === mark;
  const size = double ? 2 : 1;
  const token = mark.repeat(size);
  if (mark === '_' && isAlnum(text[i - 1])) return null;
  if (isSpace(text[i + size]) || text[i + size] === mark) return null;
  let close = text.indexOf(token, i + size);
  while (close !== -1 && !closes(text, close, size, mark, i + size)) close = text.indexOf(token, close + 1);
  if (close === -1) return null;
  return { node: { t: double ? 'strong' : 'em', c: parseInline(text.slice(i + size, close), depth + 1, inLink) }, end: close + size };
};

const closes = (text, close, size, mark, from) => {
  if (close === from || isSpace(text[close - 1]) || text[close + size] === mark) return false;
  if (size === 1 && text[close - 1] === mark) return false;
  return !(mark === '_' && isAlnum(text[close + size]));
};
