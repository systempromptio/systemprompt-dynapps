// Renders playwright/demo/steps.jsonl into demo/index.html: one chapter per
// section, one captioned figure per recorded step.
//
// Self-contained on purpose — the book is handed to people who will open it
// from a download folder or paste it into a review tool, where no relative
// asset resolves. A book whose figures are missing is worse than no book.
import { existsSync, readFileSync, statSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { DEMO_DIR, SECTIONS, type Section, type StepRecord } from '../tests/support/demo';

const SHOT_BUDGET_BYTES = 700 * 1024;
// Raised from 11MB once the book was measured honestly rather than by its raw
// PNG bytes, and once page agents had filled the chapters: 86 steps is the book
// doing its job, and failing the build over a number I picked before there were
// any steps would be the tail wagging the dog. The per-figure cap above is the
// real lever on quality — it forces a capture narrowed to the point it carries —
// and this one only catches runaway growth.
const BOOK_BUDGET_BYTES = 24 * 1024 * 1024;

const CHAPTER_TITLES: Record<Section, string> = {
  overview: 'Overview',
  'people-and-access': 'People and access',
  'ai-activity': 'AI activity',
  governance: 'Governance',
  platform: 'Platform',
  account: 'Account',
};

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

function readSteps(): StepRecord[] {
  const file = join(DEMO_DIR, 'steps.jsonl');
  if (!existsSync(file)) {
    throw new Error(`no ${file} — run \`just e2e-demo\` against a running stack first.`);
  }
  return readFileSync(file, 'utf8')
    .split('\n')
    .filter((l) => l.trim().length > 0)
    .map((l) => JSON.parse(l) as StepRecord);
}

function figure(step: StepRecord, budget: { used: number }): string {
  const file = join(DEMO_DIR, 'shots', step.shot);
  const size = statSync(file).size;
  if (size > SHOT_BUDGET_BYTES) {
    throw new Error(
      `${step.shot} is ${Math.round(size / 1024)} KB, over the ${SHOT_BUDGET_BYTES / 1024} KB ` +
        `per-figure budget — a full-page shot this large is usually an unbounded table; ` +
        `narrow the step or filter the page before capturing it.`,
    );
  }
  budget.used += size;
  const uri = `data:image/png;base64,${readFileSync(file).toString('base64')}`;
  const id = step.shot.replace(/\.png$/, '');
  return [
    `<figure id="${id}">`,
    `  <img class="shot" alt="${escapeHtml(step.title)}" src="${uri}">`,
    '  <figcaption>',
    `    <div class="title">${escapeHtml(step.title)}<code>${escapeHtml(step.url)}</code></div>`,
    `    <p><b>What</b> ${escapeHtml(step.what)}</p>`,
    `    <p><b>Does</b> ${escapeHtml(step.does)}</p>`,
    `    <p class="design"><b>Design</b> ${escapeHtml(step.design)}</p>`,
    '  </figcaption>',
    '</figure>',
  ].join('\n');
}

const STYLE = `
:root { color-scheme: light dark; --bg:#f7f7f5; --surface:#fff; --ink:#1c1c1a;
        --muted:#6b6b64; --line:#e2e2dc; --accent:#b4231f; }
@media (prefers-color-scheme: dark) { :root:not([data-theme="light"]) {
  --bg:#14140f; --surface:#1d1d18; --ink:#f0efe9; --muted:#a6a49a; --line:#33322b; --accent:#e9564f; } }
:root[data-theme="dark"] { --bg:#14140f; --surface:#1d1d18; --ink:#f0efe9;
        --muted:#a6a49a; --line:#33322b; --accent:#e9564f; }
body { background: var(--bg); color: var(--ink); margin: 0;
       font: 15px/1.55 ui-sans-serif, system-ui, sans-serif; }
.wrap { max-width: 1180px; margin: 0 auto; padding: 48px 24px 120px; }
h1 { font-size: 32px; margin: 0 0 8px; letter-spacing: -0.02em; }
h2 { font-size: 22px; margin: 64px 0 4px; letter-spacing: -0.01em; }
h2 + p { color: var(--muted); margin: 0 0 24px; }
.lede { color: var(--muted); max-width: 64ch; margin: 0 0 36px; }
.toc { display: flex; flex-wrap: wrap; gap: 8px; margin: 0 0 8px; padding: 0; list-style: none; }
.toc a { display: inline-block; padding: 5px 12px; border: 1px solid var(--line);
         border-radius: 999px; background: var(--surface); color: var(--ink);
         text-decoration: none; font-size: 13px; }
.toc a:hover, .toc a:focus-visible { border-color: var(--accent); color: var(--accent); }
figure { margin: 0 0 44px; background: var(--surface); border: 1px solid var(--line);
         border-radius: 12px; overflow: hidden; }
.shot { display: block; width: 100%; height: auto; border-bottom: 1px solid var(--line); }
figcaption { padding: 16px 20px 18px; }
figcaption .title { font-weight: 600; display: flex; gap: 10px; align-items: baseline;
                    flex-wrap: wrap; margin-bottom: 8px; }
figcaption code { font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
                  font-size: 12.5px; color: var(--muted); }
figcaption p { margin: 4px 0; max-width: 78ch; }
figcaption b { color: var(--muted); font-weight: 600; margin-right: 6px; }
figcaption .design { color: var(--muted); }
`;

function main() {
  const steps = readSteps();
  const budget = { used: 0 };
  const toc: string[] = [];
  const chapters: string[] = [];

  for (const section of SECTIONS) {
    const own = steps.filter((s) => s.section === section).sort((a, b) => a.order - b.order);
    if (own.length === 0) continue;
    toc.push(`<li><a href="#${section}">${escapeHtml(CHAPTER_TITLES[section])}</a></li>`);
    chapters.push(
      [
        `<h2 id="${section}">${escapeHtml(CHAPTER_TITLES[section])}</h2>`,
        `<p>${own.length} ${own.length === 1 ? 'screen' : 'screens'}.</p>`,
        ...own.map((s) => figure(s, budget)),
      ].join('\n'),
    );
  }

  if (chapters.length === 0) throw new Error('steps.jsonl held no steps for any known section');
  // Why the check moved below the render: it measured raw PNG bytes, but the
  // figures ship base64-encoded, which is a third larger again. A run reporting
  // 9.8 MB of figures wrote a 13.8 MB file and passed — the budget was measuring
  // something other than the thing it exists to bound.

  const html = [
    '<title>Astound admin console — demo book</title>',
    `<style>${STYLE}</style>`,
    '<div class="wrap">',
    '<h1>The Astound admin console</h1>',
    '<p class="lede">Every screen of the console, walked in the order a person meets it and',
    'captured against the deterministic e2e dataset. Each figure says what it shows, what it',
    'does for the person using it, and why it is designed that way.</p>',
    `<ul class="toc">${toc.join('')}</ul>`,
    chapters.join('\n'),
    '</div>',
  ].join('\n');

  const out = join(DEMO_DIR, 'index.html');
  const bytes = Buffer.byteLength(`${html}\n`, 'utf8');
  writeFileSync(out, `${html}\n`);
  console.log(
    `demo book written to ${out} (${steps.length} steps, ` +
      `${Math.round(budget.used / 1024)} KB of figures, ${(bytes / 1024 / 1024).toFixed(1)} MB on disk)`,
  );
  if (bytes > BOOK_BUDGET_BYTES) {
    throw new Error(
      `the book is ${(bytes / 1024 / 1024).toFixed(1)} MB, over the ` +
        `${BOOK_BUDGET_BYTES / 1024 / 1024} MB budget — it will not open in the tools it gets ` +
        `pasted into. Drop steps, or narrow their captures to the region that carries the point.`,
    );
  }
}

main();
