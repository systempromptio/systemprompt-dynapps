# Install on Google Kubernetes Engine (Helm)

The chart is `deploy/helm/systemprompt-dynapps/`. It is the Kubernetes form of
the self-host runbook (`deploy/self-host/README.md`): same image, same
profile keys, same `secrets.json`, same one-shot `migrate` step. **Status:**
the chart is new. It passes `helm lint` and `helm template`, and its rendered
profile validates against `docs/profile.schema.json`, but it has not yet been
installed on a real GKE project. Treat the first install as the test.

## Topology

```
                 GKE external ALB  (/readyz health check, 600 s backend timeout)
                 /v1/*                         everything else
                   |                                 |
        gateway Deployment (HPA, PDB)        admin Deployment (1 replica, Recreate)
        server.role: gateway                 server.role: admin
        /v1, bridge, OAuth, health           console, scheduler, MCP, agents
                   \                                 /
           each pod: cloud-sql-proxy native sidecar -> Cloud SQL PG16 (private IP, IAM auth)
           each pod: secrets.json from Secret Manager (CSI), files in GCS (Workload Identity)
```

Before each install or upgrade, a Helm hook Job runs the image with the
argument `migrate`. It runs `infra db migrate`, then `admin bootstrap`, then
exits. Pods never migrate on boot (`database.migrate_on_boot: false`) and
refuse to start against a schema that is behind.

## Prerequisites

| item | setting |
|---|---|
| Cluster | GKE 1.29 or newer, Autopilot or Standard. Workload Identity on. Dataplane V2 on, so NetworkPolicy is enforced. Secret Manager add-on on (`--enable-secret-manager`). |
| Namespace | `kubectl create ns dynapps` then `kubectl label ns dynapps pod-security.kubernetes.io/enforce=restricted`. Every container in the chart meets `restricted`. |
| Cloud SQL | PostgreSQL 16, HA (regional), private IP only, flag `cloudsql.iam_authentication=on`. Enable `vector`, `uuid-ossp` and `pgcrypto` in the `systemprompt` database (`deploy/self-host/provision.sql`). |
| Cloud SQL IAM user | `gcloud sql users create dynapps-gw@<project>.iam --instance=<instance> --type=cloud_iam_service_account`, then grant it ownership of the `systemprompt` database objects (see `provision.sql`). |
| GCS bucket | Uniform bucket-level access, not public. The service account below gets `roles/storage.objectAdmin` on it. |
| Secret Manager | One secret, e.g. `dynapps-secrets-json`, whose value is the whole `secrets.json` document. Every key is in `deploy/self-host/SECRETS.md`. Render it once with `deploy/self-host/render-profile.sh` and upload the generated `secrets.json`. |
| Artifact Registry | Mirror the signed image: verify it with cosign, then `crane copy ghcr.io/systempromptio/systemprompt-dynapps:<tag> <region>-docker.pkg.dev/<project>/dynapps/systemprompt-dynapps:<tag>`. Grant the node service account `roles/artifactregistry.reader`. |
| Google service account | `dynapps-gw@<project>.iam.gserviceaccount.com` with `roles/cloudsql.client`, `roles/cloudsql.instanceUser`, `roles/secretmanager.secretAccessor` on the secret, `roles/storage.objectAdmin` on the bucket, and `roles/aiplatform.user` when Vertex is used through Workload Identity. |
| Workload Identity | Bind **two** KSAs: the app KSA and the migrate hook's KSA (below). |
| Static IP, certificate | A global static IP (`gcloud compute addresses create dynapps-gw-ip --global`) and DNS for the host. The chart creates a ManagedCertificate by default. |

Workload Identity bindings, with release name `dynapps` in namespace `dynapps`:

```bash
for ksa in dynapps-systemprompt-dynapps dynapps-systemprompt-dynapps-migrate; do
  gcloud iam service-accounts add-iam-policy-binding dynapps-gw@<project>.iam.gserviceaccount.com \
    --role roles/iam.workloadIdentityUser \
    --member "serviceAccount:<project>.svc.id.goog[dynapps/$ksa]"
done
```

