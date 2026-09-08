---
name: sf-integration
description: Use when implementing or reviewing Integration (Domain Knowledge) in
  Salesforce.
---

## Systemprompt execution contract

The marketplace provisions authenticated `atlassian`, `salesforce`, and `github`
MCP connectors for the signed-in user. Use their advertised tools and input
schemas. Never request, print, or copy credentials, and never bypass Systemprompt
with direct provider HTTP requests. Missing tools or rejected authentication are
failures: name the connector and stop the affected operation. Do not substitute
mock data or report an unexecuted action as successful.

Use the current repository as project context. Confirm unknown project identifiers
from authenticated reads; never invent an org, Jira key, schema, or repository.
Respect the user's existing authorization and the review requirements below.
For provider writes, use the format accepted by the advertised input schema.

Atlassian v2 exposes primary tools plus `discover` and `executeRead`/`executeWrite`.
Inspect live schemas; do not assume legacy Jira or Confluence tool names.

# Skill: Integration (Domain Knowledge)

Use this when implementing or reviewing integrations.

## Expertise areas

- **REST / SOAP callouts:** Outbound HTTP, authentication (OAuth, basic, certificate, JWT).
- **Named Credentials:** Setup and use for endpoints and auth; no hardcoded URLs / secrets.
- **External Services:** Declarative external callouts and OpenAPI.
- **Inbound:** REST resources (`@RestResource`), Platform Events, Apex Web Services.
- **Testing:** Mock callouts (`Test.setMock`, `HttpCalloutMock`), Stub API for complex flows.

## Key patterns

- Use **Named Credentials** for URLs and auth; no hardcoded secrets.
- Async for high-volume or long-running callouts (Queueable, future) where appropriate.
- All callout paths covered by tests with mocks; no live calls in unit tests.
- For inbound REST, validate caller identity via Connected App + named user, not anonymous endpoints.
- Idempotency: integration endpoints should accept a client-supplied id (or compute a deterministic one) so retries do not create duplicates.

## Bypass for integration users

- Use a **Custom Permission** (assignable via permission set) to gate bypass logic — e.g. allow an integration user to update records past a guard rule that blocks regular users.
- Assign the permission set to the integration user in each environment as part of post-deploy setup.
