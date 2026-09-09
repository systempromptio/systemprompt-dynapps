> Astound marketplace adaptation: Historical upstream project template, not Claude Code installation instructions or facts about your org. Cursor paths and agent rosters describe the source kit and are not installed by this marketplace. Use the installed sf-salesforce-project-context skill and the current repository instructions for this client.

# Branch and Deploy Practice

The full version of the rule that lives in `.cursor/rules/always-active.md`. Apply this for **every feature and every bug**; no exceptions.

---

## 1. Branch from `develop` (or `main` where there is no `develop`)

Branch naming:

| Work type | Pattern |
|---|---|
| Feature for a story / ticket | `feature/<STORY-ID>` (e.g. `feature/PROJ-123`) |
| Bug fix for a ticket | `bug/<TICKET-ID>` (e.g. `bug/PROJ-456`) |
| Refactor (no story) | `refactor/<short-description>` |

Do not commit feature / bug work directly to `develop` or `main`.

```bash
git checkout develop
git pull
git checkout -b feature/<STORY-ID>
```

## 2. Story folder (when there is a story / ticket ID)

Create the story folder before implementation:

```
stories/<STORY-ID>/
    requirement.md         # original story text, verbatim
    design-note.md         # design decisions, schema impact, deployment ordering, rollback
    deployment.md          # exact commands and order
    qa-test-request.md     # automated + manual tests + video capture
```

See `stories/README.md` for the templates.

The story folder is updated **in the same commit** as code / metadata changes. PR Reviewer blocks merges where the story folder is missing or stale.

## 3. Deploy from the branch to a non-prod sandbox

Use an **org alias** (never a username, never a hardcoded identifier):

```bash
sf project deploy start \
  --source-dir force-app \
  --target-org <sandbox-alias> \
  --test-level NoTestRun \
  --wait 15
```

For changes that touch many metadata types, prefer a **manifest**:

```bash
sf project deploy start \
  --manifest manifest/<STORY-ID>-package.xml \
  --target-org <sandbox-alias> \
  --test-level NoTestRun \
  --wait 15
```

After deploy, run impacted tests:

```bash
sf apex run test \
  --class-names <TestClass1> <TestClass2> \
  --result-format human \
  --code-coverage \
  --target-org <sandbox-alias>
```

## 4. PR into `develop`

- Open a PR; the description references `<STORY-ID>`.
- PR Reviewer checks: code standards, story folder freshness, schema doc updated, manual + automated test cases, video link or file present.
- After merge, the integration branch (`develop`) is deployed to the integration sandbox.

## 5. Promotion to UAT / PreProd / Prod

- Cut a release branch (e.g. `release/<version>`) from `develop`.
- Validate to UAT with `sf project deploy validate --test-level RunLocalTests`.
- Quick-deploy to UAT, then to PreProd / Prod with environment approvals and a release tag.
- Production deploys only from `main` (or a release branch promoted to `main`) with explicit approval and rollback steps in the release notes.

## 6. Hotfix path

For a production hotfix:

- Branch `hotfix/<TICKET-ID>` from `main`.
- Apply the minimal fix; story folder still required (a thin one is fine).
- Validate + deploy to a sandbox first.
- Promote to production with the same approval gate as a normal release.
- Merge `main` back into `develop` after the hotfix lands so the fix is not lost in the next release.

## 7. Hard rules

- Never commit secrets.
- Never deploy to production without an approved release tag and rollback steps.
- Never run destructive changes outside an approved destructive workflow.
- Never deploy a partially-edited profile (especially `Admin`) without retrieving the full profile first.
