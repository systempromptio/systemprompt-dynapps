---
name: openspec-onboarding
description: Use when someone new to this project asks how OpenSpec and the skills/operate workflow work, why we use them, what the pipeline / roles / approval gates are, where to start on a Jira ticket, or wants a guided walkthrough or hands-on practice. Triggers on "how does OpenSpec work", "explain the pipeline", "where do I start", onboarding, and getting-started questions.
---

# OpenSpec Onboarding

Orientation for anyone new to this project's OpenSpec pipeline and the `skills/operate` layer: the mental model, the roles, the human gates, and where to start — then a guided deepen-or-practice loop. This skill teaches the *map*; it never carries phase procedures. **The `/opsx:*` commands and `openspec-workflow` rule are the source of truth — never run a phase from memory.**

## How to run an onboarding session

When this skill is invoked, coach — do not dump the whole file:

```
- [ ] 1. Ask who is onboarding (BA/SA, developer, QA) and how familiar they are.
- [ ] 2. Give a short, tailored conceptual overview (mental model + the one or two
         phases their role owns). A few sentences — not a wall of text.
- [ ] 3. Offer, via AskQuestion, to either GO DEEPER into one area or PRACTICE together.
- [ ] 4. Deepen or run the practice loop below, then re-offer step 3 until they're set.
```

## Mental model

Spec-driven development: before code, a change moves through **artifacts**, each **human-reviewed and approved** before the next is generated. Goal — kill requirement ambiguity, capture decisions with an explicit human sign-off, and avoid bureaucracy where it isn't needed.

Two data zones:

- **`openspec/specs/`** — the timeless baseline: describes what the system does *now*. No "Phase N", no change narrative, no forward-references.
- **`openspec/changes/<JIRA-KEY>/`** — one folder per Jira ticket, holding the change's artifacts + delta specs. On `archive`, deltas merge into the baseline **verbatim** (matched by capability folder + exact requirement header).

> This repo *is* the tooling library shipped as a project's `.cursor/`. The live `openspec/`, Confluence, and Jira exist only on a **deployed** project (see the `repo-context` rule).

## Pipeline map

`feature` schema (each arrow is a human review gate — the agent stops, summarizes, and waits):

```
discovery → design → proposal → grooming ─[grooming approval gate]→ specs → (optional fsd | isd)
  → [propose-submit: commit + review PR + publish FSD/ISD to Confluence]
  → [proposal approved]
  → [propose-approved: script stories.md + create Jira stories (+Epic)]
  → technical-design → tasks → apply → verify → archive
```

`bugfix` is shorter: `proposal → specs → tasks → apply → archive` (no design).

| Artifact | Owner | In one line |
|---|---|---|
| discovery | BA/SA | RQ/WBS trace from BRD/SRD, reconciliation, open questions, FSD-vs-ISD scope |
| design | SA | architecture / solution shape (depth scales with the change) |
| proposal | BA/SA | Requirements Lock + Delivery Breakdown (story slices + AC seeds) + deps/access |
| grooming | humans decide | approval gate — agent prepares Q/options/recommendations, named humans approve |
| specs | BA/SA | strict behaviour contract (RFC 2119 + Given/When/Then) |
| fsd / isd | BA/SA | client (FSD) or engineering (ISD) doc; exactly one type per change |
| technical-design | dev/SA | developer HOW — file maps, wiring, test design |
| tasks | dev | parseable checkbox queue with `@BE`/`@FE`/`@SHARED` + dependency tags |
| stories.md | *scripted* | NOT authored by the agent — a deterministic projection at `propose-approved` |

## Commands (source of truth)

| Command | Purpose |
|---|---|
| `/opsx:explore` | browse existing specs/changes, think, clarify requirements |
| `/opsx:propose` | start a change: discovery→grooming, then specs + optional FSD/ISD (never authors stories) |
| `/opsx:propose-submit` | commit artifacts, open review PR, publish FSD/ISD to Confluence (offer→preview→yes) |
| `/opsx:propose-approved` | script `stories.md`, create Jira stories (+Epic), backfill keys/links, push |
| `/opsx:plan` | Dev Planning: `technical-design.md` then `tasks.md` |
| `/opsx:apply` | implement from `tasks.md` (BE/FE/FULL scope + dependency gates) |
| `/opsx:archive` | merge deltas into the baseline, archive the change |
| `/project-sync` | refresh tooling + project-context; mandatory preflight first step of every `/opsx:*` command |

