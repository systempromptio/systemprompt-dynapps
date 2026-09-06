// Builds playwright/screenshots/groups/review.html: one self-contained page
// embedding every PNG of the pack as a data URI, captioned with the page it
// shows, the URL it came from and the principal that was signed in.
//
// Self-contained on purpose — the page is published as an artifact for review,
// where no relative asset would resolve, and a review pack whose images are
// missing is worse than no pack at all.
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { OUT_DIR, SHOTS } from './capture-groups';

function dataUri(file: string): string {
  return `data:image/png;base64,${readFileSync(file).toString('base64')}`;
}

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

const STYLE = `
:root {
  color-scheme: light dark;
  --bg: #f7f7f5;
  --surface: #ffffff;
  --ink: #1c1c1a;
  --muted: #6b6b64;
  --line: #e2e2dc;
  --accent: #b4231f;
}
@media (prefers-color-scheme: dark) {
  :root:not([data-theme="light"]) {
    --bg: #14140f;
    --surface: #1d1d18;
    --ink: #f0efe9;
    --muted: #a6a49a;
    --line: #33322b;
    --accent: #e9564f;
  }
}
:root[data-theme="dark"] {
  --bg: #14140f;
  --surface: #1d1d18;
  --ink: #f0efe9;
  --muted: #a6a49a;
  --line: #33322b;
  --accent: #e9564f;
}
body { background: var(--bg); color: var(--ink); font: 15px/1.55 ui-sans-serif, system-ui, sans-serif; }
.wrap { max-width: 1180px; margin: 0 auto; padding: 40px 24px 96px; }
h1 { font-size: 30px; margin: 0 0 8px; letter-spacing: -0.02em; }
.lede { color: var(--muted); max-width: 62ch; margin: 0 0 32px; }
.toc { display: flex; flex-wrap: wrap; gap: 8px; margin: 0 0 40px; padding: 0; list-style: none; }
.toc a { display: inline-block; padding: 5px 11px; border: 1px solid var(--line); border-radius: 999px;
         background: var(--surface); color: var(--ink); text-decoration: none; font-size: 13px; }
.toc a:hover { border-color: var(--accent); color: var(--accent); }
figure { margin: 0 0 40px; background: var(--surface); border: 1px solid var(--line);
         border-radius: 10px; overflow: hidden; }
figcaption { padding: 14px 18px; border-bottom: 1px solid var(--line); }
figcaption .title { font-weight: 600; }
figcaption .meta { color: var(--muted); font-size: 13px; margin-top: 3px; }
figcaption code { font-family: ui-monospace, SFMono-Regular, Menlo, monospace; font-size: 12.5px; }
.shot { display: block; width: 100%; height: auto; }
.badge { display: inline-block; margin-left: 8px; padding: 1px 8px; border-radius: 999px;
         background: var(--bg); border: 1px solid var(--line); font-size: 12px; color: var(--muted); }
`;

function main() {
  const figures: string[] = [];
  const toc: string[] = [];
  const missing: string[] = [];

  for (const shot of SHOTS) {
    const file = join(OUT_DIR, `${shot.name}.png`);
    if (!existsSync(file)) {
      missing.push(shot.name);
      continue;
    }
    toc.push(`<li><a href="#${shot.name}">${escapeHtml(shot.title)}</a></li>`);
    figures.push(
      [
        `<figure id="${shot.name}">`,
        '  <figcaption>',
        `    <div class="title">${escapeHtml(shot.title)}<span class="badge">${shot.principal}</span></div>`,
        `    <div class="meta"><code>${escapeHtml(shot.path)}</code> — ${shot.name}.png</div>`,
        '  </figcaption>',
        `  <img class="shot" alt="${escapeHtml(shot.title)}" src="${dataUri(file)}">`,
        '</figure>',
      ].join('\n'),
    );
  }

  if (figures.length === 0) {
    throw new Error(
      `no screenshots in ${OUT_DIR} — run \`just e2e-screens-groups\` against a running stack first.`,
    );
  }

  const html = [
    '<title>Groups and marketplace access — review pack</title>',
    `<style>${STYLE}</style>`,
    '<div class="wrap">',
    '<h1>Groups, projects and marketplace access</h1>',
    '<p class="lede">Every admin surface added by this change, captured against the',
    'deterministic e2e seed. Each shot names the URL it came from and the principal',
    'that was signed in, so a reviewer can reproduce it exactly.</p>',
    missing.length > 0
      ? `<p class="lede">Not captured in this run: ${missing.map(escapeHtml).join(', ')}.</p>`
      : '',
    `<ul class="toc">${toc.join('')}</ul>`,
    figures.join('\n'),
    '</div>',
  ].join('\n');

  const out = join(OUT_DIR, 'review.html');
  writeFileSync(out, `${html}\n`);
  console.log(`review page written to ${out} (${figures.length} figures)`);
}

main();
