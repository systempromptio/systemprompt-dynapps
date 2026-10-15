# Workflows — find, read, verify

Retrieval recipes: orient in a corpus you don't know, walk a big tree without dumping it,
de-noise a search, and check an answer against the document it came from. Tool details in
`../read-tools.md`; the index of every workflow is `README.md`.

Golden thread: **orient → scope → drill → read.** The structural tools are cheap
(`docs_list_spaces`, `docs_outline`, `docs_metadata`); spend the expensive ones
(`docs_get`, `docs_export`) only where they pay.

---

## 1. Orient in an unknown corpus

You don't know what documentation exists. Start wide, then narrow.

```text
docs_list_spaces {}                       # what documentations exist? ids + article_count
  → pick space 5 ("Platform")
docs_outline { space: 5, depth: 1 }       # its top-level sections (child_count each)
  → "Infrastructure" (id 3747, 6 children) looks right
docs_outline { node: 3747, depth: 1 }     # drill into that section
  → "Backups" (id 3865)
docs_get { article_id: 3865 }             # read the leaf
```
Why: `docs_list_spaces` gives valid `space` ids; `docs_outline` walks the tree cheaply
(titles only) so you spend a `docs_get` only on the page that matters.

---

## 2. Discover a big tree without pulling it all

A documentation with hundreds of pages. Don't `depth:-1` it — page and drill.

```text
docs_outline { space: 5, depth: 1, max_nodes: 50 }
  → response has truncated:true, has_more:true, hint:"narrow to a child or page with offset"
docs_outline { space: 5, depth: 1, offset: 50 }        # next page of top-level nodes
docs_outline { node: 3747, depth: 2, max_nodes: 100 }  # expand just the branch you need
```
And the reverse — you hold ONE page and need its section root (not the giant global
root):
```text
docs_outline { node: 3882, mode: "ascend" }
  → ancestors show subtree_count jumping 5 → 210 at the space root
  → recommended_ceiling: "Administration guide" (id 3881, 5) ← work from here
```
Why: `descend` with `depth`/`max_nodes`/`offset` keeps every response bounded;
`ascend` finds the natural boundary (where subtree size jumps) so you never accidentally
scope work to a huge aggregator.

A section page's children are also right there in `docs_get` → `children`, as data next to
the document rather than inside its `markdown`. Use that when you already hold the page.

---

## 3. Narrow a search that returns junk

Broad search is noisy → scope it, don't just rephrase blindly.

```text
docs_search { query: "how are backups done and how long are they kept" }
  → results span 3 unrelated documentations, low scores
docs_list_spaces { query: "platform" }        # find the right documentation's id
  → space 5
docs_search { query: "database backup frequency and retention", space: 5, limit: 5 }
  → tight, high-score hits from the right corpus
```
Escalation ladder when hits are still off:
1. **Scope** with `space` (biggest lever).
2. **Sharpen** the query with the concrete entities/terms ("CNPG", "Velero", a code, an
   identifier) — the keyword arm catches exact tokens.
3. **Switch tool**: if you want the fact, not a list, `docs_answer { question, space }`.
Leave `lang` off — a Spanish query finding English docs is a feature, not noise.

---

## 4. Answer a question and verify the source

Get a synthesized answer, then confirm it against the real document before you rely on it.

```text
docs_answer { question: "What is the backup retention and who can approve a publication?", space: 5 }
  → answer cites [1] admin/backups.md, [2] admin/roles.md
docs_get { path: "admin/backups.md" }          # open citation [1] to confirm the exact wording
```
Why: `docs_answer` is fast and cited, but for anything you'll act on, open the citation
with `docs_get` (by its `path`) to read it in full — the snippet is only a preview.

---

## 5. From a vague ask to the authoritative document

Several docs mention the topic; pick the right one with metadata.

```text
docs_search { query: "how do we handle client data residency", limit: 5 }
  → three candidate hits
docs_metadata { article_ids: [3754, 3762, 3765] }
  → 3754 is is_synced:true (official, imported), most recently updated, in the Trust space
docs_get { article_id: 3754 }
```
Why: `docs_search` ranks by relevance, but `docs_metadata` tells you which is
authoritative (source, space, author, updated, imported-vs-native) so you cite the right
one. It is also the call that tells you whether you could *edit* it — see
`authoring.md`.

---

## 6. Ask the corpus a question about itself (typed metadata)

Not "what does this say" but "which of these is in what state" — what nobody owns, what is
still a draft, what has not been reviewed this year. Retrieval cannot answer it; the typed
metadata can.

```text
docs_list_spaces {}                                  # which documentation are we auditing?
docs_metadata_schema { space: 7 }                    # what fields does it declare, and what do they expect?
  → status (selection: draft|approved), owner (many2one, expects res.users.login), reviewed_on (date)
docs_list { space: 7, where: { owner: false } }      # the inventory question: what is unowned
  → 14 documents
docs_update { article_id: 3884, metadata: { owner: "j.smith", status: "approved" } }
  → { metadata: ["owner","status"], metadata_warnings: [] }   # read the warnings, always
docs_list { space: 7, where: { owner: false } }      # 13 left
```

Why in that order: the fields are **declared per documentation and per section**, so
`docs_metadata_schema` before anything else — it is what tells you that `owner` wants a
*login* and `status` wants a *key* and not the label, and it is what keeps a name the
descriptor does not declare from **failing the whole write**. `docs_list` for inventory
(everything that matches, paginated), `docs_search { where }` when the question is also about
meaning ("escalation policies nobody owns").

Three things that bite here, all in `../../gotchas.md`: **`metadata_warnings` is a field that
did not get written**; **you set a relation by text but filter it by id** — take the id from
`docs_metadata` → `metadata`, not from the login you just wrote; and **a date is not
validated**, so `reviewed_on: "yesterday"` is stored and committed exactly like that.

On a **managed** documentation this is a real edit: it reaches the file's header, is saved in
Odoo and is pending publication. Say so (`pending.md` §1); the publication is not your step.

---

**Large documents** (`markdown_url` instead of inline `markdown`) and whole-documentation
exports are one recipe, shared with the write side: `authoring.md` §6.

**The page shows a diagram, a capture or a PDF?** `markdown` gives you the reference
(`![](red.svg)`), never the file. Reaching it — and reading what is inside it — is
`files.md` §1–2.
