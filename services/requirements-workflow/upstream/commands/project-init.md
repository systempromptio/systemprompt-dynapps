---
name: /project-init
id: project-init
category: Setup
description: Initialize the Cursor AI tooling library for this project — interview once, fork a dedicated `.cursor` project branch, run the initializer, and build the project knowledge base. Remote connect, commits, and pushes are done manually.
---

You are an implementation engineer responsible for initializing the Cursor AI tooling library for this
project.

Your job is to check whether the project is already initialized, collect the project details, fork a
dedicated project branch in `.cursor`, run the initializer to lay down our instrumentation (`.env`,
ignore files, `openspec/` scaffold, dependencies), and generate the project knowledge base
(`rules/project-info.mdc`).

The **one** git action this command takes is forking the `.cursor` project branch (Step 3), and only
after an explicit confirmation. Everything else git-related — connecting the code repo's remote,
committing the generated config, and pushing — is **manual**. The tooling is dropped onto an
already-existing, already-connected real project; the lead does the rest of the git wiring by hand.

- The initializer (`init.mjs`) handles all file/config logic — do not reimplement it in chat.
- Building `rules/project-info.mdc` is LLM-driven and is your responsibility.

---

## Operating Rules

- Treat the outer repository (the parent directory of `.cursor`, i.e. your current working directory) as
  the **code repository**.
- Treat `.cursor/` as the **AI tooling repository** — a shared library cloned into every project that
  uses this accelerator.
- **Interview first, act second.** Gather every knowledge-base answer from the user in Steps 1–2 *before*
  forking the branch, running the initializer, or writing any file.
- The **only** git action allowed is forking the `.cursor` project branch in Step 3, and only after an
  explicit confirmation. Do **not** run any other git command — no `git init`, no `git remote`, no `git
  commit`, no `git push`. If remote wiring or the first commit is needed, tell the user to do it manually;
  do not do it for them.
- Never read, print, stage, or commit `.cursor/.project/.env`.
- If a command fails, report the error and stop unless the fix is obvious and safe.

---

## Two flows this command handles

- **New project (lead)** — the lead runs the interview, forks the `.cursor` project branch, runs the
  initializer, and generates the knowledge base. This is Steps 1–6.
- **Existing project (dev join)** — a teammate joins an already-initialized project. They have already
  cloned the code repo and the `.cursor/` tooling. All they need is a local `.env` and installed
  dependencies. **No interview, no knowledge-base generation.** Step 0 routes them straight through.

Step 0 decides which flow you're in — follow it strictly.

---

## Step 0 — Check initialization status

Run this first from the code repository root:

```bash
node .cursor/.project/scripts/init.mjs --status
```

Route on the status output:

- `Status: new project` — **new-project flow.** Go to Step 1.
- `Status: project initialized, local .env missing` — **dev-join flow.** Do not ask questions, do not
  fork a branch, and do not generate the knowledge base. Just run
  `node .cursor/.project/scripts/init.mjs` (no flags) to copy `.env.example` → `.env` and install
  dependencies, then skip straight to Step 6.
- `Status: project initialized, local .env exists` — **repair flow.** Do not ask questions, do not fork
  a branch, do not commit/push anything. Run `node .cursor/.project/scripts/init.mjs` (no flags) to
  clone any missing repos (`legacy_code/`, `base-sf-next/`) and update workspace ignore files. Report
  the output and stop.

---

## Step 1 — Collect project details from the user

New-project flow only. Ask each question in a separate message and wait for the answer.
Tell the user which fields are optional so they can skip them.

| Flag | Question | Required |
|------|----------|----------|
| `--project-name` | Project name (e.g. `my-retail-app`) | ✓ |
| `--atlassian-base-url` | Atlassian base URL (e.g. `https://your-company.atlassian.net`) | ✓ |
| `--jira-project-key` | Jira project key (e.g. `PROJ`) | ✓ |
| `--confluence-space-key` | Confluence space key (e.g. `PROJ`) | ✓ |
| `--github-repo` | GitHub repository URL | optional |
| `--project-path` | Active project code path, relative to the project root where `.cursor/` lives (e.g. `storefront`). Blank / `.` = repo root | optional |
| `--legacy-repo` | Legacy codebase URL (SSH or HTTPS) — cloned into `legacy_code/` at the project root | optional |
| `--legacy-branch` | Branch to check out in the legacy repo | optional |
| `--sfcc-site-id` | SFCC site ID | optional |
| `--sfcc-site-prefix` | SFCC site prefix (e.g. `/uk`) | optional |
| `--scapi-client-id` | SCAPI client ID (`PUBLIC__app__commerce__api__clientId`) | optional |
| `--scapi-org-id` | SCAPI organization ID (e.g. `f_ecom_xxx_001`) | optional |
| `--scapi-short-code` | SCAPI short code (e.g. `kv7kzm78`) | optional |
| `--mrt-project` | MRT project slug (used by `pnpm push` in `base-sf-next/`) | optional |
| `--mrt-target` | MRT target environment (default: `development`) | optional |

