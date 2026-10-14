# Phase 4 § 4.2 — The parts: `.gitmodules` / `repositories.yml` → `dyn-build.yaml` — LOCAL

> **docker2k8s — Phase 4, section 4.2 — the long one, split out.** Before starting: Read it when you are actually converting the manifest. `phases/04-equivalence.md` is the rest of Phase 4 and points here.
> The router, the safety rules and the Prerequisites live in `SKILL.md`.


The pin format is a negotiation with `dyn-tools`, and getting it wrong breaks the build in
a way that reads like something else. It lives in `cloud_k8s/lib/dyn_build.py`.

**The rules, all three load-bearing:**

1. **A commit-pinned part carries `commit:` and NO `branch:` key.** `dyn-tools`'
   `process_repo_part_git` unconditionally runs `git checkout <branch>` after a shallow
   commit-based clone, where no local branch exists — with both keys set, the build fails.

2. **The branch is written as an end-of-line comment next to `commit:`.**

   ```yaml
   - name: my-module
     source: git
     url: git@gitlab.example.com:group/repo.git
     commit: 1a2b3c4d5e6f7890abcdef1234567890abcdef12 # branch: 18.0
   ```

   This is not decoration. The manager parses it back (`_BRANCH_HINT_RE`) to know which
   branch a commit came from — it is what lets the submodules dialog show a Branch column
   at all, and what the next reader needs to bump the part later. **A commit with no
   branch comment is an unexplained sha.**

3. **A tag is a ref, not a commit.** It goes in `branch:`, the part carries **no**
   `commit:`, and it is marked `# tag`:

   ```yaml
   - name: pinned-by-tag
     source: git
     url: git@gitlab.example.com:group/repo.git
     branch: v1.4.0 # tag
   ```

   Both builders hand `branch` to git as the thing to fetch, and git resolves a tag there.
   Resolving the tag to a bare sha instead loses the tag name, and the next reader cannot
   tell which tag was chosen.

**Said the other way round, because this is the decision that goes wrong: THE TAG WINS.**
When a part is meant to be tag-pinned, the tag is the pin and there is no `commit:` at
all. When a part is meant to be commit-pinned, the commit is the pin and the branch is
only the comment beside it. There is no third shape. Written as a table, because two of
the four combinations are the whole bug surface:

| Intent | `commit:` | `branch:` | Result |
|---|---|---|---|
| Pinned by tag | absent | `X.Y-YYYYMMDD # tag` | correct |
| Pinned by commit | `<sha> # branch: X.Y` | absent | correct |
| Both keys | `<sha>` | `X.Y` | **build fails** — rule 1 |
| Tag name in `commit:` | `X.Y-YYYYMMDD` | absent | **build fails** — clone-by-commit resolves shas only |

The last row is not hypothetical: the odoo-skeleton made exactly that mistake once
(`346cb5d`) and it is why the scaffold now carries a comment explaining the split.

> **VERIFY THE FILE YOURSELF, whatever produced it.** A conversion script is worth
> writing (below) but it will not cover every shape a Docker manifest can take, and a
> hand-edited file drifts. **Before the first build, read the finished `dyn-build.yaml`
> part by part and check each one against the table above** — every part matches exactly
> one row, and any part that does not is corrected in place, right then. This is a
> mandatory read of the whole file, not a spot check: it costs a minute and the failure
> it prevents surfaces as a build error that reads like a missing module, a wrong URL or
> a broken runner.

**Where the commits come from — all of it is in the repository. No SSH.**

| Source | Has commits? | How to get them |
|---|---|---|
| `repositories.yml` | **Yes**, in the file | Read the file. Every field is there. |
| `.gitmodules` | Not in that file — but yes in the repo | `.gitmodules` carries url and path only. The pin is a **gitlink in the superproject's tree**, and a tree is part of the repository like any other object. |

Clone the code repo — submodules not needed, they are never checked out — and read the
tree at the ref the environment actually deploys:

```bash
git clone --no-recurse-submodules <code-repo-url> repo && cd repo
git ls-tree <branch> -r | grep '^160000 commit'
```

Each line is `160000 commit <sha>\t<path>`: the pinned commit, straight out of the tree.
Pair it with the url and branch from `.gitmodules` at the same ref and the part is
complete.

