# 2026-09-02 — Files categories and `add_knowledge_document` restored

**Status:** decided (Ed, 2026-09-02). Amends decision 4 of
`2026-09-01-knowledge-bank-remote-shared-index.md`, which dropped the file
sources and the PDF/DOCX parsers.

## Context

The Phase 0 requirements (`docs/migration/project-context/PHASE0-REQUIREMENTS.md`)
were checked requirement by requirement against the knowledge-bank crate and its
tests. Four functional requirements had no implementation at all after the
2026-09-01 reshaping: hand-authored notes and ADRs (FR-ING-1), spec and archive
categories as distinct sources (FR-ING-4), Markdown/text/PDF/DOCX intake
(FR-ING-5), and adding a document without a rebuild (FR-ING-6). The Node engine
being migrated has four of its six categories on disk (`meeting_notes`, `adrs`,
`specs`, `openspec_archive`), and the parity corpus for M1 lives there. Routing
that material through Confluence would have amended the requirements rather than
met them.

## Decisions

1. **`files` is a third `SourceKind`.** A files category reads a directory under
   `files.root` (`storage/knowledge`, the `app_knowledge` volume in the
   container) or its own `path`, selected by `include`/`exclude` globs. Markdown
   and text pass through; PDF and DOCX are extracted; other suffixes are skipped,
   never fatal. The listing is always complete, so a vanished file is deleted
   from the index on the next sync.

2. **Files documents are granted to configured subjects, not to the syncing
   caller.** They are team material with no upstream reader list. A category's
   `grants:` (kind + value from the `user | role | adfs_group` vocabulary)
   defaults to `role:user`, everyone signed in. Atlassian categories keep the
   per-caller grants and refuse a configured `grants:` at load, because it would
   widen access past what Atlassian enforced. A rebuild replaces a files
   document's configured grants; an incremental sync is additive.

3. **`sync_knowledge` needs Atlassian headers only for Atlassian categories.**
   Naming a Confluence or Jira category without them is refused; an unscoped sync
   without them runs the files categories alone and says so.

4. **`add_knowledge_document` is the sixth tool.** It writes the bytes to
   `<category>/uploads/<file_name>` atomically, then runs the same pipeline a sync
   uses and stamps the index, so the document is searchable at once and the next
   directory sync or rebuild converges on the same row. It is open to every
   signed-in user, because the result is readable by exactly that audience and
   every call is audited; `overwrite` of an existing name requires the admin
   role. File names are bare names with a supported suffix; uploads are capped at
   20 MiB.

5. **Open items O-1, O-2, O-3 and O-5 are closed**: Postgres + pgvector is the
   store; an unscoped search means every live category; archive opt-in is an
   enforced filter; ingestion is user-triggered (`sync_knowledge`, the upload
   tool), with no schedule.

## Consequences

- `pdf-extract`, `zip` and `glob` return to the workspace; no schedule crate.
- On the multi-node self-host topology the uploads directory is node-local until
  the `app_knowledge` volume is shared; `storage/knowledge/README.md` says so.
- Requirement-to-test traceability is enforced by
  `tests/unit/mcp/src/phase0_traceability.rs` against
  `docs/migration/project-context/TRACEABILITY.md`.
