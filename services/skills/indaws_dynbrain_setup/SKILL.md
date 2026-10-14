---
name: indaws-dynbrain-setup
version: 1.1.0
effort: low
license: Interno Dynapps
argument-hint: "[install | status | disable] [--role author|steward|both]"
description: >-
  One command that puts Dyn Brain's scheduled tasks on a person's Mac: the Friday novelty
  harvest, the Monday sweep and the daily canonical pass, each ending with a desktop
  notification. Which ones a person gets follows their role: the harvest is per-person, the
  sweep and the daily pass per-corpus. Renders the prompts from the person's config, registers
  them through the Claude Desktop scheduled-tasks tool, adds the allowlist an unattended run
  needs, and proves the notification channel works. Use it when someone says "configura Dyn
  Brain", "instala las tareas de Dyn Brain", "dynbrain setup", "programa la cosecha", or asks
  why a Dyn Brain task is not running. NOT the tasks themselves — those are
  dynbrain-decision-harvest, dynbrain-weekly-queue and dynbrain-canon.
---

# Dyn Brain — setup of the scheduled tasks

Dyn Brain only maintains documentation if three things run on their own:

```
FRIDAY   dynbrain-decision-harvest  → candidates on a page, the owner confirms   (publishes nothing)
MONDAY   dynbrain-weekly-queue      → sweeps, triages, one activity per owner
DAILY    dynbrain-canon             → deadlines, escalation, the problem report
```

This skill installs them as Claude Desktop scheduled tasks for **one person**, in their own
session and with their own connectors. It changes nothing in Odoo.

## Who installs what

Not everyone installs all three, and the line is **not** "who edits articles". An article
owner receives the Monday notice as a `mail.activity` opened by whoever ran the sweep — they
install nothing at all to be told their document went stale. The line is **per-person vs
per-corpus**:

| Role | Cadences | Who |
| --- | --- | --- |
| `author` | harvest | Anyone whose week produces decisions **and** who has Memory OS. The harvest reads *their* week and only ever runs in their own session. Install widely. |
| `steward` | canon, queue | **One machine**, or one per territory. Both passes are corpus-wide: the sweep walks every novelty published last week and notifies every affected owner. |
| `both` | all three | Today: Víctor. |

Two `steward` installs mean two sessions sweeping the same novelties on the same Monday. The
notice key is `novelty_id:owner`, computed inside a run — it does not protect against a second
run elsewhere. Before adding a second steward, read `references/gotchas.md`.

Ask for the role if the person did not say one, and record it as `cadences` in their config.
A config written before this existed has no `cadences` and installs all three, as it did.

## Language

Write everything **in English** inside this skill and its templates — the convention of this
group repository. The task titles, the summaries a run writes and the notifications go in the
**person's language** (`language` in the config). When you talk to the user, reply in the
language they wrote in.

## What only the tool can do

The registry of scheduled tasks lives inside Claude Desktop. Writing a `SKILL.md` into
`~/.claude/scheduled-tasks/` registers **nothing**: the two folders that were created that
way in March 2026 never appeared in the task list. So the split is:

- `scripts/render_tasks.py` — deterministic: config + templates → three task specs.
- **You** — hand each spec to `create_scheduled_task` (or `update_scheduled_task`).
- `scripts/patch_permissions.py` — the allowlist, merged idempotently into `settings.json`.
- `scripts/notify.sh` — one test notification, so the channel is proven before Friday.

Identify the scheduled-tasks tool by its family (`list_scheduled_tasks`,
`create_scheduled_task`, `update_scheduled_task`), never by a server name. **If it is not
available** — a terminal session, a cloud session — say so, render the specs with `--out` so
the person can create the tasks from the app's Routines panel, and stop. Do not pretend.

## Process — `install` (the default)

### 0. The working folder

A task inherits the working folder of the session that creates it, and every run then boots
with that folder's `CLAUDE.md` and memory hook. Before registering anything, be in
`~/.dynbrain`: if a `change_directory` tool is available, create the folder and switch to it;
otherwise ask the person to reopen Claude there and run the setup again. Do not register the
tasks from a project folder — the first install did, and every Dyn Brain run since boots
inside an unrelated project.

### 1. The person's config

`~/.dynbrain/config.json`. If it exists, read it back to the user in five lines and ask
whether it still holds. If not, build it from `config.example.json` and **ask** for what you
cannot derive:

