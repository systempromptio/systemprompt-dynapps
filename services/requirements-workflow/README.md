# UK requirements

Three skills in the Europe storefront plugin build context, write a lean
specification, and publish approved content. OpenSpec is no longer a runtime
dependency. Deterministic validation runs in Rust without a Node evaluator or
an unmetered external model call.

## Workflow

1. Search Knowledge Bank for relevant project evidence. Call
   `start_requirements_workflow` with `name`, `project`, a description in
   `delivery_scope`, and `sources: [{source_type, source_ref}]`. Select 1–40
   sources; Jira is optional. The server reads caller-granted indexed sources
   and re-fetches selected Confluence pages through the connected Atlassian MCP.
   Missing evidence fails explicitly. The 120 KB evidence limit rejects oversized
   selections without truncation; narrow the scope before retrying.
2. Submit `context.md` using `submit_context`. Validate `phase: context`, inspect the saved result, and request `gate: context`. The owner reviews
   scope, evidence and the draft in their signed-in browser.
3. Once approved, submit `specification.md` using `submit_specification`.
   Validate `phase: specification`. Set the Confluence destination using
   `set_requirements_destination` with cloud ID, space ID, parent ID and title.
   Request `gate: publication` and wait for the owner's browser approval.
4. `publish_requirements` creates the approved specification through Atlassian
   MCP, with a bundle-specific title suffix. Readback verifies canonical Markdown.
   `verify_requirements_publication` detects later modifications.

Every mutation requires the current `expected_revision`. Changing evidence or
context invalidates downstream artifacts and approvals. Changing the destination
invalidates publication approval. Source freshness is rechecked before mutations.
Model evaluations never substitute for human approval. Deterministic checks
explicitly report that semantic quality has not been scored.

Publication reserves an attempt before sending a write. An uncertain outcome
requires read-only reconciliation; a second create is not automatically sent.
Existing source grants, owner checks, browser nonces and signing keys are retained.
Keep the `requirements_workflow_signing_seed` secret unchanged when deploying.

## Validation

Use `just requirements-check`, `just requirements-test` and
`just requirements-integration`. The evaluation platform implementation and
paid benchmark remain separately tracked in `evals.md`; this workflow's structural
validation is not an agent/model benchmark.
