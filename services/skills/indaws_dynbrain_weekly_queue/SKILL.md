---
name: indaws-dynbrain-weekly-queue
version: 1.0.2
effort: medium
model: opus
license: Interno Dynapps
description: >-
  The Monday sweep for Dyn Brain: takes the novelties published last week, finds the documents
  they invalidate, triages each match, and opens one review notice per owner. Closes the gap that
  motivated Dyn Brain — a decision that changes how we work, and documentation that never hears
  about it. Use it when the user wants to propagate a published novelty, run or inspect the weekly
  review queue, see which documents a decision affects, or understand why a document was or was
  not flagged. NOT the Friday capture of decisions — that is dynbrain-decision-harvest. NOT the
  canonical state itself — that is dynbrain-canon.
---

# Dyn Brain — the weekly sweep

<objective>
That no decision changing how Dynapps works leaves stale documentation being served as valid.
Every Monday, each owner gets **one** notice listing the documents of theirs that a decision just
invalidated — with the paragraph, the reason, and four ways to answer.
</objective>

## Where this sits

```
FRIDAY   dynbrain-decision-harvest   → the owner confirms → 01 · Novedades
MONDAY   this skill                  → sweeps, triages, notifies the affected owners
DAILY    dynbrain-canon              → deadlines, escalation, the problem report
```

Confirming on Friday and sweeping on Monday is deliberate: it gives the weekend as a buffer and
gets the novelty to the people affected **before** they start working from the old version.

## Language

Write everything **in English** inside this skill and its references. When you **talk to the user,
reply in the language they wrote in** — and the queue page an owner reads goes in their language.

## Which MCP am I using? (never hardcode a name)

- The **generic ORM proxy** — `orm_search_read`, `orm_write`, `orm_create`, `orm_post_message`.
  The lexical pass, the activities and the notes. Which server answers, and what to fall back to
  when none does, is `references/shared/odoo-connection.md`; this skill opens no
  connection of its own.
- The **documentation MCP**, the `docs_*` family — the semantic pass only.

Either can be **entirely absent** from a session, and a scheduled run is where that happens. If the
ORM proxy is missing, **stop**: there is no sweep without it. If the documentation MCP is missing,
run passes A and B, and **say in the report that the semantic pass did not run** — a sweep with a
missing pass is not a clean sweep.

## Where the novelties are

`novelties_root_article_id` in `~/.dynbrain/config.json` (written by `dynbrain-setup`) is the
`01 · Novedades` node whose children are last week's novelties. If it is null, resolve by name —
and if more than one article carries that name (production has two: 4026 under `Dyn Brain`, 4103
under `[ES] Internal`), **stop and say so** rather than sweep from the wrong one: Friday and
Monday have to agree on the node or a confirmed novelty is never propagated.

If the ORM proxy answers with a session or authentication error mid-sweep, `whoami` and retry the
call **once**, then report the retry. If `whoami` itself fails, the connector's authorisation is
gone for the session — there is no sweep without it: stop and say so. It expired mid-run on
2026-09-11, and later that day `whoami` failed too until the connector was re-authorised.

## Before sweeping: is this novelty sweepable?

`validate_novelty()`. Two entry conditions:

- `propagate: False` → **nothing happens.** It is a real decision recorded for memory. Without that
  option people stop recording decisions for fear of the noise, which costs more than the noise.
- Propagating with no `invalidates_terms` → **an error, not an empty sweep.** A sweep that finds
  nothing because the novelty is malformed must never look like one that found nothing because the
  corpus is clean.

## The universe

Two populations, read through `sweep_view()` in `dynbrain-canon`, which gives both the same shape
(`canon_owner`, `canon_territory`, `canon_truth_source`) so triage and grouping take them unchanged:

```
canon:     canon_managed = True  ∧  canon_state ∈ {verified, expiring}
doctrine:  owner set  ∧  truth_source ∈ {decided, derived}  ∧  doc_type ∉ {index, bridge, home, post_mortem}
both:      active = True  ∧  not a child of 01 · Novedades
```

The doctrine is read through the schema it already carries; nobody seeds `canon_*` on it to make
it sweepable. Until 2026-10 the universe was the canon population alone, and it never held a single
document that prescribed anything: four sweeps in a row came back empty for that reason. Novelties
are out because a novelty records a decision — sweeping one against another only matches things like
«bloqueado» the adjective against «Bloqueado» the stage.

A doctrine article with no owner of its own — its parent declares no properties — inherits owner,
territory and truth source from the nearest ancestor that has one (`ancestors=`, nearest first);
its own doc_type is the only one that counts.

Query with `sweep_domains()`, **paginate** each domain (~2.900 articles, and an unpaginated
`orm_search_read` drops rows in silence), then keep only what `sweep_view()` returns. A doctrine
article whose territory has no Dyn Brain key comes back with `canon_territory = None`: list it in
the report, never guess its root.

`in_sweep_universe()` is unchanged and still drives the daily deadline pass: doctrine is swept, but
it never receives an expiry or an escalation from `dynbrain-canon`.

If the novelty declares `scope`, restrict further to territories that declare it. **Known
limitation:** per-article scope does not exist, so `scope` filters by the article's territory only.

## Three passes, in order

- **A — lexical.** `orm_search_read` with `body ilike` per term, paginated. Cheap, wide, noisy.
- **B — word boundaries. Not optional.** `find_matches()` from `match_terms.py` over every
  candidate. This is what stops `Studio` matching the Spanish word «estudio», which without it
  produces hundreds of false positives across the corpus.
