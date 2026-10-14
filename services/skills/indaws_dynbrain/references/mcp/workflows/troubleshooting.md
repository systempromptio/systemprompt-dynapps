# Workflows — when something does not add up

What to look at when the answer and reality disagree, and how to say it to the user without
either alarming them or claiming more than happened. The model behind all of it is
`../pending-work.md`; the index of every workflow is `README.md`.

The thread running through every entry: **the failures here are silent by nature** — the
user believes something happened that did not, and every answer looked green.

---

## 1. "It says it's done" — is it?

Four answers look like success and are not the whole story:

```text
docs_trash → { trashed: true, publish_required: true }
```
The page left Knowledge; its file did not leave the repository. The removal is pending
(`authoring.md` §4).

```text
docs_trash → { trashed: true, publish_required: false }
```
Worse, and the one that reads like good news. On a `mkdocs`/`rst` documentation the file
**stays in the repository permanently** — nothing in these tools will ever remove it, and the
next import can bring the page back. Say that, don't leave the user waiting for a publication
that is not coming. Same split on `docs_move`.

```text
docs_update → { updated: true, metadata_warnings: ["owner: no user matches 'j.smth'"] }
```
`updated: true` with a warning means **that field was not written**. Same for
`front_matter_warnings`. They are the only trace, so pass them on.

```text
docs_sync_status → pending_publication: { waiting: false }
```
"Nothing waiting to be sent" — which also covers work already handed to a change proposal and
not yet merged. It is not "the text is live". If the user needs to know whether the change is
in the branch everyone reads, that is a question about the proposal, and on a server without
the publish family the honest answer is *"I cannot see that from here — the repository can."*

---

## 2. Work that has been pending far too long

Written days ago, and the repository still does not have it. Read before guessing:

```text
docs_sync_status { space: 5 }
  → pending_publication: { waiting: true, documents: 3, files: 0 },
    pending_since: "2026-08-18T02:00:00", last_error: "…"
```

Three things to check, in order:

1. **`last_error`** — one redacted line, the last failure of the circuit. Relay it; the full
   log lives in Odoo.
2. **Did you move, rename or retire a page something else still links to?** A change that
   would leave an internal link pointing at nothing **aborts the whole publication**, every
   pass, before any commit — deliberately, because a reader finding the hole is worse than a
   lost pass. Nothing is lost and nothing self-heals: the content has to change. Fix the
   pointing pages (`docs_update` on each, with the new path) or put the target back. The
   message naming the links is only produced by a caller-run publication, so on most servers
   you have to reason it out from what you did rather than read it (`authoring.md` §7).
3. **`pending_since`** — a backlog clock stamped by the publication. Long and growing, with no
   `last_error`, points at a circuit nobody is running rather than at your change.

**Do not "re-save" the document to nudge it.** Re-writing the same text changes nothing about
why the pass is refusing, and it re-marks the document for indexing for no reason.

---

## 3. A document that never gets indexed

`docs_metadata` → `indexed: false` / `chunk_count: 0` long after the write. Indexing is
asynchronous and belongs to the system, not to your call, so first give it time. If it stays
that way, it is a Knowledge-side configuration matter — RAG not configured, the document not
eligible, no embeddings provider — and **not** something a rewrite fixes. Say so and let the
user check the configuration; re-creating or re-editing in a loop only adds noise. Details in
`../write-tools.md` → "Authoring so the document gets indexed (RAG)".

One thing that *is* yours: a document with no real Markdown content has nothing to chunk, and
a translation with no `description` of its own is a page nobody finds **in that language**
even when the document is perfectly indexed.

---

## 4. A write that was refused whole

```text
docs_create → { "error": "\"audience\" is not a metadata field of this documentation.
                Declared fields are set in knowledge.yml of the \"Dyncloud docs\"
                repository -- add it there and synchronize, or edit the file in Git.
                Nothing was written: Declared fields are: owner, reviewed_on, status." }
```

**Nothing exists.** Not the document, not the good keys, not half a subtree — the call unwinds
together, which is the right behaviour and the reason to read the message rather than retry.

Two shapes, both about a name the documentation does not declare:

- **a `metadata` key** — the message lists the declared fields. Call `docs_metadata_schema`
  **anchored on the parent** (fields are per section) and use those names.
- **a header key** — on a Git-backed document the writable set is `title`, `icon`,
  `description` plus the declared typed fields, and nothing else. A `weight: 3` in
  `front_matter` is refused on a documentation that does not declare `weight`, whatever any
  example shows.

**Neither is fixed by trying another spelling, and neither is yours to fix.** Declaring a
field means editing the repository's `knowledge.yml` in Git — no tool here can do it. Say
that, and offer the write without the key if the user wants the document now.

And the good news worth saying: a refused call does **not** burn a single-use
`content_upload_id`. Fix the argument, call again with the same id.

---

## 5. The text in Knowledge still looks old

A change that reached the repository does not repaint Knowledge on its own: Knowledge shows
what it last read, so the new text appears on the documentation's next import. Say that
plainly instead of letting the user conclude something failed — and **never re-save the
document to "make it stick"**, which only creates a second pending change of the same page.

**And do not offer to synchronise.** Importing is the other direction: on a documentation
Odoo owns it **replaces what Odoo holds**, so somebody reaching for it while anything is
pending is about to overwrite the very work they were trying to see. It is a button on the
repository's form, for whoever administers repositories (`can_synchronize`), and reporting
what `incoming` holds is the whole of your part.
