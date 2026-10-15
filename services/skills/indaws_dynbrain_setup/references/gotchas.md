# Gotchas — scheduling Dyn Brain

Read before reporting a setup as done, and before reading a scheduled run's summary as
truth. Every item below was measured on a real run; the date says which.

## Registration

- **Writing a `SKILL.md` into `~/.claude/scheduled-tasks/` registers nothing.** The registry
  is inside Claude Desktop. Two folders created that way in March 2026 (`odoo-mentor`,
  `org-chart-weekly`) still sit there and have never appeared in the task list. Only
  `create_scheduled_task` registers. (2026-09-11)
- **A task inherits the working folder of the session that created it.** Every run then
  starts there, with that folder's `CLAUDE.md` and memory hook. Run the setup from a neutral
  folder, or accept that the runs will boot with an unrelated project's context. (2026-09-11)
- **The app adds a few minutes of jitter** to every cron. "Friday 12:30" runs at 12:37. Do not
  chase the difference.
- **Tasks run only while the app is open and the Mac is awake.** A missed slot runs on the
  next launch; a skipped Friday is usually sleep, not failure, and it leaves no trace.

## Who installs which cadence

- **The sweep and the daily pass are corpus-wide, not per-person.** Monday's sweep walks every
  novelty published last week and opens a `mail.activity` for **every** affected owner,
  whoever ran it. So an article owner needs no install to be notified — and two people who
  both install it sweep the same novelties on the same Monday. The notice key
  (`novelty_id:owner`, `group_notices.py`) is computed **inside a run**: nothing in the skill
  checks for an activity another session already opened. **Not yet measured** — Víctor is the
  only steward so far — so before adding a second one, either scope the sweep by territory or
  make the create step check for an open activity with the same key first. The failure mode is
  an owner getting the same notice N times and filing the lot under noise. (design review
  2026-09-21)
- **`cadences` absent means all three.** Configs written before 1.1.0 keep behaving exactly as
  they did. Do not "fix" one by adding `cadences` unless the person's role actually narrowed.
- **A `steward`-only install does not need Memory OS.** Only the harvest reads the worklog. The
  render stopped demanding `memory_os_vault` for the other two in 1.1.0; if an older setup asked
  someone for a vault they do not have, that is why.

## Permissions in an unattended run

- **Don't-ask mode denies, it does not prompt.** Any tool call not covered by
  `permissions.allow` returns a denial the model reads as text. The run does not fail — it
  reports a clean result with a hole in it. This is why `patch_permissions.py` exists.
- **The skill's own files are outside the working folder.** The desktop app unpacks plugins
  under `~/Library/Application Support/Claude/local-agent-mode-sessions/<…>/rpm/<plugin>/`;
  the CLI under `~/.claude/plugins/`. The first harvest could not Read
  `what-is-a-decision.md`, `sources.md` or `decision-record.md`, and filtered from the
  SKILL.md summary alone. Both homes are now in the allowlist, and the second run of the day
  **confirmed that a `**` in the middle of a Read rule matches**: it read its three references
  and the worklog without a single denial. `patch_permissions.py` adds the concrete plugin
  folders as well, so nothing rests on that one behaviour. (2026-09-11)
- **A redirection, a heredoc, a variable prefix or a `;` chain makes a Bash call deniable
  whatever its first word.** Third run of the day: `find "/Users/…" -type d -name x 2>/dev/null`
  denied, `python3 - <<'PY'` denied, `BASE="…"; ls "$BASE" | head` denied — while
  `ls -la "/Users/…"` and `python3 -c "…"` with the same literal paths went through. The
  allowlist matches the first word; the permission engine still treats redirections, heredocs
  and compounds as needing approval. Step 0 of every prompt now says: one plain command,
  literal paths, `python3 -c` or `python3 script.py`. (2026-09-11)
- **A Bash command carrying `$HOME` is denied even when `find` is allowed.** Two `find
  "$HOME/…"` calls were denied in the second run while `find "/Users/…"` with a literal path
  went through, and so did `python3 "/Users/…"` and `osascript`. The templates now carry the
  resolved home. Escaped spaces (`Application\ Support`) were denied too. (2026-09-11)
