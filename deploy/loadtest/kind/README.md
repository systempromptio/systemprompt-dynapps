# Local Kubernetes load-test rig (kind)

This rig runs the real Helm chart (`deploy/helm/systemprompt-dynapps`) on a local
kind cluster. It uses in-cluster stand-ins for the GKE attachments, so it makes
no paid model calls. It exists to find gateway blockers before Cloudfresh runs
the same test on GKE.

| Piece | Stand-in for | Where |
|---|---|---|
| `postgres` Deployment, pg16, `pg_stat_statements`, lock-wait logging, exporter sidecar | Cloud SQL | `manifests/20-postgres.yaml` |
| `mock-primary`, `mock-eu` (core `crates/tests/mock-inference`) | Vertex Claude global + europe-west1 | `manifests/30-mock-inference.yaml`, `Dockerfile.mock` |
| Prometheus with pod discovery on `prometheus.io/*` annotations | Managed Prometheus | `manifests/40-prometheus.yaml` |
| inline Secret rendered by `deploy/self-host/render-profile.sh` | Secret Manager CSI | `cluster.sh secrets` |
| `services/` ConfigMap mounted through `pod.extraVolumes` | the image's provider catalog, routes and policies | `services/` |

The cluster is one control plane plus three workers. Two workers run the app
and the generator. The third runs Postgres and Prometheus alone. Node CPUs are
capped at 2 + 5 + 5 + 4, which leaves the 24-CPU host usable.

## Bring-up

```bash
just loadtest-kind-up          # cluster, CPU caps, images, manifests
just loadtest-kind-secrets     # .profile/secrets.json (gitignored)
just loadtest-kind-deploy      # services ConfigMap + helm upgrade --install (migrate hook first)
just loadtest-kind-seed 100    # 100 users, PAT + session each -> .tokens/u100.csv + Secret
just loadtest-kind-seed 20     # the quota-enforce set
just loadtest-kind-seed 100 hot commerce   # every user on one project
```

Check it by hand:

```bash
curl -s localhost:18080/v1/models          # only the mock ids
IFS=, read -r T S < deploy/loadtest/kind/.tokens/u100.csv
curl -s localhost:18080/v1/messages -H "Authorization: Bearer $T" -H "x-session-id: $S" \
  -H 'content-type: application/json' -H 'anthropic-version: 2023-06-01' \
  -d '{"model":"claude-sonnet-5-5","max_tokens":32,"messages":[{"role":"user","content":"ping"}]}'
```

## Running

```bash
just loadtest-kind-run smoke --concurrency 20 --duration 20s
just loadtest-kind-matrix                 # every entry in scenarios.json
just loadtest-kind-matrix 's1|s5'         # a subset
```

Each run writes `reports/loadtest/kind/<ts>/<name>/` (raw data) plus
`<name>.json` and `<name>.md` (summary). Ramps record their ceiling in
`ceilings.json`. A ramp stops at the first step with any non-200 status, or
with p99 `gateway_overhead_seconds` above 200 ms.

The generator is an Indexed Job of `hey` pods built from the app image, which
ships `/app/bin/hey`. It calls the `dynapps-gateway` ClusterIP Service, and
every active credential gets its own `hey` process. The core harness
(`crates/tests/loadtest`, `--model`, `--token-file`) also works from the host
through NodePort 18080.

Each run also proves that nothing left the cluster: the mocks' `/stats`
counters must match `gateway_deployment_selected_total`.

## Phase 3: a locally built image

```bash
just build --release
just loadtest-kind-image-local
LOADTEST_IMAGE_TAG=loadtest-local just loadtest-kind-deploy
```

## Things the rig had to work around

These were all found on first boot, and every one of them also applies to GKE.

- **The `/app/storage` emptyDir hid the image's bundled assets.** The chart now
  creates the directory tree in the init container, and the entrypoint
  refills it from `/app/share`.
- **The authz hook runs in-process** (`governance.authz.hook.mode: extension`)
  on every role, so no inference request crosses the admin pod.
- **`render-profile.sh` does not generate `encryption_master_key`.** The rig
  passes the key in with `--secret`.
- **Plain-http provider endpoints need `SYSTEMPROMPT_TRUSTED_HTTP_HOSTS`.** The
  chart sets it from `pod.trustedHttpHosts`.
- **Sessions are minted at `/api/public/gateway/sessions`.** 0.64.0 has no
  `/v1/sessions` route.
- **`admin users api-key issue --scope project=` is refused from the CLI.** The
  seed script sets `user_scope_defaults.primary_project_id` instead.

`just loadtest-kind-down` deletes everything.