> **Which ref?** The branch the environment deploys, and that is not a guess since
> 2026-08-21: `platform_deploy_config(environment_id=…)` returns the deploy target, whose
> **name IS the branch** (`target_type` says whether it is a release, hotfix or feature
> branch). Read it, do not assume `main`.

> **When SSH is warranted, and only then.** The repository tells you what the branch
> pins. The host tells you what is actually checked out, and the two can disagree — a
> submodule bumped on the host and never committed. Read the repository first; reach for
> `docker_ssh_exec` and `git submodule status --recursive` only to confirm the running
> checkout when you have reason to suspect drift, and say which of the two answers you
> used.

> **Do NOT take the commits from `platform_submodules_plan`.** Its rows return `sha: ""`
> deliberately: reading the real sha costs an SSH round trip per row, and the tool is not
> in that business. It gives you repository and branch, never the pin. Using it as the
> source of commits would produce a `dyn-build.yaml` that builds *a* version, not *the*
> version.

**Parts that do NOT travel — check this list before converting anything.**

| Docker part | Why it stops here |
|---|---|
| `asoo-tools` | It holds exactly one module, `asoo_connector` — the old connector, replaced by `dyncloud_api` + `dyncloud_auth` (§ 6.3 (`phases/06-move-data.md`)). Carrying it over is carrying a corpse: the module it provides is the one this migration removes. |

The consequence is not only that the part is dropped. **The k8s image therefore has no
`asoo_connector` code at all** — which does not stop it being uninstalled on the
Kubernetes side, and does mean the first boot needs the platform's orphan-state repair.
Both are § 6.3 (`phases/06-move-data.md`).

**The `indawsodoo` parts come from GitLab mirrors now, not from GitHub.** The CI token
does not cover every repository in that GitHub organisation, and a part it cannot read
fails the build with a 403 that reads like a missing repository. Rather than widening the
token, those repositories are **pull-mirrored into `dynapps-es/mirrors/`** and referenced
as `source: dynapps` — the same mechanism `odoo/odoo` and `odoo/enterprise` already use,
with no GitHub credential in the pipeline at all.

**So every `github.com/indawsodoo/*` part is rewritten during the conversion:**

| Docker manifest points at | The k8s part becomes (`source: dynapps`) |
|---|---|
| `indawsodoo/indaws-odoo` | `dynapps-es/mirrors/indaws-odoo` |
| `indawsodoo/indaws-odoo-enterprise` | `dynapps-es/mirrors/indaws-odoo-enterprise` |
| `indawsodoo/vertical-odoo-inbuilt` | `dynapps-es/mirrors/vertical-odoo-inbuilt` |
| `indawsodoo/connector-sale-odoo` | `dynapps-es/mirrors/connector-sale-odoo` |
| `indawsodoo/connector-stock-odoo` | `dynapps-es/mirrors/connector-stock-odoo` |
| `indawsodoo/connector-delivery-odoo` | `dynapps-es/mirrors/connector-delivery-odoo` |
| `indawsodoo/indaws-edi` | `dynapps-es/mirrors/indaws-edi` |
| `indawsodoo/ai-odoo-ocr` | `dynapps-es/mirrors/ai-odoo-ocr` |
| `indawsodoo/indaws-tools` | `dynapps-es/mirrors/indaws-tools` |
| `indawsodoo/asoo-tools` | **nothing — dropped, see above** |
| `odoo/odoo` | `odoo/odoo.git` — **and pinned by tag, see below** |
| `indawsodoo/enterprise` | `odoo/enterprise` — **and pinned by tag, see below** |
| `odoo/design-themes` | `odoo/odoo-themes.git` — **and pinned by tag, see below** |

The commit does not change: a mirror carries the same shas as its origin. Only the `repo`
and `source` keys change, and the branch comment (§ rule 2) stays exactly as it was.

**Three parts are the exception, and they are pinned by TAG:** Odoo core, Enterprise and
the design themes. Whatever the Docker manifest points at, they always come from
`gitlab.dynapps.be/odoo/odoo`, `…/odoo/enterprise` and `…/odoo/odoo-themes` — 100% of the
time — and unlike every other part they carry **no commit at all**:

