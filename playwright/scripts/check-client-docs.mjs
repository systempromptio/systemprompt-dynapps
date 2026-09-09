// Read-only checks for published client documentation; no account or database seeding.
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { chromium } from '@playwright/test';

const base = process.argv[2];
assert(base && /^https?:\/\//.test(base), 'Usage: node scripts/check-client-docs.mjs <site-url>');
const slugs = ['index', 'bridge-install', 'connect-claude-code', 'connect-cowork', 'install-macos', 'install-windows', 'downloads'];
const sourceRoot = new URL('../../services/content/documentation/', import.meta.url);
const sources = new Map();
for (const slug of slugs) {
  const source = await readFile(new URL(`${slug}.md`, sourceRoot), 'utf8');
  assert.match(source, /public: true/);
  assert.match(source, /updated_at: "2026-09-09"/);
  assert.doesNotMatch(source, /pinned-release|API Usage Billing|xattr -d|More info → Run anyway|signing resumes|\[1m\]/i);
  sources.set(slug, source);
}
assert.match(sources.get('connect-claude-code'), /install --host claude-code/);
assert.match(sources.get('connect-claude-code'), /uninstall --host claude-code/);
assert.match(sources.get('connect-claude-code'), /--settings.*claude-code-settings\.json/);
assert.match(sources.get('connect-claude-code'), /--model gemini-2\.5-flash/);
assert.match(sources.get('bridge-install'), /Generate a connect code/);
assert.match(sources.get('install-windows'), /inside WSL/);
assert.match(sources.get('downloads'), /\$ActualHash -ne \$ExpectedHash/);

const browser = await chromium.launch();
const context = await browser.newContext();
const page = await context.newPage();
const links = new Map();
try {
  for (const slug of slugs) {
    const path = `/documentation${slug === 'index' ? '' : `/${slug}`}`;
    const response = await page.goto(new URL(path, base).href);
    assert.equal(response.status(), 200, path);
    const main = page.locator('main');
    const title = sources.get(slug).match(/^title: "(.+)"$/m)[1];
    assert(await main.getByRole('heading', { name: title, exact: true }).count(), `${path}: missing title`);
    const text = await main.innerText();
    const firstParagraph = sources.get(slug).split(/^# .+$/m)[1].trim().split('\n\n')[0]
      .replace(/\[([^\]]+)\]\([^)]+\)/g, '$1');
    assert(text.includes(firstParagraph), `${path}: published introduction is stale`);
    assert(!text.includes('<!--'), `${path}: leaked source comment`);
    const codeBlocks = [...sources.get(slug).matchAll(/```[^\n]*\n([\s\S]*?)```/g)].map(match => match[1].trim());
    assert.deepEqual((await main.locator('pre code').allTextContents()).map(value => value.trim()), codeBlocks, `${path}: stale or altered commands`);
    for (const href of await main.locator('a[href]').evaluateAll(nodes => nodes.map(n => n.getAttribute('href')))) {
      const url = new URL(href, page.url());
      if (url.origin === new URL(base).origin && url.pathname.startsWith('/documentation')) links.set(url.href, url);
    }
    await page.setViewportSize({ width: 390, height: 844 });
    assert(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1), `${path}: mobile page overflow`);
    await page.setViewportSize({ width: 1280, height: 900 });
    const toc = await page.locator('.toc-nav').boundingBox();
    const related = await page.locator('.toc-related-playbooks').boundingBox();
    if (toc && related) assert(toc.y + toc.height <= related.y + 1, `${path}: overlapping table of contents`);
    console.log(`PASS ${path}: current text, heading, mobile width`);
  }
  for (const url of links.values()) {
    const response = await context.request.get(url.href);
    assert.equal(response.status(), 200, `broken documentation link: ${url.href}`);
    if (url.hash) {
      await page.goto(url.href);
      const id = decodeURIComponent(url.hash.slice(1));
      assert(await page.evaluate(value => Boolean(document.getElementById(value)), id), `missing anchor: ${url.href}`);
    }
  }
  console.log(`PASS ${links.size} documentation links and anchors; sources: ${fileURLToPath(sourceRoot)}`);
} finally {
  await context.close();
  await browser.close();
}
