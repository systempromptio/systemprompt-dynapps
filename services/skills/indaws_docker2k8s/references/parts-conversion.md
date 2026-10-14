# Parts: Docker manifests → `dyn-build.yaml`

The authority is `cloud_k8s/lib/dyn_build.py` in the manager. Its module docstring
explains the format and why it is what it is; this page is the migration-side view.

## The two Docker manifests

A Docker checkout uses **one** of them, never both.

| Manifest | Mechanism | Carries commits? |
|---|---|---|
| `.gitmodules` | plain git submodules | **No.** Only `submodule.<name>.url` and `submodule.<name>.branch`. The pinned commit lives in the superproject tree. |
| `repositories.yml` | the Cloud manifest | **Yes.** Everything is in the file. |

`.gitmodules` supports a branch and nothing else — a tag would have to be resolved to a
commit and checked out detached, which the Docker engine deliberately does not do. So a
Docker environment on `.gitmodules` has no tag pins to convert.

## Getting the commits for `.gitmodules`

On the host, in the environment's checkout:

```bash
git submodule status --recursive
```

`<sha> <path> (<describe>)` per line. The sha is the gitlink actually checked out, which
is the thing to pin.

**Not `platform_submodules_plan`.** Its rows carry `sha: ""` on purpose — reading the real
sha costs an SSH round trip per row and the tool does not do it. It is the right tool for
*which repositories and which branches*, and the wrong one for *which commits*.

## The `dyn-build.yaml` format

Three rules, each of which breaks something when ignored.

### 1. Commit pins carry no `branch:` key

```yaml
- name: my-module
  source: git
  url: git@gitlab.example.com:group/repo.git
  commit: 1a2b3c4d5e6f7890abcdef1234567890abcdef12 # branch: 18.0
```

Both builders run `git checkout <branch>` after a shallow clone made by commit, and in
that clone no local branch exists — so with both keys set **the build fails**. The Odoo
image pipeline uses `update-repos.py` (`_update_or_clone_git`, guarded by `if
branch_name:`); `dyn-tools`' `process_repo_part_git` does the same checkout
unconditionally. Neither prefers one key over the other and quietly carries on: the commit
wins the clone, and then the checkout that follows it dies.

This is the exact opposite of the Docker manifests, where neighbouring entries carry both
and it is fine. Carrying the habit across is a real trap.

### 2. The branch rides as an end-of-line comment

Not decoration: the manager parses it back out (`_BRANCH_HINT_RE`) so a reader — the
submodules dialog, or the next person bumping the part — can tell which branch a commit
came from. `ruamel` preserves it across round-trips.

**A commit with no branch comment is an unexplained sha.**

### 3. A tag is a ref, not a commit

```yaml
- name: pinned-by-tag
  source: git
  url: git@gitlab.example.com:group/repo.git
  branch: v1.4.0 # tag
```

No `commit:` at all. Both builders hand `branch` to git as the thing to fetch and git
resolves a tag there: `dyn build` clones with `--branch <X> --single-branch`, leaving HEAD
detached on the tag.

The first version of this resolved tags to their sha, on the theory that a tag name in a
branch slot would be checked out as a branch that does not exist. It is not true, and what
the theory cost was the tag name: the file kept a bare sha and reopening the dialog could
not say which tag had been chosen. The `# tag` mark records the namespace, because a bare
`branch: 19.0` cannot say whether `19.0` is a branch or a tag — and they are different
refs that can point at different commits.

## Sources

`dyn-build.yaml` uses `dynapps`, `public`, `git`, `local`. Only the first three resolve
against a remote host and can be bumped; **`local` parts live inside the code repo**
(`config`, `parts/custom`, `parts/oca_pull`) and have no *pinned* Docker counterpart to
convert. The exception in shape, not in kind: the Docker fleet's separate custom-addons
repository (`addons-<customer>`) does land inside the code repo — as `parts/custom/`
**content**, copied at the running commit, never emitted as a converted part. That step
is SKILL.md § 4.3; if the custom repo is pinned in the Docker manifest, the conversion
must **exclude** it so the modules do not exist twice.

