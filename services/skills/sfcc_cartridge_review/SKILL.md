# Cartridge Change Review

You review a diff that touches Salesforce B2C Commerce server-side code — cartridges, controllers,
ISML, `hooks.json` and hook scripts, `rest-apis/` custom SCAPI endpoints, job steps, `services.xml`,
and metadata XML — and produce a findings table plus one verdict: **SAFE TO DEPLOY** or **NOT SAFE
TO DEPLOY**. The verdict is derived from the table, never from impression.

## Use when

- "review this cartridge change", "is this hook safe", "check my controller", "review the ISML"
- "review the custom API", "can we deploy this to staging", "look at the metadata delta"
- Do not use for React/Storefront Next UI review; that is `code_review`.

## Inputs

Ask for:

- Which diff: `git diff <integration-branch>...HEAD`, `git diff --cached`, a PR number, or files.
- The ticket key if the branch name does not carry it.

Infer from the repo:

- Compatibility mode from `sfcc/dw.json` or the project docs; it decides whether ES6 syntax is a
  Blocker (Rhino 1.7R5 under 18.10-era modes) or fine.
- Cartridge path order from `sites/<SiteID>/site.xml`; it decides whether an override is reachable.
- The project lint command, normally `cd sfcc && npm run lint`.

## Workflow

1. **Ground the review in project knowledge.** Call the `knowledge-bank` MCP tool `search_knowledge`
   with `{"query": "<cartridge name> <hook or controller names> <ticket key>", "top_k": 8}`, then
   with `source_types: ["project_tickets"]` for past incidents on the same hook or endpoint. Cite
   hits as `title — uri#lines` in the finding they support. If the server is unavailable, say so
   in the report header and continue.
2. **Scope.** Run the diff command, list every changed file, and read each one in full; never
   review from the hunk alone. Only findings inside the diff count, unless the change breaks
   something it touches.
3. **Run the lint** on the sfcc workspace. Errors introduced by the change are Blockers.
4. **Pass A — Runtime safety (Rhino).** ES5 shape when the compatibility mode demands it; no
   `java.*` or `Packages.*`; no async, promises, or timers; `require` inside functions, not at
   module scope, except `dw/` modules; `JSON.stringify` never on a Java object; collections
   iterated with iterators when large; `Transaction.wrap` around every persistent write.
5. **Pass B — Extension discipline.** Overrides go through `module.superModule` or a controller
   `server.extend`/`append`/`prepend`; no platform source copied in; no `.ds` or `.isml` fork of
   a base template when a slot exists; the cartridge is actually in the site path.
6. **Pass C — Hooks and SCAPI.** Hook names in `hooks.json` match exported function names;
   `afterPOST`-style hooks return `undefined` to let the chain continue unless stopping it is the
   point; `modifyResponse` hooks only add `c_` fields; hooks never widen what a shopper can change
   (price, discount, shipping, role). Custom endpoints: `api.json` path and handler name match,
   custom query params carry the `c_` prefix, no `additionalProperties` in request bodies, no
   `Cache-Control` set by hand (the gateway blocks it), `response.setExpires` used deliberately.
7. **Pass D — Security and money.** No credentials inline (encrypted `services.xml` is fine and
   must be committed); order and basket math server-side; no PII, tokens, or card data in
   `Logger` calls or cache keys; ISML output encoded (`encoding="on"` or `${...}` defaults, never
   `encoding="off"` on user data); redirects validated; every `dw.svc` call checks
   `result.status`.
8. **Pass E — Metadata and preferences.** Changes live in `sfcc/meta_delta/`, never
   `meta_sandbox/`. A new boolean preference is a Blocker unless it ships a committed default for
   every environment, a default-on read helper, and a `Logger.warn` on the disabled branch. Any
   attribute referenced in code is defined in the delta.
9. **Pass F — Cache correctness.** `CacheMgr` keys include site, locale, and catalog scope;
   values are primitives or plain objects; `null` not `undefined`; entries under 128 KB; the code
   survives a cold cache.
10. **Write the report** in the Result shape. Derive the verdict: any Blocker means NOT SAFE.

## Result

Header: files reviewed, lint command and outcome, knowledge-bank status.

| # | Severity | File:line | Pass | Finding | Fix |
|---|---|---|---|---|---|
| 1 | Blocker / Major / Minor | `cartridge/scripts/hooks/basket.js:42` | C | one sentence | concrete patch or pseudocode |

Severity calibration:

| Severity | Meaning |
|---|---|
| Blocker | Breaks the storefront, leaks data or money, silently disables a feature, fails lint |
| Major | Wrong on some path: unreachable override, unscoped cache key, unchecked service result |
| Minor | Style, naming, comment discipline, JSDoc noise |

Verdict line, exactly one of:

- `SAFE TO DEPLOY — 0 Blockers, <n> Majors listed for follow-up`
- `NOT SAFE TO DEPLOY — Blockers #<list> must be fixed first`

Followed by one sentence of reasoning and the deploy note: which of code deploy, metadata import,
and cache flush the change needs, in that order.

The user verifies by opening each `File:line`, re-running the lint, and confirming that the verdict
follows the calibration table. A verdict that contradicts the table is wrong.

## Guardrails

- Never invent a `dw.*` class, hook id, endpoint, or scope. Verify in `sfcc_api_classes` or the
  platform docs before flagging; if unsure, mark the finding `Assumption`.
- An OCAPI 404 is not proof a resource is missing; do not report infrastructure as absent from it.
- Do not flag committed encrypted `services.xml` credentials as a security issue.
- Findings on money, auth, or hooks around mutable resources are at least Major.
- Do not inflate severity. If unsure, go one level lower and say why.
- Do not fix the code. Report, verdict, stop.
- If a finding reveals a repeatable pattern, add one line under `Process:` naming the rule or
  lint that would catch it next time.
