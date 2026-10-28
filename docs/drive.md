# Drive

Drive is this install's view of core's shared document store. It is on when
`files.documents.enabled` is `true` in `services/config/files.yaml`; while it is
off, every page below answers 404 and no sidebar entry is shown.

## People

`/drive` is every signed-in person's drive, linked as **My drive** in the
Account group of the sidebar. It mounts core's `sp-document-browser`, which
switches between the person's own documents, the ones shared with them and the
ones they worked on last. **New document** opens a dialog that lists the
templates under `services/documents/templates/` and creates a document through
`POST /api/v1/documents`, then opens it.

`/drive/{id}` shows one document. The page asks the document service for the
viewer's access before it renders, so a person without a grant gets a 404. It
picks the body by format and access:

| Format | Can edit | Body |
|---|---|---|
| HTML, Markdown | yes or no | `sp-document-editor` (read-only without edit access) |
| xlsx | yes | `sp-sheet-grid` |
| pptx | yes | `sp-slide-text` |
| any other, or no edit access | | `sp-document-preview` |

Beside the body sit `sp-presence-bar`, `sp-comments`, `sp-proposal-list` and
`sp-history`. A person with `share` access gets a **Share** button that opens
`sp-share-dialog`. The components and their styles are core's, served from
`/assets/documents/`. This install sends no `Content-Security-Policy`, so the
editor's WebAssembly needs no `wasm-unsafe-eval` exception.

## Administrators

`/admin/drive` lists folders and documents. Each folder row has an **Agent
edits** control that sets the folder's `agent_edit_policy` through
`PATCH /api/v1/document-folders/{id}`: `direct` lands an agent's edits, and
`propose` turns them into proposals for review. `/admin/drive/{id}` adds
sharing, comments and proposals beside the document itself, which reads as a
page in a Dynapps-branded sheet and switches to an **Edit** mode that commits
block edits against the loaded revision. Owners, authors, grants and editors
show names through `/admin/drive/people`. **New document** on the listing
creates one from a template, `dynapps-word` first.

## Analytics

The skill and conversation pages under `/admin/analysis` show a **Documents**
panel: the documents the subject created or edited, with their revision count,
each linked to `/drive/{id}`. Revisions come from core's
`report_document_revisions` view. A conversation owns a revision that carries its
context id, and a skill owns the revisions of the conversations its harness
sessions ran: the sessions come from `analysis_skill_version_events`, the source
the skill page counts runs from, joined to their conversations through
`conversation_facts`. The overview shows a **Documents touched** tile: distinct
documents that gained a revision in the window.

## Smoke test

`just docs-smoke` runs the store end to end on a throwaway database, and the
Gates workflow runs it as the `docs-smoke` job.