Routing lives in the `openspec-workflow` skill; cross-cutting orchestration (change sizing, the per-artifact review gate, the post-grooming consistency pass) in the `openspec-workflow` rule; per-artifact authoring detail, spec hygiene, and generation gates in the schema (`schema.yaml` instructions + `config.yaml`), pulled in when the command runs.

## The non-negotiables

- **Never self-approve.** Every gate needs a named human (`Agent`/`LLM`/`Cursor` is invalid). Grooming, Planning, and the per-artifact review gate all stop for a real person.
- **Commands are the source of truth.** Skills orient and route; they never carry phase procedures, templates, or gates.
- **`specs/` is behaviour authority.** FSD/ISD and Jira stories are handoff/publication, not truth.
- **Size the change first.** Quick fix (skip OpenSpec) → `--schema bugfix` (lean) → `--schema feature`/`bugfix` → full `feature` pipeline + grooming. Bias one size smaller than instinct.
- **External systems = offer → preview → execute.** Confluence/Jira are mutated by the agent only after an explicit yes + dry-run preview.

## Where skills/operate fits

`skills/operate` is the pipeline's hands into external systems — the hand-off points out of `opsx-*`:

- `/opsx:propose-submit` → **`operate/atlassian`** `publish.mjs` publishes the FSD/ISD and stamps `- Confluence:` / `- Page ID:` back into the working copy. The doc's chrome/canon (Document header card, approvals, references, title suffix) is owned by **`operate/atlassian-doc-templating`**.
- `/opsx:propose-approved` → **`operate/atlassian`** `jira.mjs create-stories` reads `stories.md`, creates/reuses issues by idempotency hash, and backfills keys + `- Jira:` links.
- The rest of `operate/*` (`b2c-sandbox`, `b2c-logs`, `b2c-job`, `b2c-config`, …) are runtime SFCC/MRT operations — the "operate" bucket, but not part of the OpenSpec change flow.

Credentials come from `.project/.env`; Jira/Confluence go through scripts (no MCP).

## Practice loop (hands-on)

When the user picks PRACTICE, run a **safe dry walkthrough** — teach by doing without mutating anything:

```
- [ ] Pick a real or hypothetical Jira ticket as the change id.
- [ ] Walk ONE phase end to end (usually discovery or the grooming gate): explain
      its purpose, show the artifact skeleton, and produce a tiny grounded example.
- [ ] At each external step, demonstrate offer → preview (dry run) → (stop; no execute).
- [ ] Point to the exact command + rule section that owns what they just saw.
- [ ] Ask what to try next: the next phase, a different role's view, or a real change.
```

Keep it low-stakes: no commits, no Confluence/Jira writes, no self-approval during practice.

## Common mistakes

- Treating this skill (or any skill) as the procedure — always defer to `/opsx:*` and the rule.
- Chaining phases without stopping at the review gate.
- Authoring `stories.md` by hand instead of fixing upstream and re-running the script.
- Running the full pipeline on a quick fix.
- Publishing to Confluence/Jira without offer → preview → yes.

## Pointers

- Routing: `openspec-workflow` skill · Orchestration (sizing, review gate, consistency pass): `openspec-workflow` rule · Authoring detail + gates: the artifact schema
- Commands: `/opsx:explore` `/opsx:propose` `/opsx:propose-submit` `/opsx:propose-approved` `/opsx:plan` `/opsx:apply` `/opsx:archive`
- Operate: `operate/atlassian` · `operate/atlassian-doc-templating`
- Doc lifecycle & publishing: the `fsd`/`isd` schema instruction + `/opsx:propose-submit` · AC authoring: the `proposal` artifact schema
- Repo layout / source-vs-deployed: `repo-context` rule
