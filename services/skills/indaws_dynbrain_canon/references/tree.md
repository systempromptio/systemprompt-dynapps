# The Dyn Brain tree

A **new, separate** tree. The existing 2.880 articles are not reorganised and not moved: they stay
exactly where they are, indexed exactly as today, outside this system until somebody puts them in
deliberately.

23 articles. Create with `docs_create_tree`, then provision the property definition on **every**
node, then activate canonicity on the 22 that can carry it.

```
Dyn Brain                                  ← root: declares, cannot carry. No doctrine here.
├── 00 · Cómo funciona Dyn Brain           states, buckets, what to do with a notice
├── 01 · Novedades                         one article per novelty, YAML front-matter
├── 10 · Consultoría
│   ├── Cómo trabajamos          decided    doctrine. A person signs it. Expires yearly
│   ├── Referencia viva          derived    maintained by its source. Never expires
│   └── Aprendido                observed   post-mortems. Never contradicts
├── 20 · Desarrollo y Plataforma           ┐
├── 30 · Producto — Core & inBuilt         │  same three children,
├── 40 · Comercial y Marketing             │  same exact names
└── 50 · Administración                    ┘  high_risk: deadline 7 days, not 21
```

**No `90 · Bandeja de entrada`.** Capture lives outside Odoo, in the Friday harvest.

## Why the three fixed children

They are not filing bureaucracy: `canon_truth_source` **drives the triage**. An `observed` article
never contradicts anything, because it describes what happened rather than prescribing what to do.
A `derived` article never gets a human review deadline, because its importer maintains it.

So putting a document under the right child *is* an instruction to the weekly sweep. Get it wrong
and the sweep misbehaves in one of two ways: doctrine filed under *Aprendido* becomes invisible to
propagation, and a post-mortem filed under *Cómo trabajamos* generates dated reviews that are all
false.

## Why the names are identical across the five territories

The sweep and every report group by them. A *Referencia viva* renamed to *Documentación técnica* in
one territory silently drops out of every count.

## The root carries nothing

`Dyn Brain` is a root, and a root can never carry `article_properties` — verified, and it fails
silently. Two consequences:

- **22 managed nodes, not 23.** The root stays a pure container.
- **A root's body is never swept**, because the sweep only looks at managed articles. So the rule
  is: *a root holds one sentence saying what hangs below it, and never prescriptive content.*
  `lint_roots.py` checks it against the prescriptive markers of `classify_bucket.py`.

The same applies to any root seeded later, including Dyncloud (3740).

## Territory membership is a field, not a location

`canon_territory`, never "where it hangs". That is deliberate and it is what lets Git-imported
content belong to a territory **while staying physically where the importer put it** — no fight
with the sync, no moved `parent_id`.

This tree is the home of content **written in Odoo**. Imported content is attached by field.

## Provision every node, not just the parents with children

Today only 6 nodes have children: the root (7) and the 5 territories (3 each). Provisioning only
those 6 is a trap — the first curated article created under *Cómo trabajamos* would lose its
properties in silence, because its parent declares no definition.

So provision all 23. One write each, once, and the whole class of bug disappears. Use
`nodes_to_provision()`, and confirm every write with `assert_written()`.