| Field | How to fill it |
| --- | --- |
| `owner.name`, `owner.odoo_uid` | `odoo_whoami` / `whoami` on the ORM proxy if present; otherwise ask |
| `territory.key` | one of `consultoria`, `desarrollo`, `producto`, `comercial`, `administracion`. Ask; the name fills itself |
| `language` | the language the user is writing in |
| `memory_os_vault` | **only needed for `harvest`.** The Memory OS folder: try `find ~/Library/CloudStorage -maxdepth 3 -type d -name MemoryOS`; confirm. If the person has no Memory OS, say the harvest will run degraded and point them to `memory:setup`. A `steward`-only install does not ask for it |
| `servers.*` | the MCP server names **as this session sees them**: the part between `mcp__` and `__` for the server exposing `docs_*` (docs), `orm_search_read` (orm), `list_meetings` (granola), `d365_*` (m365). A family the person does not have stays an empty list |
| `cadences` | from the role — `["harvest"]`, `["canon","queue"]` or all three. Ask (see **Who installs what**); never assume `both` because you are configuring Víctor's successor |
| `schedule.*` | keep the defaults unless asked: daily 08:00, Monday 08:30, Friday 12:30, local time. Only the cadences being installed are validated |
| `novelties_root_article_id` | `orm_search_read` on `knowledge.article` with `name = "01 · Novedades"`, fields `id`, `parent_id`. One hit → pin it. Several (production has two: 4026 under `Dyn Brain`, 4103 under `[ES] Internal`) → ask which; the one under the `Dyn Brain` root is the tree's. No ORM proxy → leave null and say the runs will resolve by name and stop if ambiguous |

Write the file. Never put credentials in it — it holds names and paths, nothing else.

### 2. Render

```bash
python3 scripts/render_tasks.py ~/.dynbrain/config.json --out ~/.dynbrain/rendered
```

It renders exactly the cadences the config selects; `--role author|steward|both` overrides
them for a one-off. Config errors come back all at once on stderr with exit 2: fix the config,
do not patch the output. Read each rendered prompt once — it is what will run unattended.

### 3. Register

For each spec **that was rendered** — never for a cadence this person did not take —
check `list_scheduled_tasks` first:

- **Not there** → `create_scheduled_task` with `taskId`, `title`, `description`,
  `cronExpression`, `prompt`, `notifyOnCompletion: true`.
- **Already there** → `update_scheduled_task` with the new `prompt`, `cronExpression`, `title`,
  `description` and `enabled: true`. Same id, same URL in the app, no duplicates.

Cron is evaluated in the person's **local** time. The app adds a few minutes of jitter.

### 4. Permissions

```bash
python3 scripts/patch_permissions.py --config ~/.dynbrain/config.json --dry-run
python3 scripts/patch_permissions.py --config ~/.dynbrain/config.json
```

Show the dry run, then apply. It writes a timestamped backup first and never removes an
entry. The list is least privilege: reads plus the exact writes the skills make (`canon_*`
properties, `mail.activity`, chatter notes). **Never** add `orm_unlink`, `orm_send_mail`,
`d365_send_*` or `docs_create` — the harvest publishes only after a person confirms, in an
interactive session that asks as usual.

New sessions pick the allowlist up; the current one does not.

### 5. Prove the notification

```bash
bash scripts/notify.sh "Dyn Brain" "setup" "Three tasks scheduled. This is what a run's last step looks like."
```

If nothing shows: System Settings › Notifications › **Script Editor** must be allowed. The
first `osascript` notification on a Mac triggers that prompt; the person has to answer it
once. Without it every task will still run and still write its summary — only the
notification is lost.

### 6. Report

One table: the tasks installed, their schedule, and `nextRunAt` from `list_scheduled_tasks`.
Say which cadences the person did **not** take and why, so nobody later reads a missing task
as a broken install.
Then the three facts every person needs to hear once:

- Tasks run **only while Claude Desktop is open and the Mac is awake**. A missed slot runs on
  the next launch; a skipped Friday is usually sleep, not failure.
- The harvest **never publishes**. The notification says how many candidates wait; the person
  opens that run from the task's runs and answers per candidate.
- The daily pass notifies **only when something happened**. Silence on a weekday is normal.
- Tool approvals granted during a run are stored on the task. If the person's permission mode
  is not `dontAsk`, tell them to click **Run now** on each task once, while at the keyboard, so
  the first real run does not stall on a prompt nobody sees.

## `status`

`list_scheduled_tasks` filtered to the ids this person installed (`cadences` in the config), plus `list_task_runs` for each: last run,
status, one-line summary. Then `patch_permissions.py --dry-run`: anything it would add is a
permission a run has been silently missing. Then read the last run's transcript
(`list_events` on its session) and grep it for `has been denied`: every hit is a step that
silently did not happen, and the run's own summary will not always say so. Report all three.

## `disable`

`update_scheduled_task` with `enabled: false` for the ids this person installed. Never delete from here —
deletion is the person's click in the app. Permissions stay: they are harmless and removing
them is not this skill's business.

## Gotchas

`references/gotchas.md` — every failure measured on a real run, with its date. **Read it before
reporting a setup as done**, and before taking a scheduled run's summary at face value.

## Scripts

| Script | What it owns |
| --- | --- |
| `render_tasks.py` | Config defaults and validation, template substitution, the three specs |
| `patch_permissions.py` | The least-privilege allowlist and its idempotent merge |
| `notify.sh` | The one-line macOS notification, quoted safely |

Tests are in `tests/test_render_tasks.py` and `tests/test_patch_permissions.py` at the
repository root.

## Delegates to

`dynbrain-decision-harvest`, `dynbrain-weekly-queue`, `dynbrain-canon` — the tasks
themselves. This skill only schedules them.
