# Gotchas — the traps this MCP actually sets

Every one of these has bitten somebody. They share a shape: **a call succeeds, the answer
looks like success, and something the user believes happened did not.** Read this before
reporting a result back, not after.

## Writing

- **A second language is NOT a second document.** `docs_update {article_id, lang}` on the
  SAME document. Two `docs_create` calls give two unrelated documents, two files, two tree
  entries — both calls succeed and nothing warns you. The single easiest mistake to make
  with these tools (`mcp/write-tools.md` → "A language is a VARIANT").
- **`docs_create {lang}` inside a repository does not create a translation.** It creates the
  document, in the repository's own base row, with the canonical file — because an addition
  has no text to translate. The answer says which variant it made; read it. The translation
  is the `docs_update {lang}` that comes after.
- **A title has two doors, and an explicit one wins in both places.** The `title` **parameter**
  names the ARTICLE and is realigned into the header too, so the file and the document cannot
  disagree. A `title:` in a **pasted header** with no parameter beside it renames the document
  — it has to, because that header is what gets committed and the next import reads it. Say
  the name out loud when it changed, and check it with `docs_metadata` rather than trusting
  the echo.
- **`front_matter_warnings` means keys were dropped**, even though the answer says
  `updated: true`. Pass them to the user — they are the only trace of what was not saved.
- **A move does not rename the file.** It changes the page's place in the descriptor; the
  folder and file name stay. Every reader of the documentation still sees the new place.
- **A link between pages is a relative `.md` path, never an Odoo URL.** `/odoo/knowledge/
  article/<id>` renders fine on screen and reaches the repository as a dead reference to a
  database no reader of the file has (`mcp/workflows/authoring.md` §7).
- **A title-only `docs_create` still writes a file** — a `# Title` stub, because a tree
  entry pointing at a missing file is removed by the next import.
- **A refused write leaves nothing behind, upload token included.** The whole call unwinds:
  no half-built document, no half-applied metadata, and the single-use `content_upload_id` it
  would have consumed is still good. Fix the argument and retry with the same id.

## The two refusals that fail the whole call

Both only on a document that belongs to a **repository**; on a native document they are
warnings and the header stays permissive.

- **A `metadata` key the documentation does not declare.** Nothing is written — not even the
  keys that were fine. **No tool can declare a field**: it is added to the `knowledge.yml` in
  Git, and the error message says so rather than pointing at a tool that does not exist. So
  the fix is never "try another spelling", it is `docs_metadata_schema` first.
- **A header key outside what the documentation manages.** What Odoo may write into a
  Git-backed header is **`title`, `icon`, `description` plus the declared typed fields**, and
  nothing else. `front_matter: {weight: 3}` is refused on a documentation that does not
  declare `weight` — even though `weight` is a perfectly ordinary front-matter key and the
  tool's own examples show it. It is not a bug: the published file is composed from that
  closed set, so a key outside it would sit in Odoo forever and never reach the file.

## Typed metadata

- **A `selection` takes the KEY, not the label.** `"approved"`, never `"Approved"` — and
  this is the one that fails *while looking right*, because the label is what every
  interface shows. `docs_metadata_schema` → `options` lists the keys.
- **A relation takes TEXT, never an id and never a display name.** The field it travels as
  is in `expects` (`res.users.login` → the login). An id would mean nothing in the
  customer's repository, and a display name matches nobody.
- **A date and a free-text field are NOT VALIDATED.** "yesterday afternoon" in a `date`
  property is accepted, stored, and committed to the customer's file exactly as typed. There
  is no warning, and Odoo's own form does not paint a date property holding text — so nobody
  downstream will spot it either. `expects` is an instruction to you, not a guard.
- **Writing and filtering do not speak the same language.** `docs_update {metadata}` takes
  the text; `where` matches the stored **id**. Both correct, not interchangeable — take the
  id from `docs_metadata` → `metadata`.
- **`metadata_warnings` means those fields were NOT written**, though the answer says
  `updated: true`. Same contract as `front_matter_warnings`, and the same duty: pass it on.
- **"this document declares no metadata fields" is about its PARENT.** The typed
  fields are read from the immediate parent and inherit nothing, so a document
  written under a section that had none carried none either — its metadata
  complete in the header and invisible in Odoo. Creating a document now gives
  that parent the fields, so if you still meet this warning the documentation
  genuinely declares none: check `docs_metadata_schema` before assuming a bug.
- **An ambiguous lookup leaves the field empty and says so.** Two people match that login,
  nothing is written — never the first match, which would file a document under a
  colleague's name.
