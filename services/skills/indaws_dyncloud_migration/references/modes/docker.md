# Mode A — deploy the connector on a Docker environment

The full procedure, in order, with the check that closes each step. Validated end to end on a
real legacy Docker environment; every gotcha below was hit for real on the way.

**Shape of the change:** two files in the environment's own code repository (the parts manifest and
the compose file), **one commit**, and one manual install in the database. Nothing is edited on the
server by hand — the webhook does the deploy.

> **Both files go in the SAME commit.** Every push to the environment's branch triggers a deploy, so
> splitting them means two deploys, and the first one is a known-broken state: the module is on disk
> with no configuration, or configured with nothing to configure. One commit, one deploy.

## Which manifest does this repo use? Check before editing

Two formats are in the field and they are **not interchangeable** — Dyncloud runs a different code
path for each (`asoo_submodules` vs `git_submodules` actions, `incloud submodule add` vs
`git submodule add`). Clone the environment's branch and look:

```bash
git clone --branch <environment-branch> <environment-repo> && cd <repo>
ls repositories.yml .gitmodules 2>/dev/null
```

| Found | Format | What you add |
|---|---|---|
| `repositories.yml` | Dyncloud parts manifest (newer) | A YAML entry — § *Step 2* |
| `.gitmodules` | Standard git submodules (legacy, still common) | A real submodule — § *Step 2b* |

If both exist, the environment is mid-migration: **ask** before assuming which one the build reads.
Editing the wrong one changes nothing and the module never appears.

---

## Step 0 — confirm the series is published

`dyncloud-odoo-auth` must have a **branch and a tag for the target Odoo series**. Check the
repository before anything else:

```bash
glab api "projects/<project-path-urlencoded>/repository/branches" --hostname <gitlab-host>
glab api "projects/<project-path-urlencoded>/repository/tags?per_page=100" --hostname <gitlab-host>
```

No branch for that series → **stop**. The module needs porting first (branch, adapt the
version-specific code, green suite, tag). Say so instead of improvising a pin to another series:
the module has code that differs per Odoo version, so a 19.x tag on an 18.0 environment is not a
shortcut, it is a broken environment.

Note the **tag's commit SHA** — it is what goes into `repositories.yml`.

---

## Step 1 — mint a read-only deploy token

On the **project** `dyncloud-odoo-auth` (not a group, unless you deliberately want it to cover
every project under it): **Settings → Repository → Deploy tokens**, scope **`read_repository`**.

Two things decide whether this works, and both are easy to get wrong:

- **Scope.** `read_registry` is for pulling container images and will **not** clone a repository.
  Existing tokens in the groups are often `read_registry` — they look right and fail.
- **Username.** Leaving the *Username* field empty makes GitLab generate `gitlab+deploy-token-N`,
  and **that** is what must go in the URL — not the token's display name. Fill the field if you
  want a readable username.

Verify the token **before** committing anything with it:

```bash
GIT_TERMINAL_PROMPT=0 git ls-remote "https://<user>:<token>@<gitlab-host>/<group>/dyncloud-odoo-auth.git" <series>
```

It must print the SHA of the series head. `HTTP Basic: Access denied` means the credentials do not
exist **for this repository** — the usual cause is a token created somewhere else, or the wrong
username. Checking here costs seconds; checking after a deploy costs a full cycle.

### Why not the SSH deploy-key route

Dyncloud can mint an SSH deploy key and register it on the platform, and for GitHub it does. On
GitLab it fails: the platform token holds **Reporter (20)** on the group, and creating a deploy key
requires **Maintainer (40)**. GitLab answers **403**, and that 403 dies inside an
`except Exception: pass`. The action finishes `done`, the key exists in Dyncloud with an **empty
`external_id`**, and the later clone runs without credentials inside a command invoked with
`warn=True` whose exit status nobody inspects.

Net effect: **two green actions and nothing downloaded.** Until the platform token is raised to
Maintainer, HTTPS + deploy token is the working route. See `troubleshooting.md`.

**Warn the user before committing the token:** the URL goes into `repositories.yml`, which is
committed and pushed. It lands in that repository's history and cannot be taken out without
rewriting it. If that is unacceptable, the alternative is a credential helper on the server
(`~/.git-credentials` + `credential.helper store`) with a clean URL in the file — same result, the
secret never enters the repo.

---

## Step 2 — add the module to `repositories.yml`

In the environment's repo, on **its own branch** (each environment is a branch), append one entry:

```yaml
- path: src/addons/<namespace>/<module-group>-<series>/dyncloud-odoo-auth
  url: https://<user>:<token>@<gitlab-host>/<group>/dyncloud-odoo-auth.git
  branch: '<series>'
  commit: <sha-of-the-tag>
  depth: 1
```

Match the `path` convention already used by the neighbouring entries in that same file rather than
inventing one — the layout differs between repos.

Both `branch` and `commit` are normal here; the other entries carry both. (In the Kubernetes build
the same combination is a trap — see `k8s.md`.)

**Check:** the file still parses, and the entry count grew by exactly one.

---

## Step 2b — the same, on a `.gitmodules` repository

