# Per-Module Migration Loop

Exact steps to migrate one custom module (Step 5 of the workflow).
Repeat for every module in topological order (see `migration-order.md`).

## 1. Create feature branch

```bash
git checkout develop
git pull
git checkout -b <ticket>_migrate_<module>_v<target>
# e.g.: MIG-42_migrate_dynapps_invoicing_v<version_destino>
```

## 2. Edit `__manifest__.py`

Changes required:
- `"installable": True`
- `"version": "18.0.1.0.0"` (adapt target version)
- Replace Odoo-standard depends with `"xx_base"` (keep other custom module deps)
- Delete the `migrations/` folder entirely:
  ```bash
  rm -rf parts/custom/<module>/migrations/
  ```

Example before/after for `depends`:
```python
# Before
"depends": ["account", "sale", "stock", "xx_base"],

# After
"depends": ["xx_base"],
```

## 3. Remove from pre-commit exclude

In `.pre-commit-config.yaml`, remove the module's line from the `exclude` block.

## 4. Delegate code port to odoo-module-upgrader

Invoke `odoo-module-upgrader` on `parts/custom/<module>/`.
Pass source and target versions.

The upgrader will:
- Apply version-specific XML/Python/JS patterns.
- Leave `# TODO-PORT-v<target>` comments where no automatic pattern exists.

After the upgrader completes, review all TODO comments. Resolve or escalate (STOP point).

## 5. Run pre-commit and fix

```bash
pre-commit run -a
# Fix any remaining issues, then:
git add --all
```

## 6. Uncomment module in `xx_all/__manifest__.py`

Find the commented-out line and uncomment it:
```python
# Before
# "dynapps_invoicing",

# After
"dynapps_invoicing",
```

## 7. Test installation

```bash
./parts/odoo/odoo/odoo-bin -c odoo.cfg -d <db> -u <module> --stop-after-init
```

Watch for:
- `ERROR` lines in log
- Missing views / broken XML
- Field type mismatches

Fix iterate errors in coordination with the DB track (kit `indaws-migrate` commands).

## 8. Commit

```bash
git add --all
git commit -m "[MIG] Migrate <module> to v<target>"
```

## 9. Push branch — CONFIRM BEFORE EXECUTING

Before running `git push`, show the user:

```
Ready to push branch <ticket>_migrate_<module>_v<target> to GitLab.

  Branch: <ticket>_migrate_<module>_v<target>
  Remote: origin (git@gitlab.dynapps.be:...)
  Commits to push: N

Confirm? [Yes / No]
```

Only push after explicit confirmation. Never push autonomously even if the user
previously approved pushes for other modules — each push is a separate confirmation.

```bash
# After user confirms:
git push -u origin <branch>
```

## 10. Create MR — CONFIRM BEFORE EXECUTING

Show the full MR details before creating it:

```
Ready to create MR on GitLab:

  Title:  [MIG] <module> v<version_origen>→v<version_destino>
  Source: <ticket>_migrate_<module>_v<target>
  Target: develop
  Files:  N changed, +X -Y lines

Confirm MR creation? [Yes / No]
```

Only run `glab mr create` after explicit confirmation:

```bash
# After user confirms:
glab mr create \
  --title "[MIG] <module> v<version_origen>→v<version_destino>" \
  --target-branch develop \
  --assignee @me
```

## 11. Wait for pipeline — MANDATORY GATE

```bash
glab ci view
```

**Pipeline must be fully green** — no tolerated failures, no skipped jobs.
If pipeline fails → diagnose with `git-manager`, fix root cause, confirm push of fix commit,
wait for green again. Never advance with a red pipeline.

Checklist before notifying the technician:
- [ ] All pre-commit hooks pass on the MR branch
- [ ] All CI jobs green (lint, tests, install check)
- [ ] No `ERROR` or `CRITICAL` in Odoo install log
- [ ] Module visible in `ir.module.module` with state `installed`

## 12. Notify technician — STOP (manual merge)

Once pipeline is green, **stop and notify**:

```
MR ready for review and merge:
  Title:    [MIG] <module> v<version_origen>→v<version_destino>
  URL:      <mr_url>
  Pipeline: ✅ green

Please review and merge the MR in GitLab.
Confirm when merged to continue with the next module.
```

**The skill does NOT merge the MR.** Merging is always done manually by the technical team.

## 13. After technician confirms merge — update status and continue

```bash
indaws-migrate modules set <module> --status code_ready --note "MR merged, pipeline green"
```

> **Use `code_ready`, not `verified`.** The kit's status machine is explicit about what each one
> means (`indaws_migration_kit/src/indaws_migration_kit/migration/module_status.py:19-28`, `MODULE_STATUSES`):
> `code_ready` = *"código portado existe, sin probar"*, `verified` = *"-u all OK"*. A merged MR with
> a green pipeline is the first, not the second — the module has not yet survived an `-u all` against
> the migrated database.
>
> And it would fail anyway: transitions are validated
> (`indaws_migration_kit/src/indaws_migration_kit/migration/module_status.py:47-56`). `verified` is only reachable **from `code_ready`** (or from
> `blocked`), so a module still in the default `pending` cannot jump there. The CLI offers
> `--force` to skip the check — use it only when you know why you are lying to the tracker.

### The status machine, in one table

| Status | Means | Legal next |
|---|---|---|
| `pending` (default) | decided, port not started | `porting`, `code_ready`, `blocked`, `skipped` |
| `porting` | dev working on target-version code | `code_ready`, `blocked`, `pending` |
| `code_ready` | ported code exists, **untested** | `verified`, `blocked`, `porting` |
| `verified` | `-u all` OK | `smoked`, `blocked`, `code_ready` |
| `smoked` | smoke RPC + UI OK | `done`, `blocked`, `verified` |
| `done` | consultant signed off | `blocked` |
| `blocked` | waiting on something | anything |
| `skipped` | NEUTRALIZE / REMOVE / STUB / STUDIO | `pending`, `blocked` |

Anything can go to `blocked`, and `blocked` can go anywhere. Actions `NEUTRALIZE`, `REMOVE`, `STUB`
and `STUDIO` move a module to `skipped` **automatically** on load (`indaws_migration_kit/src/indaws_migration_kit/migration/module_status.py:35-40`).

Pull `develop` and start next module:

```bash
git checkout develop && git pull
```

Back to step 1 with the next module in topological order.

## Rules

- **One branch, one module, one MR** — always. Never mix modules.
- **Pipeline green before notifying** — hard requirement, no exceptions.
- **Skill creates MRs, never merges them** — merge is always a manual technical decision.
- **Every GitLab write action requires explicit confirmation** — push, MR create, MR update.
  A previous "yes" for one module does not carry over to the next.
- If a module depends on one not yet migrated → reorder or extract shared dependency first.
- `dyn_test_suite_common` — never set installable False, never migrate; ships with template.
- After each merge, always pull `develop` before starting the next module's branch.
