# Knowledge sources

Hand-authored material the knowledge bank indexes. One subdirectory per
`kind: files` source in `services/knowledge-bank/config.yaml`:

| directory | category | what goes here |
|---|---|---|
| `meeting_notes/` | `meeting_notes` | workshop / discovery / tech-call transcripts (md, txt, pdf, docx) |
| `adrs/` | `adrs` | Architecture Decision Records (md) |
| `specs/` | `specs` | the OpenSpec `openspec/specs/**/spec.md` tree |
| `openspec_archive/` | `openspec_archive` | `openspec/changes/archive/**/{proposal,design,technical-design}.md` |

Contents are gitignored. In a compose deployment this directory is the
`app_knowledge` volume; land files here (a `git pull` of the Cursor repo, an
`scp`, a mount) and run `systemprompt-mcp-knowledge-bank ingest` or wait for
the schedule. Unsupported file types are skipped, never fatal.
