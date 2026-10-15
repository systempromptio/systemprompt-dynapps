# The novelty record

One format. This skill writes it, the owner confirms it, Monday's sweep consumes it.

```yaml
---
id: nov-<territory>-<yyyy-mm>-<slug>
type: decision          # decision | external_source | internal_doc | error_report
                        # | end_of_life | people_change
title: "A full sentence, in the present tense, saying what we do now"
decided_by: [name.surname, or the group]
decided_on: 2026-07-12          # when it was closed, NOT when it was registered
registered_on: 2026-08-07
territory: comercial
scope: [propuestas, presupuestos, demos]
invalidates_terms: ["Studio", "reconstruir en Studio", "personalización sin código"]
false_positives: ["x_studio_price", "x_studio_task_id"]
supersedes: null                 # id of the novelty this replaces, if any
expires_on: null                 # only if provisional
propagate: true                  # false = registered, but triggers no sweep
status: Verified
---

## What changes
Two or three sentences. What we did before and what we do now.

## Why
One or two sentences. The reason, not the long justification.

## What does NOT change
The limits. This is the section that stops somebody over-applying it.

## Evidence
> "literal quote"
— Who, where, when.
```

---

## The six types

`decision` is the common case and what the filter in `what-is-a-decision.md` is tuned for. The
other five exist because a novelty is not always a decision somebody took:

| Type | What it is |
| --- | --- |
| `decision` | We decided to work differently from now on |
| `external_source` | Something outside changed: a version, a law, a supplier |
| `internal_doc` | A new internal document that supersedes scattered practice |
| `error_report` | Something we documented turned out to be wrong |
| `end_of_life` | Something we used stops being supported or offered |
| `people_change` | An ownership change: who signs what from now on |

The type does not change the mechanics — every one of them can invalidate terms and trigger a
sweep. It changes how the notice reads to the owner, and it is what lets "why do we do this?" be
answered six months later.

---

## How each field is written

### `title` — one sentence, present tense, quotable

It is what people read in the notice, in the channel and in Monday's queue. It has to be
understandable **out of context and six months later**.

✅ "Dynapps deja de proponer Odoo Studio; los GAP se cubren con desarrollo asistido por IA"
✅ "El importe mínimo de un proyecto de implantación es de 15.000 €"
❌ "Cambio en la política de desarrollos" — says nothing
❌ "Studio" — not a sentence
❌ "Se ha decidido que a partir de ahora, y tras la reunión del martes, ya no…" — all of it is
   surplus

Rule of thumb: if it does not fit in an email subject line, it is too long.

### `decided_on` — the real date

The date the discussion closed, not the date of the harvest. It matters because it defines **how
far back** documents and proposals are potentially contaminated. A July novelty registered in
August has a month of retroactive scope, and that has to be countable.

### `territory` — whose it is

From the closed list in `config.json`. Never `domain`: that word already means something else in
Odoo, and the collision is paid for in every conversation.

### `scope` — where it applies

From the closed list in `config.json`: `propuestas`, `presupuestos`, `demos`, `analisis`,
`implantacion`, `soporte`, `interno`, `fiscal`, `rrhh`, `marca`. It narrows the sweep: without
`scope`, the sweep walks everything and multiplies the false positives.

### `invalidates_terms` — the field that makes this work

The terms which, **in a document that prescribes what to do**, mean that document went stale.

Think like whoever wrote the old documentation, not like whoever states the new novelty. Include
synonyms and paraphrases, not just the proper noun: "personalización sin código" catches documents
that never wrote the word Studio.

### `false_positives` — the safety net

Terms that **match but must not be touched**: field names, models, installed modules, known
historical references.

This field exists because of a real case: a sweep for "Studio" finds `x_studio_price` and
`x_studio_task_id`, which are live database fields. **An agent told to "remove Studio" would have
edited production.** Fill it in whenever you suspect.

**It is the same field Monday's sweep reads.** Written on Friday by this skill, consumed on Monday
by `dynbrain-weekly-queue`, where it is one of the three things that put a match in the
`infrastructure` bucket. So a term you leave out here becomes somebody's dated review.

### `propagate` — internal novelty yes, noise no

`false` when it is a real novelty that invalidates no documentation: internal reorganisation, team
agreements, meeting cadences. It is registered — it has memory value — but it triggers no sweep
and notifies nobody.

Without this option, people discard valid novelties for fear of bothering others. With it,
everything gets registered and only what matters propagates.

### `expires_on` — only for the provisional

If somebody says "for now we do X until we see", give it a date. The provisional with no date is
what does the most damage: nobody knows whether it still holds and nobody dares change it.

### `supersedes` — the chain

When a novelty reverts or modifies another, link them. It lets "why do we do this?" be answered
with the full history, which is usually the real question behind most repeating arguments.

---

## The four body sections

**What changes** — the before and the after. Concrete. If you cannot write the "before", check
that the novelty actually changes anything.

**Why** — one or two sentences. This is not minutes: it is the reason that will let somebody
review it in a year with some judgement.

**What does NOT change** — the most underrated section. Explicit limits prevent over-application,
which is the typical side effect: we decide to stop proposing Studio and somebody concludes the
existing fields have to be ripped out.

**Evidence** — the literal sentence, who and where. It is what lets the owner confirm in seconds
and what stops somebody being credited with a novelty they never stated.

---

## Before proposing, check

- [ ] The `title` stands on its own, six months later.
- [ ] `decided_on` is the real date, not today.
- [ ] `invalidates_terms` includes at least one paraphrase, not just the proper noun.
- [ ] You have thought about the false positives and written them down.
- [ ] "What does NOT change" is filled in.
- [ ] The evidence is literal and has an author.
- [ ] It was not taken by somebody else who was not present. If it was, it is marked as pending
      confirmation by whoever took it.