`database_url` in `secrets.json` points at the sidecar. With IAM database
authentication there is no password, and the proxy carries TLS:

```
postgres://dynapps-gw%40<project>.iam@127.0.0.1:5432/systemprompt?sslmode=disable
```

Leave `database_write_url` unset. Replica reads stay off by policy because
replica lag breaks read-after-write for sessions.

## Values to fill in

Start from `deploy/helm/systemprompt-dynapps/values-gke-example.yaml`. The
install fails with a named error while any of these is empty:

| value | example |
|---|---|
| `image.repository`, `image.tag`, `image.digest` | the Artifact Registry mirror, pinned by digest |
| `profile.externalUrl` | `https://ai.example.com` |
| `profile.adminEmail` | the first administrator, who signs in through AD FS |
| `profile.storage.bucket` | `<project>-dynapps-files` |
| `secrets.csi.secretVersion` | `projects/<project>/secrets/dynapps-secrets-json/versions/1` |
| `cloudSqlProxy.instanceConnectionName` | `<project>:<region>:<instance>` |
| `serviceAccount.gcpServiceAccount` | `dynapps-gw@<project>.iam.gserviceaccount.com` |
| `routing.host` | `ai.example.com` |
| `ingress.staticIpName` | `dynapps-gw-ip` |
| `networkPolicy.cloudSqlCidrs` | the private services access range of the Cloud SQL VPC peering |

## Install and upgrade sequence

```bash
helm upgrade --install dynapps deploy/helm/systemprompt-dynapps \
  -n dynapps -f values-gke.yaml --wait --timeout 20m
```

1. **Migrate.** The pre-install or pre-upgrade hook creates its own KSA,
   profile ConfigMap and SecretProviderClass, then runs the `migrate` Job. A
   failed Job stops the release, and the running pods are untouched. Read it
   with `kubectl -n dynapps logs job/dynapps-systemprompt-dynapps-migrate -c migrate`.
2. **Admin.** The single admin pod is replaced with `Recreate`. The console
   and scheduler are down for one pod start, typically one to two minutes.
   Inference is not affected, because `/v1` is served by the gateway pods.
3. **Gateway.** Rolling update with `maxUnavailable: 0`. Each new pod must
   pass `/readyz` before an old one is removed, and the PDB keeps two up
   during node maintenance.

Helm applies all three in one `upgrade`. The Job runs first, and the two
Deployments roll together after it.

## Rollback

`helm rollback dynapps <revision>` is safe only while the schema has not moved
forward. The migrate hook does not run on rollback, and a pod refuses to boot
on a schema newer than it expects. If a release added migrations, restore the
Cloud SQL backup taken before the upgrade, then roll back. Take an on-demand
backup before every upgrade:

```bash
gcloud sql backups create --instance=<instance> --description="pre-<tag>"
```

## How the role split and autoscaling behave

- **Gateway pods** mount only the AI gateway, bridge, OAuth, well-known and
  health routes. They run no scheduler, MCP or agent workers, so they scale
  freely. The HPA scales on CPU at 65 percent, from 3 to 12 pods. It can
  double the pod count each minute and removes at most one pod every two
  minutes.
- **Authorization runs in each pod.** The profile sets
  `governance.authz.hook.mode: extension`, so every gateway pod decides
  entitlement in-process; no request hops to the admin pod's
  `/api/public/govern/authz`.
- **The admin pod** must be exactly one. Two would run the scheduler twice,
  so the chart fixes `replicas: 1` and gives it no HPA and no PDB.
- **Load shedding.** Past `server.max_in_flight` a pod answers 503 with
  `Retry-After`, and `/readyz` answers `503 {"status":"saturated"}`. The
  load balancer health check and the readiness probe both use `/readyz`, so a
  saturated pod leaves rotation until it drains.
- **Custom metric.** `gateway.autoscaling.customMetric` adds a per-pod metric
  to the HPA. It needs a metrics adapter, and the metric name must exist on
  `/metrics` first.
