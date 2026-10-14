# Gotchas — the weekly sweep

Read before reporting any result. Each of these has cost something already.

## The one that motivated the matcher

**`Studio` matches the Spanish word «estudio».** Pass A is `body ilike '%Studio%'`, and «hicimos un
estudio de viabilidad» matches. Without pass B — whole-word boundaries — one term produces hundreds
of false positives across the corpus, each with a deadline on it. **Pass B is not optional.**

The mirror of it: `x_studio_price` is a live database field. The underscore counts as a word
character, so the boundary rule protects it on its own — but `false_positives` on the novelty is the
belt to that braces, and a real agent told to "remove Studio" would have edited production.

## Silent failures

- **`orm_search_read` without pagination drops rows.** ~2.900 articles, and no error when it
  truncates. Always paginate, and check the count adds up.
- **Writing `article_properties` where the direct parent declares no definition returns `ok: true`
  and stores nothing.** Not an error — a silence. Always re-read. A **root** can never carry them at
  all.
- **`orm_post_message` takes no `partner_ids`.** No mention, no notification. If you only post the
  note, the owner never learns anything. The **activity** is the notification.
- **OR-ing two property paths returns nothing.** `["|", ["parent_id.article_properties.owner",
  "!=", False], ["parent_id.parent_id.article_properties.owner", "!=", False]]` gave **0 rows** on
  2026-10-06 while each side alone gave 73 and 13. No error. Keep every property-path condition in
  its own domain — that is why `sweep_domains()` returns four domains and not one.
- **A parent with no property definition leaves its children with no properties at all.** «🎯 CRM»
  did that to «Las estrellas y el commit», «El embudo» and two more. They have no owner of their
  own; `sweep_view()` takes it from the nearest owned ancestor, and the orphan domains reach two
  levels up. Deeper orphans are not reached: count them in the report.
- **A pre-escaped body comes out double-escaped.** Pass real HTML: `<p><b>x</b></p>`, never
  `&lt;p&gt;`. `body_is_html` is auto-detected.

## Silent absences

- **The `docs_*` MCP can be entirely missing from a session**, and a scheduled run is exactly where
  it happens. Passes A and B still work. **Say in the report that the semantic pass did not run** —
  an incomplete sweep reported as complete is worse than no sweep, because the gap is invisible.
- **`dynapps.knowledge.rag.rule` is blocked for `odoo_describe_model` and `odoo_get_selection`** on
  the ORM proxy, while `orm_search_read` works. Read its metadata from `ir.model.fields` instead.

- **The ORM proxy session can expire mid-run.** Measured 2026-09-11 on the harvest: the call after
  the idempotency check came back with a session error and the rest of the pass never ran. One
  `whoami` reopens it; retry once and say you did.

## Counting and reporting

- **`historical` and `infrastructure` must appear in the log.** A sweep that found 40 matches and
  opened 3 reviews has to show where the other 37 went. Report only the 3 and the number looks
  either wrong or suspiciously convenient.
- **A malformed novelty is not an empty corpus.** `propagate: True` with no `invalidates_terms`
  finds nothing, and reporting "no documents affected" hides a broken novelty. `validate_novelty()`
  first.
- **A candidate found only by the semantic pass has no matched paragraph**, so it cannot justify a
  contradiction. It enters as `maybe`.

- **A page that already applies the novelty reads as a contradiction.** «Nunca con el descuento del
  primer año» matches the term and carries a prescriptive marker. Measured 2026-10-06: all four
  `contradicts` of the first doctrine dry run were pages already fixed. Pass
  `edited_since_decision(last_edition_date, decided_on)` into the meta: an edit on or after the
  decision day turns `contradicts` into `maybe`. Use `last_edition_date`, never `write_date`.
- **A generic term turns the rest into noise.** «número de usuarios», «Bloqueado» and «commit» each
  matched ten or more pages that had nothing to do with the decision. The fix lives in the novelty —
  sharper terms or `false_positives` — not in the matcher.

## Data traps in this corpus

- **`write_date` is useless as a freshness signal**: a bulk reprocess left 65 distinct timestamps
  across 2.880 articles, and one `write_uid` on 84.6% of them.
- **44.1% of article names are duplicated** and **174 bodies are empty**. Any query assuming a name
  is unique will be wrong, and empty bodies pass through the matcher without matching and inflate
  the candidate count if you count before filtering.
- **Article items do not nest**, so do not walk them expecting depth.

## Ordering

Transition or write **first**, re-read **second**, note **third**. A note written before a write
that did not land is a lie in the audit trail, and the audit trail is the only reason anyone will
trust the sweep in six months.