- **C — semantic.** `docs_search` with the novelty's title, for documents that never wrote the term
  literally. Candidates found **only** this way enter as `maybe`, never as `contradicts`: there is no
  matched paragraph to justify a deadline.

## Triage and notices

`classify_bucket()` per match — the four buckets and their order are in `references/triage.md`.
Put `edited_since_decision(last_edition_date, decided_on)` in the meta: a page a person edited on or
after the decision day never gets a `contradicts`, because a page that already applies the novelty
matches its terms exactly like one that contradicts it.
`historical` and `infrastructure` go to the **sweep log** and no further, and the log has to be in
the report: a sweep that found 40 matches and opened 3 reviews must be able to show where the other
37 went, or nobody believes the 3.

Then `group_by_owner()`: **one notice per owner and novelty, never one per document.** 12 articles
across 3 owners is 3 notices. Only a notice containing a `contradicts` carries a deadline.

## The write workflow: order matters, and the note is not the notification

1. Create the **activity** on the owner's territory root — `mail.activity`, `user_id` the article's
   owner, `date_deadline` from the notice. **This is the notification.**
2. Post the note with `orm_post_message`, real HTML, never pre-escaped.
3. Re-read to confirm before reporting anything as done.

`orm_post_message` takes **no `partner_ids`**: it neither mentions nor notifies. The note is the
**record**; the activity is what reaches the person. Post only the note and the owner never finds
out.

## Never

- **Never write to an article body without the owner's yes in the Phase B session**, and never to
  one whose `ai_writable` is false unless the owner chose to open it. The sweep proposes; the owner
  approves, in their own session, under their own name.
- **Never assign to `decided_by`.** The notice goes to the article's owner.
- **Never skip pass B.**
- **Never let a `maybe` carry a deadline.**
- **Never report a sweep as complete when a pass did not run.**

## Phase B — the owner decides in the chat

Nobody opens the activity menu of Knowledge. The activity stores the review; the owner **sees and
decides it in a Claude session**, the same way the harvest is confirmed.

1. **Who sweeps what.** Each owner's Monday run sweeps only the novelties of its own territory
   (Phase A), then gathers **every open review whose owner is this user**, whoever's sweep opened
   it, and drafts a proposed paragraph for each. A review whose paragraph already applies the
   novelty gets no proposal.
2. **Unattended, it stops there.** Save the list to
   `<output_dir>/queue/<ISO week>/<owner slug>.reviews.json` (shape in `review_questions.py`),
   notify, and end. **Never call `AskUserQuestion` in an unattended run**: nobody is there.
3. **The owner opens that run and says anything.** Then `review_questions.py` builds the
   batches — four questions per call, contradictions first, the proposal inside the question —
   and `AskUserQuestion` asks them. Check the date of the file first: a questionnaire answered a
   week late acts on a proposal drafted a week ago.
4. **Read every answer with `read_answer()`.** «Other» comes back as `interpret`: it may be the
   final paragraph or an instruction. The first pilot got «hazlo AI writable y escribe» four
   times — written literally, that sentence would have landed in four documents. Follow an
   instruction; show a replacement paragraph back before writing it.
5. **Read the whole page before writing a paragraph.** In the pilot, the approved paragraph of
   «Las estrellas y el commit» contradicted the rest of its own page: the novelty had not said
   whether the stars stayed in Spain. Ask, then write — and fix the novelty, not only the page.
6. **Write through the documentation MCP.** Most doctrine is synced with a Git repository:
   `docs_metadata` says `is_synced` and `can_write`; `docs_update` commits under the owner's name.
   Documents written in Markdown refuse an ORM write to `body` — edit their source, never the
   rendered HTML. `allow_and_apply` sets `ai_writable: true` in the same call.
7. **Record, then close.** Write `decision`, `owner_note` and `answered_on` into the reviews file
   after each batch, so a session that dies resumes without asking twice; then close the activity.

## The answers an owner has

`apply` · `allow_and_apply` (opens a closed document to AI, then applies) · `apply_manual` (the
text is handed over to paste) · `keep` (renews `reviewed_on`, or `canon_verified_on`) ·
`not_applicable` (**requires** a note) · `snooze` (**once only**) · free text (`interpret`).

## Scripts

| Script | What it owns |
| --- | --- |
| `match_terms.py` | Whole-word, accent-insensitive matching over an HTML body |
| `classify_bucket.py` | The four buckets and their order |
| `group_notices.py` | One notice per owner, deadlines, idempotency |
| `render_queue.py` | The page an owner reads |
| `review_questions.py` | Phase B: the owner's reviews as `AskUserQuestion` batches, and their answers back |

## Gotchas

The full list is `references/gotchas.md` — **read it before reporting any result.** The two that
bite hardest are already above, and both fail quietly: `orm_post_message` takes no `partner_ids`,
so posting only the note leaves the owner never notified; and an unpaginated `orm_search_read`
over ~2.900 articles drops rows without saying so, which looks exactly like a clean corpus.

## Delegates to

`dynbrain-canon` — the canonical state, the sweep universe, the deadline arithmetic.
`dynbrain` — how to query the documentation MCP, and the origin check before proposing any change.

## Consumed by

`dynbrain` — to turn a published change into reviews for the owners it affects.
`dynbrain-decision-harvest` — the sweep it hands each confirmed novelty to.
`dynbrain-canon` — for the review side of a document's canonical lifecycle.

Nobody outside the Dyn Brain group.