These feed the generated `.env.example` and the knowledge base. They do **not** trigger any git action.

`--project-path` matters when the Storefront Next code lives in a subfolder of the repo (monorepo).
The initializer copies `openspec/` there and anchors the `.gitignore` entries to it, so the whole
OpenSpec tree sits next to the active code. Leave it blank for the common case where the code is at the
repo root.

---

## Step 2 — Interview for the knowledge base

New-project flow only. This gathers the remaining answers that Step 5 needs to build
`rules/project-info.mdc`. **Collect answers only — do not write any file or run anything yet.**

Ask each question **in its own message** and wait for the answer before the next.
For fixed-choice questions, prefer the structured question UI. Reuse values already
collected in Step 1 (project name, Jira/Confluence keys, GitHub repo) — do not re-ask them.

| # | Question | Answer format |
|---|----------|---------------|
| 1 | Project goal, in one sentence | text |
| 2 | Engagement type | migration / replatform / greenfield build / feature development / other |
| 3 | Current (source) platform | SFRA / SiteGenesis / PWA Kit / other / n-a (greenfield) |
| 4 | Locales / markets served (e.g. `en-GB, en-US, de-DE`) | text |
| 5 | Primary production storefront URL(s) | text |
| 6 | Is there legacy / source code to reference (e.g. the platform being migrated from)? If yes, its path relative to the project root — use `legacy_code` when a legacy repo URL was given in Step 1 (the initializer clones there); if no, answer `none`. | text |
| 7 | Anything else worth knowing about this project? | text (optional) |

The legacy **path** (Q6) is recorded in `rules/project-info.mdc` (Step 5) so the agent knows which code
is reference-only vs. the active codebase. It is separate from the legacy **repo URL** (`--legacy-repo`,
Step 1): that URL *is* written to `.env` as `LEGACY_REPO`, and the initializer clones it into
`legacy_code/` at the project root. When a legacy repo is provided, the Q6 path is therefore
`legacy_code`.

At this point the interview is complete.

---

## Step 3 — Fork the `.cursor` project branch

New-project flow only. This is the "fork point": every project gets its own branch in the shared
`.cursor` repo, so its config/knowledge-base commits (which you make manually later) never land on the
shared `master`/`main` branch that other projects branch from. Fork it now — **after** the interview and
**before** the initializer — so the whole project setup lives on the project branch from the start.

This is the single git action this command performs, and only after an explicit confirmation.

1. Confirm the `.cursor` working tree is clean: `git -C .cursor status --porcelain`. If it's dirty,
   report it and stop — do not fork a branch on top of uncommitted changes.
2. Determine the current branch: `git -C .cursor branch --show-current` (this is the shared base, e.g.
   `master`).
3. **Ask the user for permission** (use the structured question UI) before branching, e.g.:

   > Fork a dedicated branch `project/<project-name>` in the AI tooling repo (`.cursor`) off
   > `<current-branch>`? All project-specific changes (config, knowledge base, custom skills) will live
   > there instead of on the shared branch. Committing and pushing it stays manual.

   - **Approved** → `git -C .cursor checkout -b project/<project-name>`. If the branch already exists
     locally, check it out instead of failing.
   - **Declined** → warn explicitly that continuing will place any later manual commits on
     `<current-branch>`, which is shared with every other project using this library, and confirm the
     user still wants to proceed on that branch.

4. Report which branch `.cursor` is now on before moving to Step 4. Do **not** push it — pushing is a
   manual step the lead does after committing the generated config.

---

## Step 4 — Run the initializer

New-project flow. Pass all collected answers as flags. Omit flags for skipped optional fields.
The initializer is fully non-interactive when flags are provided.

```bash
node .cursor/.project/scripts/init.mjs \
  --project-name=<value> \
  --atlassian-base-url=<value> \
  --jira-project-key=<value> \
  --confluence-space-key=<value> \
  [--github-repo=<value>] \
  [--project-path=<value>] \
  [--legacy-repo=<value>] \
  [--legacy-branch=<value>] \
  [--sfcc-site-id=<value>] \
  [--sfcc-site-prefix=<value>] \
  [--scapi-client-id=<value>] [--scapi-org-id=<value>] [--scapi-short-code=<value>] \
  [--mrt-project=<value>] [--mrt-target=<value>]
```

Omit `--project-path` (or pass `.`) when the code is at the repo root. There is no legacy **path** flag:
the initializer always clones `--legacy-repo` into `legacy_code/` at the project root (the destination is
fixed). The legacy **path** is recorded separately in the knowledge base (Step 5) as `legacy_code`.

