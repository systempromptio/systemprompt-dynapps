# Workflows — moving an environment forward

Releases, submodules, configuration, domains. Tool details in `../tools-write.md`; the
index is `README.md`.

Every recipe here ends the same way: **the call returning is not the change landing.**

---

## 1. Upgrade an environment to a new release

**k8s only.**

```text
list_release_tags { environment_id }
  → [ { id: 127, tag: "odoo-19.0-r24-rc8", is_current: false, has_content: true },
      { id: 126, tag: "odoo-19.0-r24-rc7", is_current: true },
      … ]
```

Read two fields before choosing:

- **`is_current`** — the one already pinned. Upgrading to it is a no-op that still enqueues
  a chain and still costs a re-render.
- **`has_content`** — `false` means the tag was discovered but its spec was never fetched,
  so there is nothing to deploy from.

Then:

```text
platform_deploy { op: "upgrade_release", environment_id, release_tag_id: 127,
                  target_environment_ids: [environment_id] }
  → { ok: true, action_ids: [228] }

platform_list_actions { environment_id, limit: 1 }
  → "Upgrade release main (odoo-19.0-r24-rc8)"
    registries: "Re-render overlay + fleet" → done, "Await ready after upgrade" → done
```

The tool needs `platform_ops`, which on production means a grant. Note the permission is
`:k8s_image_change:` — being allowed to scale or to create environments says nothing about
this.

**A no-op upgrade still un-quiesces an environment**, incidentally: if one is down because
a job left it that way, an upgrade re-renders the overlay and brings it back. Useful to
know, not a reason to use it as a repair tool.

---

## 2. Pin submodules

```text
platform_submodules_plan { environment_id }
  → { engine: "…", rows: [ … ], capabilities: { … },
      intro: "<what confirming will do, in the engine's own words>",
      warning: "<no submodules | could not read the manifest>" }
```

**Read the `intro` to the user and pass the `warning` through verbatim.** The two engines
do different things, and the `warning` distinguishes "this scope has no submodules" from
"the manifest could not be read" — opposite problems that look alike when summarised.

To choose a ref before pinning it:

```text
platform_submodules_refs { … row … }
  → { items: [ … ], has_more: true, truncated: false }
```

`truncated: true` means GitHub could not search server-side and the scan was capped; the
ref you want may exist beyond it. GitLab searches properly.

Then apply, and read the answer for what it actually is:

```text
platform_submodules_apply { … }
```

- **docker** → `action_ids`, including **one restart action**. The change lands when the
  chain finishes; verify with `platform_list_actions`.
- **k8s** → `branch` and `commit_url`, `action_ids` **empty**. Nothing was enqueued: a
  repository was patched and committed. Whatever picks that commit up runs on its own
  schedule, so **do not report a deployed change** — report a commit, with its URL.

---

## 3. Apply a configuration change on docker

If someone changed configuration and restarted and nothing happened, this is why.

```text
platform_rebuild { environment_id }
```

A `restart` does not pick configuration up, because it never goes through
`put_compose_extend`. `platform_rebuild` regenerates the compose extend and recreates the
containers with it. It is **not** destructive — no image removal, no volumes touched — so
it is the right reflex, not a last resort.

Docker only: `E_NOT_SUPPORTED_FOR_RUNTIME` on k8s, where configuration travels through
gitops instead.

---

## 4. Add or move a domain

```text
platform_list_domains { environment_id }
  → existing FQDNs, DNS state, TLS/CORS flags — and on k8s the release's available purposes
```

Read it first: on k8s the write needs a **valid `purpose_name`**, and this is where the
valid ones are listed.

```text
platform_domains { environment_id, op: "create", … }    # ORM write on cloud.fqdn
platform_domains { environment_id, op: "apply" }        # make DNS reflect it
```

`apply` is where the runtimes diverge and where people get caught:

- **docker** — zone records are created **synchronously**. When the call returns, DNS is
  changed.
- **k8s** — enqueues `k8s_update_domains` and returns an `action_id`. The effect lands
  through gitops, **later**. The call returning means the intent was recorded.

Validation on create/update comes from the `cloud.fqdn` model, not from the tool, so a
rejection is the model talking and its message is the real one.
