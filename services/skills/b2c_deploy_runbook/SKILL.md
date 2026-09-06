# Sandbox and MRT Deployment Runbook

You produce a deployment runbook for one change to a B2C Commerce instance and, when the project
has one, its Managed Runtime storefront: an ordered list of numbered commands covering code
version, metadata import, site export where a snapshot is required, MRT bundle push, cache
invalidation, verification, and rollback. Every step names the command, the expected output, and
the check that proves it landed. The runbook is written to disk, not only spoken.

## Use when

- "deploy this to <sandbox>", "push to staging", "write the deploy steps for the release"
- "what order do we deploy in", "how do we roll back", "the sandbox is out of sync"
- Do not use to create or delete sandboxes; that is billable and needs an explicit request.

## Inputs

Ask for:

- Target: instance hostname or `dw.json` profile, site id, and for MRT the project slug and
  environment. Never assume production.
- The change set: which of cartridges, `meta_delta`, site preferences, jobs, services, and the
  storefront bundle are in it. Derive from `git diff --stat <base>...HEAD` when the user is unsure.
- The code version name to deploy to. Default to the branch name plus the short SHA.

Infer from the repo:

- `sfcc/dw.json` for hostname and the current code version; `package.json` for `pnpm build` and
  `pnpm push` scripts; the MRT project from `MRT_PROJECT` or the docs.
- Whether `sfcc/meta_sandbox/site_template` is CI-generated (it usually is; never hand-edit it).

## Workflow

1. **Ground the runbook in project knowledge.** Call the `knowledge-bank` MCP tool `search_knowledge`
   with `{"query": "<site id> deployment environment <hostname> cartridge path code version", "top_k": 8,
   "source_types": ["project_docs"]}` for the project's environment list, OCAPI client
   permissions, and any documented deploy order. Cite `title — uri#lines` beside the steps they
   inform. If the server is unavailable, say so and derive everything from `dw.json` and scripts.
2. **Record the starting state** so rollback has a target:
   `b2c code list` (active version), `b2c mrt bundle history -p <project> -e <env>` (current
   bundle id), `git status` (uncommitted `sfcc/**` edits are not deployed and must be said so).
3. **Order the steps** by dependency, and write them all with numbered commands:
   1. Metadata first when code references new attributes or preferences:
      `cd sfcc && b2c job import meta_delta/site_template --show-log` on a bootstrapped
      sandbox; the full `meta_sandbox/site_template` archive first on a fresh one. Preference
      definitions live in the delta, values in `sites/<SiteID>/preferences.xml`; both must land.
   2. Cartridge code: `cd sfcc && b2c code deploy -v <version> --reload`, then `b2c code list`
      shows it active. Confirm the cartridge is in the site path with `b2c sites cartridges`.
   3. Custom SCAPI registration when `rest-apis/` changed:
      `b2c scapi custom status --tenant-id <t> --short-code <sc>` shows every endpoint `active`.
   4. Storefront bundle: `pnpm build`, `pnpm bundlesize:test`, then
      `pnpm sfnext push --environment <env> -m "<release message>" --wait`, or
      `b2c mrt bundle deploy -p <project> -e <env> -m "<message>"`. Page Designer metadata, if
      changed, goes with `pnpm deploy:cartridge` to the instance, not to MRT.
   5. Environment variables: `b2c mrt env var set KEY=value -p <project> -e <env>` for anything
      new, listed by name, never by value in the runbook.
   6. Caches, in this order: BM custom caches if a `CacheMgr` cache is involved; the JWA gateway
      cache waits out `setExpires`; `b2c mrt env invalidate -p <project> -e <env>` for the CDN.
4. **Write the verification block**: `b2c logs get --since 5m --level ERROR --level FATAL`
   is empty of new symbols; one smoke `curl -s -o /dev/null -w "%{http_code}"` per new endpoint;
   the storefront URL renders the change; the Playwright spec from `sfnext_prove_with_playwright`
   passes against the target.
5. **Write the rollback section**, one command per step in reverse: `b2c code activate
   <previous version>`; `b2c mrt bundle deploy <previous bundle id> -p <project> -e <env>`;
   metadata rollback by re-importing the previous delta snapshot, or an explicit statement that a
   new attribute is left in place because removing it breaks old data; `b2c mrt env var delete`
   for variables you added; cache invalidation repeated.
6. **Write the file** to `docs/runbooks/deploy-<env>-<change-id>.md` (or the project's runbook
   directory) and print it. Do not execute it unless the user says "run it".

## Result

The runbook markdown at `docs/runbooks/deploy-<env>-<change-id>.md` with, in order:
`## Target`, `## Starting state`, `## Steps` (numbered, each with command, expected output, and
check), `## Verification`, `## Rollback` (numbered, reverse order), `## Not in this deploy`
(uncommitted or excluded items). Each step reads:

```
### 3. Deploy cartridge code
cd sfcc && b2c code deploy -v feat-basket-a1b2c3d --reload
Expect: "Activated code version feat-basket-a1b2c3d"
Check:  b2c code list   → feat-basket-a1b2c3d marked active
```

The user verifies by running the checks in `## Starting state` before execution and the
`## Verification` block after; a runbook whose rollback section has fewer steps than its Steps
section is incomplete.

## Guardrails

- Never run a deploy, import, push, invalidation, or `env var set` without the user saying so
  for that target in this session. Writing the runbook is the default; executing it is opt-in.
- Never target production from an inferred value; the user names it explicitly.
- Never create, delete, or restart a sandbox.
- Never edit `sfcc/meta_sandbox/`; never import the full archive onto a bootstrapped instance
  (it overwrites unrelated attributes); prefer a minimal targeted archive for a single attribute.
- Secrets never appear in the runbook; reference variable names only.
- Uncommitted `sfcc/**` changes are named in `## Not in this deploy`, never silently included.
- An OCAPI 404 is not proof of a missing resource; check WebDAV or `b2c scapi custom status`.