## The indawsodoo repositories: mirrors, not GitHub

The CI token does not cover every repository in the `indawsodoo` GitHub organisation, and
a part it cannot read fails the build with a 403 that reads like a missing repository.
Instead of widening that token, those repositories are pull-mirrored into
`dynapps-es/mirrors/` and referenced as `source: dynapps` — no GitHub credential in the
pipeline at all. Rewriting a part is mechanical: `repo` becomes `dynapps-es/mirrors/<name>`,
`source` becomes `dynapps`, **the commit and its `# branch:` comment do not change**, because
a mirror carries the same shas as its origin.

The mirrors are filtered to version branches (`\A\d+\.0(-develop)?\z`). A pin whose commit
only ever lived on a feature branch is therefore unreachable in the mirror — a real sha
with no mirrored ref leading to it. It fails at fetch time and does not look like what it
is.

One repository is not mirrored but **dropped**: `asoo-tools`, whose single module
`asoo_connector` is exactly what the migration replaces with `dyncloud_api` +
`dyncloud_auth`. Dropping it also means the k8s image carries no `asoo_connector` code,
which is what forces the uninstall to happen on the Docker side — SKILL.md § 6.3.

## Odoo core, Enterprise and the themes: pinned by date tag

Always `gitlab.dynapps.be/odoo/odoo`, `…/odoo/enterprise` and `…/odoo/odoo-themes`,
whatever the Docker manifest says. Written as `repo: odoo/odoo.git` (the `.git` is
load-bearing — it is the only form `_list_addons` recognises as core Odoo, and without it
`parts/odoo/odoo/addons` never reaches `addons_path`), `repo: odoo/enterprise` and
`repo: odoo/odoo-themes.git`, all three `source: dynapps`.

The themes are the easiest to miss: the skeleton and the Docker manifests carry them as
`source: public` against `https://github.com/odoo/design-themes.git`, so the conversion
changes host, name, source **and** pin type at once.

They carry **no `commit:`**. Both GitLab repositories are tagged weekly on Sundays as
`<series>-<YYYYMMDD>`, and the pin is that tag in the `branch:` slot with `# tag` — rule 3
of the format, and the reason that rule exists.

Pick it from the **commit date of the `odoo/odoo` pin on GitHub**: the next Sunday after
it, and **the same tag on all three repositories**. One tag across them is what keeps core,
Enterprise and the themes from drifting; the gap is bounded at seven days. A quiet
repository like the themes may have consecutive tags on the same commit — a week without
changes, not a missing tag.

The tag is cut after the week's commits are synced, so it contains everything committed
before it — a pin older than the tag date is an ancestor by construction. Its own commit
is just the last one that existed: on `19.0` that is the Saturday evening before, one day,
consistently. A wider gap on a quiet series means nobody committed that week.

The only open case is a pin from the **current** week, whose Sunday has not arrived: no tag
exists yet. Wait for it, or pin the latest tag and accept core Odoo going back a few days —
a developer's call, and one to state out loud.

**This is the one conversion the script does not do.** Everything else is a mechanical
reference rewrite; this is a commit resolved into a date against a tagging calendar.

## The conversion script

Not written yet, and **not part of the manual procedure** — the runbook has to work by
hand today. When it is written it belongs in **`cloud_k8s/lib/`, next to `dyn_build.py`**,
for the same reason `dyn_build.py` is ORM-free and lives there: the pin format is a
negotiation with `dyn-tools`, and one module holding it is what makes it testable without
a database and impossible to reimplement by accident.

## Writing the file

Round-trip it with `ruamel`, configured as `dyn_build.build_yaml()` does — `preserve_quotes`
and an indent triple matching prettier's defaults. A round-trip that reformats produces a
diff on every run even when nothing changed, which fights the customer repo's pre-commit
hook and makes the commits non-idempotent.
