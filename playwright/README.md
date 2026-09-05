# The Playwright harness

Browser coverage for the admin console: one deterministic dataset, seven signed-in
principals plus five bridge principals with their own tokens, a page spec per page,
and a narrated demo book built from the same run.

Everything here runs against an **already-running** stack. It never boots a server:
this clone's server is shared between agents, so global setup pings `/health` and
fails fast instead of starting one.

```bash
just start                    # once, in another shell
just e2e-install              # once per clone
just e2e-seed --reset         # rebuild the dataset from scratch
just e2e                      # the functional + requirement suites
just e2e --project chromium   # what CI runs
just e2e --grep @REQ-marketplaces   # the signed manifest + knowledge-bank tool audit
just e2e-demo                 # re-seed, walk the console, render the demo book
```

`GATEWAY_URL` overrides the base URL, `E2E_DATABASE_URL` the database the seed
writes to, and `E2E_SIGNING_KEY` the key tokens are minted with. Without them the
harness reads the `local` profile.

## The dataset

`setup/seed.ts` is the connection, the order and the reset; the data itself lives in
`setup/seed/`. Every row carries an `e2e-` id prefix or an `@e2e.local` email, and
`--reset` deletes exactly those rows — never a TRUNCATE, never a developer's data.

| module | what it owns |
|---|---|
| `kit.ts` | `T0`, the offsets, the seeded PRNG, the id vocabulary |
| `principals.ts` | the seven principals, the five UUID-keyed bridge principals, their roles, their sessions, the twelve traffic actors |
| `people.ts` | groups, projects, memberships, marketplace entitlement rules, the knowledge workers' Salesforce links |
| `pats.ts` | one deterministic PAT per bridge principal, real sha256 hash, secrets written to `.auth/pats.json` |
| `entitlements.json` | what each bridge principal's signed manifest must contain, exactly |
| `traffic.ts` | sessions, contexts, 240 AI requests, 60 tool executions, 40 skill invocations |
| `governance.ts` | decisions at all four chain stages, safety findings, approvals, anomalies |
| `bridge.ts` | bridge sessions, devices, certificates, API keys, pending device links |
| `legacy.ts` | the two older analytics trails the requirements evidence pack was written against |

Times are fixed offsets from `T0`, the instant the seed started, so two runs an hour
apart produce the same shape shifted forward. That is what lets a KPI assertion name
an exact number while the dashboards still show data that looks recent.

Mutating specs never edit a seeded row. They create their own subject with
`uniqueEmail()`, which the reset backstop cleans up.

### The bridge principals

`/v1/auth/bridge/pat` parses the token owner's id as a UUID, so the readable
`e2e-*` principals can never fetch a manifest. `principals.ts` therefore adds five
fixed-UUID users, one per entitlement band, all `e2e-bridge-<band>@e2e.local`:

| principal | roles | membership | manifest carries |
|---|---|---|---|
| `europe` | developer, user | europe-devs + storefront | enterprise-demo, astound-europe-dev; knowledge-bank |
| `india` | developer, user | india-devs | enterprise-demo, astound-india-dev; knowledge-bank |
| `knowledgeWorker` | knowledge_worker, user | none (role-gated, Salesforce linked) | enterprise-demo, astound-cowork; knowledge-bank, salesforce |
| `unassigned` | user | none | enterprise-demo only |
| `admin` | admin, user | none | enterprise-demo incl. astound-admin; systemprompt, knowledge-bank |

`entitlements.json` is the expected set per principal; `tests/bridge-manifest.spec.ts`
checks the file against `services/` (so it cannot drift) and then asserts set equality
against the signed, signature-verified manifest each PAT receives. Because these ids
carry no `e2e-` prefix, every reset predicate that keys on `user_id LIKE 'e2e-%'` has
an `@e2e.local` twin in `seed.ts` — extend both when you add a table.

`tests/gateway-skill-kb.spec.ts` drives one skill-shaped knowledge-bank tool call
through `/v1/messages` as the europe developer; the mock upstream answers the
`[e2e:kb]` marker with a `tool_use`, and the spec asserts the `ai_request_tool_calls`
row the gateway audits.

## Writing a page spec

Copy `tests/pages/_template.spec.ts`. Four describe blocks, always these four names:

- **renders** — the page draws its content from the seeded dataset
- **actions** — every mutation the page offers, and its visible result
- **authorization** — a table over the seven principals plus anonymous, each with a
  literal expected status
- **design language** — active nav item, breadcrumb, `h1`, `--sp-` tokens in computed
  style, the empty state when filtered to zero, a visible focus ring, no horizontal
  scroll at 1440 or 1024, and zero serious or critical axe violations

`scripts/check-spec-shape.sh` fails a spec missing one of them.

Never write a literal admin URL or a literal CSS class. Routes come from
`tests/support/paths.ts`, component classes from `tests/support/pages/selectors.ts`.
Both exist so a rename is one edit instead of a sweep.

## Fixtures

`tests/support/fixtures.ts` exports a page per principal — `adminPage`,
`platformAdminPage`, `developerPage`, `businessPage`, `unassignedPage`, `pmPage`,
`userPage`, `anonPage` — plus `cookieFor`, `apiAs`, `uniqueEmail`, `snapshot` and
`dbRow` / `dbRows`. Playwright resolves `storageState` per file, not per test, which
is why principals are page fixtures rather than an option override: two principals in
one spec compose only this way.

Bridge-side helpers live in `tests/support/manifest.ts`: `seededPat(role)` reads the
token the seed minted, and `fetchManifest(baseURL, bearer)` verifies the Ed25519
signature against `/v1/bridge/pubkey` before it hands the payload back.

`snapshot(page, name)` writes baselines under
`tests/<dir>/__screenshots__/<file>/chromium-linux/`. It asserts nothing off Linux, so
a developer on macOS never rewrites the baselines CI compares against.

## The demo book

`just e2e-demo` re-seeds, runs the `demo` project, and renders
`playwright/demo/index.html` — one chapter per section, one captioned figure per step.
Both the book and its shots are gitignored: they are a deliverable, not a source.

A demo spec asserts nothing. It walks one section in the order a person meets it and
calls `step(page, title, {what, does, design}, fn)` at each stop. The recorder throws
if a step lands on the login page, because a book full of screenshots of a sign-in
form looks plausible and is worthless. `tests/demo/_template.demo.ts` is the contract;
there is one file per section, named for it.

## Gates

Two read-only shell checks run in `just lint-gates`, in `just e2e`, and in CI:

- `scripts/check-spec-shape.sh` — every page spec has all four describe blocks
- `scripts/check-req-tags.sh` — no frozen `@REQ-NNN` id in `requirements.txt` has
  stopped being tagged by a spec. `@REQ-marketplaces` is a feature tag, not a
  register row: it groups the manifest and knowledge-bank specs for
  `just e2e --grep @REQ-marketplaces` and is deliberately absent from the file.

`just preflight` gains an `e2e-gate` tier that fails loudly, naming the command to
run, when no stack is up. A preflight that silently omits the browser tier is worse
than one that stops.

CI runs the suite in the `e2e` job of `.github/workflows/gates.yml`: debug build,
`setup-local` against its own Docker Postgres, `just start`, wait on `/health`, then
`just e2e --project chromium`, with `playwright-report/` uploaded on every result.