- **One denied `cat` became "Bash is denied".** The run tried `cat ~/.dynbrain/config.json`,
  got a denial, and from then on never tried `python3` or `osascript` — both allowed. So no
  `render_harvest.py` (the page was hand-written) and **no desktop notification**. Step 0 of
  every prompt now names what is pre-approved and forbids that inference. (2026-09-11)
- **`Read(//Users/x/dir/**)` is the spelling.** Two slashes, then the path without its leading
  one. `///Users` matches nothing, silently. The first version of `patch_permissions.py` got
  this wrong and its tests encoded the mistake. (2026-09-11)
- **The allowlist is read at session start.** The session that ran `patch_permissions.py`
  does not see the new entries; the next run does.
- **"Invalid or missing session" can outlive a reconnect.** After the ORM connector and the local
  `dynapps-docs` server both failed with it, the user re-authorised in claude.ai; this session
  still got the same error from both while the claude.ai `Dynbrain` connector to the same
  backend worked. `session_connectors_status` showed every one of them "connected": the error
  is the backend's Odoo session, not the MCP transport, and `reconnect_session_connector`
  refuses a server that is not "failed". A fresh session is the test. (2026-09-11)
- **The Odoo ORM proxy session can expire mid-run.** The harvest lost it right after the
  idempotency check and ran no scope estimate. A `whoami` re-establishes it — unless the
  connector's authorisation itself is gone, in which case `whoami` fails the same way and only
  re-authorising the connector in claude.ai helps. The prompts say: retry once, and if `whoami`
  fails too, treat the proxy as absent and say so. (2026-09-11)
- **When a source is denied the model improvises a route to it.** Denied the vault by Read,
  the harvest listed the same files through the OneDrive connector and asked for their
  content — also denied, harmlessly. Grant the direct route and there is nothing to
  improvise.

## Notifications

- **`osascript` notifications belong to "Script Editor"** in System Settings › Notifications.
  The first one on a Mac triggers a permission prompt nobody sees in a scheduled run; fire
  `notify.sh` once during setup, with the person at the keyboard.
- **The task's own completion notice (`notifyOnCompletion`) is the second channel.** It lands
  in the app, not on the desktop, and it fires even when the run could not `osascript`.
- **A daily task that notifies on quiet days trains people to ignore it.** The canon pass
  notifies only on transitions, escalations, problems or failure to run.

## The harvest specifically

- **Running it twice in one day proposes the same candidates twice.** Idempotency is checked
  against what is *published* under `01 · Novedades`, not against the page already written
  under `~/.dynbrain/out/harvest/<week>/`. A "Run now" test on a Friday morning is followed by
  the real 12:30 run. Fixed in harvest 2.1.0: an existing `cosecha.json` for the week makes the
  run a refresh that keeps the refs. (2026-09-11)
- **There are two `01 · Novedades` nodes, and they are two different systems.** 4026 is the
  native node of the Dyn Brain tree (created 2026-08-08, parent 4024, `can_write`, not indexed,
  `canon_*` all null). 4103 is a **Git-synced** index from the `dynapps-es/doc-internal`
  repository (`01-novedades/index.md`, author David Gómez, `truth_source: decided`, indexed):
  a decision register built in the docs repo in parallel to Dyn Brain's. The harvest creates
  native articles through the ORM, so it publishes under **4026**; anything under 4103 goes
  through Git. Both empty today, so nothing broke; the day one gets an article, the harvest
  and the sweep must agree on which — and somebody has to decide whether the two registers
  merge. (2026-09-11, read through `docs_metadata`) `novelties_root_article_id` in `~/.dynbrain/config.json`
  pins it; the setup resolves it through the ORM proxy (step 1) and both prompts carry it.
- **`render_harvest.py` wants `domain`, the skill says `territory`.** The JSON schema in the
  script header used the word the vocabulary section forbids. Fixed in harvest 2.1.0: the
  renderer takes `territory`, accepts `domain`, validates the whole file and prints its schema
  with `--schema`.
- **The confirmation page's buttons did nothing.** A static HTML cannot reach Claude. Since
  harvest 2.1.0 the page is read-only with a reference per candidate, and the confirmation is
  the `AskUserQuestion` menu in the owner's session — never in the unattended run, where it
  would hang waiting for nobody.
