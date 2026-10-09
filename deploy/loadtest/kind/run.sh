#!/usr/bin/env bash
# One load-test run against the kind rig; writes reports/loadtest/kind/<ts>/<name>.*
#   run.sh <name> [--concurrency 200] [--set u100] [--body buffered|stream|long]
#          [--duration 120s] [--pods 4] [--mock-mode ok|5xx] [--mock-latency 300:900]
#          [--chain on|off] [--policies default|enforce] [--helm k=v[,k=v]]
#          [--kill-pod-after SECONDS] [--out DIR]
# Generator: an Indexed Job of `hey` pods (the image ships /app/bin/hey) calling
# the gateway ClusterIP Service; each active credential gets its own hey
# process. Knobs are reverted on exit (mock mode, chain, policies).
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
NAME="${1:?usage: run.sh <name> [options]}"; shift
C=200 SET=u100 BODY=buffered DUR=120s PODS=4 MOCK_MODE=ok MOCK_LAT=300:900
CHAIN=on POLICIES=default HELM="" KILL_AFTER="" OUT=""
while [ $# -gt 0 ]; do
    case "$1" in
        --concurrency) C="$2" ;; --set) SET="$2" ;; --body) BODY="$2" ;;
        --duration) DUR="$2" ;; --pods) PODS="$2" ;; --mock-mode) MOCK_MODE="$2" ;;
        --mock-latency) MOCK_LAT="$2" ;; --chain) CHAIN="$2" ;; --policies) POLICIES="$2" ;;
        --helm) HELM="$2" ;; --kill-pod-after) KILL_AFTER="$2" ;; --out) OUT="$2" ;;
        *) echo "unknown option $1" >&2; exit 2 ;;
    esac
    shift 2
done
OUT="${OUT:-$ROOT/reports/loadtest/kind/$(date -u +%Y%m%dT%H%M%SZ)}"
RUN="$OUT/$NAME"
[ ! -e "$RUN" ] || RUN="$OUT/$NAME-$(date -u +%H%M%S)"
NAME="$(basename "$RUN")"
mkdir -p "$RUN"
K="kubectl --context kind-astound-loadtest -n loadtest"
psqlc() { $K exec -i deploy/postgres -c postgres -- psql -U postgres -d systemprompt -At "$@"; }
mock_stats() {
    for m in mock-primary mock-eu; do
        printf '%s=%s\n' "$m" "$($K exec deploy/astound-admin -c server -- curl -s "http://$m:9100/stats" | sed -n 's/.*"requests":\([0-9]*\).*/\1/p')"
    done
}
secs="${DUR%s}"

set_mock() {
    for m in mock-primary mock-eu; do
        local mode=ok
        [ "$m" = mock-primary ] && mode="$1"
        $K patch deploy "$m" --type=json -p "[{\"op\":\"replace\",\"path\":\"/spec/template/spec/containers/0/args\",\"value\":[\"--port\",\"9100\",\"--latency-ms\",\"$MOCK_LAT\",\"--mode\",\"$mode\"]}]" >/dev/null
        $K rollout status deploy "$m" --timeout 120s >/dev/null
    done
}
set_chain() { psqlc -c "SELECT set_audit_guard((SELECT coalesce((SELECT mode FROM audit_guard_settings LIMIT 1),'warn')), $1);" >/dev/null 2>&1 \
    || psqlc -c "UPDATE audit_guard_settings SET chain_enabled = $1;" >/dev/null; }

cleanup() {
    for p in $(jobs -p); do kill "$p" 2>/dev/null || true; done
    [ "$MOCK_MODE" = ok ] && [ "$MOCK_LAT" = 300:900 ] || set_mock ok
    [ "$CHAIN" = on ] || set_chain true
    [ "$POLICIES" = default ] && [ -z "$HELM" ] || "$HERE/cluster.sh" deploy >/dev/null 2>&1 || true
}
trap cleanup EXIT

# --- knobs -------------------------------------------------------------------
if [ -n "$HELM" ] || [ "$POLICIES" != default ]; then
    args=(--policies "$POLICIES")
    IFS=',' read -ra kv <<< "$HELM"
    for s in "${kv[@]+"${kv[@]}"}"; do args+=(--set "$s"); done
    "$HERE/cluster.sh" deploy "${args[@]}" >"$RUN/helm.log" 2>&1
    # Policies are loaded at admin boot; restart so the table matches the file.
    $K rollout restart deploy/astound-admin deploy/astound-gateway >/dev/null
    $K rollout status deploy/astound-admin --timeout 300s >/dev/null
    $K rollout status deploy/astound-gateway --timeout 300s >/dev/null
fi
{ [ "$MOCK_MODE" = ok ] && [ "$MOCK_LAT" = 300:900 ]; } || set_mock "$MOCK_MODE"
[ "$CHAIN" = on ] || set_chain false

