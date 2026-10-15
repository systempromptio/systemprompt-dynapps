# Documentation structures — where the layout comes from

**The layouts are not in this folder, and that is deliberate.** They are
documentation: they live in Knowledge, they are versioned like any other document,
and they are **found** — not remembered — with a typed filter:

```jsonc
docs_search { "query": "how this documentation is organised, where a document goes",
              "where": { "doc_type": "governance" } }
```

`../knowledge-as-source.md` has the full flow, the narrower filters per kind of
documentation, and what to do when it comes back empty.

A layout copied here would be a second original. The day somebody adds a category
or renames a block, the corpus moves and the copy keeps answering with yesterday's
truth — and it answers confidently, which is worse than not answering. A *name* or
a *path* copied here would be just as bad in a different way: it would tie the
skill to one installation.

## The kinds, and how each one's layout is found

| Kind | How to find its layout |
| --- | --- |
| **Internal, per department** | `where: { "doc_type": "governance" }` — the blocks a domain has, what goes in each and its regime |
| **Internal, general** | same filter: governance covers the root documentation's own tree |
| **Cross-department processes** | `where: { "territory": "processes", "doc_type": "index" }` — a process has a shape of its own, documented in its section index |
| **Client-facing** | **not defined yet** — see `client.md` |

Those field names and values are examples of the shape of the question, not a
catalogue: `docs_metadata_schema` says which fields a documentation actually
declares, and an undeclared one is refused with the list of the real ones.

## What stays on this side

Only what is not doctrine and cannot be looked up:

- **How to find it** — the query, the filters, the order of calls — in
  `../knowledge-as-source.md`.
- **The rule of stopping** when nothing is published.
- The **mandatory rules of authoring** that belong to the tools rather than to the
  corpus — a second language is a variant, the header belongs to the document — in
  `../../SKILL.md` and `../gotchas.md`.
