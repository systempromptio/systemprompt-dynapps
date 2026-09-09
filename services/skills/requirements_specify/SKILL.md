---
name: requirements-specify
description: Write a lean, client-ready specification with cited requirements and testable acceptance criteria from human-approved context.
---

# Write Requirements Specification

The requirements-workflow MCP server owns revisions, validation and approvals. Read current state before mutations and supply `expected_revision`. Inspect the saved validation result; only a passing result permits review. A quality score never grants approval. Give the human the returned review URL and stop until their signed browser decision appears in workflow state. Never submit the browser approval form yourself.

1. Read the workflow and require the signed context approval. Use the frozen context and sources; newly discovered scope belongs in a revised context and renewed approval.
2. Write `specification.md` for the people deciding and delivering the change. Include an executive summary, outcomes, scope, requirements and measurable acceptance criteria, relevant dependencies, risks, assumptions and unresolved decisions. Use stable requirement identifiers and cite frozen source URLs. Include negative and boundary behavior where relevant. Avoid invented implementation details and boilerplate sections that add no decision value. Explain why the change matters and what success looks like.
3. Review every requirement against the approved scope and supporting source. Make acceptance criteria observable and unambiguous. Flag unsupported proposals and unresolved decisions; never describe them as approved. Submit using `submit_specification` and validate `phase: specification`. Deterministic validation does not establish semantic quality; inspect the actual document before requesting review.
4. Confirm the intended Confluence cloud, space, parent and title through the connected Atlassian MCP and user context. Set them using `set_requirements_destination` with `destination: {cloud_id, space_id, parent_id, title}`. Request `gate: publication`. Show the human the exact document, destination and outstanding decisions with the returned review URL. Stop for their decision.