# --- bodies --------------------------------------------------------------------
B="$RUN/body.json"
case "$BODY" in
    buffered) echo '{"model":"claude-sonnet-5","max_tokens":32,"messages":[{"role":"user","content":"ping"}]}' > "$B" ;;
    stream) echo '{"model":"claude-sonnet-5","max_tokens":32,"stream":true,"messages":[{"role":"user","content":"ping"}]}' > "$B" ;;
    long) python3 -c 'import json
m=[{"role":("user" if i%2==0 else "assistant"),"content":"turn %d: lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor." % i} for i in range(39)]
m.append({"role":"user","content":"and now?"})
print(json.dumps({"model":"claude-sonnet-5","max_tokens":32,"messages":m}))' > "$B" ;;
    *) echo "unknown body $BODY" >&2; exit 2 ;;
esac
$K create configmap loadtest-body --from-file=body.json="$B" --dry-run=client -o yaml | $K apply -f - >/dev/null

# --- baseline marks ------------------------------------------------------------
# rules.yaml declares gateway_route/* `default: open`, but each admin boot's
# catalog sync (source services:/app/services/ai/gateway.yaml) rewrites it to
# closed, and core's rule-based check sees roles [] for a PAT caller, so every
# call 403s. Restore the declared default; this is a product bug, not a knob.
# Fixed from 0.65.0: LOADTEST_NO_ROUTE_RESTORE=1 skips the restore and records
# the route state instead, so a flip shows up as a failed run.
psqlc -F $'\t' -c "SELECT entity_id, default_included FROM access_control_entities WHERE entity_type = 'gateway_route';" > "$RUN/routes-before.tsv" </dev/null
[ -n "${LOADTEST_NO_ROUTE_RESTORE:-}" ] || psqlc -c "UPDATE access_control_entities SET default_included = true WHERE entity_type = 'gateway_route' AND NOT default_included;" >/dev/null </dev/null
# Fresh backends per run: 0.64.0 backends grow without bound (prepared
# statements are never released), so a run must not inherit the last one's.
psqlc -c "SELECT count(pg_terminate_backend(pid)) FROM pg_stat_activity WHERE usename = 'systemprompt';" >/dev/null </dev/null
sleep 35
# Warm the fresh pools (connect + first prepares) outside the measured window.
IFS=, read -r wtok wsid < <($K get secret "loadtest-tokens-$SET" -o jsonpath='{.data.tokens\.csv}' | base64 -d | head -1)
$K exec deploy/astound-admin -c server -- /app/bin/hey -z 20s -c 12 -m POST -T application/json \
    -H "Authorization: Bearer $wtok" -H "x-session-id: $wsid" -H "anthropic-version: 2023-06-01" \
    -d '{"model":"claude-sonnet-5","max_tokens":32,"messages":[{"role":"user","content":"warm"}]}' \
    http://astound-gateway:8080/v1/messages >/dev/null 2>&1 </dev/null || true
sleep 15
psqlc -c "SELECT pg_stat_statements_reset();" >/dev/null </dev/null
before="$(mock_stats | tr '\n' ' ')"
pg_restarts() { $K get pod -l app=postgres -o jsonpath='{.items[0].status.containerStatuses[?(@.name=="postgres")].restartCount}'; }
pg_r0="$(pg_restarts)"
start="$(date -u +%s)"

# --- samplers ------------------------------------------------------------------
( while :; do psqlc -F $'\t' -f /dev/stdin < "$HERE/sql/sample-locks.sql" >> "$RUN/waits.tsv" 2>/dev/null || true; sleep 1; done ) &
( while :; do t=$(date -u +%H:%M:%S); $K top pods --no-headers 2>/dev/null | sed "s/^/$t /" >> "$RUN/top.txt"; \
    $K get hpa --no-headers 2>/dev/null | sed "s/^/$t /" >> "$RUN/hpa.txt"; \
    $K get deploy astound-gateway --no-headers 2>/dev/null | sed "s/^/$t /" >> "$RUN/replicas.txt"; sleep 5; done ) &