It generates the `.cursor` config (`.env.example`, `.env`, project-specific `README.md`), installs
skill-script and project-context dependencies, and — when the code repo is a git repository — writes
(but does **not** commit) the code-repo config: `.gitignore`, `.cursorignore`, `.cursorindexingignore`,
and the `openspec/` scaffold.

When `--project-path` is set, `openspec/` is copied under that path (e.g. `storefront/openspec`) and the
`.gitignore` entries are anchored there; with no path it sits at the repo root as `openspec/`. When
`--project-path` is set and `GITHUB_REPO` is provided, the initializer will automatically clone the repo
into that subfolder if it doesn't exist yet (SSH first, HTTPS fallback).

When `--legacy-repo` is set, the initializer clones the legacy codebase into `legacy_code/` at the
project root. This folder carries its own `.git` history and is for reference only. If you lack access
to the legacy repo, copy the legacy sources into `legacy_code/` by hand instead: the initializer detects
the existing folder and skips cloning (for a hand-copied folder without `.git` it prints a "not a git
repository — skipping clone" notice, which is expected — leave the folder in place). Either way the agent
picks legacy up from the knowledge-base path.

When `--scapi-client-id`, `--scapi-org-id`, or `--scapi-short-code` are set, the initializer generates
`base-sf-next/.env` by copying `base-sf-next/.env.default` and overwriting the SCAPI and MRT variables
with the provided values. On repair runs the file is regenerated from the project `.env.example`/`.env`
so credentials stay current. Leave all three blank to keep the demo backend (`zzrf-001`) from `.env.default`.

**One-time lead action after first init — `base-sf-next/config.server.ts`:**

The cloned `base-sf-next/` contains a vanilla `config.server.ts` with demo sites (`RefArchGlobal`,
`RefArch`). The lead must replace it with the project's real site configuration before the reference
deployment will work:

1. Edit `base-sf-next/config.server.ts` — replace `commerce.sites`, `defaultSiteId`, and `siteAliasMap`
   with the project's sites (copy from `<project-path>/config.server.ts`).
2. Save the customised file as a template so every future dev gets it automatically:
   ```bash
   mkdir -p .cursor/.project/templates/base-sf-next
   cp base-sf-next/config.server.ts .cursor/.project/templates/base-sf-next/config.server.ts
   ```
3. Commit `.cursor/.project/templates/base-sf-next/config.server.ts` to the `.cursor` project branch.

After this one-time step, `/project:init` (repair / dev-join) applies the template automatically —
no manual editing required for any future teammate.

When `--scapi-client-id`, `--scapi-org-id`, or `--scapi-short-code` are set, the initializer generates
`base-sf-next/.env` by copying `base-sf-next/.env.default` and overwriting the SCAPI and MRT variables
with the provided values. On repair runs the file is regenerated from the project `.env.example`/`.env`
so credentials stay current. Leave all three blank to keep the demo backend (`zzrf-001`) from `.env.default`.

**One-time lead action after first init — `base-sf-next/config.server.ts`:**

The cloned `base-sf-next/` contains a vanilla `config.server.ts` with demo sites (`RefArchGlobal`,
`RefArch`). The lead must replace it with the project's real site configuration before the reference
deployment will work:

1. Edit `base-sf-next/config.server.ts` — replace `commerce.sites`, `defaultSiteId`, and `siteAliasMap`
   with the project's sites (copy from `<project-path>/config.server.ts`).
2. Save the customised file as a template so every future dev gets it automatically:
   ```bash
   mkdir -p .cursor/.project/templates/base-sf-next
   cp base-sf-next/config.server.ts .cursor/.project/templates/base-sf-next/config.server.ts
   ```
3. Commit `.cursor/.project/templates/base-sf-next/config.server.ts` to the `.cursor` project branch.

After this one-time step, `/project:init` (repair / dev-join) applies the template automatically —
no manual editing required for any future teammate.

Two things to surface to the user from the initializer output:

- The OpenSpec `config.yaml` and `schemas/` (under `openspec/`, or `<project-path>/openspec/`) are added
  to `.gitignore` — they hold our tooling IP (schema + artifact prompts) and stay local-only, refreshed
  later by `/project-sync`. Only the client-facing scaffold (`specs/`, `changes/`) is meant to be
  committed.
- If the initializer prints a **`⚠ CODE REPO SKIPPED`** block, the code repo isn't a git repository yet.
  Tell the user to make it a git repo manually (init or clone their project into this folder), then
  re-run this command. Do **not** `git init` for them.

The initializer never commits or pushes. Committing the code-repo config and any `.cursor` config is a
manual step for the lead (the initializer prints the suggested `git add` / `git commit` commands).

---

## Step 5 — Build the project knowledge base (`rules/project-info.mdc`)

New-project flow only. This writes the file from the answers you already collected in Step 2 — do not
ask anything new here. Do not delegate it to the initializer script.

**Fill the template — do not reconstruct the file from chat.** The exact procedure:

1. **Read** the template file `.cursor/.project/templates/rules/project-info.mdc.tmpl` in full.
2. **Replace only the `{{PLACEHOLDER}}` tokens** with the collected/normalized values (mapping below).
   You provide the *values* (normalizing free-text interview answers — e.g. locales → `en-GB`, source
   platform → `SiteGenesis`); the *structure, prose, tables, and headings come from the template
   verbatim*.
3. **Preserve everything else exactly** — do not rewrite sections, drop rows, reorder, or "improve" the
   wording. In particular, leave the discovery placeholders (`<…>`, `customfield_XXXXX`) untouched;
   they're filled later during setup.
4. **Write** the result to `rules/project-info.mdc` in the AI tooling repository.

Why from the template and not from memory: the template is the source of truth for the file's shape and
gets updated over time — copying remembered content from the chat drifts from it and drops sections.

Placeholder sources:

| Placeholder | Source |
|-------------|--------|
| `{{PROJECT_NAME}}` | Step 1 (`--project-name`) |
| `{{PROJECT_GOAL}}` | Step 2 Q1 |
| `{{PROJECT_TYPE}}` | Step 2 Q2 |
| `{{SOURCE_PLATFORM}}` | Step 2 Q3 |
| `{{LOCALES}}` | Step 2 Q4 |
| `{{STOREFRONT_URLS}}` | Step 2 Q5 |
| `{{LEGACY_CODE_PATH}}` | Step 2 Q6 (write `None.` if the answer was `none`; otherwise the path in backticks, e.g. `` `legacy_code` ``) |
| `{{NOTES}}` | Step 2 Q7 (write `None.` if skipped) |
| `{{PROJECT_CODE_PATH}}` | Step 1 (`--project-path`; write `.` if blank/repo root) |
| `{{SFCC_SITE_PREFIX}}` | Step 1 (`--sfcc-site-prefix`; leave blank if not provided) |
| `{{GITHUB_REPO}}`, `{{JIRA_PROJECT_KEY}}`, `{{CONFLUENCE_SPACE_KEY}}`, `{{ATLASSIAN_BASE_URL}}` | Step 1 answers |

Leave no `{{...}}` markers in the final file. Do not invent facts — if a value is genuinely missing,
prefer the placeholder row over guessing.

`project-info.mdc` is the **single** generated project-specific rule; everything else in `rules/` and
`skills/` is generic and ships as-is. Its **Environments** and **Jira Configuration** sections contain
data that is usually discovered later (SFCC hostnames, MRT preview URLs, Jira custom-field IDs,
workflow transitions). Keep the `<…>` / `customfield_XXXXX` placeholder rows as-is and fill them in
during discovery/setup — do not invent values at init.

---

## Step 6 — Prompt the user to fill in credentials

Both flows. Tell the user to open `.cursor/.project/.env` and fill in:

```
ATLASSIAN_EMAIL=their@email.com
ATLASSIAN_API_TOKEN=<token>
```

Token URL: https://id.atlassian.com/manage-profile/security/api-tokens

---

## Final Response

Report:
- Which flow ran (new-project lead init vs dev join).
- Which branch `.cursor` is on, and whether it was forked this session (new project) or left as-is
  (declined / dev join).
- Whether the initializer completed successfully.
- The code repository's state as reported by the initializer: whether ignore files and the `openspec/`
  scaffold were written, and whether the `⚠ CODE REPO SKIPPED` block appeared (code repo not a git repo).
- Whether `base-sf-next/` was cloned from the SF Next template (or already present / skipped with a warning).
- If `base-sf-next/` was **freshly cloned** (new project): remind the lead of the one-time manual step —
  customise `base-sf-next/config.server.ts` with the project's real sites and save it as
  `.cursor/.project/templates/base-sf-next/config.server.ts`, then commit to the `.cursor` project branch.
  Until this is done, the reference MRT deployment will fail with a `validateConfig` site mismatch error.
  If `.cursor/.project/templates/base-sf-next/config.server.ts` already exists, the template was applied
  automatically — no manual action needed.
- That `openspec/config.yaml` and `openspec/schemas/` are gitignored (tooling IP, local-only) and are
  refreshed from the library by `/project-sync`.
- Whether `rules/project-info.mdc` was generated (new project) or skipped (dev join).
- That the user must fill credentials in `.cursor/.project/.env` before using Atlassian tools.
- That the remaining git wiring is manual: connecting the code repo's remote, the first commit of the
  code-repo config and the `.cursor` config, and pushing the project branch are done by the lead by hand.
  Forking the `.cursor` project branch is the only git action this command took.
