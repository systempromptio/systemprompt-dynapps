# Forward-Porting a Change Across Coexisting Version Branches

A different problem from a one-time X→Y upgrade: a module is maintained on
**several live branches at once** (e.g. one repo with `16.0` / `17.0` / `18.0` /
`19.0` all in production), and a single new feature or fix must land on **all of
them without drift**. The risk isn't a broken migration — it's the branches
quietly diverging because a change (or a reviewer's nit) was applied to one and
forgotten on the rest.

## Principle: adapt by API family, not per version

The four live versions collapse into **two API families** for almost everything:

| | v16 / v17 | v18 / v19 |
|---|---|---|
| ACL probe (bool) | `check_access_rights("read", raise_exception=False)` | `has_access("read")` |
| translation | module-level `_()` | `env._()` (env carrying `lang`) |
| read-group | `read_group(domain, fields=, groupby=, lazy=False)` | `_read_group(domain, groupby=, aggregates=)` |
| list view tag | `<tree>` | v18 `<tree>`, **v19 `<list>`** |

Plus a v19-only twist: authorization checks use `all_group_ids` (direct+implied),
not `group_ids` (direct). Full detail: `common-skills:odoo-version-api-reference`
(diffs/) and `indaws-core-developer` (odoo18-fixes / odoo19-fixes). Write **one
branch of code per family**, parameterised by the one differing call — not a
bespoke implementation per version.

## Workflow

1. **Land a canonical version first.** Implement + validate + get reviewed/merged
   on one version (usually the lowest, or the repo's "source of truth"). That is
   the reference the others copy. Don't fan out from an unreviewed draft.
2. **Branch off the *correct, fresh* release.** `git fetch <remote> <NN.0>` first
   — local `origin/NN.0` goes stale, especially right after a merge (the merge
   commit isn't in your stale ref, so you'd branch off old code and the MR diff
   would wrongly include already-merged commits).
3. **Apply the same change, adapted by family.** For mechanical, *identical* edits
   across N repos, drive them with a small script that asserts each replacement
   matched **exactly once per file** and errors loudly otherwise — divergence
   (a renamed symbol, an extra import) surfaces immediately instead of silently
   skipping. Keep per-family deltas in a tiny map (`{"16": check_access_rights…,
   "18": has_access…}`).
4. **Validate each version independently.** Bring up that version's stack and run
   its tests; "passes on v17" does **not** imply "passes on v19" (group split,
   list tag, `has_access`). Cheap to run, expensive to skip.
5. **Version correctly.** A change folded into an *unmerged* MR keeps that MR's
   version. A follow-up on an *already-merged* branch gets its own patch bump —
   never ship two different code states under one version string.
6. **Forward-port nits in the same pass.** When a reviewer asks for a change on
   the version they reviewed, apply it to the *other* family members immediately
   (same script, same families) — that is the whole point of not letting branches
   drift. A nit fixed on one branch and not the rest is exactly how drift starts.

## Git / CI gotchas (these bite in practice)

- **Merged MR deletes its source branch.** If you push more commits to that
  branch name afterwards you *recreate an orphan* with no MR. Base follow-ups on
  the **release branch**, not the old feature branch.
- **Know your remotes.** A repo can have `origin` pointing at a GitHub *mirror*
  and a separate `gitlab` remote that is the real target (or vice versa).
  `git remote -v` before every push; push/MR to the right one. A push that says
  `[new branch]` when you expected an update is the tell.
- **Retrying one failed CI job can fail spuriously** if a later stage already
  cleaned up shared state (e.g. a `clean-image` step deleted the CI image the
  retry needs). Re-run the **whole pipeline**, not the single job.
- **`origin/NN.0` stale after others merge** — always fetch before comparing or
  branching, or you'll reason about code that's already moved.

## Relationship to upgrades

This is *not* an upgrade (`odoo16_to_17.md` et al. handle moving a module forward
one version). It's **parallel maintenance** of the same feature across versions
that all stay alive. The version-diff knowledge is shared — read the upgrade
patterns and `common-skills:odoo-version-api-reference` for the per-family deltas — but
the *workflow* (canonical-first, family-parameterised edits, per-version
validation, no-drift discipline) is what this file adds.
