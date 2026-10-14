# TODO-DECISIONES

Everything the functional document marked "verify in code", what the answer turned out to be, and
what is still open. The addon sources for `dynapps_*` are **not on this machine** and
gitlab.dynapps.be rejects the SSH key, so every answer below came from ORM introspection or from a
bounded experiment against production — never from reading the module.

Full reasoning per item: `plans/active/2026-08-dynbrain-canon-skills/decisions.md`.

## Closed by measurement

| Question | Answer | How |
| --- | --- | --- |
| Are `article_properties` searchable by domain? | **Yes**, and they filter. `[('article_properties.canon_state','not in',[…])]` returned exactly the intended article out of 2.884 | Test articles, created and deleted |
| Can the property keys be chosen? | **Yes** — `canon_state`, not a generated hex. The hex keys on the 6 pre-existing articles are just what the UI generates | Write + re-read |
| Does the properties definition cascade? | **No.** Only the **direct parent** counts, and a **root can never carry properties at all**. Both failures are **silent**: the write returns `ok: true` and stores nothing | Grandchild and root probes |
| Does `dynapps.knowledge.rag.rule.mode` allow exclusion? | **Yes**, `exclude` exists. And rules are **first-match-wins by ascending sequence** | `ir.model.fields` (the model is blocked for `describe_model`) |
| Are Git-imported articles read-only? | `docs_metadata` returns `repo` / `is_synced` / `can_write`, and `can_write` folds all three cases | Field help + ORM |
| Is there a `mail.activity.type` for articles? | **No.** Only the 4 generic ones with `res_model: false`, which work on any model | ORM |
| Does `orm_post_message` preserve HTML? | **Yes, if you pass real HTML.** Pre-escaped entities come out double-escaped. And it takes **no `partner_ids`**: it neither mentions nor notifies | Two messages, one as a control |
| Is HTML indexed by the RAG? | **Yes.** A native HTML article with `rag_mode: auto` is re-chunked as soon as its markdown changes | Bounded experiment |

## Corrected during the delivery

**Phase 8 concluded that HTML was not indexed and that the curated tree would be invisible. That was
wrong.** It was inferred from counts — 2.552 HTML articles with content, 113 with chunks, nothing
pending — and phase 9 refuted it with an experiment. The real explanation is a **backlog**: indexing
is driven by content changing, and nobody had edited those articles since the RAG module arrived. A
whole phase was added on the false premise and closed without building anything.

Two smaller corrections in the same area: an "identical before and after" count presented as proof
was weak evidence, because the number moves on its own; and the sweep does **not** need the index —
it uses `body ilike` over the ORM, and only the semantic pass uses `docs_search`.

## Still open

- **The exact no-match default of the RAG engine is inferred, not read.** The measurements are
  consistent with "an article is indexed when its content changes and no `exclude` rule matches it",
  but `odoo_describe_model` and `odoo_get_selection` are blocked for
  `dynapps.knowledge.rag.rule` on the ORM proxy, and the source is not available. Nothing in this
  delivery depends on the answer: the exclusion rule only ever removes a subset that is empty today.
- **`dynapps_rag_mode` beats the rules.** A per-article `always` / `never` overrides every rule. 0
  articles use it today, but any "why did this leave the index?" has to check that field first.
- **Per-article `scope` does not exist.** A novelty's `scope` narrows the sweep by the article's
  *territory* only. Known limitation, in the sweep's SKILL.md.
- **Table cells are not detected.** `find_matches` does not know a match came from a `<td>`, so a
  priority table with no verb reads as `maybe` until the sweep declares `in_table`. Fails safe.
- **Whether property writes survive a Git sync is unverified.** That is why the Dyncloud seed is
  delivered and not executed: it would need the definition written to 29 Git-synced articles.

## Production state at close

| | |
| --- | --- |
| `dynapps.knowledge.rag.rule` **id 2** | `Sólo canon verificado`, `exclude`, `sequence: 5`, active. Matched **0** articles when created |
| `ir.cron` 200 (RAG indexing) | **Active.** Disabled and re-enabled the same day — see below |
| Articles with canonicity | **0.** Nothing was marked managed; the tree and the seed are not executed |
| Chunks | grew from 1.148 to 6.737 during the work |

**The chunk growth was caused by this delivery, and not deliberately.** Writing `dynapps_markdown` on
a test article triggers the daily indexing cron, which then processed its whole backlog — about 598
previously unindexed articles. Nothing was removed or damaged; the index only grew. The cost is
embeddings for those 598.

The cron was then disabled (do not index what nobody reviewed) and re-enabled the same day, once it
was established that **indexing is not a prerequisite for propagation** and that curating without an
index means facing each owner with "review your territory" instead of with a prioritised list the
sweep produces. The criterion did not change — owner-signed, up-to-date documentation — the order did.

**Consequence to keep in mind:** the corpus is being indexed **without review**. ~1.841 articles
still pending, draining in daily batches. No article declares itself verified, so the absence of a
canonical state is the signal that nobody has signed it. `00 · Cómo funciona` must say that plainly,
and Dyn Brain must never be described as curated before it is.

## Not delivered, deliberately

The tree (23 articles), the Dyncloud seed, and the three scheduled cadences are **written, tested and
not executed**. Creating them is a human-confirmed step, and the three cadences should wait for one
full manual cycle: the triage has never run over real content.
