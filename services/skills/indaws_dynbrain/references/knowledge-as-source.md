# The doctrine lives in Knowledge, and it is FOUND, not remembered

Where a document belongs, how it is written, which value each metadata field takes
— none of that is written here. It is documentation, it lives in the corpus, and it
is read at the moment it is needed.

That is not tidiness. A rule copied into a skill is a second original: the day
somebody adds a category or renames a block, the corpus changes and the copy keeps
answering with yesterday's truth, confidently. **The corpus is the company brain;
this skill is how to consult it.**

And it is not enough to keep the *rules* out. Keeping a **name**, a **path** or an
**id** in here would tie the skill to one installation just as tightly. So the
doctrine is not addressed by where it sits: **it is identified by what it is**, and
what it is happens to be a typed field on the document — which means it can be
asked for.

## What is never written down, because a tool answers it

| Question | Tool |
| --- | --- |
| Where is the doctrine of THIS installation | `docs_search` with a `where` filter (below) |
| Which metadata fields exist here, of what type, with what values | `docs_metadata_schema` |
| What documentations exist, and their ids | `docs_list_spaces` |
| What this documentation's tree looks like | `docs_outline` |
| Whether this document is native, mirror or managed | `docs_metadata` |

Never carry a field list, a `space` id, a document title or a path in your head or
in a prompt. Every one of them is per-installation, and several are per-section.

## Finding the doctrine: one call

A document that explains how the documentation itself works is tagged as such. Ask
for that class of document and let relevance order the results:

```jsonc
docs_search {
  "query": "how this documentation is organised, where a document goes, how it is written",
  "where": { "doc_type": "governance" }
}
```

**The filter is what makes this safe, and it is why this is not "searching for the
doctrine".** `where` is a typed filter, not a guess: nothing that is not governance
can come back. Relevance only decides the order inside that class. Compare it with
a bare `docs_search`, which cannot fail and will happily return something that
merely *reads* like a rule.

For a narrower class, add the field that narrows it. Cross-department processes,
for instance, have a shape of their own, and it is documented in the index of
whichever documentation owns them:

```jsonc
docs_search {
  "query": "how a cross-department process is written",
  "where": { "territory": "processes", "doc_type": "index" }
}
```

Then read the ones that apply, **by the `id` the search just returned** — not by
the `path`, which is only unique inside one documentation (`index.md` exists in
every repository):

```jsonc
docs_get { "article_id": <the id from the result> }
```

The id is fine here precisely because it was born one call ago. An id written into
this file would be an id from somebody else's database.

## When there is nothing to find, say so

Two ways this comes back empty, and both are answers, not failures:

- **The field does not exist.** `where` on an undeclared field is refused, and the
  answer lists the fields the documentation does declare. That means this
  installation does not classify its documents that way.
- **The field exists and nothing carries that value.** No governance document has
  been published here.

In both cases:

1. Say it plainly: **this installation does not publish its documentation
   doctrine.**
2. **Do not fall back to a bare `docs_search`** and treat whatever comes back as
   the house standard. A plausible-looking passage is how an agent invents a
   structure and presents it as policy.
3. Agree the structure with the user for this document, and say that is what you
   are doing.

That is the normal case on a client's own Odoo, and it is the reason this page
names nothing: the skill has to work there too.

## When two rules disagree, stop — do not choose

The doctrine is written by people, over time, in separate documents. Sooner or
later two of them will answer the same question differently: one demanding
something, another accepting a laxer version of it.

**That is a finding, not a fork in the road.** Say it, name both documents and what
each one asks for, and let a person settle it. Meanwhile follow the **stricter** of
the two, because it is the one that satisfies both.

What makes this worth a rule of its own is how it fails otherwise. Picking one in
silence looks like progress: nothing errors, the work goes on, and the choice is
invisible — until somebody notices, and by then everything written under it has to
be revisited. It has already cost twenty-two documents once.

The same applies to a rule that contradicts what you can see in the corpus: a
layout everything follows and one document forbids, a field every sibling carries
and the doctrine does not mention. Report the gap. **The corpus is somebody's to
fix; it is never yours to reconcile quietly.**

### And a rule that contradicts the running system

Doctrine is what we believe; only Odoo says what is happening. `SKILL.md` → "Before
writing doctrine, check the running system" has the three cheap checks. What they
caught on the first real article, none of it visible in any document:

- The template the doctrine named — "Implantación v2" — **did not exist**. The real
  record is `Implantación Value Based` (`sale.order.template` id 15). The name came
  from a glossary shorthand and would have sent a rep looking for something that is
  not there.
- There were **four** value-based templates, not the three everyone remembered, and
  one of them had **zero** quotations: created and never used.
- The template the doctrine declared retired was **still in use that month**.
  Doctrine was right; adoption was at three quarters, and the remaining quarter is
  where the value was.
- A pricing config contradicted **itself** — `base_monthly: 320` in one key and a
  `360` fallback in another, in the same file.

## When to go and get it

Not on every activation. Loading the doctrine to answer "what do our docs say about
backups" is waste.

| What the user is asking for | Read the doctrine? |
| --- | --- |
| Search, read, answer a question | **no** — that is ordinary retrieval, and `docs_search` / `docs_answer` are the right tools |
| Create, restructure or move documentation | **yes, before writing the first line** |
| Say where a change stands, or what reaches Git | no — that is the MCP's own business (`mcp/pending-work.md`) |

The rule against searching applies to **the doctrine only**. For content — what
people actually wrote — semantic retrieval is the first tool, not the last.

## The order, when you are about to write

```jsonc
docs_search { "query": "…", "where": { "doc_type": "governance" } }   // where the doctrine is, here
docs_get    { "article_id": <from the result> }                        // read it
docs_metadata { "article_id": <the parent> }                           // native / mirror / managed
docs_metadata_schema { "article_id": <the parent> }                    // ← the PARENT, not the repository
// read a sibling already written: what this section actually looks like
docs_create { … }                                                      // header filled from the schema
docs_metadata { "article_id": <the new one> }                          // did the fields land?
```

Two of those steps are the ones people skip, and both fail quietly:

- **The schema is asked of the parent**, never of the repository. Fields are
  declared per section: a repository-level answer gives you the common layer and
  loses whatever that section adds, so a perfectly valid value gets dropped.
- **The last check is not ceremony**, but read it right. A value that does not
  resolve is discarded while the write still reports success — and the trace of
  that is `front_matter_warnings` / `metadata_warnings`, not the gap between
  `front_matter` and `metadata`. On a managed documentation the header is what
  travels, so a value in `front_matter` and not in `metadata` is queryable-later,
  not lost.
- **A name the descriptor does not declare does not warn: it fails the call.** On a
  document that belongs to a repository, a `metadata` key or a header key outside
  what the documentation manages unwinds the whole write, so the schema call above
  is the difference between a document created and nothing at all. And declaring a
  field is not something any tool can do — it is a change to the `knowledge.yml`
  in Git.
