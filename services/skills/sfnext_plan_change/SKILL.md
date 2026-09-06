# Plan a Storefront Next Change

You produce an open-spec proposal for one change to a Storefront Next (React Router 7 + SCAPI on
MRT) or hybrid SFRA project, written to `openspec/<change-id>/proposal.md`, plus a step list a
reviewer can approve line by line. No code is written by this skill. The proposal is judged on one
thing: can a reviewer approve or reject each step without opening the codebase.

## Use when

- "plan this feature", "write the spec for", "propose an approach to", "how would we build"
- "break JIRA-1234 into reviewable steps", "what are the risks of", "which files does this touch"
- Any change touching more than one route, component, cartridge, or data contract.
- Do not use for a quick fix (< 4h, one file, no ambiguity) — commit that directly.

## Inputs

Ask for:

- The ticket key or a pasted requirement. Refuse to plan from a one-line title alone.
- The change size the user believes it is: quick fix, small, standard, cross-team.

Infer from the repo:

- Project shape: `react-router.config.*` + `src/routes/` means Storefront Next;
  `cartridges/*/cartridge/controllers` means SFRA; both means hybrid.
- Approved libraries from `package.json`. Anything not in it is a decision, not a default.
- The integration branch and the ticket prefix from `git branch -r` and recent commit subjects.
- An existing `openspec/` directory and its schema, which win over the layout in this skill.

## Workflow

1. **Ground the plan in project knowledge.** Call the `knowledge-bank` MCP tool `search_knowledge`
   with `{"query": "<ticket key> <feature nouns> <route or cartridge names>", "top_k": 8}`.
   Run it twice more with `source_types: ["project_docs"]` for the FSD/ISD and
   `source_types: ["project_tickets"]` for prior decisions on the same area. Record every hit you
   rely on as `title — uri#lines` in the proposal's Context section. If the server is unavailable,
   write `Knowledge bank unavailable at <time>; plan grounded in repo only` and continue.
2. **Restate the requirement** in one paragraph, including what is explicitly out of scope. If the
   ticket and the code disagree, write the conflict down as an open question; do not pick a side.
3. **Read the code you will touch.** `rg` for the route files, loaders, actions, components,
   hooks, or cartridge scripts named in the requirement. Name the file you will model each change
   on. Stop at ~90% confidence; a plan is not the place to resolve every unknown.
4. **Decide the change size** using the table in Guardrails. Start one size smaller than instinct.
5. **Write `openspec/<change-id>/proposal.md`** where `<change-id>` is `<ticket>-<kebab-slug>`.
   Sections, in order:
   - `## Context` — the restated requirement and the knowledge-bank citations from step 1.
   - `## Approach` — routes, components, loaders/actions, cartridges, hooks, and data flows
     touched. State whether data is fetched in a loader, mutated in an action, or fetched on
     demand with `useScapiFetcher`; client-side fetching where a loader belongs is a rejected
     alternative, not an option.
   - `## Decisions` — dated records for anything that constrains later work: a new dependency, a
     data model, an API contract, a new site preference. Each names the alternative rejected.
   - `## Risks` — each with the step that retires it. A `null` site preference silently disabling
     a feature is a risk to name whenever a preference is introduced.
   - `## Steps` — a numbered list. Each step is one pull request, names its files, and names its
     verification: `proven by playwright/tests/<spec>.spec.ts` or `unit test <file>`. A step with
     no verification line is not a step.
   - `## Open questions` — each addressed to a named role (BA, tech lead, client PO).
   - `## Review & Approval` — `- [ ] Approved` and `Approved by:` left blank. Never fill it in.
6. **Print the step list** to the user as a table: `#`, summary, files, verification, and hand
   over for approval. Do not start implementation; that is `sfnext_implement_feature`.

## Result

- `openspec/<change-id>/proposal.md` on disk, uncommitted, with every section above present.
- A step table in the reply that matches `## Steps` exactly.

The user verifies by opening the file and checking that every step has a verification line and
every citation in Context resolves. A reviewer who has to open the codebase to judge a step is
the failure signal; rewrite that step.

## Guardrails

| Size | Criteria | What you write |
|---|---|---|
| Quick fix | < 4h, one file, no ambiguity | Nothing — tell the user to commit directly |
| Small | < 1 day, one package, clear requirements | Context, Approach, Steps only |
| Standard | Multi-day, some ambiguity | All sections |
| Cross-team | Multi-package, architectural | All sections plus a grooming question list |

- Never write or modify application code, and never commit or push. Planning is read-only.
- Never self-approve. The approval box is filled by a named human.
- Never propose a new dependency without a Decisions entry naming what in `package.json` was
  considered first.
- Never propose editing `sfcc/meta_sandbox/`; metadata changes go to `sfcc/meta_delta/`.
- Do not claim a platform limitation without a link to docs or an issue. No link, no claim.
- Prefer extending an existing pattern over introducing one; the modelled-on file is mandatory.
- Keep the proposal under one page of prose. A spec nobody reads governs nothing.