```yaml
- repo: odoo/odoo.git
  source: dynapps
  branch: <series>-<YYYYMMDD> # tag      # e.g. 19.0-20260809 — NOT a constant
- repo: odoo/enterprise
  source: dynapps
  branch: <series>-<YYYYMMDD> # tag      # the SAME tag as above
- repo: odoo/odoo-themes.git
  source: dynapps
  branch: <series>-<YYYYMMDD> # tag      # the SAME tag again
```

> **The themes also change `source`.** The skeleton and the Docker manifests reach them as
> `source: public` at `https://github.com/odoo/design-themes.git`. On Kubernetes they are
> `odoo/odoo-themes.git` with `source: dynapps` — a different host, a different name, and
> a tag instead of a commit. It is the part most easily left behind on the old form,
> because it is the one nobody thinks about.

**The date is computed for every migration, never copied from this page.** It comes from
the commit the customer is actually running, so it is different for each of them — and
different again if the same customer is migrated later. Any date written here is an
example.

> **`odoo/odoo.git` keeps the `.git`, and that is load-bearing.** `dyn-tools`'
> `_list_addons` recognises core Odoo **only** in that exact path form, and only then adds
> both `parts/odoo/odoo/odoo/addons` (base + tests) and `parts/odoo/odoo/addons`
> (community: web, account…) to `addons_path`. Written any other way the community addons
> directory is simply missing and the environment dies at init with
> `ModuleNotFoundError: odoo.addons.web`. Enterprise carries no `.git`.

**Choosing the tag — the LLM does this by hand, per environment.** The GitLab repositories
are tagged **once a week, on Sundays**, as `<series>-<YYYYMMDD>`. So:

1. Read the **commit date** of the `odoo/odoo` pin in the Docker manifest — from GitHub,
   where that commit lives.
2. Take the **next Sunday after that date**. A commit of Tuesday 2026-08-04 on `19.0`
   gives `19.0-20260809`.
3. Use **that same tag for all three** — `odoo/odoo.git`, `odoo/enterprise` and
   `odoo/odoo-themes.git`.

The third step is the whole point. These are repositories that must move together; giving
each its own nearest pin is how they silently drift apart. One weekly tag across the three
bounds the gap at **seven days at most** (a Monday commit waiting for the following Sunday)
and guarantees the combination is one that existed.

> **A quiet repository still has the tag.** `odoo-themes` is committed to far less often
> than core, so its `19.0-20260816` can point at a commit from ten days earlier — and
> `19.0-20260809` and `19.0-20260816` can even be the same commit. That is a week with no
> changes, not a missing tag. Take the date, not the nearest commit.

> **Why the next Sunday is always safe.** The tag is cut *after* the week's commits are
> synced, so it carries everything committed before it: a pin older than the tag date is
> an ancestor of it by construction. The tag's own commit is simply the last one that
> existed — on `19.0`, where upstream commits daily, that is the Saturday evening before,
> exactly one day, week after week. A bigger gap on a quiet series (`16.0-20260809` points
> at a commit of 2026-08-07) means nobody committed that week, not that the tag is missing
> anything.
>
> **The one case with no answer yet: a pin from the current week.** If the commit is more
> recent than the last tag, its "next Sunday" has not happened and the tag does not exist.
> Then it is a choice, and it belongs to the developer: wait for Sunday, or pin the latest
> existing tag and accept that core Odoo goes back a few days from what Docker runs. Say
> which one was done.

**This part is manual and stays manual.** The conversion script of § 4.2 can rewrite every
other part mechanically — the mirror carries the same shas, so only `repo` and `source`
change. These two are a different operation: a commit becomes a *date*, resolved against a
tagging calendar the manifest knows nothing about. Leave them to the human.

> **The mirrors carry version branches only.** They are filtered with
> `\A\d+\.0(-develop)?\z`, so `16.0`, `18.0-develop` and the like are there and feature
> branches are not. **A part pinned to a commit that only ever lived on a feature branch
> will not fetch** — the sha is real, it is simply not reachable from any mirrored ref.
> That is not an auth problem, and it will not look like one: read the pin first, and if
> it comes from a feature branch, decide deliberately whether to merge it upstream or
> widen that one mirror's regex.

