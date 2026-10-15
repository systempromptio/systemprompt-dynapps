# Workflows — destroying an environment

One recipe, written out in full because it is the one where a wrong id cannot be taken
back. Tool details in `../tools-destructive.md`; the index is `README.md`.

---

## 1. Destroy an environment, end to end

### Confirm the target before anything else

```text
list_environments { project_id }
  → id, name, stage_type, project_type, state
```

**Read the name and the stage back to the user and get an explicit yes.** An id is not a
name, ids are adjacent, and `stage_type: "production"` on the row you are about to destroy
is the single most important thing on the screen.

Worth checking what you would be throwing away:

```text
platform_list_backups { environment_id }     # is there a recent backup, if this was a mistake?
platform_env_status { environment_id }       # deletion protection on? any work in flight?
```

If `health_issues` contains `unprotected`, the environment is **missing** the deletion
protection its stage expects — which is why the destroy will go through on the first try.
That is a reason to be more careful, not less.

### Hit the gate

```text
platform_destroy_environment { environment_id }
  → E_CATEGORY_NEEDS_GRANT        ← nothing prepared, nothing enqueued
```

Expected. The access engine runs before the handler, so discovering the requirement this
way is safe.

### Ask, and wait for a human

```text
request_mcp_access { environment_id, scope: "category",
                     category: "platform_destructive", duration_hours: 1,
                     reason: "Destroy <project>/<env> at <who>'s request — <why>" }
  → { name: "BKRG/2026/00010", state: "pending", auto_approved: false,
      approver_count: 4, next_step: "Grant is pending approval…" }
```

**Stop.** Give the user the grant name and what it authorises. You cannot approve it from
here — that is the design, not an obstacle, and the `reason` you wrote is all the approver
will see. Then poll:

```text
get_my_mcp_access { environment_id }
  → effective_state: "active", remaining_minutes: 59
```

Read `effective_state`, not `state`: a row can still say `active` while the clock says
otherwise.

### Destroy

```text
platform_destroy_environment { environment_id }
  → { ok: true, action_ids: [218] }
```

**The grant was the approval.** There is no second confirmation and no dry run: this
prepares the full destruction chain and enqueues it immediately.

### Follow it out

```text
platform_list_actions { environment_id, limit: 1 }
  → the destroy chain, step by step
```

The chain takes a while — a cluster cascade is not instant — and the environment record
disappears partway through, so a later `platform_env_status` on that id returning "not
found" is the expected ending, not an error.

If a step fails, note that **`requeue` on a `k8s_destroy_*` action is unavailable** due to
a known platform defect and returns a typed error. Escalate to a human rather than
improvising a cleanup.

### Give the access back

```text
revoke_mcp_access { grant_id }
```

Do not leave a destructive grant open for the rest of its TTL because the task finished
early.
