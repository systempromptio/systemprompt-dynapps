#!/usr/bin/env bash

# Why: the app also holds the agent farm's runner pool (stamped
# metadata.farm_runner=pool by extensions/web/admin/farm/src/services/runner/fly/api.rs),
# whose machines run the API's image but are not the deploy target. Fail closed:
# exactly one API machine, and any machine that is neither shape is an error.
api_machine_state() {
    jq -er '
        def worker: (.config.metadata.farm_runner // null) == "pool"
            and (.config.init.exec // []) == ["/app/bin/systemprompt-farm-worker"]
            and ((.config.services // []) | length) == 0;
        def api: (.config.metadata.farm_runner // null) == null
            and ((.config.services // []) | length) > 0;
        if any(.[]; (worker or api) | not) then error("unrecognised Fly machine: neither the API nor a farm runner")
        else [.[] | select(api)] end |
        if length != 1 then error("expected exactly one API Fly machine") else .[0] end |
        select((.image_ref.digest | type) == "string" and (.updated_at | type) == "string") |
        select(.image_ref.digest | startswith("sha256:")) |
        "\(.image_ref.digest) \(.updated_at)"'
}

machine_state() {
    flyctl machines list -a "$FLY_APP" --json | api_machine_state
}

read_health() {
    local body
    body="$(curl -sS -m 10 -w '\n%{http_code}' "$HOST/health" 2>/dev/null)" || return 1
    local code="${body##*$'\n'}"
    local payload="${body%$'\n'*}"
    echo "    $(date -u +%H:%M:%S) $code $payload"
    # /health reports the core crate version, not dynapps's.
    [ "$code" = 200 ] && jq -e --arg version "$CORE_VERSION" '.status == "healthy" and .version == $version' <<<"$payload" >/dev/null
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
    log "waiting for healthy core $CORE_VERSION before the observation window"
    deadline=$((SECONDS + HEALTH_READY_SECONDS))
    until read_health; do
        [ "$SECONDS" -lt "$deadline" ] || die "no healthy core $CORE_VERSION response within ${HEALTH_READY_SECONDS}s; worktree kept at $WORKTREE"
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
