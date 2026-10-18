#!/usr/bin/env bash
# Retry a network operation (registry login, push, crate index fetch) with
# backoff. Only for transport failures: the command's last exit status is
# returned, so a command that keeps failing still fails the step.
#
#   scripts/ci-retry.sh <attempts> <command> [args...]
set -uo pipefail

attempts="${1:?usage: ci-retry.sh <attempts> <command> [args...]}"
shift
delay=5
for ((i = 1; i <= attempts; i++)); do
    "$@" && exit 0
    status=$?
    [ "$i" -lt "$attempts" ] || break
    echo "::warning::attempt $i/$attempts of '$1' failed (exit $status); retrying in ${delay}s" >&2
    sleep "$delay"
    delay=$((delay * 2))
done
echo "::error::'$1' failed after $attempts attempts (exit $status)" >&2
exit "$status"
