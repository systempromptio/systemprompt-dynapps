# Astound Digital documentation

Public installation, configuration and workflow references for this repository.

## Install and configure

- [Docker installation](install/docker.md)
- [Oracle Cloud deployment](install/oracle-cloud.md)
- [Required secrets](install/required-secrets.md) and [Vertex configuration](install/vertex.md)
- [Knowledge database](install/knowledge-db.md) and [model providers](install/knowledge-model-provider.md)
- [Google Drive ingestion](install/google-drive-knowledge.md)
- [Profile schema](profile.schema.json)

## Use the system

- [Gateway routing](gateway-routes.md)
- [Personal MCP accounts](CONFIGURED-CONNECTORS.md)
- [Centralized MCP connections](integrations/centralized-mcp.md)
- [India development marketplace](india-marketplace.md)
- [UK requirements workflow](UK-EVALUATIONS.md)
- [Evaluation status and remaining work](evaluation-completion.md) — incomplete; component checks are not end-to-end acceptance.

## Maintain the repository

- [Branching](BRANCHING.md)
- [Release and rollback procedure](RELEASING.md)

Keep customer diagnostics, access audits, deployment transcripts and internal
planning out of this public folder. Store local working records in the ignored
`docs-internal/` directory; do not link public documentation to those local files.
Public capability descriptions must distinguish implemented behavior from planned
work and must not imply live acceptance from component tests.
