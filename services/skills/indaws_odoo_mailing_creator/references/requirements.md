# Technical requirements — odoo-mailing-creator

What Claude MUST check (and install or request if missing) **before** running the pipeline. Do not assume anything is installed: always verify as the first step.

> Language note: this file and all scripts are written in English. When the skill is active, Claude must reply to the user **in the language the user is speaking** (Spanish, English, etc.) — only the artifacts (code, docs) stay in English.

## Table of contents
- [1. Runtime environment](#1-runtime-environment)
- [2. npm dependencies (skill-local)](#2-npm-dependencies-skill-local)
- [3. Odoo access (MCP)](#3-odoo-access-mcp)
- [4. Quick check (single block)](#4-quick-check-single-block)
- [5. What to do if something is missing](#5-what-to-do-if-something-is-missing)
- [6. Verified reference versions](#6-verified-reference-versions)

---

## 1. Runtime environment

| Requirement | Minimum | Verified on | Check with | Why |
|---|---|---|---|---|
| **Node.js** | **>= 18** | v22.22.0 | `node -v` | The pipeline (mjml/juice/minify) runs as a Node library in a single process. |
| **npm** | >= 9 | 11.7.0 | `npm -v` | Install the skill-local dependencies. |
| **OS** | macOS / Linux | darwin 25.1.0 | — | Scripts use POSIX paths and `bash`/`node`. |

No browser required. **Forbidden**: Playwright/Selenium/Puppeteer/headless Chromium. Visual validation is by human checklist + real test send (see `validation.md`).

**Optional system tool** — `rsvg-convert` (librsvg), only needed when converting SVG brand assets to PNG via `svg_to_png.cjs`. Check with `rsvg-convert --version` (verified 2.61.1). macOS: `brew install librsvg`. Not required for the core pipeline (decorative icons use emoji).

## 2. npm dependencies (skill-local)

Declared in the skill's `package.json` and installed into `.claude/skills/odoo-mailing-creator/node_modules/`. They are **not** global: the skill is self-contained and portable across clients.

| Package | PINNED version | Pipeline role | Why this version |
|---|---|---|---|
| `mjml` | **4.18.0** | Compiles `.mjml` → HTML with tables, ghost tables `[if mso]`, background VML, responsive | v5.x made `mjml2html()` **async** and breaks the sync wrapper. **Do not bump to v5** without rewriting scripts with `await`. |
| `juice` | **8.1.0** | Inlines `<head>` CSS (Gmail drops `<style>`) while preserving `@media`/dark | juice 10/11 changed defaults and crashes on malformed HTML in this flow. |
| `html-minifier-terser` | **7.2.0** | Minifies (~20% smaller) **without removing** `[if mso]` comments | `minify()` is async; `removeComments:false` is mandatory. |
| `cheerio` | **1.0.0** | Odoo adaptation (extract `<body>`, fold `<style>`, wrap) and structural validation | Browserless HTML parser. |

> If `doiuse-email` (caniemail compatibility lint) is added later, document it here with a pinned version before use.

## 3. Odoo access (MCP)

Uploading the mailing and sending the real test use MCP tools from the `odoo-indaws` server (the skill does not run Odoo locally):

| MCP tool | Use | Note |
|---|---|---|
| `mcp__odoo-indaws__orm_write` | Write `body_html` **and** `body_arch` of the `mailing.mailing` | **Only reliable** option for large payloads. |
| `mcp__odoo-indaws__orm_create` | Create the `mail.mail` for the real test | Returns the ID as `mail.mail(NNNN,)` — **read the real ID**, never guess it. |
| `mcp__odoo-indaws__orm_search_read` | Idempotency (find mailing by subject) and post-upload verification | — |
| `mcp__odoo-indaws__orm_execute_kw` | Only for `process_email_queue([id])` and reads | **Do not** use `method=write` with large payloads: it returns `-32000`. |

Check availability: these tools appear in the session's MCP tool list. If absent, the `odoo-indaws` server is not connected → tell the user; the pipeline up to validation (steps 1–6) works fine without Odoo.

**Known MCP limit (verified):** `orm_create` returns `-32000` for `ir.attachment` (and other large/binary payloads) — image hosting and large creates are NOT possible via MCP. For PNG attachments and large bodies use the **`scripts/odoo_push.cjs`** JSON-RPC script instead, which reads files from disk. It needs env vars `ODOO_URL`, `ODOO_DB`, `ODOO_USERNAME`, `ODOO_API_KEY` (Odoo API key from Settings → Account → Security → API Keys). Keep them in a **gitignored `.env.local`** in the skill dir — `odoo_push.cjs` **auto-loads it** from its own directory (no `cd`/`source` needed; real `process.env` always wins). The DB name is the Odoo database, not the company name (discover via `curl -s -X POST <URL>/jsonrpc -d '{"jsonrpc":"2.0","method":"call","params":{"service":"db","method":"list","args":[]}}'`).

## 4. Quick check (one command, runs from any cwd)

The scripts self-install their deps on first use. To pre-check Node, install deps, and detect `rsvg-convert`:

```bash
node scripts/_bootstrap.cjs
```

(Resolve the path to wherever the skill lives; it does not depend on the current directory.) Prints
`{ ok, node, deps, rsvg }`. Expected pinned versions once installed: `mjml 4.18.0`, `juice 8.1.0`,
`html-minifier-terser 7.2.0`, `cheerio 1.0.0`.

## 5. What to do if something is missing

| Symptom | Cause | Action |
|---|---|---|
| `node: command not found` | Node not installed | Ask the user to install Node 18+ (`brew install node` on macOS). Do not continue. |
| `node -v` < v18 | Old Node | Ask to upgrade; `minify` async and mjml need 18+. |
| `Cannot find module 'mjml'` | Missing `npm install` | Run `npm install --no-audit --no-fund` in the skill dir. |
| `r.html is undefined` after compile | mjml v5 (async) slipped in | Reinstall pinning `mjml@4.18.0` (`npm i --save-exact mjml@4.18.0`). |
| `Cannot read properties of undefined (reading 'replace')` in juice | juice got empty HTML (upstream MJML error) or juice 10/11 | Check MJML errors first; pin `juice@8.1.0`. |
| `mcp__odoo-indaws__*` tools absent | MCP server not connected | Tell the user; run only up to local validation (no upload/send). |

## 6. Verified reference versions

Snapshot of the machine where the skill was built and tested (2026-05-31):

```
node v22.22.0
npm  11.7.0
mjml 4.18.0
juice 8.1.0
html-minifier-terser 7.2.0
cheerio 1.0.0
```

Pipeline smoke test (compile → inline → minify → prepare_odoo → validate) confirmed: keeps `[if mso]` comments (balanced), preserves `@media`, keeps ghost tables and `mso-*` CSS, exit code 0 on a clean fragment.
