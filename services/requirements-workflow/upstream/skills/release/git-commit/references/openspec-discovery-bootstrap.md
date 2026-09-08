# OpenSpec discovery-time branch bootstrap (+ EXTEND baseline seeding)

The full recipe behind the one-line pointer in the `git-commit` skill's **OpenSpec branch workflow**.
Run this ONCE per change, at the start of BA/SA work (driven by `/opsx:propose`), BEFORE any BA/SA
artifact is committed. It creates the feature branch and — for an **EXTEND** — the pristine FSD/ISD
commit that `/opsx:propose-submit`'s body-scoped 3-way merge later uses as its ancestor.

The second branch (`requirements/<name>`) is **opt-in**: it exists only to surface FSD/ISD changes as a
clean review diff, so `/opsx:propose` offers it ONLY when the change has a real FSD/ISD target, and the
human may decline. With no FSD/ISD (or on a decline), everything happens on `feature/<name>` alone.

It is **local-only**: it never pushes. Pushing happens at `/opsx:propose-submit`, on an explicit yes.

```
Two-branch (FSD/ISD present AND offer accepted):
integration (develop/main)
  └── feature/<name>            ← baseline: pristine EXTEND pull commit(s) (the merge base); NEW seeds nothing
        └── requirements/<name> ← all BA/SA artifacts + FSD/ISD edits

Single-branch (no FSD/ISD, or offer declined):
integration (develop/main)
  └── feature/<name>            ← pristine EXTEND initial commit (the merge base) + ALL BA/SA artifacts
```

## Procedure

Let `<integration>` be the project integration branch (see `project-info`) and `<name>` the change slug.

1. **Baseline branch off integration** (skip if it already exists):
   ```sh
   git fetch origin <integration>
   git checkout -b feature/<name> origin/<integration> --no-track
   ```

2. **Author `discovery.md` here as an UNCOMMITTED working-tree change.** It names the FSD/ISD Publication
   Target inventory (its `Delivery Scope & Targets` table: Type, NEW/EXTEND, Confluence target). Leave it
   unstaged — it rides along to the edit branch in step 4 and is committed there at submit. Nothing but the
   EXTEND baseline (step 3) is committed on `feature/<name>`.

3. **Seed EXTEND baselines ON `feature/<name>`, BEFORE the edit branch diverges.** For each `EXTEND`
   target (a live Confluence page exists), pull the live page verbatim into the working-copy path via the
   typed reverse pull and commit ONLY that file (nothing else — `discovery.md` stays unstaged):
   ```sh
   node .cursor/skills/operate/atlassian/scripts/confluence.mjs get-page <pageId> --type=<fsd|isd> \
     --into openspec/changes/<name>/<type>/<feature>.md \
     --assets-dir openspec/changes/<name>/<type>/assets
   git add openspec/changes/<name>/<type>/<feature>.md openspec/changes/<name>/<type>/assets
   git commit -m "<SCOPE>:DOCS PROJ-000 Seed <name> <type> baseline from Confluence"
   ```
   `--assets-dir` localizes the page's body images + diagram sources into the working copy's `./assets`
   (kept, unlike a baseline pull) so the pristine commit is a COMPLETE, editable snapshot.
   Use the project's commit format (see the skill's **Commit format** section; `<SCOPE>` from the
   project's `scope-enum`, `PROJ-000` when there is no ticket).

   For a `NEW` target there is no live page — seed nothing here; the scaffold is first added on the edit
   branch (step 5) and becomes its own merge base.

4. **Choose the branch model** from `discovery.md`'s `Delivery Scope & Targets` table:
   - **No FSD/ISD target** — single-branch. Skip step 5; stay on `feature/<name>`.
   - **At least one FSD/ISD target** — `/opsx:propose` OFFERS the two-branch model (human may decline,
     rationale: a clean FSD/ISD review diff). On accept, do step 5; on decline, stay single-branch.

5. **(Two-branch only) Edit branch off the (now baseline-bearing) feature branch**, then switch to it —
   the uncommitted `discovery.md` and any working-tree files follow across the switch:
   ```sh
   git checkout -b requirements/<name> feature/<name> --no-track
   ```

6. **Do ALL BA/SA work on the active branch:** generate the artifacts, and for an EXTEND edit the pristine
   working copy already present from the `feature/<name>` initial commit. Commit them at
   `/opsx:propose-submit`.
   - **Two-branch:** work on `requirements/<name>`; submit opens the review PR `requirements/<name>` →
     `feature/<name>` (baseline already lives on the target, so the PR shows exactly the BA/SA delta).
   - **Single-branch:** work on `feature/<name>`; submit commits there directly and opens NO PR. The
     EXTEND merge base is still the pristine initial commit (`git log --diff-filter=A` first-add on
     `feature/<name>`), so the 3-way merge is unaffected.

## Why the order is load-bearing

`/opsx:propose-submit` derives the 3-way merge base from git history:

```sh
git log --diff-filter=A --format=%H -- <feature>.md | tail -1   # the file's first-add commit
```

For an EXTEND this MUST resolve to the pristine `feature/<name>` commit, so the merge base is the live
document as it stood at discovery — `base` = pristine, `ours` = your edits, `theirs` = the freshly
re-pulled live page. Committing the pristine pull as its OWN initial commit BEFORE any edit (in
two-branch mode, before branching `requirements/<name>`; in single-branch mode, before editing on
`feature/<name>`) is exactly what guarantees it.

If you instead commit an already-edited working copy as the file's first-add, that first-add becomes your
own edited version: `base == ours`, and the 3-way merge degenerates to "take theirs", silently discarding
local edits or manufacturing false conflicts. This is the R1 (pristine commit before any edit) and R5
(keep that commit stable — never rebase/rewrite it) discipline from the OpenSpec 1.1 design, and it holds
in both branch models.

## Relationship to the rest of the flow

- **Discovery inventory drives it:** the `EXTEND` rows + Confluence Page IDs come from `discovery.md`'s
  `Delivery Scope & Targets` table (or a project-context / live Page-ID lookup when authoring it).
- **Submit consumes it by branch existence:** `/opsx:propose-submit` detects the model — if
  `requirements/<name>` exists it commits there and opens the PR `requirements → feature`; otherwise it
  commits on `feature/<name>` and opens NO PR. Either way it reads the merge base from git history, and
  this bootstrap is the half that creates that base.
- **`.baseline.md` is unrelated:** the throwaway `<feature>.baseline.md` pulled at submit is the merge
  `theirs` input (gitignored, never committed). The durable ancestor is the pristine `feature/<name>`
  commit created here.
