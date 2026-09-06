# Release Notes from Commit History

You turn the conventional-commit history of a branch or tag range into three artefacts: a client-
facing release-notes block, a CHANGELOG entry inserted into the project's changelog file, and a
pull-request body with a verification section. Every line in the notes traces to a commit hash;
every user-visible commit appears in the notes. Nothing is invented and nothing is dropped.

## Use when

- "write the release notes", "update the changelog for", "draft the PR body", "summarise the branch"
- "what changed between v1.4.0 and HEAD", "prepare the release for the client"
- Do not use to write commit messages; that is `git_commit`.

## Inputs

Ask for:

- The range: a base ref (integration branch or last tag) and a head ref. Default to
  `<integration-branch>...HEAD`.
- The audience of the release notes: client PO, internal QA, or both. Default to client.
- Whether the PR uses squash-merge (then the PR title becomes the changelog line and must be
  commitlint-valid).

Infer from the repo:

- The changelog file (`CHANGELOG.md` at the root or the project's documented location) and its
  existing heading style. Match it exactly.
- The ticket prefix and browse URL from recent commit bodies (`Refs: PROJ-1234`) and the docs.
- The last tag: `git describe --tags --abbrev=0`.

## Workflow

1. **Ground the notes in project knowledge.** Call the `knowledge-bank` MCP tool `search_knowledge`
   with `{"query": "<ticket keys from the range> release notes acceptance", "top_k": 8}`, then
   with `source_types: ["project_tickets"]` to get each ticket's summary in the client's own
   words. Use those words for the outcome lines and cite `title — uri#lines` in the PR body's
   Summary. If the server is unavailable, say so and write outcomes from the commit bodies.
2. **List the commits.** `git log <base>...<head> --format='%h%x09%s%x09%b' --no-merges`. Parse
   `type(scope): summary` and the `Refs:` trailer. A commit that does not parse is a finding,
   not a guess: list it under `Unclassified` for the author to fix.
3. **Classify.** `feat`, `fix`, and behaviour-affecting `perf` are user-visible and go in the
   notes. `refactor`, `chore`, `test`, `docs`, `build` are internal and go only in the PR body.
   A `!` or a `BREAKING CHANGE:` footer goes in its own section at the top, always.
4. **Write the release notes** for the client: outcome first, no internal jargon, one line per
   user-visible commit, grouped Added / Changed / Fixed / Breaking, each ending with the ticket
   key. "Guests can now save a basket for 30 days (PROJ-1234)", not "add basket persistence".
5. **Insert the CHANGELOG entry** under a new heading in the file's existing style
   (`## [X.Y.Z] - YYYY-MM-DD` or `## Unreleased`), above the previous entry. Do not touch any
   other line. The version is the one the user gave or `Unreleased`; never bump a version on
   your own.
6. **Write the PR body** in the project's order: Summary (ticket links first, then two or three
   sentences), Changes Made (files and components), Testing (the exact commands and counts from
   `sfnext_prove_with_playwright`, or `VERIFICATION MISSING` in capitals if none was run),
   Blocked Tests (any `fixme` with owner), Screenshots (if UI), Breaking Changes (if any),
   Out of scope. Title = lead commit subject; for squash-merge it must be commitlint-valid.
7. **Cross-check.** Every user-visible hash appears in the notes; every ticket key in the range
   appears in the PR Summary; the CHANGELOG diff touches only the new entry.

## Result

- `git diff -- CHANGELOG.md` in the reply, showing only the inserted entry.
- The PR body as a fenced markdown block ready to paste, with a `Title:` line above it.
- A trace table:

| Hash | Type | Ticket | Notes line | Section |
|---|---|---|---|---|
| `a1b2c3d` | feat | PROJ-1234 | Guests can now save a basket... | Added |
| `e4f5g6h` | chore | — | (internal, PR body only) | — |

The user verifies by comparing the table against `git log` for the range: a hash missing from the
table, or a notes line with no hash, is a defect.

## Guardrails

- Never commit, push, tag, create the PR, or edit the ticket. Present the diff and the body; the
  user acts.
- Never write a notes line without a commit behind it, and never omit a user-visible commit
  because it is awkward to phrase.
- A PR body with no Testing evidence says `VERIFICATION MISSING`; do not paraphrase it away.
- Client notes contain no file paths, hook names, cartridge names, or internal component names.
- English only. Do not translate ticket summaries.
- Do not reformat or reflow existing CHANGELOG entries.
