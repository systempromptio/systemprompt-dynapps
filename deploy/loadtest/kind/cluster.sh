#!/usr/bin/env bash
# Lifecycle of the kind load-test cluster (README.md in this directory).
#   cluster.sh up            create the cluster, cap node CPUs, load images, apply manifests
#   cluster.sh secrets       render .profile/secrets.json (deploy/self-host/render-profile.sh)
#   cluster.sh deploy [--set k=v ...] [--policies default|enforce]
#                            services ConfigMap + helm upgrade --install, wait for both roles
#   cluster.sh image-local   build systemprompt-astound:loadtest-local (Dockerfile.local) and load it
#   cluster.sh lint          helm lint + template with values-kind.yaml, profile schema check
#   cluster.sh down          delete the cluster
# Portable bash (macOS + Linux); needs kind, kubectl, helm, docker, jq, openssl.
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
CLUSTER=astound-loadtest
NS=loadtest
CHART="$ROOT/deploy/helm/systemprompt-astound"
VALUES="$CHART/values-kind.yaml"
# Optional extra values file under the chart directory (LOADTEST_VALUES).
EXTRA_VALUES=()
[ -z "${LOADTEST_VALUES:-}" ] || EXTRA_VALUES=(-f "$CHART/$LOADTEST_VALUES")
IMAGE_REPO=ghcr.io/astound-digital/systemprompt-astound
IMAGE_TAG="${LOADTEST_IMAGE_TAG:-0.65.0}"
MOCK_IMAGE=astound-loadtest/mock-inference:local
CORE="${CORE_DIR:-$ROOT/../systemprompt-core}"
SECRETS="$HERE/.profile/secrets.json"
KCTX="kind-$CLUSTER"
k() { kubectl --context "$KCTX" "$@"; }

# CPU budget per node container (host 24 CPU; leave the host usable).
cap_cpus() {
    docker update --cpus 2 "$CLUSTER-control-plane" >/dev/null
    docker update --cpus 5 "$CLUSTER-worker" >/dev/null
    docker update --cpus 5 "$CLUSTER-worker2" >/dev/null
    docker update --cpus 4 "$CLUSTER-worker3" >/dev/null
}

load_image() {
    local img="$1"
    docker image inspect "$img" >/dev/null 2>&1 || docker pull -q "$img" >/dev/null
    kind load docker-image --name "$CLUSTER" "$img" >/dev/null
}

cmd_up() {
    if ! kind get clusters 2>/dev/null | grep -qx "$CLUSTER"; then
        kind create cluster --config "$HERE/kind-config.yaml" --wait 120s
    fi
    cap_cpus
    if ! docker image inspect "$MOCK_IMAGE" >/dev/null 2>&1; then
        docker build -f "$HERE/Dockerfile.mock" -t "$MOCK_IMAGE" "$CORE/crates/tests/mock-inference"
    fi
    for img in "$MOCK_IMAGE" "$IMAGE_REPO:$IMAGE_TAG" pgvector/pgvector:pg16 \
        quay.io/prometheuscommunity/postgres-exporter:v0.15.0 prom/prometheus:v2.54.1 \
        registry.k8s.io/metrics-server/metrics-server:v0.7.2 alpine/socat:1.8.0.1; do
        echo "loading $img"
        load_image "$img"
    done
    k apply -f "$HERE/manifests/"
    k -n "$NS" rollout status deploy/postgres --timeout 180s
    k -n "$NS" rollout status deploy/mock-primary --timeout 120s
    k -n "$NS" rollout status deploy/mock-eu --timeout 120s
    k -n "$NS" rollout status deploy/prometheus --timeout 120s
    echo "cluster up: gateway NodePort http://localhost:18080, Prometheus http://localhost:19091"
}

cmd_secrets() {
    mkdir -p "$HERE/.profile"
    # render-profile.sh (0.64.0) does not generate encryption_master_key, which
    # the server refuses to boot without; keep an existing one stable.
    local emk=""
    [ -f "$SECRETS" ] && emk="$(jq -r '.encryption_master_key // empty' "$SECRETS")"
    [ -n "$emk" ] || emk="$(openssl rand -hex 32)"
    "$ROOT/deploy/self-host/render-profile.sh" --out "$HERE/.profile" \
        --external-url https://gateway.loadtest.local \
        --admin-email loadtest-admin@astounddigital.com \
        --trusted-proxies 10.0.0.0/8 \
        --database-url 'postgres://systemprompt:loadtest@postgres.loadtest.svc.cluster.local:5432/systemprompt?sslmode=disable' \
        --anthropic-key sk-ant-loadtest-mock \
        --secret encryption_master_key="$emk" >/dev/null
    echo "rendered $SECRETS ($(jq -r 'keys | join(", ")' "$SECRETS"))"
}