- **`false` clears a field**, and a cleared field leaves the header rather than sitting in
  it as `null`. Clearing works even on a document with no repository behind it.
- **A declared field inside a pasted `.md` header IS written**, through the typed layer. So
  a paste can change `owner` without anybody typing `metadata` — read `metadata` in the
  answer to see what it moved. The one place a typed field is refused is the header of a
  **translation**, and `metadata_warnings` says so.

## Pending work, and who publishes

- **You do not publish, and on most servers you cannot.** The four publish tools are a family
  a server exposes or does not. If you are about to name `docs_propose_changes`, check your
  own tool list first: no answer from the read or write families will ever have named it to
  you (`mcp/pending-work.md` §6).
- **"Pending" is not "unsaved".** The change is stored the moment the tool returns and
  nothing is at risk while it waits. Telling somebody their work "does not count yet" or
  "could be lost" is false, and it makes people re-type what they already have.
- **"Nothing pending" also covers work already sent.** The pending set is *edited and not yet
  delivered*, so the moment a publication hands the work to a proposal it stops counting —
  while the change is still not in the branch anyone reads. `pending_mine: 0` and
  `pending_publication.waiting: false` mean "nothing waiting to be sent", never "it is live".
- **`up_to_date` answers the other direction.** It is Git → Odoo. `up_to_date: true` beside
  pending work is a normal, consistent pair, and quoting it as "everything is in Git" is the
  classic mix-up. So is `pending_since`, which is a backlog clock stamped by the publication,
  not the answer to "is something waiting".
- **`trashed: true` is not "removed from the documentation".** The file is still in the
  client's repository. On a **`knowledge`** documentation the removal is pending; on a
  **`mkdocs`/`rst`** one `publish_required` is **false** and the file **stays there forever**
  — nothing in these tools will ever remove it. Read the boolean and the `next_step`, and say
  which of the two it is.
- **A move on a `mkdocs`/`rst` documentation never leaves Odoo either.** Same split, same
  `publish_required: false`, same duty to say so instead of leaving the user waiting.
- **Anything an assistant wrote waits for a person.** A proposal carrying work produced
  through this MCP comes back `waiting_because: "awaiting_review"` and does not merge
  unattended, however many times anything asks. That is the design; say it rather than
  "it will go out tonight".
- **An import is not what rescues pending work.** It is the other direction, and on a
  documentation Odoo owns it **replaces what Odoo holds**. Somebody reading "waiting: true"
  and reaching for a synchronisation is about to overwrite the work they wanted to save.
- **An attached file is publishable work with no edited text behind it.** It counts nowhere a
  text counter looks; read the `file` entry's own `pending_publication`.
- **A broken internal link stops the WHOLE publication, and you will not see the message.**
  A change that would leave a link pointing at nothing aborts the pass before the commit —
  deliberately: losing a pass is cheaper than a reader finding the hole. Nothing is lost and
  the work stays pending, but the pass that reports it in words is the one a caller runs, and
  you probably do not have that tool. So when work has been pending far longer than a
  publication cycle, read `docs_sync_status` → `last_error` and `pending_since` before
  concluding the mechanism is broken, and check whether you moved, renamed or retired a page
  something else still links to. A link that was **already** broken blocks nothing.

## Files

- **A file attached is not a file visible.** `docs_asset_upload` puts the bytes there;
  nothing shows until the page **links it**, by its relative name, with `docs_update`
  (`![](diagram.svg)`). Publication moves the file; it does not write that link. Never write
  an attachment id into the prose — it works in Odoo and nowhere else.
- **`attached: false` in the first answer is not a failure.** It is the honest answer until
  the second call; the first one only hands you the `upload_url`.
- **The URLs expire in 15 minutes and the upload one is single use.** If the person takes
  longer, issue another. Retrying the old one gets "Invalid or expired upload link."
  forever.
- **Re-uploading the same name does NOT replace the file.** New bytes land beside the old
  ones, numbered (`diagram-2.svg`), and the page keeps linking the old file.
- **An SVG carrying script, `on…=`, `javascript:`, `<foreignObject>` or an entity is
  stored and published, but not labelled as an image** — so it never renders inline. If
  somebody uploads a diagram and it does not show, that is why.
- **A diagram pasted into the Markdown as `<svg>…</svg>` empties out when the page is
  edited in Odoo.** The body is sanitised HTML: the wrapper survives and the drawing
  inside it does not, and only an IMPORT extracts it into an attachment. So a diagram
  written or changed in Odoo renders as a HOLE until the change reaches Git and comes
  back. Upload it as a `.svg` file and link it by name (`mcp/workflows/files.md` §5).