# --- generator -----------------------------------------------------------------
$K delete job loadgen --ignore-not-found --wait >/dev/null
# No more generator pods than active credentials: an idle pod would finish
# at once and, with backoffLimit 0, a failing one ends the whole Job early.
ntok="$($K get secret "loadtest-tokens-$SET" -o jsonpath='{.data.tokens\.csv}' | base64 -d | grep -c ,)"
active=$(( ntok < C ? ntok : C ))
[ "$PODS" -le "$active" ] || PODS="$active"
IMG="$($K get deploy astound-gateway -o jsonpath='{.spec.template.spec.containers[0].image}')"
cat > "$RUN/job.yaml" <<YAML
apiVersion: batch/v1
kind: Job
metadata: {name: loadgen, namespace: loadtest}
spec:
  completionMode: Indexed
  completions: $PODS
  parallelism: $PODS
  backoffLimit: 0
  template:
    metadata: {labels: {app: loadgen}}
    spec:
      restartPolicy: Never
      nodeSelector: {loadtest/role: app}
      securityContext: {runAsNonRoot: true, runAsUser: 1000, seccompProfile: {type: RuntimeDefault}}
      containers:
        - name: hey
          image: $IMG
          imagePullPolicy: IfNotPresent
          command: ["/bin/sh", "-c"]
          args:
            - |
              set -u
              n=\$(grep -c , /t/tokens.csv); active=\$(( n < $C ? n : $C )); per=\$(( $C / active ))
              i=0
              while IFS=, read -r tok sid; do
                if [ \$i -lt \$active ] && [ \$(( i % $PODS )) -eq \$JOB_COMPLETION_INDEX ]; then
                  /app/bin/hey -z $DUR -c \$per -t 60 -m POST -T application/json \
                    -H "Authorization: Bearer \$tok" -H "x-session-id: \$sid" -H "anthropic-version: 2023-06-01" \
                    -D /b/body.json -o csv http://astound-gateway:8080/v1/messages > /tmp/h\$i.csv 2>/dev/null &
                fi
                i=\$((i + 1))
              done < /t/tokens.csv
              wait
              cat /tmp/h*.csv 2>/dev/null | grep -v '^response-time' || true
          securityContext: {allowPrivilegeEscalation: false, capabilities: {drop: ["ALL"]}}
          resources: {requests: {cpu: 500m, memory: 256Mi}, limits: {cpu: "2", memory: 1Gi}}
          volumeMounts:
            - {name: tokens, mountPath: /t}
            - {name: body, mountPath: /b}
      volumes:
        - {name: tokens, secret: {secretName: loadtest-tokens-$SET}}
        - {name: body, configMap: {name: loadtest-body}}
YAML
$K apply -f "$RUN/job.yaml" >/dev/null
if [ -n "$KILL_AFTER" ]; then
    ( sleep "$KILL_AFTER"; v=$($K get pod -l app.kubernetes.io/component=gateway -o name | head -1); \
      echo "$(date -u +%H:%M:%S) deleting $v" >> "$RUN/events.txt"; $K delete "$v" --wait=false >/dev/null ) &
fi
$K wait --for=condition=complete job/loadgen --timeout="$((secs + 180))s" >/dev/null 2>&1 \
    || $K wait --for=condition=failed job/loadgen --timeout=5s >/dev/null 2>&1 || true
end="$(date -u +%s)"
for p in $($K get pod -l app=loadgen -o name); do $K logs "$p" >> "$RUN/hey.csv" 2>/dev/null; done

# --- collection ----------------------------------------------------------------
after="$(mock_stats | tr '\n' ' ')"
psqlc -f /dev/stdin < "$HERE/sql/top-statements.sql" > "$RUN/statements.tsv" 2>&1 || true
# Statements per request: every systemprompt statement since the reset.
psqlc -c "SELECT sum(calls) FROM pg_stat_statements WHERE dbid = (SELECT oid FROM pg_database WHERE datname = 'systemprompt') AND userid = (SELECT oid FROM pg_roles WHERE rolname = 'systemprompt');" > "$RUN/statement-calls.txt" </dev/null 2>&1 || true
# Audit chain seal lag (0.65.0+ sealer job); absent on 0.64.0.
psqlc -F $'\t' -c "SELECT audit_chain_unsealed('ai_requests'), audit_chain_unsealed('ai_safety_findings');" > "$RUN/chain-unsealed.tsv" </dev/null 2>&1 || true
$K logs -l app.kubernetes.io/component=gateway -c server --since="$((end - start + 30))s" --tail=-1 2>/dev/null \
    | grep -ciE 'saturated|load shed|acquire timeout|pool timed out' > "$RUN/log-pressure.txt" || true
$K get pods -o wide > "$RUN/pods.txt"
cat > "$RUN/meta.json" <<JSON
{"name":"$NAME","concurrency":$C,"set":"$SET","body":"$BODY","duration_s":$secs,"pods":$PODS,
 "mock_mode":"$MOCK_MODE","chain":"$CHAIN","policies":"$POLICIES","helm":"$HELM",
 "kill_pod_after":"$KILL_AFTER","start":$start,"end":$end,"image":"$IMG",
 "mock_before":"$before","mock_after":"$after","pg_restarts":$(( $(pg_restarts) - pg_r0 ))}
JSON
python3 "$HERE/report.py" "$RUN"
