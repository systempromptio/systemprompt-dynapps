#!/usr/bin/env bash
# Exercise the coordinator without touching a real build or its ledger.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
SCRATCH="$(mktemp -d "${TMPDIR:-/tmp}/astound-coordinator.XXXXXX")"
cleanup() {
    local result=$?
    if [ "$result" -ne 0 ]; then
        for log in "$SCRATCH"/*.log; do
            echo "$log"
            tail -15 "$log"
        done
    fi
    for pid in $(jobs -pr); do kill "$pid" 2>/dev/null || true; done
    wait || true
    rm -rf "$SCRATCH"
    exit "$result"
}
trap cleanup EXIT
mkdir -p "$SCRATCH/repo/scripts"
cp "$ROOT/scripts/build-coordinator.sh" "$SCRATCH/repo/scripts/"
cp "$ROOT/Cargo.toml" "$SCRATCH/repo/"
git -C "$SCRATCH/repo" init --quiet
git -C "$SCRATCH/repo" -c user.name=Fixture -c user.email=fixture@example.test \
    commit --quiet --allow-empty -m initial
COORD="$SCRATCH/repo/scripts/build-coordinator.sh"
export COORD_STATE_DIR="$SCRATCH/state"

# All contenders can observe the same dead owner without deleting a successor.
mkdir -p "$COORD_STATE_DIR/lock.owner.99999999.fixture"
printf '99999999\n' > "$COORD_STATE_DIR/lock.owner.99999999.fixture/pid"
ln -s "$COORD_STATE_DIR/lock.owner.99999999.fixture" "$COORD_STATE_DIR/lock"

# Distinct requests must never overlap their critical sections.
pids=()
for request in {1..12}; do
    bash "$COORD" run contention "$request" -- bash -c '
        set -e
        mkdir "$1/critical"
        sleep 0.02
        rmdir "$1/critical"
    ' bash "$SCRATCH" > "$SCRATCH/request-$request.log" 2>&1 &
    pids+=("$!")
done
for pid in "${pids[@]}"; do wait "$pid"; done
test ! -e "$COORD_STATE_DIR/lock.owner.99999999.fixture"

# Identical callers attach and run the command only once.
bash "$COORD" run identical "" -- bash -c '
    printf x >> "$1/count"
    sleep 1
' bash "$SCRATCH" > "$SCRATCH/leader.log" 2>&1 &
leader=$!
until [ -s "$COORD_STATE_DIR/lock/pid" ]; do sleep 0.01; done
bash "$COORD" run identical "" -- bash -c '
    printf x >> "$1/count"
    sleep 1
' bash "$SCRATCH" > "$SCRATCH/follower.log" 2>&1 &
follower=$!
wait "$leader"
wait "$follower"
test "$(wc -c < "$SCRATCH/count" | tr -d ' ')" = 1
grep -q 'attaching to its output' "$SCRATCH/follower.log"

# A failing leader and its attached caller both retain the original status.
bash "$COORD" run failing "" -- bash -c 'sleep 1; exit 42' > "$SCRATCH/failing-leader.log" 2>&1 &
leader=$!
until [ -s "$COORD_STATE_DIR/lock/pid" ]; do sleep 0.01; done
bash "$COORD" run failing "" -- false > "$SCRATCH/failing-follower.log" 2>&1 &
follower=$!
leader_status=0
follower_status=0
wait "$leader" || leader_status=$?
wait "$follower" || follower_status=$?
test "$leader_status" = 42
test "$follower_status" = 42

# Real-directory locks from the previous implementation remain recoverable.
mkdir "$COORD_STATE_DIR/lock"
printf '99999999\n' > "$COORD_STATE_DIR/lock/pid"
bash "$COORD" run stale "" -- true > "$SCRATCH/stale.log" 2>&1
grep -q 'clearing stale lock' "$SCRATCH/stale.log"

# A queued request must record the checkout it actually builds.
bash "$COORD" run holding "" -- sleep 3 > "$SCRATCH/holding.log" 2>&1 &
leader=$!
until [ -s "$COORD_STATE_DIR/lock/pid" ]; do sleep 0.01; done
bash "$COORD" run queued "" -- true > "$SCRATCH/queued.log" 2>&1 &
waiter=$!
until grep -q 'waiting for it' "$SCRATCH/queued.log"; do sleep 0.01; done
git -C "$SCRATCH/repo" -c user.name=Fixture -c user.email=fixture@example.test \
    commit --quiet --allow-empty -m changed
wait "$leader"
wait "$waiter"
expected="$(bash "$COORD" key)"
actual="$(python3 -c 'import json, sys; print(json.load(open(sys.argv[1]))["tree"])' \
    "$COORD_STATE_DIR/latest/queued.json")"
test "$actual" = "$expected"
test ! -e "$COORD_STATE_DIR/lock"
echo 'build coordinator: contention, attachment, failure propagation, stale recovery and fresh identity passed'
