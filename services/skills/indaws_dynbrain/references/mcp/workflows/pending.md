# Workflows — work that is written and not yet in the repository

A change made through the MCP on a `managed` documentation is **saved in Odoo** and **held
pending** until the documentation's next publication carries it to Git. These recipes are
about saying that correctly, checking it when the user asks, and knowing the two cases where
"pending" is a word that does not apply. The model is `../pending-work.md`; the index of
every workflow is `README.md`.

**None of this applies to a native document.** It never travels: saving it *is* the change.
Check the origin before you start (`docs_metadata` → `repo` / `can_write`).

**You do not publish.** No recipe here ends in a publication step, and on most servers the
tools for one are not in your list at all. `../pending-work.md` §6 is the exception, for the
servers that do have them and only when the user asks by name.

---

## 1. Finish an edit honestly

```text
docs_metadata { article_id: 3852 }        # is_synced:true AND can_write:true → managed
docs_update  { article_id: 3852, markdown: "# OpenBao …\n\n(corrected policy steps)\n" }
  → { article_id: 3852, updated: true, path: "docs/openbao.md",
      pending_publication: true }
```

Then say it, in one sentence, about *their* work:

> "Corrected. It is saved in Odoo and goes to the repository with the next scheduled
> publication — nothing for you to do. Because I wrote it, somebody there has to approve it
> before it becomes the official version."

Three things not to say, each of which has cost somebody an afternoon: *"it is not saved
yet"* (it is), *"it could be lost"* (it cannot), and *"it is in the repository"* (it is not,
and `pending_publication: true` is the tool telling you so).

**Read the answer rather than your memory of the origin.** `pending_publication` is simply
absent on a native document, and its absence is what lets you say "done".

---

## 2. A change of SHAPE, or an attached FILE — and the format decides

Creating a page, moving it, renaming it or retiring it changes the documentation's *tree*.
No document comes back as "edited", so any counter of edited text reads 0 while there is very
much something waiting. But the tree only travels on one format, and the answer says which:

```text
docs_move { article_id: 3886, new_parent_id: 3879 }
  → { moved: true, repo_id: 4, publish_required: true,
      next_step: "… moved inside the documentation in Odoo only. Its file has NOT moved …
                  the new tree is held PENDING publication …" }
```
```text
docs_move { article_id: 3886, new_parent_id: 3879 }        # a mkdocs documentation
  → { moved: true, repo_id: 4, publish_required: false,
      next_step: "… moved in Odoo only, and stays that way: this repository publishes from
                  'mkdocs' files, which have no tree a publication can carry a move into …" }
```

`publish_required: false` is not "already done" — it is **"nothing will ever carry this to
Git"**. Say that. The same split governs `docs_trash`, where it is more expensive: the page
leaves Knowledge and its `.md` stays in the client's repository, permanently, on a
`mkdocs`/`rst` documentation.

**A pending FILE is the other half of this trap.** A diagram attached with
`docs_asset_upload` is publishable work with no edited text behind it; its `file` entry
carries `pending_publication` and the `message` says what carries it. It reaches the
repository next to the page that owns it — and only if a page **links it** (`files.md` §3).

---

## 3. "Did my change go through?"

Read it, do not reason it out.

```text
docs_sync_status { space: 5 }
  → { repositories: [ { repo: "Platform docs", sync_mode: "manual",
        pending_publication: { waiting: true, documents: 2, files: 1, refs: ["main"],
                               note: "… the documentation's next publication, which is
                                      scheduled and which a repository administrator can
                                      also launch." },
        up_to_date: false, remote_ahead: false, pending_since: "", last_error: "" } ] }
```

- **`pending_publication.waiting`** is the Odoo → Git answer, and the only one that answers
  their question. It is counted the moment work is written.
- **`up_to_date`** answers the *other* direction (is anything arriving from Git). Quoting it
  here is the classic mix-up: `up_to_date: true` with work pending is a perfectly normal,
  perfectly consistent pair.
- **`pending_since`** is a backlog clock — "how long has this gone unpublished" — stamped by
  the publication. Empty does not mean "nothing waiting".

**And `waiting: false` does not mean "it is live".** Work already handed to a change proposal
stops counting as pending while the proposal is still open (`../pending-work.md` §1). If the
user needs to know whether the text is in the branch everyone reads, that is a question about
the proposal, and on a server without the publish family the honest answer is *"I cannot see
that from here — the repository can."*

---

## 4. A mirror, and a document that is not going anywhere

```text
docs_metadata { article_id: 4021 }
  → { repo: "Upstream docs", is_synced: true, can_write: false }
```

Read-only: content flows Git → Odoo only, the write tools refuse it, and correctly so. Tell
the user the document is edited at its source and where that is (`source_url`). A mirror also
answers `pending_publication: { waiting: false }` forever, which is true rather than
suspicious: nothing leaves a mirror.

The other one that goes nowhere: a **file attached to a document with no repository behind
it** comes back `in_repository: false, pending_publication: false` — Odoo holds it and
nothing will ever publish it, which for a native document is exactly right and the whole job.

---

## 5. Synchronising is not yours, and it is not the opposite of publishing

There is no tool in this MCP that reads a repository into Odoo: it is a button on the
repository's form, for whoever administers repositories (`can_synchronize`). What you can do
is *report* the other direction:

```text
docs_sync_status { repo_id: 7 }
  → incoming: { known: true, file_count: 2, refs: ["main"], modified_only: true,
                files: [ { path: "docs/setup/install.md", document: "Installation",
                           article_id: 1234 },
                         { path: "docs/setup/limits.md" } ] }
```

Say *"the Installation page would change"*, not *"two files"*. `modified_only: false` means
files are being added or **removed**, and a removal never travels inside the word "changes".
`known: false` means no push described it — read `note`, and do not turn it into "nothing is
coming".

**Never offer the import.** On a documentation Odoo owns it **replaces what Odoo holds**, and
pending work is not something an import rescues — importing is the other direction. When
something genuinely has to be imported, name what `incoming` holds and who has to press it.
