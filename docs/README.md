# Dynapps documentation

Installation, configuration, architecture, workflow, and engineering references for
this repository. This is the single repository-documentation tree. Content compiled
into the product documentation site remains under `services/content/documentation/`.

## Install and configure

- [Docker installation](install/docker.md)
- [Oracle Cloud deployment](install/oracle-cloud.md)
- [GKE deployment with the Helm chart](install/gke.md)
- [Required secrets](install/required-secrets.md) and [Vertex configuration](install/vertex.md)
- [Profile schema](profile.schema.json)

## Use the system

- [Gateway routing](gateway-routes.md)
- [Personal MCP accounts](CONFIGURED-CONNECTORS.md)
- [Centralized MCP connections](integrations/centralized-mcp.md)
- [UK requirements workflow](UK-EVALUATIONS.md)

## Maintain the repository

- [Branching](BRANCHING.md)
- [Release and rollback procedure](RELEASING.md)
- [Security policy and telemetry statement](../SECURITY.md)

Do not commit customer diagnostics, credentials, or raw deployment transcripts.
Capability descriptions must distinguish implemented behavior from planned work and
must not imply live acceptance from component tests.
