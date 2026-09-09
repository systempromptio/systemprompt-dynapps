---
name: requirements-publish
description: Publish the exact human-approved specification to Confluence and verify its remote content and approval receipt.
---

# Publish Approved Requirements

The requirements-workflow MCP server owns revisions, validation and approvals. Read current state before mutations and supply `expected_revision`. Inspect the saved validation result; only a passing result permits review. A quality score never grants approval. Give the human the returned review URL and stop until their signed browser decision appears in workflow state. Never submit the browser approval form yourself.

1. Read current workflow state and require the signed publication approval for the current specification and destination. If either changed, obtain a new review. A chat message, model score or editable checkbox is not a signed workflow approval.
2. Call `publish_requirements`. The server sends the approved content through the connected Atlassian MCP. Do not use another write tool to bypass this operation. An uncertain write requires read-only reconciliation against its returned status and intended page; never blindly repeat a create or claim success.
3. Call `verify_requirements_publication`. Return the actual Confluence URL, workflow revision and verified result. If readback differs, report verification failure with evidence. Preserve the artifacts and receipt for review.
