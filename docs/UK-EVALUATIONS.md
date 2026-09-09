# UK requirements workflow and evaluation implementation

The UK workflow uses three skills: `requirements_context`, `requirements_specify`, and `requirements_publish`. It requires a signed browser decision on the context and a second decision on the specification and Confluence destination. Changes to the approved inputs invalidate the relevant approval. The server validates structure synchronously and verifies the published content through Confluence readback.

The workflow freezes explicitly selected, caller-readable Knowledge Bank sources. Selected Confluence pages are read through the connected Atlassian MCP server. Evidence records preserve both the indexed hash and the live content hash. Missing access, missing version metadata, and oversized evidence stop the operation; the server does not truncate evidence or fall back to the demonstration corpus.

The OpenSpec engine and its bundled upstream checkout have been removed. Existing historical workflow records remain available as read-only downloads. New workflows use `requirements-v2.1`; historical approvals do not authorize new content.

## Automated evaluation

Automated evaluation is under development. Creating an experiment queues records;
it does not start a native client or establish a measured quality result.

See [evaluation status and remaining work](evaluation-completion.md) for the
implemented components, outstanding work, command availability and acceptance
requirements. Do not use component tests or Docker version probes as evidence
that the full evaluation workflow is ready.
