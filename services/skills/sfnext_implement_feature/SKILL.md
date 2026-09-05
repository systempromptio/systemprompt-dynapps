# Implement a Storefront Next Feature

You build one approved step of a change in a Storefront Next project (React 19, React Router 7
framework mode, Tailwind 4, SCAPI on MRT) or its hybrid SFRA cartridges, and hand back a diff plus
a filled house-rules checklist. The diff is not done until every checklist row is `PASS` or has a
written reason, and lint and typecheck are clean.

## Use when

- "implement step N of the proposal", "build the <route/component/hook>", "add a loader/action for"
- "extend the cartridge to", "add the SCAPI hook", "make this test-addressable", "translate this"
- Do not use to plan (`sfnext_plan_change`) or to prove the change (`sfnext_prove_with_playwright`).

## Inputs

Ask for:

- The approved `openspec/<change-id>/proposal.md` step, or the ticket plus a one-line scope if the
  change was sized as a quick fix. Without one of these, stop and ask.

Infer from the repo:

- Shape: `src/routes/` with flat-route names (`_app.product.$productId.tsx`) is Storefront Next;
  `sfcc/cartridges/**` is the SFRA/hook side. Confirm the actual layout from a sibling file.
- The API client factory (`createApiClients(context)`) and fetcher hook the project already
  exports; the shadcn components under `src/components/ui/`; the i18n namespaces under
  `src/locales/<locale>/`.
- The quality-gate commands from `package.json` scripts, normally `pnpm lint && pnpm typecheck`,
  plus `cd sfcc && npm run lint` when cartridge files change.

## Workflow

1. **Ground the build in project knowledge.** Call the `knowledge-bank` MCP tool `search_knowledge`
   with `{"query": "<feature nouns> <route or component names> pattern convention", "top_k": 8}`,
   then again with `source_types: ["project_docs"]` for the FSD/ISD acceptance criteria. Note the
   hits you follow as `title — uri#lines`; they go in the checklist's Evidence column. If the
   server is unavailable, say so once and continue from the repo.
2. **Read the modelled-on file** named in the proposal step and its nearest sibling. Match its
   naming, layout, error handling, and comment density. Do not introduce a second way of doing
   something the file already does.
3. **Write the data layer first.** Route data lives in `loader`; mutations live in `action` with
   forms that work without JS; on-demand fetches after load use the project's `useScapiFetcher`.
   Independent requests are returned as parallel promises, never awaited in sequence. Await only
   what the shell or SEO needs and stream the rest through `<Suspense>` + `<Await>`.
4. **Write the UI.** Presentational components receive props; logic lives in hooks. App-wide and
   feature-local client state is a scoped React context provider, not a new store. Tailwind
   utilities and `cn()` only; no CSS modules. Add shadcn primitives with `npx shadcn@latest add`,
   never by hand. Every user-facing string goes through `useTranslation('<namespace>')` with a
   key added to each locale file; Zod messages use the `createSchema(t)` factory.
5. **Add `data-testid` where a test will select it**: feature root, forms, CTAs, list rows with a
   stable data id suffix, dialogs, skeletons, empty states. Kebab-case, no prefixes, on the control
   itself for form fields. Skip layout-only wrappers and anything `getByRole` already finds.
6. **Cartridge side, if any.** Extend through `module.superModule`, hooks registered in
   `hooks.json`, or a route override; never copy platform source into the project. Rhino is
   ES5-shaped unless the compatibility mode says otherwise. Metadata goes to `sfcc/meta_delta/`,
   never `sfcc/meta_sandbox/`. A new site preference needs a committed default for every
   environment and a `Logger.warn` on the disabled branch, or it is not added.
7. **Run the quality gate** once, after the whole step: `pnpm lint && pnpm typecheck`, plus the
   sfcc and playwright lints when their files changed. Fix every error you introduced; note
   pre-existing ones without blocking.
8. **Fill the checklist** (Result) from the diff, not from memory. Re-read `git diff` while
   filling it. Then hand off to `sfnext_prove_with_playwright`; the step is not complete until
   that skill reports a passing run.

## Result

- The working-tree diff (`git diff --stat` and the full diff in the reply), uncommitted.
- This table, every row filled:

| Rule | Status | Evidence |
|---|---|---|
| Data fetched in loader/action, not in components | PASS / FAIL / N/A | file:line |
| Independent requests parallel, critical data awaited only | PASS / FAIL / N/A | file:line |
| Extend, don't fork (superModule / hook / override) | PASS / FAIL / N/A | file:line |
| No new dependency outside `package.json` | PASS / FAIL | decision ref |
| `data-testid` on roots, CTAs, forms, rows, dialogs | PASS / FAIL / N/A | selector list |
| i18n: no hard-coded copy, keys in every locale | PASS / FAIL / N/A | namespace, keys |
| SSR-safe: no `window`/`document` outside effects | PASS / FAIL / N/A | file:line |
| No `any`, no `as` casts, no `@ts-ignore`, inferred returns | PASS / FAIL | grep output |
| Comments: one file header, one-line intent, no narrative | PASS / FAIL | file list |
| No kill-switch preference without the four conditions | PASS / N/A | pref id |
| `pnpm lint && pnpm typecheck` clean | PASS / FAIL | command output |
| Knowledge-bank citations followed | PASS / UNAVAILABLE | title — uri#lines |

The user verifies by running the quality-gate command themselves and by opening each Evidence
pointer. A `FAIL` row with no fix in the diff means the step is not ready for review.

## Guardrails

- Never start without an approved step or an explicit quick-fix scope.
- Never commit, push, or open a PR; that is `b2c_release_notes` territory and needs authorisation.
- Never reset or discard file changes you did not make.
- Never `.skip()` or `fixme()` a failing test to get green.
- When two implementations are plausible, stop and present both with trade-offs; do not guess.
- Never claim a platform bug without a link. Investigate config, frontmatter, and the API call
  first.
- If you loop on the same error three times, stop, write what you tried, and ask.