- **Metrics.** `server.metrics_port` (9090) serves `/metrics` on its own
  listener. `metrics.podMonitoring.enabled` creates Managed Service for
  Prometheus `PodMonitoring` objects. `metrics.podMonitor.enabled` does the
  same for prometheus-operator.

## What is still node-local

Each pod has `emptyDir` volumes for the paths the server writes. They are
lost when the pod is replaced:

| path | content | consequence of loss |
|---|---|---|
| `/app/storage/data` | gateway accounting journal, scratch | receipts not yet flushed to Postgres are lost with the pod |
| `/app/storage/exports` | generated exports | a user re-runs the export |
| `/app/storage/files` | image-shipped templates, refreshed at boot | none, the entrypoint restores them |
| `/app/web` | rendered public site | none, `publish_pipeline` renders it at boot |
| `/app/services-cache`, `/app/logs`, `/app/.systemprompt/sessions` | caches, logs, CLI sessions | none |
| `/tmp` | MCP child scratch, memory-backed | none |

Uploaded and generated files go to the GCS bucket, which every pod shares.

## Secrets and rotation

The Secret Store CSI driver mounts the Secret Manager version as a file. An
init container copies it, with the profile ConfigMap, into an in-memory
profile directory. The server reads `secrets.json` once, at boot. To rotate,
add a new secret version, set `secrets.csi.secretVersion` to it and run
`helm upgrade`. The Deployments carry a checksum of the secret settings, so
they roll. Rotating `signing_key_pem` signs every user out, and rotating
`oauth_at_rest_pepper` orphans stored OAuth tokens (`deploy/self-host/SECRETS.md`).

Other modes: `secrets.mode: existingSecret` reads a Kubernetes Secret you
manage. `secrets.mode: inline` is for development only.

## Network policy

The chart's NetworkPolicy denies everything for the release's pods, then
allows the following:

- **Ingress** on 8080 from the Google Front End and health-check ranges.
- **Ingress** on the metrics port from the `gmp-system` namespace.
- **Egress** to kube-dns and to the GKE metadata server.
- **Egress** to the Cloud SQL private range on 3307.
- **Egress** on 443 to the Private Google Access ranges.

Two cases need more. `*.googleapis.com` must resolve to those ranges through a
private DNS zone, or `networkPolicy.allowPublicHttps` must be on. A provider
outside Google, such as the `api.anthropic.com` failover, needs
`allowPublicHttps` or an `extraEgress` rule.

## Requirement coverage

| IT registry item | requirement | how the chart meets it |
|---|---|---|
| NFR-2.2 / UC-19 | gateway and admin role split | two Deployments, `server.role: gateway` scaled by HPA, `server.role: admin` fixed at one replica with `Recreate` |
| NFR-4.2 / UC-20 | Secret Manager mounted as files | `SecretProviderClass` for the GKE Secret Manager CSI add-on; `secrets.json` as a file; rotation by restart |
| NFR-6.1 / UC-43 | Helm chart with probes, limits, HPA, PDB | startup, liveness (`/livez`) and readiness (`/readyz`) probes; requests and limits on every container; HPA on CPU plus an optional custom metric; PDB `minAvailable: 2` |
| NFR-2.1 / UC-44 | GCS storage | `storage.backend: gcs` with `credentials: workload_identity` |
| NFR-4.3 / UC-46 | Cloud SQL Auth Proxy with IAM auth | native sidecar with `--auto-iam-authn --private-ip`; the app connects to `127.0.0.1:5432` |
| NFR-4.4 / UC-47 | Pod Security `restricted` | uid 1000, `runAsNonRoot`, `readOnlyRootFilesystem`, drop `ALL`, `RuntimeDefault` seccomp, no privilege escalation, `emptyDir` for every writable path, memory-backed `/tmp` |

## Verify

```bash
kubectl -n dynapps get pods -l app.kubernetes.io/instance=dynapps
kubectl -n dynapps port-forward svc/dynapps-systemprompt-dynapps-gateway 8080:8080 &
deploy/self-host/verify-node.sh --url http://localhost:8080 --role gateway --version <tag>
```

`verify-node.sh` also accepts `--role admin`. Without `--container` it runs
only its HTTP checks, which is what works against a port-forward.
