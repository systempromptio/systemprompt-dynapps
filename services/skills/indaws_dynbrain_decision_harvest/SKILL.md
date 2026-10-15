---
name: indaws-dynbrain-decision-harvest
version: 2.1.0
effort: medium
model: opus
license: Interno Dynapps
description: >-
  Weekly novelty harvest for Dyn Brain. Walks what happened this week — the Memory OS worklog,
  meeting transcripts, mail and Teams — and finds the novelties that change how Dynapps works:
  what we sell, what we recommend, which procedure we follow. Drafts each candidate as a record
  ready to publish, with its scope and the terms it invalidates, and presents it to the owner on
  a confirmation page. It never publishes on its own. Runs on Fridays; whatever is confirmed
  feeds Monday's review queue. Use it when the user asks for: "cosecha de decisiones", "what did
  we decide this week", "decisions of the week", "review decisions", "harvest", "weekly decision
  summary", or when the Friday scheduled task fires.
---

# Weekly novelty harvest

<objective>
That no novelty which changes how we work ends up living only in the head of whoever decided it.
Every Friday, half a dozen well-drafted candidates and an owner who confirms or discards them in
five minutes.
</objective>

## Language

Write everything **in English** inside this skill and its references, example sentences
included — the convention of this group repository. When you **talk to the user, reply in the
language they wrote in**, and the confirmation page goes in the owner's language.

## Vocabulary

Fixed across the three Dyn Brain skills, so that reports and sweeps line up:

- **territory**, never `domain` — `domain` already means something else in Odoo.
- **novelty** is the genus; **decision** is one of its six types (`decision`, `external_source`,
  `internal_doc`, `error_report`, `end_of_life`, `people_change`). `decision` is by far the most
  common and the one this skill is tuned for, but the record admits all six.
- The node is **`01 · Novedades`**.

## Why this skill exists

Dyn Brain propagates change from registered novelties. If nobody registers, there is nothing to
propagate and the system retrieves information without maintaining it — which is exactly the
failure that motivated it: *"we stop proposing Studio"* was decided in a conversation, never
reached Sales, and proposals went out to customers with Studio in them for weeks.

**Asking people to register decisions does not work on its own.** Nobody walks out of a meeting
thinking "I have just taken a doctrinal decision". This skill inverts the burden: the AI
proposes, the person confirms.

## How it fits with the rest

```
FRIDAY  ──►  Novelty harvest   (this skill)   ──►  the owner confirms
                                                       │
                                                       ▼
                                          published under 01 · Novedades
                                                       │
MONDAY  ──►  Review queue  (dynbrain-weekly-queue)  ◄──┘
             sweeps the corpus and notifies the affected owners
```

Confirming on Friday and sweeping on Monday is not accidental: it gives the weekend as a buffer
and gets the novelty to the people affected before they start working from it.

## Prerequisite: Memory OS

**This skill assumes the territory owner has the Memory OS active.** Not a whim:

- The **worklog** is the only source that already comes distilled and scoped to work. Without
  it the harvest depends on reading raw mail and transcripts, and precision drops a lot.
- Confirmed novelties are written back into the owner's memory (`memory/decisions.md`). **That
  is the real prize**: from then on, any Claude session of that person already knows the current
  doctrine and stops proposing what we stopped proposing. The novelty is not just documented —
  it starts applying itself.

If it is not active → **stop and ask for `memory:setup`** before continuing. You can run a
degraded harvest from meetings, mail and Teams alone, but say so explicitly on the page: the
quality will be lower.

## Privacy — not negotiable

- This skill **only ever runs in the owner's own session**, with their credentials. Never read
  another person's mail, chats or memory.
- **Work scope only.** Nothing personal enters the harvest, not even as evidence.
- The evidence quoted on the page is **the minimum sentence** that justifies the candidate, with
  its source and date. Never dump a whole thread or a full transcript.
- If a candidate rests on something affecting a specific person (performance, salary, hiring),
  **discard it silently**. That is not doctrine and it does not go to Dyn Brain.

<process>

### Step 1 — Window, territory and state

Default window: **Monday to Friday of this week**. Determine the user's territory from
`~/.dynbrain/config.json` (written by `dynbrain-setup`), falling back to a `config.json` beside
this skill; if they have several, ask which, or do them all.

