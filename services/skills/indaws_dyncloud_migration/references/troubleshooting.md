# Troubleshooting — the failures that look like success

The expensive thing about this procedure is not the failures; it is that **several of them report
success**. Two `except Exception: pass`, one shell command run with `warn=True` whose exit status
is never checked, and a module that installs into an image but not into a database. Each section
below gives the symptom, the tell, and the fix.

Diagnosis order, cheapest first: **is the module installed? → did the clone happen? → did the key
exchange happen?**

---

## 1. `/dyncloud/api/v1/*` returns 404 but `/web/login` returns 200

**The most frequent one by far.** Odoo is alive; `dyncloud_api` is simply **not installed in the
database**, so its controllers were never registered.

Do not go looking at networking, the token or the certificate. Check the module state:

- Is `dyncloud_api` in state `installed`, or `uninstalled`?
- Is the code even on the server (the addons path of the container should include the module's
  directory)?

Two different causes, two different fixes:

| Finding | Cause | Fix |
|---|---|---|
| Code present, module `uninstalled` | `auto_install` does not fire on an existing database (`must_install()`) | Step 5: install it by hand |
| Code absent from the addons path | The clone never happened | § 2 below |

On **Kubernetes** the equivalent cause is different: `odoo.requiredModules` in the environment's
GitOps values lists only `dyncloud_auth`, so `dyncloud_api` is never installed there either. Same
404, different mechanism — see `modes/k8s.md`.

---

## 2. The deploy action says `done` and nothing was downloaded

The SSH deploy-key route. The chain is: Dyncloud mints the key → tries to register it on GitLab →
**403, swallowed** → the submodule/clone command then runs **without credentials** → it fails
inside a call made with `warn=True` → nobody checks the exit code → the action reports `done`.

**The tell:** the SSH key record in Dyncloud has an **empty `external_id`**. When the upload
succeeds, that field holds the id GitLab returned. One key with an empty `external_id` among many
populated ones is the fingerprint of exactly this failure.

**Why the 403:** the platform token holds **Reporter (20)** on the group; creating a deploy key
needs **Maintainer (40)**.

**Fix (unblocks today):** HTTPS + a `read_repository` deploy token, as in `modes/docker.md` § 1.
The code path that looks for an SSH key is skipped entirely for `http(s)://` URLs, so the whole
deploy-key mechanism is bypassed.

**Proper fix (needs a decision):** raise the platform token to Maintainer on the group. That is a
GitLab permissions change and belongs to whoever owns those groups — not something to do quietly
as part of connecting one environment.

**Worth reporting upstream:** the two `except Exception: pass` and the unchecked `warn=True` are
what turned a perfectly clear 403 into a long investigation. Fixing those in the manager would make
this class of failure self-diagnosing.

---

## 3. `HTTP Basic: Access denied` when cloning over HTTPS

The message reads like a wrong password. It usually means **those credentials do not exist for
this repository**.

Check, in this order:

1. **Does the token exist on that project?** A token created on a different project, or on a group
   that does not cover it, authenticates against nothing.
2. **Is its scope `read_repository`?** `read_registry` is for images and cannot clone. Pre-existing
   tokens on the parent groups are frequently `read_registry`.
3. **Is the username right?** With the *Username* field left empty at creation, GitLab generates
   `gitlab+deploy-token-N`, and that generated username — not the token's display name — is what
   belongs in the URL.
4. **Has it expired or been revoked?**

Always reproduce with `git ls-remote` before touching the repository again: it is instant and
isolates the credential from everything else.

---

## 4. It worked, and weeks later tokens are rejected

Look for the environment variables in the compose file. If someone removed them after the first
successful sync — a reasonable-looking cleanup — the JWKS refresh has been returning early ever
since, leaving one `JWKS_REFRESH_FAIL` audit entry per tick and nothing else. The environment kept
working on cached keys until the manager rotated its signing key.

They are read on **every** cron tick, not once at boot. See `values-and-sources.md`.

Same symptom, different cause: `DYNCLOUD_AUTH_SIGNING_KEYS_PATH` present on a Docker environment
puts the module in disk mode and **disables the cron** outright.

---

## 5. TLS / certificate errors talking to the manager

Almost always `DYNCLOUD_AUTH_BACKEND_SPKI` being set when it should not be. Setting it disables
normal validation and pins the hash of the full DER certificate, which changes at every Let's
Encrypt renewal. Unset it and let the module do TOFU on first contact.

---

## 6. The deploy action fails on `git pull` of the environment repo itself

Not related to this procedure. The error names the **environment's own** repository host, and the
pull runs with the server's default SSH key — not with any key Dyncloud manages for the modules.
Fix the server's access to its own repo; the connector work is untouched by it.

---

## Reporting upward

When the cause is a permission or a platform-level decision — the Reporter/Maintainer token, the
fleet-wide `requiredModules`, a series that needs porting — **stop and hand it over** with the
concrete finding rather than working around it silently. Each of those affects every environment,
not just the one in front of you.
