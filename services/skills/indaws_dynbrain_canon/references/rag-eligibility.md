# RAG index eligibility

The only part of Dyn Brain that changes what the whole company gets served. The
design goal is not "exclude the stale ones" — it is **never remove anything that is
in the index today**.

## How the engine decides

`dynapps.knowledge.rag.rule` records, **first match wins, ascending sequence**
(`sequence` help: *"Rules are evaluated in ascending order and the first one matching
an article decides its eligibility"*). `mode` is `include` or `exclude`; `scope` is
`all`, `repo`, `space` or `domain`.

There is also a **per-article override**, `dynapps_rag_mode`: `auto` (let the rules
decide), `always` (force in), `never` (force out, chunks purged). **It beats the
rules.** So a rule is never the last word — check the field before concluding that a
document's eligibility follows from the rules.

Measured 2026-08-08: **0 articles** carry anything other than `auto`, so nothing is
pinned today.

## What is indexed today, measured

| | HTML | Markdown |
| --- | --- | --- |
| Articles | 2.745 | 143 |
| With `dynapps_markdown` (the index source) | 2.552 | 126 |
| **With chunks — i.e. indexed** | **113** | **126** |
| Pending reindex (`dynapps_rag_stale`) | 0 | 0 |

A first reading of that table said the no-match default was **exclude**, and that an
article written in Odoo would therefore never be retrievable. **That was wrong**, and
a bounded experiment refuted it: a native HTML article with `dynapps_rag_mode = 'auto'`
was **re-chunked** as soon as its markdown changed. HTML **is** eligible.

The real explanation is a **backlog, not an exclusion**. Indexing is driven by content
changing: a write fills `dynapps_markdown` and chunks appear immediately. Articles
nobody has edited since the RAG module arrived were never queued, so they sit with
content and no chunks — and `dynapps_rag_stale` stays false because nothing changed.
The daily `_cron_index_articles` works through them in batches.

**So the curated tree needs nothing special: writing it indexes it.** What remains is
the backlog of untouched articles, which is a decision about the existing corpus, not
about this delivery.

## The exclusion rule

```
name:     "Sólo canon verificado"
scope:    "domain"
mode:     "exclude"
sequence: 5
domain:   [['article_properties.canon_managed', '=', True],
           ['article_properties.canon_state', 'not in', ['expiring', 'verified']]]
```

**Record id: 2** (created 2026-08-08 on db `master`).

Why the complement rather than a disjunction: with `exclude` semantics, an article
without canonicity **does not match**, falls through to the markdown rule at 10, and
behaves exactly as before. An `include` rule cannot express "leave everything else
alone", and a domain with an `'|'` would match the whole corpus through its first
branch.

Why `sequence: 5`: ahead of the existing rule at 10, or it never gets to decide.

Verified before creating: the domain matched **0 articles**, so the rule is a no-op
until somebody activates canonicity — which is also why creating it now is the safe
moment. The more managed content exists, the more surface a mistake has.

## Reverting it

Written before the record was created, on purpose.

```
orm_write('dynapps.knowledge.rag.rule', [2], {'active': False})
```

Deactivating is enough and is preferable to deleting: the record documents what was
tried. If it must go entirely, `orm_unlink('dynapps.knowledge.rag.rule', [2])`.

Reverting **does not re-index anything by itself**. Articles excluded while the rule
was live had their chunks purged, and they come back on the next indexing pass — the
daily `_cron_index_articles`, or by setting `dynapps_rag_stale` on the affected
articles.

## Forbidden

- Touching `dynapps_rag_base` or `dynapps_knowledge_rag`. Everything goes through
  records and properties.
- Deleting `dynapps.rag.chunk` rows by hand. The engine owns them.
- Using `active` or archiving on an **article** to keep it out of the index. That
  hides the document from everybody, not just from retrieval.

## Still unverified

`odoo_describe_model` and `odoo_get_selection` are blocked for
`dynapps.knowledge.rag.rule` on the ORM proxy — its metadata has to be read from
`ir.model.fields`. So the **exact** no-match default is inferred from the data above
rather than read from the source, which is not on this machine. The inference does not
affect this rule, which only ever adds an exclusion to a subset.
