# Documentation MCP — workflows (index)

The tools are most useful **in combination**. A single call rarely answers a real task; the
skill is knowing how to chain them. Each recipe is a concrete sequence with the actual calls
and the reasoning behind the chaining. Tool details are in `../read-tools.md` /
`../write-tools.md` / `../pending-work.md`; shared conventions in `../README.md`.

Golden thread: **orient → scope → drill → read/act.** Never dump a whole corpus, tree or
diff; move deliberately, using the cheap structural tools (`docs_list_spaces`,
`docs_outline`, `docs_metadata`) before the expensive content ones.

**This page exists so you can choose a recipe without opening anything.** Read the line,
open the one file you need.

## `reading.md` — find, read, verify

| § | Workflow | Open it when |
| --- | --- | --- |
| 1 | Orient in an unknown corpus | You don't know what documentation exists |
| 2 | Discover a big tree without pulling it all | Hundreds of pages; you need the shape, not the text |
| 3 | Narrow a search that returns junk | Hits are off-target or span the wrong documentations |
| 4 | Answer a question and verify the source | You will act on the answer, so the citation must be read |
| 5 | From a vague ask to the authoritative document | Several docs say it; you need the official one |
| 6 | Ask the corpus about itself (typed metadata) | "What is unowned / still a draft / unreviewed" — not a question about prose |

## `authoring.md` — create, edit, reorganize, retire

| § | Workflow | Open it when |
| --- | --- | --- |
| 1 | Author a whole section | Laying down a new documentation or a new branch of one |
| 2 | Edit an existing document | Correcting or extending a page |
| 3 | Reorganize: move a document | Changing where a page hangs — **works on Git documentations too** |
| 4 | Retire a page | Removing a page — and what the repository's **format** decides about it |
| 5 | Set the header and the typed fields | A description, an icon, an owner, a status — **without resending the body** |
| 6 | Move large content out-of-band | A document too big to travel through the model channel |
| 7 | Link one document to another | Writing a cross-reference — and what a move does to the links |

## `files.md` — the files a documentation carries

| § | Workflow | Open it when |
| --- | --- | --- |
| 1 | What files does this document have | The page shows a diagram, a PDF, a capture |
| 2 | Download a file — and read it | You must see what is inside, or the user wants it |
| 3 | Attach a file to a document | Adding an image, a diagram, a PDF — **two calls, and the bytes are not yours to carry** |
| 4 | A new version of an already published file | Re-uploading the same name does **not** replace it |
| 5 | An SVG that does not show | It uploaded, it published, it does not render |

## `pending.md` — work that is written and not yet in the repository

| § | Workflow | Open it when |
| --- | --- | --- |
| 1 | Finish an edit honestly | You edited a document of a `managed` repository and have to say where it stands |
| 2 | A change of shape, or an attached file | You created, moved, renamed, retired or attached — and the **format** decides what happens next |
| 3 | "Did my change go through?" | The user asks, and the two directions are easy to mix up |
| 4 | A mirror, and a document going nowhere | The tools refuse, or nothing will ever publish it |
| 5 | Synchronising is not yours | The repository has moved on and somebody has to import it |

## `troubleshooting.md` — when something does not add up

| § | Workflow | Open it when |
| --- | --- | --- |
| 1 | "It says it's done" — is it? | `trashed: true`, `publish_required: false`, a counter reading 0 |
| 2 | Work that has been pending far too long | It was written days ago and the repository still does not have it |
| 3 | A document that never gets indexed | It stays out of `docs_search` long after being written |
| 4 | A write that was refused whole | The call failed and nothing at all was created |
| 5 | The text in Knowledge still looks old | The change reached Git and the page did not change |

## Composition cheatsheet

| Goal | Chain |
| --- | --- |
| **Where does this document come from?** | `docs_metadata` → `repo` + `can_write` → native / managed / mirror (decide **before** writing) |
| What docs exist? | `docs_list_spaces` |
| Explore a documentation's shape | `docs_list_spaces` → `docs_outline` (descend, page/drill) |
| Find the right "root" from one page | `docs_outline` (ascend) → `recommended_ceiling` |
| Answer a question | (`docs_list_spaces` →) `docs_answer` → `docs_get` a citation |
| Find + read a doc | `docs_search` → (`docs_metadata` →) `docs_get` |
| De-noise a search | `docs_list_spaces` → `docs_search { space }` → sharpen terms |
| Author a section | classify → `docs_metadata` (can_write) → `docs_metadata_schema` (the **parent**) → `docs_create_tree` → `docs_metadata` |
| Edit / reorganize | `docs_metadata` (can_write) → `docs_update` / `docs_move` / `docs_trash` → read `pending_publication` / `publish_required` |
| Add a second language | `docs_update {article_id, lang, markdown}` → `docs_update {article_id, lang, front_matter:{description}}` → `docs_metadata {article_id, lang}` |
| Set a header key or a typed field | `docs_metadata_schema` → `docs_update {front_matter}` / `{metadata}` → read both `*_warnings` |
| Retire a page | `docs_metadata` (origin) → `docs_trash` → read `publish_required` **and** `next_step` |
| Where does my work stand? | `docs_sync_status` → `pending_publication` (Odoo → Git), never `up_to_date` (Git → Odoo) |
| Paste a whole `.md` file | `docs_update` → read `front_matter` (renamed?), `front_matter_warnings` and `metadata` (what the header moved) |
| What files does this page carry? | `docs_asset_list` → read `in_repository` **+** `pending_publication` as a pair |
| Read a file (a PDF, an image) | `docs_asset_link` → `curl -o` → open it with the file-reading tool |
| Add a file | `docs_asset_upload {filename}` → POST the bytes → `docs_asset_upload {attachment_id}` → `docs_update` with a **relative link** |
| Large content | `docs_get`→`markdown_url` / `docs_request_upload`→`content_upload_id` / `docs_export`→`download_url` (curl) |
