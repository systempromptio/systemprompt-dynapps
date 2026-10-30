// The document as a page: core's sanitised preview HTML set inside an
// Dynapps-branded sheet. The frame stays sandboxed without scripts or same
// origin, so the preview can style itself but never run or reach the console.
import { errorMessage, rawResponse } from './api.js';

const SHEET_CSS = `
  :root { color-scheme: light; }
  html { background: #eceae6; }
  body {
    box-sizing: border-box; max-width: 816px; min-height: 1056px; margin: 32px auto;
    padding: 72px 88px 88px; background: #fff; color: #1f1d1a;
    border-top: 6px solid oklch(0.5 0.18 255); box-shadow: 0 1px 3px rgb(0 0 0 / 12%), 0 8px 24px rgb(0 0 0 / 8%);
    font: 11pt/1.55 "Inter", "Segoe UI", Calibri, system-ui, sans-serif;
  }
  h1, h2, h3, h4 { line-height: 1.25; color: #111; margin: 1.4em 0 .5em; }
  h1 { font-size: 22pt; font-weight: 700; }
  h2 { font-size: 16pt; font-weight: 700; }
  h3 { font-size: 13pt; font-weight: 600; }
  p { margin: 0 0 .8em; }
  p:empty::before { content: "\\00a0"; }
  ul, ol { padding-left: 1.4em; margin: 0 0 .8em; }
  a { color: oklch(0.45 0.18 255); }
  table { border-collapse: collapse; width: 100%; margin: 1em 0; font-size: 10pt; }
  th, td { border: 1px solid #d8d4cd; padding: 6px 8px; text-align: left; vertical-align: top; }
  th { background: #f6f4f0; font-weight: 600; }
  section, .slide { border: 1px solid #d8d4cd; border-radius: 6px; padding: 24px 28px; margin: 0 0 24px; }
  img { max-width: 100%; }
`;

export const sheetDocument = (fragment) =>
  `<!doctype html><html><head><meta charset="utf-8"><style>${SHEET_CSS}</style></head><body>${fragment}</body></html>`;

export const showPaper = async (frame, previewApi) => {
  const resp = await rawResponse(previewApi);
  if (!resp.ok) throw new Error(await errorMessage(resp));
  frame.srcdoc = sheetDocument(await resp.text());
};