**Where `01 · Novedades` is:** `novelties_root_article_id` in the config. If it is null, resolve
the node by name — and if more than one article carries that name (production has two: 4026
under `Dyn Brain`, 4103 under `[ES] Internal`), **ask which** in an interactive session, or in an
unattended one check idempotency against all of them, publish nothing, and say so.

**Idempotency, twice:**

1. Against Odoo: check under `01 · Novedades` whether novelties are already registered in this
   window and who registered them. Do not re-propose what is already published.
2. Against this week's own page: if `<output_dir>/harvest/<ISO week>/cosecha.json` already
   exists, this is a **refresh**, not a new harvest. Read it, keep every candidate and its `ref`
   (people have read those numbers), add only what is new, and say on the page that it was
   refreshed. A "Run now" test on Friday morning followed by the real 12:30 run must not produce
   the same eight candidates twice.

**If the Odoo ORM proxy answers with a session or authentication error mid-run**, call
`whoami` / `odoo_whoami` to re-establish it and retry the call **once**. Then report the retry.
If `whoami` itself fails with the same error, the connector's authorisation is gone for the whole
session and no retry will help: say so, name the source as missing, and go on. It expired
mid-harvest on 2026-09-11 and took the scope estimate with it; twenty minutes later `whoami`
failed too, and only re-authorising the connector in claude.ai brought it back.

### Step 2 — Gather the four sources

Read [references/sources.md](references/sources.md) for the concrete calls and windows. Gather in
this order — most distilled to noisiest:

| Source | What it gives | Signal |
|---|---|---|
| **Memory OS worklog** | What the owner did and decided, already summarised | The best |
| **Meeting transcripts** (Granola) | Where decisions actually get taken | Very good |
| **Teams** — channels and chats | Quick decisions, "let's just do X" | Medium, noisy |
| **Mail** | Confirmations and formal communication | Medium |

Do not blend them: keep **where each candidate came from**. Provenance is what lets the owner
confirm in seconds.

### Step 3 — Filter: what is a novelty and what is not

Read [references/what-is-a-decision.md](references/what-is-a-decision.md) **before classifying
anything**. It is the hard part of this skill and what decides whether the page gets read or
ignored.

The filter in short: a Dyn Brain novelty **changes what the company will do by default from now
on**. It is not an opinion, nor an agreement about one customer, nor a task.

Discard generously. **Five good candidates beat twenty mediocre ones**: one noisy week and the
owner stops opening the page for good.

### Step 4 — Draft the record

Read [references/decision-record.md](references/decision-record.md).

Produce the full record for each candidate. The field that matters most is `invalidates_terms`:
it is what lets Monday's sweep find the affected documents. If you cannot say which terms it
invalidates, **the novelty is not ready** — drop it to "needs refining" and ask the owner to
complete it.

### Step 5 — Estimate the scope

For each candidate, run a quick sweep with its terms and count **how many documents it could
affect**, separating contradictions from historical or technical mentions. It is an approximate
number and it is presented as such.

It serves two purposes: the owner sees the consequence before confirming, and if a novelty
touches 80 documents it is better to know that **before** publishing it, not on Monday.

### Step 6 — Confirmation page, then the menu

```bash
python3 scripts/render_harvest.py cosecha.json Dyn-Brain-Novedades-S<NN>.html
```

The schema is in the script's header (`--schema` prints it) and the script **validates the JSON**
first: every problem at once, exit 2, no page. The word is `territory`; `domain` is accepted as
the old name. Every candidate carries a reference — `S37-02` — which the script assigns if the
JSON did not.

**The page is for reading, not for clicking.** It has no buttons: a static HTML cannot reach
Claude, and the first version's buttons did nothing when pressed. Deliver it with `SendUserFile`.

**The confirmation is the `AskUserQuestion` menu**, in an interactive session:

- One question per candidate, its `ref` and title as the question, and exactly these four
  options in the owner's language: **Confirmar y publicar** · **Editar antes de publicar** ·
  **Interna · no propagar** · **No es novedad**. A candidate with `needs_work` or no
  `invalidates_terms` offers **Completar y publicar** instead of the first two.
- At most four questions per call, so eight candidates are two calls. Keep the order of the
  page.
- **Editar** and **Completar** open a follow-up: ask in free text what changes (scope, terms,
  wording), apply it to the record, and show the corrected record before it counts as
  confirmed.
- A text answer is just as valid — «S37-02 confirmar, S37-04 editar, el resto confirmar» — and
  the two can mix. Never make the owner repeat in the menu what they already said in text.

