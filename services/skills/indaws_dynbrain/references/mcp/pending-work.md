# What happens to what you wrote — pending work, and who publishes it

Some documentation does not live in Knowledge: it is **imported from a Git repository**, and
Knowledge holds it. A change to such a document is **saved in Odoo the moment the tool
returns** and **held pending** until the documentation's next publication carries it to Git.

**You are not the one who publishes.** That is the whole shape of this page. The publication
is a scheduled pass, and whoever administers the repository can also launch it; on most
servers the tools that would let a caller do it by hand are not even in your tool list. What
you owe the user is an honest sentence about where their work stands — not a step you cannot
take.

---

## 1. Four states, and the two that read alike

An edit of a `managed` document is always in exactly one of these. The first three are what
you will be asked about; the fourth is the one that confuses counters.

```text
1. PENDING       written in Odoo, in no repository. Saved. Nobody outside Odoo sees it.
      │          (docs_create / docs_update / docs_move / docs_trash / docs_asset_upload leave it here)
      ▼
2. SENT          handed to a change proposal by the publication. In Git, on its own branch,
      │          with a URL. The official documentation is untouched.
      ▼
3. MERGED        accepted into the documentation's branch. Now the official text.
      │
      ▼
4. ABSORBED      read back into Odoo, and the document is level with its file again.
```

**States 2 and 3 both report "nothing pending".** The pending set is *edited **and** not yet
delivered*, so the moment the publication hands the work to a proposal it stops counting —
while the change is still not in the branch anybody reads. So `pending_mine: 0` and
`pending_publication.waiting: false` answer *"nothing is waiting to be sent"*, never *"it is
in the documentation"*. If the user's question is "is my change live?", the answer lives in
the proposal, not in a counter.

**A native document has none of these states.** It never travels: saving it is the whole
change. Check the origin first (`docs_metadata` → `repo` / `can_write`, table in `SKILL.md`)
— walking somebody through "pending → sent → merged" for a document that only lives in
Knowledge is a detour towards a URL that will never exist.

**And "a change" means any change, not just edited text.** Creating a page, moving it,
renaming it, retiring it or attaching a file changes the documentation, and the documentation
lives in the repository too.

## 2. What the repository's format changes, and it is not a detail

`docs_move` and `docs_trash` answer with a `publish_required` boolean and a `next_step`
sentence. **Read them; they are not always what you expect:**

| Format | A move or a removal | What to say |
| --- | --- | --- |
| `knowledge` | `publish_required: true` — the tree lives in the descriptor, and the descriptor is published | "It is out of the documentation in Odoo now; the file leaves the repository with the next publication." |
| `mkdocs`, `rst` | `publish_required: **false**` — their tree is never written back | "It is out of the documentation in Odoo. The file stays in the repository and nothing here will remove it — that has to be done in Git." |

The second row is the one people get wrong, and it costs a page that vanishes from search
while its `.md` sits in the client's repository forever. The answer says so in `next_step`;
say it too, rather than leaving the user waiting for a publication that is never coming.

Edited **text** is different: it is published from the file, so it travels on any managed
format. It is the *shape* that only a `knowledge` documentation carries back.

## 3. Anything an assistant wrote waits for a person

A change proposal that carries work produced through this MCP is **deliberately held**. It
comes back with `waiting_because: "awaiting_review"` and a `waiting_note` that spells it out:
a person has to read it and approve it **on the platform** before it can merge unattended.
Nothing else moves it, and retrying moves nothing either.

That is not a fault to report as one — it is the point. What it changes is the sentence you
say:

> "It is saved and it will go out with the next publication. Because I wrote it, the change
> waits for one of you to approve it in GitLab before it becomes the official documentation —
> it will not merge on its own."

Never say "it goes out tonight" as if that were the end of it.

## 4. The honest read: `docs_sync_status`

One call, both directions, no network and no clone — so ask it as often as you need
(`read-tools.md` → `docs_sync_status`).

- **`pending_publication: {waiting, documents, files, refs, note}`** — Odoo → Git. `waiting`
  is counted the moment the work is written, not when a publication first notices it, and
  `note` says what carries it. This is the field to read before telling anyone where their
  work stands. A **mirror** always answers "nothing waiting", correctly: nothing leaves a
  mirror.
- **`incoming` / `remote_ahead` / `up_to_date`** — Git → Odoo, and `up_to_date` answers **that
  direction only**. It says nothing about work waiting to go the other way.
