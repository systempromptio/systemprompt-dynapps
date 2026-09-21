#!/usr/bin/env bash

machine_state() {
    flyctl machines list -a "$FLY_APP" --json | jq -er '
        if length != 1 then error("expected exactly one Fly machine") else .[0] end |
        select((.image_ref.digest | type) == "string" and (.updated_at | type) == "string") |
        select(.image_ref.digest | startswith("sha256:")) |
        "\(.image_ref.digest) \(.updated_at)"'
}

read_health() {
    local body
    body="$(curl -sS -m 10 -w '\n%{http_code}' "$HOST/health" 2>/dev/null)" || return 1
    local code="${body##*$'\n'}"
    local payload="${body%$'\n'*}"
    echo "    $(date -u +%H:%M:%S) $code $payload"
    [ "$code" = 200 ] && jq -e --arg version "$VERSION" '.status == "healthy" and .version == $version' <<<"$payload" >/dev/null
}

observe_deploy() {
    log "waiting for $FLY_APP digest and update timestamp to move"
    local deadline=$((SECONDS + DIGEST_WAIT_SECONDS))
    local state
    while :; do
        state="$(machine_state)" || die 'could not read Fly machine state'
        after="${state%% *}"
        if [ "$after" != "$before" ] && [ "${state#* }" != "${before_state#* }" ]; then break; fi
        [ "$SECONDS" -lt "$deadline" ] || die "Fly digest and update timestamp did not both move within ${DIGEST_WAIT_SECONDS}s; worktree kept at $WORKTREE"
        sleep "$DIGEST_WAIT_INTERVAL"
    done
    echo "    $FLY_APP image after: $after (updated ${state#* })"
    log "waiting for healthy $VERSION before the observation window"
    deadline=$((SECONDS + HEALTH_READY_SECONDS))
    until read_health; do
        [ "$SECONDS" -lt "$deadline" ] || die "no healthy $VERSION response within ${HEALTH_READY_SECONDS}s; worktree kept at $WORKTREE"
        sleep "$DIGEST_WAIT_INTERVAL"
    done
    log "watching $HOST/health for ${WATCH_SECONDS}s (every ${WATCH_INTERVAL}s)"
    deadline=$((SECONDS + WATCH_SECONDS))
    while [ "$SECONDS" -lt "$deadline" ]; do
        local remaining=$((deadline - SECONDS))
        local pause="$WATCH_INTERVAL"
        [ "$remaining" -ge "$pause" ] || pause="$remaining"
        sleep "$pause"
        read_health || die "$HOST failed the healthy $VERSION observation window; worktree kept at $WORKTREE"
    done
}