- **`in_repository` means two different things in one answer.** At the top of
  `docs_asset_list` it is the document's; inside a `files[]` row it is the file's. Read the
  row's pair with `pending_publication`.
- **`in_repository: false` + `pending_publication: false` means nothing will ever publish it.**
  On a native document that is right and complete; on a Git one it is a file that fell out of
  the flow.
- **No tool of yours moves the bytes.** `curl` (or the user's browser, on the URL) does. To
  *read* a PDF or an image, download it to disk and open it with the file-reading tool —
  never paste bytes into the conversation.

## Reading

- **`markdown` from `docs_get` is the file's canonical text**, with no generated index; the
  children come separately in `children`. Never paste that list back into a body you send
  to `docs_update` — it would be committed as if a person had written it.
- **Empty results can mean "no access"**, not "nothing there": every call runs as the
  calling user.
- **A `lang`/`version` you ask for may not exist** — the answer serves the default and says
  so in `variant`. Check it before quoting the text as that language's.
- **A language with no translation of its own is still served.** A variant row can hold the
  base language's text as a stand-in, so "there is content in English" is not proof that
  anybody translated it. Read it before calling it a translation.
- **After a server deployment, reconnect `/mcp`.** The tool list and behaviour are
  negotiated when the session opens, so an old session keeps the old tools — including bugs
  fixed hours ago and fields that "do not exist yet".

## Headers and typed fields

- **The header is the FILE's; the typed fields are the DOCUMENT's.** A translation carries its
  own `title` and `description` (`docs_update {lang, front_matter}`), while `doc_type`,
  `territory`, `categories` or `owner` are written **without `lang`** — sent inside a
  translation they are refused and reported in `metadata_warnings`, because there is one
  article to hold them and the last language edited would otherwise win.
- **A `title` inside a variant's header also renames the document.** The variant keeps its
  own title in its own file, but the document's name follows the last write; restore it with
  a bare `docs_update {article_id, title}`. A `docs_update {lang, title}` reports **that
  translation's** file, which is the one you renamed.
- **The field definition lives in the immediate parent and is NOT inherited.** Ask
  `docs_metadata_schema` anchored on the parent you are writing under: asked of the
  repository it returns the common layer only, and the values a section adds are silently
  dropped on write. And the two halves do not arrive the same way: the documentation's
  **common block** reaches a new parent on its own, while what a descriptor declares for **one
  specific node** keeps coming from the descriptor. So a field that a section
  declares can be missing on a document created under it — that is the design, not a fault.
- **`front_matter` fuller than `metadata` does NOT mean the values were lost.** The first is
  the open half (whatever the author wrote), the second the typed half (only the fields the
  documentation declares AND has installed on that node). On a **managed** documentation what
  travels is the file's header, so a value sitting in `front_matter` will reach Git even while
  `metadata` is empty — typically because the field definition is not on that parent yet, and
  it arrives with the import. Read it as "not queryable yet", not as "not saved". The reading
  that does mean lost is `front_matter_warnings` / `metadata_warnings`: those keys were
  refused.
- **`icon` is not a property and not part of the title.** It is a header key that the tree
  reads; `docs_metadata_schema` never lists it and `metadata` cannot set it — it goes in the
  header, or through `front_matter`. An `icon:` in a `.en.md` is ignored, because the article is
  one record for every language. It is painted on the write, so the tree shows it at once, and
  a header with no `icon:` leaves whatever is there alone — removing one is writing `icon: ""`.
- **A translation's empty `description` is the gap, not a bug.** There is deliberately no
  inheritance: the description is what the index reads, and lending it the original's would
  make the page look described while being unfindable in its own language. Fill it; do not
  report it as a defect. (A missing title is different — it is derived from the translation's
  first heading.)
- **Reading with `lang` serves THAT variant's title, description and header.** `docs_search`
  titles each hit in the language of the passage it returns, and `docs_list` / `docs_outline`
  take `lang` — a document with no such translation **keeps its own title there** rather than
  disappearing, which is what makes the tree usable for seeing what is left to translate. In
  `docs_list` the `path` stays the DOCUMENT's (it is what `docs_get` looks up); the variant's
  file is beside it in `source_path`.
- **A YAML value containing `:` breaks the header** and the write fails with a parser error
  about mapping values — quote any `description` that carries a colon. Passing it through
  `front_matter` instead of a hand-written block avoids the whole class.
- **There is no `restore` tool.** A document already trashed has to be brought back through
  the ORM.
