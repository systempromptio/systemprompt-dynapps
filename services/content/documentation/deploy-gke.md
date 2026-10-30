---
title: "Deploy on Google Kubernetes Engine"
description: "The Helm chart for GKE: scalable gateway and api tiers, a lease-elected worker, a migration Job before every rollout, Cloud SQL through the Auth Proxy with IAM auth, Secret Manager as a file, GCS for files and Pod Security restricted."
author: "Dynapps"
slug: "deploy-gke"
keywords: "gke, google kubernetes engine, helm, cloud sql auth proxy, secret manager, workload identity, gcs, hpa, pod disruption budget, pod security restricted, deploy"
kind: "guide"
public: true
tags: ["documentation", "deployment", "configuration"]
published_at: "2026-10-08"
after_reading_this:
  - "Explain the GKE topology: gateway pods, api pods, worker pods, and the migrate Job"
  - "List the Google Cloud resources and identities to create before installing"
  - "Know what each chart piece does for probes, autoscaling, secrets, storage and network"
  - "Know what stays on each pod and how to roll back safely"
related_playbooks:
  - title: "Deploy: One Node and PostgreSQL"
    url: "/documentation/deploy"
  - title: "Proposed: Target Deployment Architecture"
    url: "/documentation/deploy-target-architecture"
  - title: "Deploy on Oracle Cloud"
    url: "/documentation/deploy-oracle-cloud"
  - title: "Deploy: Validate Every Stage"
    url: "/documentation/deploy-troubleshooting"
  - title: "Observability Export"
    url: "/documentation/observability-export"
---

# Deploy on Google Kubernetes Engine

The repository ships a Helm chart, `deploy/helm/systemprompt-dynapps/`, that
runs the same image, profile and `secrets.json` as the
[one-node deployment](/documentation/deploy) on GKE. **The chart is new.** It
lints and renders cleanly and its profile validates against the profile
schema, but it has not yet been installed on a real GKE project. The
operator runbook, with every command, is `docs/install/gke.md` in the
repository.

## Topology

| piece | what it runs | how many |
|---|---|---|
| Gateway Deployment | `server.role: gateway`: `/v1` inference, the bridge, OAuth, well-known and health routes. No scheduler, MCP or agent workers. | 3 to 12, scaled by a HorizontalPodAutoscaler on CPU, with a PodDisruptionBudget |
| API Deployment | `server.role: api`: the console, agent, MCP, content, stream, messaging and static routes. No scheduler and no MCP or agent processes. | `api.replicas` (2 by default), with a PodDisruptionBudget |
| Worker Deployment | `server.role: worker`: the scheduler and the MCP server and agent processes, reached by the api pods at the pod IP each worker advertises | 1 by default; a second replica is a standby that takes the lease when the holder goes |
| Migrate Job | the image's one-shot `migrate` step: schema migrations, then the bootstrap admin | once, as a Helm pre-install and pre-upgrade hook |

The load balancer sends `/v1` to the gateway pods and everything else to the
api pods; no balancer reaches the worker pods. Pods never migrate on boot and refuse a schema that is behind, so
the Job always runs before any new pod starts.

The api pods never render the public site or compose the services cache; the
worker does, into a ReadWriteMany volume the api pods mount read-only. Set
`pod.sharedWeb.existingClaim` to an existing ReadWriteMany claim (Filestore on
GKE). It is required: the chart refuses to render while `api.replicas` is above
zero or `api.autoscaling.enabled` is true and the claim is empty.

## What each pod carries

- **Database.** A Cloud SQL Auth Proxy runs as a native sidecar with IAM
  database authentication over private IP. The server connects to
  `127.0.0.1:5432`; the proxy handles TLS and identity. Cloud SQL is
  PostgreSQL 16 in HA mode, and replica reads stay off.
- **Secrets.** Google Secret Manager holds the whole `secrets.json`. The GKE
  Secret Manager CSI add-on mounts it as a file, and it is read once at boot.
  Rotation is a new secret version and a restart.
- **Files.** Uploaded and generated files live in a Cloud Storage bucket every
  pod shares, reached through Workload Identity.
- **Health.** Startup and liveness probes use `/livez`. Readiness and the
  load balancer health check use `/readyz`, which answers `503` with
  `{"status":"saturated"}` when the pod is over `server.max_in_flight`, so a
  saturated pod leaves rotation instead of queueing.
- **Timeouts.** The backend timeout is 600 seconds, long enough for streamed
  completions.
- **Metrics.** A separate `/metrics` listener on `server.metrics_port`, with
  optional Managed Service for Prometheus scraping.
- **Security.** Every container meets Pod Security `restricted`: uid 1000,
  read-only root filesystem, all capabilities dropped, `RuntimeDefault`
  seccomp, no privilege escalation. Each writable path is an `emptyDir`, and
  `/tmp` is memory-backed.
- **Network.** A default-deny NetworkPolicy allows DNS, the metadata server,
  the Cloud SQL private range and the Private Google Access ranges. Providers
  outside Google need an explicit egress rule.

## Before installing

Create these in the Google Cloud project first:

1. A GKE cluster, version 1.29 or newer, with Workload Identity, Dataplane V2
   and the Secret Manager add-on.
2. A Cloud SQL for PostgreSQL 16 HA instance on private IP, with IAM
   authentication and an IAM database user for the service account.
3. A Cloud Storage bucket for files.
4. A Secret Manager secret holding `secrets.json`.
5. An Artifact Registry repository holding the mirrored, verified image.
6. One Google service account with Cloud SQL client and instance user, Secret
   Manager accessor, and Storage object admin roles. Bind it to both of the
   chart's Kubernetes service accounts, the app's and the migrate Job's.

## What stays on the pod

The gateway accounting journal, exports, the rendered public site, caches and
logs live in per-pod `emptyDir` volumes. A replaced pod loses any journal
entries not yet written to Postgres. Everything else is rebuilt at boot.

## Rolling back

A Helm rollback is safe only while the schema has not moved forward. The
migrate hook does not run on rollback, and a pod refuses a schema newer than
it knows. Take a Cloud SQL backup before every upgrade, and restore it before
rolling back across a release that added migrations.
