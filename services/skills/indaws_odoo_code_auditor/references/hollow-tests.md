# Hollow tests & latent-bug smells

Two audit aids: (A) detect **hollow tests** (green but verify nothing), and
(B) flag **code smells** that hide in the happy path and break at the edge.
Both are real classes that survived green suites in production code.

## A. Hollow-test detector (dimension: Code Quality / Tests)

A test is *hollow* when it would still pass if the feature were broken. Flag
when a test's only assertions are proxies:

> **The verdict is PER ASSERTION, not per test.** A test can be hollow in its
> headline assert and sound in the next line, and reporting "this test protects
> nothing" is then as false as passing it. Say *which* assert does not
> discriminate, and whether another one in the same test does.
> *(Measured 2026-09-02: a review reported a whole test as decorative; the
> assertion right below it did go red without the fix.)*

- `assertTrue(x)` / `assertIsNotNone(x)` on "something was produced" (attachment,
  record, non-empty result) **without** asserting its content/value.
- `assertIn(substr, output)` where `substr` is a constant present regardless of
  the logic under test (e.g. a tag name, a literal), not a value derived from input.
- length/size-only assertions (`len(x) == N`, `assertEqual(count, ...)`) with no
  value check.
- "no raise" as the sole assertion (`method()` then nothing, or
  `assertEqual(state, ...)` accepting a set that includes a stuck/intermediate state).
- **negative assertion on something the pipeline can never produce anyway** —
  `assertNotIn(b"\x97", raw)` over output that already goes through an
  `encode(..., errors="ignore")`: the byte could not appear with or without the
  fix, so the assertion is vacuously true and protects nothing. Ask, for every
  negative assert: *could the thing I am forbidding ever get here at all?*
  *(Measured 2026-09-02, `indawsodoo/indaws-edi#30`: that assert named the test.)*
- the **controlled input never provably reaches the output** under test
  (e.g. asserting on a generated file while feeding a value that the format never
  writes) → the assertion is meaningless.

Report: *"Test `X` is hollow — passes without verifying behavior. Strengthen to
assert the exact value/state, and prove the input reaches the output."* Severity:
warning (advisory) unless it's the only test for fiscally/legally sensitive output
(amounts, signatures, EDI) → elevate.

## B. Latent-bug smells (map to the relevant dimension)

| Smell (grep hint) | Why it's a bug | Dimension |
|---|---|---|
| Transliteration/`to_ascii`/`unidecode`/`.encode('cp1252')` applied to the **concatenated fixed-width line** (after `''.join`) | An expanding replacement (`€`→`EUR`) shifts every later position → misaligned record. Must be per-field **before** padding. | Code Quality / Correctness |
| Amount/numeric field that **truncates** on overflow (`[:length]`, `truncate=True`, `ljust/rjust` then slice) without raising | Silent fiscal corruption (`12345678901.99`→`...901.9`). Numerics must fail loud, not truncate. | Code Quality |
| `float` used for money arithmetic where rounding matters; per-line tax recompute summed instead of `amount_tax` | Cent mismatches base/cuota/total. | Code Quality |
| `search`/domain on a multi-company model **without** company filter when iterating per-account/company | Cross-company leak. | Security / Conventions |
| `cr.commit()` inside a business method (not a cron/queue boundary), or use of an API that commits mid-flow (`_create_records_from_attachments`) without idempotency guard | Partial state on error + duplicate on retry. | Concurrency-atomicity |
| Config **`data` with `noupdate="1"`** that the team maintains as product (layouts, mappings) | Fixes to these records do NOT propagate on `-u`; need a migration or `noupdate="0"`. Flag missing migration when such data changed. | Migrability |
| `sudo()` in read/write paths without a comment justifying the privilege escalation | Possible record-rule bypass / data leak. | Security |
| Computed field with `@api.depends` that omits a dependency it reads | Stale value, not recomputed. | Code Quality |

Cross-cut: most of these "work in the happy path". When auditing tests for a
module with any of these smells, check that an **adversarial** test exists
(see odoo-test-generator `references/adversarial-cases.md`); its absence is itself
a Code Quality finding.