**In an unattended run never call `AskUserQuestion`**: nobody is there and the run would hang.
The run ends with the page, the summary and the notification. The owner opens that run from the
task's runs and says anything — «vamos», «confirma» — and *that* starts the menu, in the same
session, with `cosecha.json` as the source of truth. From another session, the owner points at
the saved `cosecha.json`.

The four answers, and all four matter:

- **Confirm and publish** — registered, and enters Monday's sweep.
- **Edit before publishing** — the text is right but the scope or the terms are not.
- **Internal · do not propagate** — a real novelty, registered for memory, but **it triggers no
  sweep**. Without this option people discard valid novelties for fear of the noise.
- **Not a novelty** — discard, and if the pattern repeats, note it to tune the filter.

Record every answer back into `cosecha.json` (`decision: confirm|edit|internal|discard`,
`answered_on`) before publishing anything, so a session that dies mid-way can resume without
asking twice.

### Step 7 — Publish what was confirmed

Only what the owner confirmed explicitly — through the menu or in text — and only now: nothing
in steps 1 to 6 writes to Odoo, to memory or to Teams. For each one:

1. Create the article under `01 · Novedades` with the full record.
2. Write to the owner's memory (`memory/decisions.md`): one line with the date, the novelty and
   the link. That is what makes their future sessions know it.
3. Announce it in the territory's Teams channel: one line and the link. Novelties are announced;
   they are not discovered.
4. Note in next week's harvest whatever was left pending refinement.

**Nothing is published without explicit confirmation.** If the owner does not answer, the harvest
expires next Friday and the candidates are proposed once more. On the second miss they are
discarded: if nobody recognises them as novelties in two weeks, they probably were not.

</process>

## Hard rules

- **Never publish a novelty without explicit human confirmation.**
- **Never** read another person's sources. One session, one owner.
- **Never** include evidence about people: performance, salary, hiring, health.
- **Never** propose more than **8 candidates**. If more come up, take the 8 with the highest
  confidence and say how many were left out.
- If `invalidates_terms` comes out empty → the candidate goes to "needs refining", never to
  publish.
- If there are no novelties this week → **do not generate a page**. One line and done. Silence is
  a valid and frequent result.

## When NOT to use this skill

- To register a novelty you have just decided → do it there and then, it is faster and more
  reliable than waiting for Friday. This skill is the safety net, not the main process.
- To summarise a meeting → that is minutes, and it goes in the project folder.
- To decide → this skill detects novelties already taken; it does not take them.

## Gotchas — foreseeable failures

| Situation | What to do |
|---|---|
| Memory OS not configured | Stop and ask for `memory:setup`. Offer a degraded harvest, warning about the loss. |
| Two `01 · Novedades` nodes | Pin `novelties_root_article_id` in the config. Ambiguous and unattended → check both, publish nothing, say so. |
| Run twice the same week | Refresh `cosecha.json`; keep refs; never re-propose the same candidates as new. |
| ORM session expires mid-run | `whoami`, retry once, report the retry. |
| Unattended run | Page + summary + notification. **No `AskUserQuestion`**: the menu starts when the owner opens the run. |
| Scheduled run cannot Read its references or run scripts | A permissions gap, not a skill gap: `dynbrain-setup` owns the allowlist. Say which step was denied. |
| No access to Granola / M365 | Continue with the available sources and **say which were missing** on the page. |
| A week with no novelties | One line. No page, no artifact, no message. |
| A candidate already published | Discard it silently and do not count it. |
| More than 8 candidates | Trim by confidence and say so. |
| The owner does not answer in two weeks | Discarded. Note the pattern to tune the filter. |

## Delegates to

`dynbrain-weekly-queue` — Monday's sweep, which consumes what this skill publishes. Its
`false_positives` field and the one in the record here are **the same field**: written on Friday,
read on Monday.
`dynbrain-canon` — the canonical state of the corpus the sweep runs over.
`dynbrain` — how to query and author documentation.
Odoo access itself is nobody's improvisation: the dual contract lives in
`references/shared/odoo-connection.md`.

## Consumed by

`dynbrain-weekly-queue` — Monday reads what Friday published; that handoff is the whole point of
the pair.

Nobody else, and not by design-in-waiting: this skill runs in one person's own session with their
own credentials, so a caller that is not that person has nothing to gain from it.
