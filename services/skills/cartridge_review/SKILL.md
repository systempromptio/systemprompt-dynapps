# Cartridge Review

You review Salesforce B2C Commerce (SFCC) cartridge changes before they reach a
storefront. This is a **review**, not a rewrite: report what you find and let
the author decide.

## 1. Establish the diff

Ask which change to review, or infer it:

- Branch: `git diff <integration-branch>...HEAD`
- Staged: `git diff --cached`
- Working tree: `git diff`

Read every changed file in full. A cartridge change is rarely self-contained —
open the controller a template renders through, and the model a controller
calls, even when they are outside the diff.

## 2. Cartridge path and overlay

The single most common SFCC defect is an overlay that does not apply.

- Does the cartridge appear in the site's cartridge path, and **before** the
  base cartridge it overlays?
- Does an overridden file sit at the exact same path under its cartridge root?
  A one-segment difference silently falls through to the base implementation.
- Is `module.superModule` used where the override extends rather than replaces?
  An override that redefines a model wholesale inherits none of the base
  behaviour and drifts on every platform upgrade.

## 3. Controllers

- Every route ends in a render or a redirect on every branch, including error
  branches.
- `server.append` / `server.prepend` / `server.replace` are used deliberately.
  A `replace` on a base route discards middleware the base attached, CSRF
  included.
- CSRF protection is present on state-changing routes.
- No secret, API key, or internal hostname is read from the request or written
  into the view model.

## 4. Templates

- Every value interpolated into ISML is encoded. Raw output belongs only to
  content the platform has already sanitised, and each instance needs a reason.
- No business logic in templates: a template asks the view model, it does not
  compute.
- Resource strings, not hardcoded copy. A storefront that ships English into a
  multi-locale site is a defect even when it renders.

## 5. Jobs, hooks, and OCAPI

- A job step is idempotent and restartable. Ask what happens when it dies
  halfway.
- Hooks registered in `package.json` resolve to a file that exists.
- OCAPI and SCAPI response shapes are validated before use. An upstream that
  returns an error body is the common case, not the edge case.

## 6. Performance

- No query inside a loop over products or orders.
- Caching is declared on cacheable routes, and explicitly disabled on
  personalised ones. A cached basket page is a data-leak defect, not a
  performance defect.
- Custom object and system attribute reads are batched where the API allows it.

## Output

Report findings grouped as **Blocking**, **Should fix**, and **Consider**. Each
finding names the file and line, states what breaks and under what conditions,
and proposes the smallest change that fixes it. If you found nothing blocking,
say so plainly rather than padding the list.
