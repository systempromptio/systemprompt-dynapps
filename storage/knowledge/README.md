# Knowledge-bank files categories

One directory per files category in `services/knowledge-bank/config.yaml`
(`meeting_notes/`, `adrs/`, `specs/`, `openspec_archive/`, …). Drop Markdown,
text, PDF or DOCX files here and run `sync_knowledge`; `add_knowledge_document`
writes into `<category>/uploads/` and indexes the file at once. Everything
under this directory except this README is content, not source, and is
gitignored. In the container it is the `app_knowledge` volume at
`/app/storage/knowledge`; on the multi-node self-host topology each node has
its own volume until the tree is shared.
