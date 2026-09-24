# Astound Digital documentation

Installation, configuration, architecture, workflow, and engineering references for
this repository. This is the single repository-documentation tree. Content compiled
into the product documentation site remains under `services/content/documentation/`.

## Install and configure

- [Docker installation](install/docker.md)
- [Oracle Cloud deployment](install/oracle-cloud.md)
- [Required secrets](install/required-secrets.md) and [Vertex configuration](install/vertex.md)
- [Profile schema](profile.schema.json)

## Use the system

- [Gateway routing](gateway-routes.md)
- [Personal MCP accounts](CONFIGURED-CONNECTORS.md)
- [Centralized MCP connections](integrations/centralized-mcp.md)
- [India development marketplace](india-marketplace.md)
- [UK requirements workflow](UK-EVALUATIONS.md)

## Maintain the repository

- [Branching](BRANCHING.md)
- [Release and rollback procedure](RELEASING.md)

Do not commit customer diagnostics, credentials, or raw deployment transcripts.
Capability descriptions must distinguish implemented behavior from planned work and
must not imply live acceptance from component tests.