services_configmap() {
    local policies="$HERE/services/gateway/policies.yaml"
    [ "${1:-default}" = enforce ] && policies="$HERE/services/gateway/policies-enforce.yaml"
    k -n "$NS" create configmap loadtest-services \
        --from-file=providers.yaml="$HERE/services/ai/providers.yaml" \
        --from-file=gateway.yaml="$HERE/services/ai/gateway.yaml" \
        --from-file=config.yaml="$HERE/services/ai/config.yaml" \
        --from-file=policies.yaml="$policies" \
        --dry-run=client -o yaml | k apply -f - >/dev/null
}

cmd_deploy() {
    local policies=default sets=()
    while [ $# -gt 0 ]; do
        case "$1" in
            --policies) policies="$2"; shift 2 ;;
            --set) sets+=(--set "$2"); shift 2 ;;
            *) sets+=("$1"); shift ;;
        esac
    done
    [ -f "$SECRETS" ] || cmd_secrets
    services_configmap "$policies"
    helm --kube-context "$KCTX" upgrade --install astound "$CHART" -n "$NS" \
        -f "$VALUES" "${EXTRA_VALUES[@]+"${EXTRA_VALUES[@]}"}" --set image.tag="$IMAGE_TAG" \
        --set-file secrets.inline.json="$SECRETS" \
        "${sets[@]+"${sets[@]}"}" --wait --timeout 15m
    k -n "$NS" rollout status deploy/astound-gateway --timeout 300s
    k -n "$NS" rollout status deploy/astound-admin --timeout 300s
}

cmd_image_local() {
    mkdir -p "$HERE/.bin"
    cp "$ROOT"/target/release/systemprompt "$HERE/.bin/"
    for b in "$ROOT"/target/release/systemprompt-mcp-*; do
        [ -x "$b" ] && [ -f "$b" ] && cp "$b" "$HERE/.bin/"
    done
    docker build -f "$HERE/Dockerfile.local" -t "$IMAGE_REPO:loadtest-local" "$ROOT"
    kind load docker-image --name "$CLUSTER" "$IMAGE_REPO:loadtest-local"
    echo "deploy with: LOADTEST_IMAGE_TAG=loadtest-local just loadtest-kind-deploy"
}

cmd_lint() {
    local tmp
    tmp="$(mktemp -d)"
    printf '{"database_url":"postgres://x@y/z"}' > "$tmp/s.json"
    helm lint "$CHART" -f "$VALUES" --set-file secrets.inline.json="$tmp/s.json"
    helm template astound "$CHART" -n "$NS" -f "$VALUES" \
        --set-file secrets.inline.json="$tmp/s.json" > "$tmp/out.yaml"
    echo "helm template: $(grep -c '^kind:' "$tmp/out.yaml") objects"
    # The gateway profile, extracted from the rendered ConfigMap, validated by the image.
    awk '/name: astound-profile-gateway/{f=1} f&&/profile.yaml: \|/{p=1;next} p&&/^---/{exit} p{sub(/^    /,"");print}' \
        "$tmp/out.yaml" > "$tmp/profile.yaml"
    [ -f "$SECRETS" ] && cp "$SECRETS" "$tmp/secrets.json" || cp "$tmp/s.json" "$tmp/secrets.json"
    docker run --rm --entrypoint /app/bin/systemprompt --user "$(id -u):$(id -g)" \
        -v "$tmp:/p:ro" -e SYSTEMPROMPT_PROFILE=/p/profile.yaml "$IMAGE_REPO:$IMAGE_TAG" \
        admin config validate --strict /p/profile.yaml
    rm -rf "$tmp"
}

cmd_down() { kind delete cluster --name "$CLUSTER"; }

case "${1:-}" in
    up) cmd_up ;;
    secrets) cmd_secrets ;;
    deploy) shift; cmd_deploy "$@" ;;
    image-local) cmd_image_local ;;
    lint) cmd_lint ;;
    down) cmd_down ;;
    *) sed -n '2,11p' "$0"; exit 2 ;;
esac