Only if the repo uses standard git submodules (§ *Which manifest*). **Skip Step 2 entirely** — do
not create a `repositories.yml` alongside; the build reads one format, and a second one is dead
weight that later confuses whoever maintains it.

**Do not hand-edit `.gitmodules`.** Step 2 edits a file; this step runs a command, and carrying the
habit over is the mistake to avoid. `git submodule add` does three things at once — writes the
`.gitmodules` entry, clones the module into the path, and records the *gitlink* in the index.
Typing the entry yourself produces only the first: no clone, no gitlink, so the build finds an empty
directory and the module never loads. Nothing errors; the addon is simply not there.

Run it from a clone of the environment's branch:

```bash
git submodule add -b <series> \
  "https://<token-user>:<token>@<gitlab-host>/<group>/dyncloud-odoo-auth.git" \
  src/addons/<namespace>/dyncloud-<series>/dyncloud-odoo-auth
```

Mirror the `path` convention of the submodules already there — read `.gitmodules` and copy the
shape rather than inventing one; the build expects the module inside the addons path.

**How the pin works here is different, and it is the part that trips people up.** `.gitmodules` has
**no `commit:` key**: the commit is recorded in the tree as a *gitlink*, created when you commit the
`git submodule add`. So pinning is not something you write, it is something that happens — and to
move the pin later you check out the wanted commit **inside** the submodule and commit the parent
again. To pin the released tag rather than whatever the branch head was at that moment:

```bash
cd src/addons/<namespace>/dyncloud-<series>/dyncloud-odoo-auth
git checkout <tag-commit-sha>
cd -
git add .gitmodules src/addons/<namespace>/dyncloud-<series>/dyncloud-odoo-auth
```

Then continue with Step 3 and commit **everything together** — the submodule, the gitlink and the
compose file, in one commit.

**The token lands in `.gitmodules`** exactly as it would in `repositories.yml`: committed, pushed,
and in that repository's history for good. Same warning as Step 2.

## Step 3 — environment variables in the compose file

In the service that runs Odoo, add the variables listed in `values-and-sources.md`. Read that file
for where each value comes from; the short version:

Get the per-environment values first — `list_environments` hands you `uuid` and `aud` in one
call, no manager database needed (`../values-and-sources.md` → *How to obtain them*).

- required: `DYNCLOUD_AUTH_BACKEND_URL`, `DYNCLOUD_AUTH_ENV_UUID`,
  `DYNCLOUD_AUTH_LINEAGE_NAME` (**`docker`**), `DYNCLOUD_AUTH_ALLOWED_AUD`
- recommended: `DYNCLOUD_API_MAX_BODY_BYTES`
- **must stay unset**: `DYNCLOUD_AUTH_BACKEND_SPKI`, `DYNCLOUD_AUTH_SIGNING_KEYS_PATH`

Comment them in the file. They look removable to anyone who does not know they are read on every
cron tick, and removing them breaks the key refresh silently months later.

**Check:** the compose file parses (`python3 -c "import yaml,sys; yaml.safe_load(open('...'))"` or
`docker compose config`) **before** pushing. A malformed compose fails the deploy at the far end,
where the message is much worse.

---

## Step 4 — push and watch the deploy

Push to the environment's branch. The repository webhook fires a deploy action on the manager;
you do not launch anything.

**Watch it — do not assume.** The action can end `failed` for reasons unrelated to this change
(the server not being able to pull the environment repo itself, for instance). Read the action's
error before concluding anything about the module.

---

## Step 5 — install the two modules in the database (MANUAL)

**A developer does this by hand.** It is not automated, on purpose.

Install **`dyncloud_auth`** and **`dyncloud_api`** in the environment's database — through Odoo's
Apps UI (update the app list first) or the repo's own update script.

### Why it cannot ride on `auto_install`

Both modules declare `auto_install: True`, which is enough on a **fresh** database:
`initialize()` walks the auto-install set recursively. On a database that already exists it does
nothing, because `must_install()` requires an auto-install dependency to be in state
`'to install'` **during that very operation** — "install me in the same movement my dependency
arrives", not "install me once my dependency is present". An environment that already had
`dyncloud_auth` and merely upgrades it (`'to upgrade'`) never marks `dyncloud_api`.

The result is the confusing one: the module is in the image, it shows up in the module list, and
it is **not installed**. Its controllers are never registered, so every `/dyncloud/api/v1/*`
returns **404** while `/web/login` happily returns 200.

---

## Verification

Through the MCP (`common-skills:dyncloud-mcp`), in this order — the order matters because each
call proves a different layer:

1. **`list_databases`** on the environment. Success proves the whole trust chain: the environment
   fetched the manager's public keys over JWKS and verified an Ed25519 signature with them.
   Failure here is an `dyncloud_auth` / bootstrap problem.
2. **`system_odoo_version`**. Must report a real `dyncloud_api_version` and a `capabilities` list.
   This proves `dyncloud_api` is installed and its controllers are live. `"unknown"` as the
   version means the build predates the fix that made it report properly.

**No cron to trigger.** The module's `post_init_hook` detects Docker mode at install time (no key
files on disk) and performs the first JWKS fetch itself. The cron is only the periodic refresh
afterwards.
