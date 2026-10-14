# Triage — which bucket a match falls in

Every match the sweep finds lands in one of four buckets. Only one of them puts a
date on somebody's calendar, and that asymmetry is the whole design: **a false
positive with a deadline destroys trust in the system; an ignored `maybe` costs
nothing.** So the rule is *when in doubt, `maybe`* — never `contradicts`.

## The four buckets

| Bucket | What it means | Review? | Deadline? |
| --- | --- | --- | --- |
| `infrastructure` | A field, model or module name. Not a mention of doctrine. | no | no |
| `historical` | Describes the past. It was true then; it is not wrong now. | no | no |
| `contradicts` | Prescribes what to do, so the novelty makes it wrong. | yes | **yes** |
| `maybe` | Anything else. Recorded so a human can look, with no clock. | yes | no |

`historical` and `infrastructure` are written to the sweep log and go no further.
They are not silently dropped — a sweep that finds 40 matches and opens 3 reviews
has to be able to show where the other 37 went, or nobody believes the 3.

## Evaluated in order, first hit wins

The order is not cosmetic. An identifier inside a prescriptive paragraph is
infrastructure; evaluating the prescriptive rule first would open a dated review
against a live database field. That exact case is what `false_positives` was
invented for — a sweep for "Studio" finds `x_studio_price`.

1. **`infrastructure`** — the match came from inside `<code>` or `<pre>`, **or**
   the term is itself an identifier (snake_case with an underscore), **or** the
   term is listed in the novelty's `false_positives`.
2. **`historical`** — the article's `canon_truth_source` is `observed`, **or** the
   paragraph carries a past marker.
3. **`contradicts`** — the sweep declared the match is inside a table cell, **or**
   the paragraph carries a prescriptive marker.
4. **`maybe`** — none of the above.

## Two things that make this behave

**The truth source does the heavy lifting.** Anything under a territory's
*Aprendido* is `observed`, and an `observed` article never contradicts: it is a
record of what happened. That is why putting a document in the right child of its
territory matters — the tree position *is* an instruction to the triage. Doctrine
filed under *Aprendido* becomes invisible to propagation; a post-mortem filed
under *Cómo trabajamos* generates dated reviews that are all wrong.

**Markers match by whole word, ignoring accents** — the same rule the invalidated
terms get. `usamos` inside `abusamos` is not a prescription, and `tenía` written
without its accent is still the past. Without that, the markers reintroduce
exactly the false positives the matcher exists to prevent.

## Replacing the heuristic

This is a floor, not the answer. Everything replaceable is in two places, on
purpose: the marker lists at the top of `scripts/classify_bucket.py`
(`PAST_MARKERS`, `PAST_PATTERNS`, `PRESCRIPTIVE_MARKERS`) and the single
`classify_bucket()` function. Nothing is decided anywhere else.

Before adding a prescriptive marker, be aware of what you are doing: each new one
is a potential false positive **with a deadline attached**. If a legitimate case
does not fit, the right home is `maybe`, which means changing a test — not growing
the heuristic.

## Known limitation

`find_matches` does not detect table cells, so `in_table` has to be declared by
the caller. Until the sweep passes it, a priority table with no verb in it reads
as `maybe` rather than `contradicts`. That is the safe direction to fail.