- **`pending_since`** is a backlog clock stamped by the publication ("how long has this gone
  unpublished"), **not** the answer to "is something waiting". Do not read one as the other.

## 5. Git in five sentences (for a non-developer)

Most people you do this for have never used Git and should never have to. Use these when a
user needs the vocabulary; don't lecture, say the one that unblocks them.

- **Repository** — the folder where the official documentation lives. Knowledge holds it; the
  repository is the copy that survives Odoo and travels to other systems.
- **Commit** — one saved change with a note explaining it. Like a labelled version in a
  document's history, and it never disappears.
- **Branch** — a private copy of the documentation where changes are prepared without anyone
  else seeing them yet.
- **Change proposal** — the request to bring that branch into the official documentation.
  GitLab calls it a **merge request**, GitHub a **pull request**; same thing. Anyone can read
  exactly what would change.
- **Merge** — accepting the proposal. From that moment it is the official documentation.

**Never invent the word.** Every answer that mentions a proposal carries `platform` and `kind`
(`merge_request` / `pull_request`); use the one that matches. With a non-technical user,
"change proposal" plus the link beats either.

---

## 6. The publish family — four tools your server may not have

**Check your own tool list before reading further.** `docs_changes_status`,
`docs_propose_changes`, `docs_review_proposal` and `docs_merge_proposal` are gated as a family
and a server exposes them or does not. On a server without them, **nothing in this section can
be called**, the rest of the skill is what applies, and no answer you get will name them
either — the tools were deliberately cleared of pointing at a tool that may be absent.

Where they do exist, three rules govern all four:

- **Only when the user names it.** "Publish this", "send it to the repository", "merge the
  proposal". Never because an answer mentioned a repository, and never as an offer.
- **Show what is being approved, in words, before the URL.** `docs_review_proposal` gives a
  summary with one line per file and no diff text: *"3 new pages, 1 changed and **2
  deleted**"* is the sentence that makes a go-ahead informed. A deletion never travels inside
  the word "changes".
- **Commit messages and proposal titles are ALWAYS in English**, Conventional Commits
  (`docs: …` for documentation content), per
  `developer:skills/git-manager/references/commit-standards.md`. Show the message inside the
  same question you use to ask permission — they catch a wrong claim in a second and the
  repository keeps it forever.

### docs_changes_status (read-only)

Where every change stands, per repository: `pending_mine` / `documents_mine`,
`pending_files_mine` / `files_mine`, `structure_change` (a document created, moved, renamed or
trashed — publishable work that no `pending_*` counter shows), the others' equivalents, and
the open `proposals` with their live state.

```json
{ "repositories": [
    { "repo": "Dyncloud docs", "repo_id": 4, "platform": "gitlab", "kind": "merge_request",
      "pending_mine": 2,
      "documents_mine": [ { "article_id": 3852, "title": "OpenBao", "path": "docs/openbao.md",
                            "edited_by": "David Gómez" } ],
      "pending_files_mine": 1,
      "files_mine": [ { "article_id": 3852, "document": "OpenBao", "name": "policy-flow.svg",
                        "path": "", "size_bytes": 18422, "attached_by": "David Gómez" } ],
      "pending_others": 1, "pending_files_others": 0,
      "structure_change": { "pending": true, "edited_by": "David Gómez", "mine": true, "note": "…" },
      "proposals": [ { "proposal_id": 17, "url": "https://gitlab.…/-/merge_requests/42",
                       "ref": "42", "state": "open", "target_branch": "main",
                       "waiting_because": "awaiting_review", "waiting_note": "…",
                       "files": ["docs/openbao.md"],
                       "file_changes": [ { "path": "docs/openbao.md", "status": "modified" } ] } ] } ] }
```

So "is there anything to publish?" is **`pending_mine > 0` OR `pending_files_mine > 0` OR
`structure_change.pending`** — never the first alone. `files` is the diary of every path Odoo
has sent the proposal; `file_changes` says what merging the whole thing does to the branch
(`added` / `modified` / `deleted`, read from Git, absent when it could not be computed — then
list the paths and say you cannot tell what each one is, rather than letting them default to
additions).

An open proposal says why it has not landed in `waiting_because`: `checking` (the platform is
still deciding — seconds to minutes, resolves itself), `awaiting_review` (§3 — a person, on
the platform), `held` (larger than this documentation merges unattended — also a person),
`unreachable`, `merge_refused`. **Say which one**: three resolve themselves and two never
will.

### docs_propose_changes (writes to the Git remote — ask first)

Commits your pending edits, pushes a branch, opens or extends the proposal. It does not touch
the documentation's own branch. Only your own edits go (others come back in
`skipped_other_authors`); calling it again adds a commit to the same open proposal.

**`committed` counts real commits.** `committed: 0` means nothing was published, and
`nothing_to_commit` says which of the two it was — everything already matched the branch (good
news) or the push did not go through and the work is **still pending** (`still_pending > 0` is
the tell). They read alike and mean opposite things. Never turn either into "published".

A broken internal link **refuses the whole publication** before the commit: `committed: 0`
with the links named. Nothing is lost and re-running changes nothing — the content has to
(`workflows/troubleshooting.md` §7).

### docs_review_proposal (read-only)

Summary by default: `summary {files, additions, deletions, commits}` and one row per file
(`{path, status, additions, deletions}`), **no diff text**. `paths: ["docs/a.md"]` for the
diff of what you actually need; `include_diff: true` is capped by `max_diff_bytes` (default
20000) and over the cap sends **nothing**, naming the biggest files instead — a diff cut in
half without saying so is how half a change gets approved.

**`files_incomplete: true` changes what you may say.** The platform returned only a prefix of
the file list, so any count would be confident and wrong. Don't count: say the list came back
incomplete and hand over the URL. It is the one case where sending somebody to GitLab is the
right answer.

Verify before narrating: something of yours missing, or something extra with no other author
in `authors`, **stops** the publication; extra work that *has* an owner does not — say it out
loud and let the approver decide. An unexpected deletion stops it always.

### docs_merge_proposal (the change becomes official)

Ask for an explicit go-ahead, **separately** from the go-ahead to propose. Identify it with
`proposal_id` or `url`, or let a scope with exactly one open proposal speak. It never forces:
a conflict comes back as `state: "conflict"` with the URL where a human resolves it, and
nothing is lost. `merged: true` is the only thing that means published.

**When it refuses for `awaiting_review` or `held`, it refuses every time.** Read
`waiting_because` rather than retrying — there is no other way to merge it from Odoo, and
saying "let me try again" wastes the user's time on a door that is closed on purpose.

**After a merge Knowledge still shows the old text** until the change comes back. That is not
a failed merge.
