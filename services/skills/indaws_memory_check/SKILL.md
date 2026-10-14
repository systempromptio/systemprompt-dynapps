---
name: indaws-memory-check
description: >-
  Mantiene .ai-memory/ al dia con el repo: detecta drift del inventario de modulos, reclama ADRs que faltan y regenera SNAPSHOT.md. Úsala para revisar o sincronizar `.ai-memory/` (drift de inventario, ADRs que faltan: "revisa la memoria", "sincroniza la memoria", "memory check").
version: 0.1.4
effort: low
model: sonnet
allowed-tools: [Bash, Read, Write, Glob, Grep, AskUserQuestion]
---

# Memory Check

Local reconciler for `.ai-memory/`. Compares the repo state with the machine-zone inventory (`modules.yml`, `studio.yml`, `integrations.yml`), regenerates `SNAPSHOT.md`, and prompts for ADRs when new modules land without a decision record. Runs as a dev tool before commit, or as a drift check consumed by other skills (see "Consumed by" at the bottom).

**The skill knows the ai-memory schema. The repo tells it what actually exists.**

## Agent routing

| Model | Actions |
|-------|---------|
| **sonnet** (subagent) | check (dry-run, deterministic) |
| **opus** (principal) | sync (interactive ADR prompts, reasoning) |

## References

| Ref | When to read |
|-----|-------------|
| [workflow.md](references/workflow.md) | Before running `check` or `sync` — full startup sequence, action steps, user messages |
| [detection-rules.md](references/detection-rules.md) | Always before scanning — defines what counts as drift and how to classify each finding |
| [snapshot-format.md](references/snapshot-format.md) | Before regenerating `SNAPSHOT.md` — format, ordering, hash algorithm |
| [adr-template.md](references/adr-template.md) | When sync detects a new module without ADR and the user accepts creating one |
| [gotchas.md](references/gotchas.md) | Before any write — known pitfalls (manifest eval, YAML round-trip, path variants) |

See `common:references/shared/` for shared ecosystem patterns.

## Startup sequence (summary)

- **Verify `.ai-memory/` exists.** Missing → exit 3 (non-fatal). Read `.schema-version`, validate `version: 1` + `layout: v2`. Malformed → exit 2.
- **Detect custom modules path.** Default `parts/custom/`, with fallbacks `custom/`, `addons/custom/`, `src/`. None found → treat list as empty.
- **Determine mode.** `--check` (dry-run), `--sync` (apply), or prompt. Flags: `--no-stage`, `--no-adr-prompt`, `--yes`, `--verbose`, `--json` (only with `check`, machine-readable output for programmatic consumers — see `workflow.md` for the full contract).

Full details (prompts, error messages, fallback order rationale) in [workflow.md](references/workflow.md).

## Actions

| Action | Model | Purpose |
|--------|-------|---------|
| `check` | sonnet | Dry-run drift report. Reads filesystem + `modules.yml`, diffs, checks ADRs, verifies SNAPSHOT hash. Never writes. |
| `sync`  | opus   | Applies drift corrections to `modules.yml`, regenerates `SNAPSHOT.md`, prompts for ADRs on new modules (interactive only), optionally stages. |

**Implementation**: both actions are implemented by `scripts/reconcile.py {check|sync}`. It orchestrates `scan_modules.py` (filesystem → modules) and `snapshot_gen.py` (YAML → SNAPSHOT.md + hash). Full action steps in [workflow.md](references/workflow.md).

## Exit code contract

`git-manager` and `quality-gate` rely on this contract:

| Code | Meaning | git-manager reaction |
|------|---------|---------------------|
| `0` | OK, no drift | Silent, continue |
| `1` | Drift detected (check) or applied (sync) | Surface as WARNING in `review`, suggest running sync in `commit` |
| `2` | Error (malformed YAML, bad schema version) | Surface as CRITICAL, block the review recommendation |
| `3` | No `.ai-memory/` folder | Silent, skip all memory-related checks |

## Guardrails

Non-negotiable invariants — never bypass:

- **Never delete files** other than entries inside `modules.yml`. Do NOT delete ADRs, knowledge/, CONTEXT.md, or the folder itself.
- **Never modify `knowledge/`** — human narrative zone, read-only from this skill.
- **Never edit existing ADR content** in `decisions/` — append new ones only. ADRs are immutable (use `supersedes:` in a new file to replace).
- **Never touch `CONTEXT.md`** — maintained manually by the team.
- **Never initialize or create `.ai-memory/`** — that is the skeleton's job. Exit 3 if missing.
- **Never invent data** — if a manifest lacks a field (e.g., no `version`), record `null` rather than guessing.
- **Never stage outside `.ai-memory/`** — only `git add .ai-memory/inventory/` and `.ai-memory/decisions/`.
- **Never auto-create ADRs** — not under `--yes`, not in non-interactive mode. ADRs are human-authored; missing ones are logged as warnings.

## Gotchas

Top failure patterns. Full list and mitigations in [gotchas.md](references/gotchas.md).

- **Manifest parsing**: use `ast.literal_eval`, never `exec`/`eval`. Missing `depends` → empty list (not a diff). Missing `version` → `null` (do not invent `"1.0.0"`).
- **YAML round-trip**: comments in `modules.yml` are lost on write (expected). Preserve empty lists as `[]`, not `null`.
- **Hash determinism**: normalize `\r\n` → `\n` and strip UTF-8 BOM before hashing, or Windows and Linux produce different hashes.
- **ADR matching**: use word-boundary grep (`\bacme_sale\b`) so `acme_sale` does not match `acme_sale_approval`. Exclude `decisions/README.md` from the scan.
- **Empty scan**: if `modules.path` resolves to an empty directory, the diff reports "all modules removed". Require `--force` in that edge case.

## Delegates to

- None directly. Scripts are self-contained (Python stdlib + PyYAML).

## Consumed by

- `git-manager` (ai-claude-developer) — action `commit` runs `memory-check --check` as a non-blocking warning gate
- `git-manager` (ai-claude-developer) — action `precheck` delegates ai-memory drift detection on the local diff (pre-MR, for the developer)
- `git-manager` (ai-claude-developer) — action `review` delegates ai-memory drift detection on an open MR diff (for the reviewer)
- `git-precheck` (ai-claude-developer) — invokes this skill (`check --json`) in step 4 to detect ai-memory drift on the local diff before opening the MR
- `quality-gate` (ai-claude-common) — invokes this skill as one of the pre-merge checks (future)
- `memory-updater` agent (future, external) — runs `sync` periodically on behalf of the team
