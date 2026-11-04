// Core's sanitised preview set inside a frame dressed for its format: a page
// for Word and OpenDocument text, a grid for spreadsheets, 16:9 cards for
// slides, a plain web page for HTML and a reading column for Markdown. The
// frame stays sandboxed without scripts or same origin.
import { errorMessage, rawResponse } from './api.js';

const BASE = `
  :root { color-scheme: light; }
  body { margin: 0; color: #1f1d1a; font: 11pt/1.55 "Inter", "Segoe UI", Calibri, system-ui, sans-serif; }
  h1, h2, h3, h4 { line-height: 1.25; color: #111; margin: 1.2em 0 .45em; }
  h1 { font-size: 22pt; } h2 { font-size: 15pt; } h3 { font-size: 12.5pt; }
  p { margin: 0 0 .8em; } ul, ol { padding-left: 1.4em; margin: 0 0 .8em; }
  a { color: #b3261e; } img { max-width: 100%; }
  table { border-collapse: collapse; }
`;

const SHELLS = {
  page: `
    html { background: #eceae6; }
    body { box-sizing: border-box; max-width: 816px; min-height: 1056px; margin: 32px auto; padding: 80px 88px;
      background: #fff; box-shadow: 0 1px 3px rgb(0 0 0 / 12%), 0 8px 24px rgb(0 0 0 / 8%); }
    body > p:first-child { font-size: 9pt; font-weight: 600; letter-spacing: .08em; text-transform: uppercase; color: #d62d20; }
    th, td { border: 1px solid #d8d4cd; padding: 6px 8px; text-align: left; }
  `,
  web: `
    html { background: #fff; }
    body { max-width: 960px; margin: 0 auto; padding: 40px 32px 64px; font-size: 16px; }
    header { padding: 32px 0 24px; border-bottom: 1px solid #e6e2dc; margin-bottom: 24px; }
    header > p:first-child { color: #d62d20; font-weight: 600; font-size: 13px; letter-spacing: .06em; text-transform: uppercase; }
    header h1 { font-size: 34px; margin: .2em 0 .3em; }
    table { width: 100%; margin: 12px 0 20px; } th, td { border-bottom: 1px solid #e6e2dc; padding: 8px 10px; text-align: left; }
    th { font-size: 13px; color: #6b665e; text-transform: uppercase; letter-spacing: .04em; }
    footer { margin-top: 40px; color: #6b665e; font-size: 14px; }
  `,
  sheet: `
    html { background: #fff; }
    body { font-size: 13px; }
    caption { text-align: left; padding: 8px 12px; font-weight: 600; background: #f6f4f0; border-bottom: 1px solid #d8d4cd; }
    table { width: 100%; }
    td, th { border: 1px solid #e3dfd8; padding: 5px 10px; white-space: nowrap; }
    tr:nth-child(2) td { font-weight: 600; background: #fbfaf8; }
  `,
  slides: `
    html { background: #2a2826; }
    body { padding: 32px; display: grid; gap: 28px; justify-items: center; }
    section { box-sizing: border-box; width: min(100%, 880px); aspect-ratio: 16 / 9; background: #fff; border-radius: 6px;
      padding: 48px 64px; box-shadow: 0 10px 30px rgb(0 0 0 / 35%); position: relative; overflow: hidden; }
    section::after { content: attr(data-slide); position: absolute; right: 20px; bottom: 14px; font-size: 11px; color: #9a948b; }
    section h2 { font-size: 30px; margin: 0 0 24px; }
    section li { font-size: 19px; margin-bottom: 10px; }
    section p { font-size: 17px; color: #4a4640; }
  `,
  note: `
    html { background: #fff; }
    body { max-width: 720px; margin: 0 auto; padding: 40px 32px 64px; font-size: 15px; }
    h1 { font-size: 26px; } em { color: #6b665e; }
  `,
};

const SHELL_FOR = {
  docx: 'page', odt: 'page', text: 'note', html: 'web', xlsx: 'sheet',
  ods: 'sheet', pptx: 'slides', odp: 'slides', markdown: 'note',
};

export const sheetDocument = (fragment, format) =>
  `<!doctype html><html><head><meta charset="utf-8"><style>${BASE}${SHELLS[SHELL_FOR[format] ?? 'page']}</style></head><body>${fragment}</body></html>`;

export const showPaper = async (frame, previewApi, format, revision = null) => {
  const url = revision ? `${previewApi}?revision=${encodeURIComponent(revision)}` : previewApi;
  const resp = await rawResponse(url);
  if (!resp.ok) throw new Error(await errorMessage(resp));
  frame.srcdoc = sheetDocument(await resp.text(), format);
};
