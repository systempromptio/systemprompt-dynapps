# Prove a Change with Playwright

You turn "it compiles" into "it works" by writing or adjusting the Playwright spec that covers the
changed flow, running it against a real server render, and reporting the command, the counts, and
the spec path. Nothing is done until this skill reports a pass. A review, a screenshot, or a unit
test is not a substitute.

## Use when

- "prove this works", "add the e2e test", "run Playwright for the change", "is step N verified"
- Before any PR is marked ready, and after every `sfnext_implement_feature` step.
- Do not use for ad-hoc browsing or screenshots; that is `playwright_cli`.

## Inputs

Ask for:

- The change: a proposal step, a diff, or the flow in words ("guest add-to-cart from PDP").
- The target: local dev server, a sandbox URL, or an MRT environment. Default to the project's
  `baseURL` in `playwright.config.ts`.

Infer from the repo:

- `playwright.config.ts` location, the specs directory (usually `playwright/tests/`), fixtures
  and helpers, and the `package.json` script (`test:e2e` or similar). Project configuration always
  wins over this skill.
- Existing `data-testid` selectors in the changed components. If a control has none and
  `getByRole` cannot name it, the fix is in the component, not a CSS selector in the spec.

## Workflow

1. **Ground the flow in project knowledge.** Call the `knowledge-bank` MCP tool `search_knowledge`
   with `{"query": "<flow nouns> acceptance criteria e2e <ticket key>", "top_k": 6, "source_types":
   ["project_docs"]}` to pull the acceptance criteria and any documented test accounts or seeded
   data. Cite hits as `title — uri#lines` next to the assertions they justify. If the server is
   unavailable, say so and derive the assertions from the proposal step.
2. **Find the nearest existing spec** for the flow: `rg -l "<route|feature>" playwright/tests`.
   Extend it if it exists; create `<feature>.spec.ts` beside it otherwise, using the same fixtures.
   If the project has no Playwright at all, scaffold the minimum (`@playwright/test`, chromium,
   a config with `baseURL`, one spec) and note that project-wide adoption is a planning decision.
3. **Write the assertions from the acceptance criteria**, one `test()` per criterion. Rules:
   `getByRole`/`getByLabel`/`getByTestId` only, never CSS or `nth-child`; web-first assertions
   (`await expect(locator).toBeVisible()`), never `waitForTimeout`; each test seeds its own state
   and assumes no order; cover the populated, empty, and error states the change introduced; a
   form that must work without JS gets one test with `javaScriptEnabled: false`.
4. **Confirm the server under test renders the change.** For a local run, restart the dev server
   so the SSR fetch cache is cold. For a sandbox or MRT target, confirm the code version or bundle
   is the one containing the change (`b2c code list`, `b2c mrt bundle history`); a green run
   against an old build proves nothing.
5. **Run scoped, then wide.** `npx playwright test <spec> --reporter=list`, then the full suite
   if the project's conventions require it before review. Chrome only during development;
   cross-browser follows CI.
6. **On failure, fix the change or the spec.** Never `.skip()`, `fixme()`, or loosen an
   assertion to pass. If the failure is environmental (missing seed data, credentials), report
   it as `BLOCKED` with the exact missing item and owner; that is a legitimate outcome, a
   silent skip is not.
7. **Lint the spec** with the playwright workspace lint (`cd playwright && npm run lint`) when
   one exists.
8. **Report** in the Result shape.

## Result

```
Spec:     playwright/tests/<feature>.spec.ts   (new | extended, +<n> tests)
Target:   <baseURL> — build <code version | bundle id | local dev, restarted HH:MM>
Command:  npx playwright test playwright/tests/<feature>.spec.ts --reporter=list
Outcome:  <passed>/<total> passed, <failed> failed, 0 skipped    (<duration>)
Covers:   AC1 <criterion> → test "<name>"   [title — uri#lines]
          AC2 ...
Status:   PROVEN | FAILED | BLOCKED (<missing item>, owner <role>)
```

Paste the reporter output verbatim under the block. The user verifies by running the same command
and by checking that every acceptance criterion maps to a named test. A report with `skipped > 0`
or without the verbatim output is not proof.

## Guardrails

- Never report `PROVEN` without a run you executed in this session against a build containing
  the change.
- Never quarantine a red test. Red means the change, the spec, or the environment is wrong, and
  the report says which.
- No `waitForTimeout`, no CSS selectors, no shared mutable state between tests.
- Do not add browsers or change `playwright.config.ts` beyond `baseURL` without a planning
  decision.
- Keep DOM snapshots out of the report; the reporter output and the spec path are the evidence.
- Test accounts and seeded orders come from the project's fixtures or the knowledge bank, never
  from a real customer.