> **Meeting a repository not in the table.** Adding a mirror is a one-time job on the
> group, not per migration: create the project under `dynapps-es/mirrors/`, point it at
> GitHub with a fine-grained read token, set the branch regex, and add the sync webhook.
> Two things bite when it is a *new group* rather than a new project in the existing one:
> `service_account_dyncloud_manager` needs at least Reporter **on the group** or every
> build 403s indistinguishably from a GitHub auth failure, and the token that calls
> `POST /projects/:id/mirror/pull` must be **Maintainer** — Developer answers 403 there
> too. Neither applies to `dynapps-es/mirrors/`, where both are already in place.

**A conversion script is worth writing, and this is what it must do** — the manual path
has too many places to slip:

- Read whichever manifest exists, **failing loudly when the read fails** rather than
  treating an unreadable repo as an empty one (§ 1b (`phases/01-survey.md`)).
- Apply the drop list and the mirror rewrite above: `asoo-tools` disappears, and every
  other `indawsodoo/*` repository becomes its `dynapps-es/mirrors/…` counterpart with
  `source: dynapps`, same commit.
- **Leave `odoo/odoo.git` and `odoo/enterprise` alone.** Their pin is a weekly date tag
  chosen by a human against a calendar, not a reference the script can rewrite.
- For `.gitmodules`, resolve each commit from `git submodule status` on the host.
- Emit `dyn-build.yaml` entries in the format above: `commit:` plus `# branch:` comment,
  or `branch:` plus `# tag` for a tag.
- Map the source: the Docker manifests distinguish *Git* (`.gitmodules`) from *Cloud*
  (`repositories.yml`); `dyn-build.yaml` uses `dynapps` / `public` / `git` / `local`.
  **`local` parts live inside the code repo and are not bumpable** — no *pinned* Docker
  counterpart enters the conversion. The one thing that does land inside the repo, the
  customer's custom-addons repository, arrives as `parts/custom/` **content** (§ 4.3 (`phases/04-equivalence.md`)),
  not as a converted part.
- Be idempotent, and round-trip the YAML without reformatting it: the file is written with
  `ruamel` configured to match prettier's defaults, so that a re-run produces no diff when
  nothing changed. A reformatting round-trip fights the customer repo's pre-commit hook.

**No such script exists yet** — none was found during this survey. It is **tracked as
separate work, not part of this procedure**: this page has to be usable by hand today, and
the script arrives after. When it is written it belongs in **`cloud_k8s/lib/`, next to
`dyn_build.py`** — that is where the pin format already lives, and keeping the two
together is what stops the format being reimplemented by accident somewhere else.

#### `dyncloud-odoo-auth` — the scaffold's own part, and the one to armour

`dyn-build.yaml` arrives from the code scaffold with one part nobody converted, because it
comes from no Docker manifest: `infra-es/dyncloud/dyncloud-odoo-auth`, which provides
`dyncloud_api` and `dyncloud_auth` — the connector that replaces `asoo_connector` (§ 6.3 (`phases/06-move-data.md`))
and the module the values overlay lists in `odoo.requiredModules`. **If the image does not
carry it, the `upgrade-odoo` init aborts with `Missing module(s): dyncloud_api` and the
environment never boots.**

Two rules, and they are not negotiable:

1. **It is pinned BY TAG and it never carries a `commit:`.** A sha here is how the part
   silently freezes: four customer repos spent three months on a May sha while everyone
   read the release number and assumed they were current (2026-08-25).
2. **Check the tag is the newest one, and bump it if it is not.** The scaffold's pin is
   whatever was current when that `code_template_ref` was cut, and a scaffold ref older
   than the tag you need is exactly the trap above:

   ```bash
   git ls-remote --tags --sort=-v:refname \
     git@gitlab.dynapps.be:infra-es/dyncloud/dyncloud-odoo-auth.git | head -5
   ```

   Take the highest tag for the customer's Odoo series, compare it with what the generated
   `dyn-build.yaml` holds, and **raise it if the scaffold gave you an older one**. Never
   lower it, and never replace it with the sha the tag points at.

The finished part reads exactly like this — no `commit:` key anywhere near it:

```yaml
- repo: infra-es/dyncloud/dyncloud-odoo-auth
  source: dynapps
  branch: 19.0.3.3.0 # tag        # the NEWEST tag, looked up — not copied from here
```

