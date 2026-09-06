#!/usr/bin/env bash
# Enrol a real client — Claude Code and OpenCode — as each seeded bridge
# principal, and prove that what lands on disk, what the loopback proxy will
# serve, and what a real model turn can reach are exactly that principal's
# entitlement. Nothing more, nothing less: the sets are compared for equality
# and the diff is printed on failure, because an entitlement that silently
# widens is the regression a marketplace exists to prevent.
#
# Three claims per (principal, host) cell:
#
#   disk   Claude Code: one marketplace dir per gateway marketplace, the
#          installed_plugins keys `<plugin>@<marketplace>`, the sidecar, and
#          every per-plugin .mcp.json naming only entitled servers.
#          OpenCode: the kebab-cased skill dirs, the `mcp.<slug>` entries in
#          opencode.json, and the sidecar's marketplace list.
#   kb     the knowledge-bank server: entitled principals list and CALL
#          search_knowledge over the loopback proxy and hit the seeded
#          sentinel document, and the server records the execution against
#          their user id; everyone else is refused by the proxy and has no
#          config entry pointing at it.
#   model  a real agent turn (claude -p / opencode run) in which the model must
#          call search_knowledge, proven by the server's own rows — an
#          ai_request_tool_calls row on a request for THIS user and model, and
#          a successful mcp_tool_executions row — never by the transcript.
#
# The expectations come from playwright/setup/seed/entitlements.json and the
# PATs from playwright/.auth/pats.json, both written by `just e2e-seed`; the
# `<plugin>@<marketplace>` keys are derived from those expectations crossed
# with the marketplace membership the signed manifest itself reports.
#
# Host mode only (E2E_CLIENT=host): every cell binds the one loopback proxy
# port, so cells run serially under a fresh throwaway HOME each. Container
# mode would need one container per cell and cannot reach this gateway from
# Docker Desktop anyway; it is refused rather than half-supported.
#
# Usage:
#   scripts/e2e-marketplaces.sh --list
#   scripts/e2e-marketplaces.sh [GATEWAY] [--principals a,b] [--hosts h1,h2] [--skip-model]
#
# Options:
#   --principals europe,india,kw,unassigned,admin   default: all five
#   --hosts claude-code,opencode                    default: both
#   --skip-model        skip the real-model cell
#   --list              print the cells and their expectations; needs no server
#
# Environment:
#   E2E_CLIENT=host     required (the default here)
#   MODELS=a,b          models for the real-model cell (default claude-sonnet-5)
#   E2E_CLAUDE_BIN      path to `claude` (default ~/.claude/local/claude)
#   OPENCODE_BIN        path to `opencode` (default: the one on PATH)
#   SKIP_BRIDGE_BUILD=1 reuse the bridge tarball already in dist/
#   SYNC_WAIT=30        seconds to wait for the bridge sync to write its sidecar
#   CELL_TIMEOUT=180    seconds allowed for one agent turn
#   KEEP_CLIENT_HOME=1  keep every throwaway HOME for inspection
#   SYSTEMPROMPT_DATABASE_URL  override the Postgres URL
set -uo pipefail

# shellcheck source=lib/e2e-client.sh
. "$(dirname "${BASH_SOURCE[0]}")/lib/e2e-client.sh"
e2e_repo_root

PATS_FILE="$REPO_ROOT/playwright/.auth/pats.json"
EXPECT_FILE="$REPO_ROOT/playwright/setup/seed/entitlements.json"
CELL_TIMEOUT="${CELL_TIMEOUT:-180}"
SYNC_WAIT="${SYNC_WAIT:-30}"
KB_SERVER="knowledge-bank"
KB_TOOL="search_knowledge"
KB_QUERY='{"query":"Larkspur pilot third Tuesday","top_k":3}'
KB_SENTINEL="Larkspur"
ALL_PRINCIPALS="europe india kw unassigned admin"
ALL_HOSTS="claude-code opencode"

# ── Arguments ─────────────────────────────────────────────────────────────────
LIST_ONLY=0; SKIP_MODEL=0
PRINCIPALS="$ALL_PRINCIPALS"; HOSTS="$ALL_HOSTS"; GATEWAY=""
while [ $# -gt 0 ]; do
    case "$1" in
        --list)        LIST_ONLY=1 ;;
        --skip-model)  SKIP_MODEL=1 ;;
        --principals)  PRINCIPALS="${2//,/ }"; shift ;;
        --principals=*) PRINCIPALS="${1#*=}"; PRINCIPALS="${PRINCIPALS//,/ }" ;;
        --hosts)       HOSTS="${2//,/ }"; shift ;;
        --hosts=*)     HOSTS="${1#*=}"; HOSTS="${HOSTS//,/ }" ;;
        -h|--help)     sed -n '2,/^set -uo/p' "$0" | sed '$d' | sed 's/^# \{0,1\}//'; exit 0 ;;
        -*)            fail "unknown option $1 (see --help)" ;;
        *)             GATEWAY="$1" ;;
    esac
    shift
done
GATEWAY="${GATEWAY:-http://127.0.0.1:8080}"
HOST_GATEWAY="$(e2e_host_gateway_from "$GATEWAY")"
GATEWAY="$HOST_GATEWAY"

# The CLI spells the knowledge worker `kw`; the seed files spell it out.
seed_key() { case "$1" in kw) printf 'knowledgeWorker' ;; *) printf '%s' "$1" ;; esac; }
for p in $PRINCIPALS; do
    case " $ALL_PRINCIPALS " in *" $p "*) ;; *) fail "unknown principal '$p' — one of: ${ALL_PRINCIPALS// /,}" ;; esac
done
for h in $HOSTS; do
    case " $ALL_HOSTS " in *" $h "*) ;; *) fail "unknown host '$h' — one of: ${ALL_HOSTS// /,}" ;; esac
done

command -v jq >/dev/null 2>&1 || fail "jq is required"
[ -f "$EXPECT_FILE" ] || fail "no $EXPECT_FILE"

# The skill the real-model prompt names, per principal: one the principal
# actually holds, so a refusal is a model or transport fault and not an
# entitlement one. Principals without a knowledge skill skip the model cell.
model_skill() {
    case "$1" in
        kw) printf 'client_research' ;;
        europe) printf 'sfnext_plan_change' ;;
        india) printf 'sf_story_intake' ;;
        *) printf '' ;;
    esac
}
# What a user pastes after the slash command — enough for the skill to run.
skill_argument() {
    case "$1" in
        client_research) printf 'Brief me on Larkspur Outdoors before my call.' ;;
        sfnext_plan_change) printf 'Requirement LARK-142 for Larkspur Outdoors: add an express checkout button on the cart page for returning EU customers; spec is in the knowledge bank.' ;;
        sf_story_intake) printf 'Jira story LARK-77 for Larkspur Outdoors: As a sales rep I want a Lost Reason picklist on Opportunity, required when Stage is Closed Lost, so pipeline reviews can see why deals are lost. AC: field appears on the Opportunity layout; validation blocks Closed Lost without a reason; existing records unaffected.' ;;
        *) printf 'Larkspur Outdoors.' ;;
    esac
}
# The plugin that carries that skill: the slash command is `/<plugin>:<skill>`.
model_plugin() {
    case "$1" in
        kw) printf 'astound-cowork-clients' ;;
        europe) printf 'astound-europe-storefront' ;;
        india) printf 'astound-india-sfcore' ;;
        *) printf '' ;;
    esac
}

# ── Expectations ──────────────────────────────────────────────────────────────
expected() { jq -r --arg k "$(seed_key "$1")" --arg f "$2" '.[$k][$f][]' "$EXPECT_FILE" | sort -u; }
expects_kb() { expected "$1" mcp_servers | grep -qx "$KB_SERVER"; }

# The bridge's two slug rules, transcribed from the vendored core:
#   kebab_dir (managed_skills.rs): lowercase; a run of non-alphanumerics is one
#     dash; no leading/trailing dash; 64 chars.
#   normalize_key (mcp_registry.rs): alphanumerics and `_` survive lowercased;
#     any other run is one dash; no trailing dash.
kebab()        { printf '%s' "$1" | tr 'A-Z' 'a-z' | sed -E 's/[^a-z0-9]+/-/g; s/^-//; s/-$//' | cut -c1-64 | sed 's/-$//'; }
normalize_key() { printf '%s' "$1" | tr 'A-Z' 'a-z' | sed -E 's/[^a-z0-9_]+/-/g; s/^-//; s/-$//'; }
map_lines() { local f="$1" x; while IFS= read -r x; do [ -n "$x" ] && "$f" "$x" && echo; done | sort -u; }

if [ "$LIST_ONLY" = "1" ]; then
    printf '%-12s %-12s %-40s %-14s %s\n' PRINCIPAL HOST MARKETPLACES MCP_SERVERS MODEL_SKILL
    for p in $PRINCIPALS; do for h in $HOSTS; do
        printf '%-12s %-12s %-40s %-14s %s\n' "$p" "$h" \
            "$(expected "$p" marketplaces | paste -sd, -)" \
            "$(expected "$p" mcp_servers | paste -sd, -)" \
            "$(model_skill "$p")"
    done; done
    echo
    info "models for the real-model cell: ${MODELS:-claude-sonnet-5}"
    exit 0
fi

# ── Preconditions ─────────────────────────────────────────────────────────────
CLIENT_MODE="${E2E_CLIENT:-host}"
[ "$CLIENT_MODE" = "host" ] || fail "E2E_CLIENT=$CLIENT_MODE is not supported: every cell binds the one
       loopback proxy port (${E2E_PROXY_PORT}) and would need its own container, and
       a container cannot reach this gateway from Docker Desktop's VM anyway.
       Run with E2E_CLIENT=host."
command -v psql >/dev/null 2>&1 || fail "psql is required — the pass is asserted against Postgres, not the transcript"
[ -f "$PATS_FILE" ] || fail "no $PATS_FILE — run \`just e2e-seed\` to mint the bridge PATs"

curl -fsS --max-time 5 "$HOST_GATEWAY/v1/models" >/dev/null 2>&1 \
    || fail "no gateway answering at $HOST_GATEWAY — start the server (\`just start\`) and re-run.
       This harness deliberately does not start or restart anything."
e2e_db_url || fail "no database_url — set SYSTEMPROMPT_DATABASE_URL"
e2e_db_reachable || fail "cannot reach Postgres at the profile's database_url — \`just db-up\` first"

CLAUDE_BIN="${E2E_CLAUDE_BIN:-$HOME/.claude/local/claude}"
OPENCODE_BIN="${OPENCODE_BIN:-$(command -v opencode 2>/dev/null || true)}"
case " $HOSTS " in *" claude-code "*) [ -x "$CLAUDE_BIN" ] || fail "no executable claude at $CLAUDE_BIN — set E2E_CLAUDE_BIN" ;; esac
case " $HOSTS " in *" opencode "*) [ -n "$OPENCODE_BIN" ] && [ -x "$OPENCODE_BIN" ] || fail "no executable opencode — set OPENCODE_BIN" ;; esac

MODEL_LIST="$(printf '%s' "${MODELS:-claude-sonnet-5}" | tr ',' '\n' | sed '/^[[:space:]]*$/d')"

# The client installs the bridge by downloading the tarball the gateway serves.
TARBALL="$REPO_ROOT/dist/astound-bridge-linux-$(uname -m).tar.gz"
if [ "${SKIP_BRIDGE_BUILD:-0}" = "1" ] && [ -f "$TARBALL" ]; then
    info "reusing $TARBALL"
else
    say "packaging the Linux bridge tarball"
    "$REPO_ROOT/scripts/package-bridge-linux.sh" || fail "bridge packaging failed"
fi

# ── Cell plumbing ─────────────────────────────────────────────────────────────
cx() { cx_host "$1"; }
CLIENT_HOME=""
cleanup_cell() {
    e2e_host_cleanup
    CLIENT_HOME=""
    # The proxy's port is released asynchronously; the next cell's guard would
    # otherwise refuse on our own dying process.
    local i
    for i in $(seq 1 50); do
        ss -ltn 2>/dev/null | grep -q "127.0.0.1:${E2E_PROXY_PORT} " || return 0
        sleep 0.2
    done
    warn "127.0.0.1:${E2E_PROXY_PORT} is still bound after our cleanup"
}
RESULTS_FILE="$(mktemp)"
LOG_DIR="$(mktemp -d)"
trap 'cleanup_cell; rm -f "$RESULTS_FILE"' EXIT INT TERM

# Set equality, sorted and unique on both sides, with the diff on failure.
#   $1 label  $2 expected (newline list)  $3 actual (newline list)
assert_set() {
    local exp act d
    exp="$(printf '%s\n' "$2" | sed '/^$/d' | sort -u)"
    act="$(printf '%s\n' "$3" | sed '/^$/d' | sort -u)"
    if [ "$exp" = "$act" ]; then
        ok "$1: {$(printf '%s' "$act" | paste -sd, -)}"
        return 0
    fi
    d="$(diff <(printf '%s\n' "$exp") <(printf '%s\n' "$act") | sed -n 's/^< /  missing: /p; s/^> /  extra:   /p')"
    printf '[FAIL] %s — expected {%s}, got {%s}\n%s\n' "$1" \
        "$(printf '%s' "$exp" | paste -sd, -)" "$(printf '%s' "$act" | paste -sd, -)" "$d"
    return 1
}
# Subset: every line of $3 must appear in $2.
assert_subset() {
    local extra
    extra="$(comm -13 <(printf '%s\n' "$2" | sed '/^$/d' | sort -u) <(printf '%s\n' "$3" | sed '/^$/d' | sort -u))"
    if [ -z "$extra" ]; then ok "$1: ⊆ {$(printf '%s\n' "$2" | sed '/^$/d' | sort -u | paste -sd, -)}"; return 0; fi
    printf '[FAIL] %s — not entitled: %s\n' "$1" "$(printf '%s' "$extra" | paste -sd, -)"
    return 1
}

# Wait for the bridge sync to land its sidecar: the installer returns when the
# proxy is up, and the host sync can finish a moment later.
wait_for() {
    local i
    for i in $(seq 1 "$SYNC_WAIT"); do
        [ -e "$1" ] && return 0
        sleep 1
    done
    return 1
}

# ── The manifest, as the gateway signs it ─────────────────────────────────────
# Fetched with the principal's PAT (exchanged for a bridge JWT) and read for marketplace membership only;
# the spec in playwright/tests/bridge-manifest.spec.ts already pins its sets.
fetch_manifest() {
    # Why: the manifest route takes the bridge JWT a client mints from its PAT
    # at /v1/auth/bridge/pat, not the raw PAT — the same exchange the bridge
    # performs after enrolment.
    local token
    token="$(curl -fsS --max-time 15 -X POST -H "Authorization: Bearer $1" \
        -H 'content-type: application/json' -d '{}' "$HOST_GATEWAY/v1/auth/bridge/pat" \
        | jq -r '.token // empty' 2>/dev/null)" || return 1
    [ -n "$token" ] || return 1
    curl -fsS --max-time 15 -H "Authorization: Bearer $token" "$HOST_GATEWAY/v1/bridge/manifest" \
        | jq -r '.payload' 2>/dev/null
}
# `<plugin>@<marketplace>` for every expected plugin under every expected
# marketplace the manifest says carries it.
expected_plugin_keys() {
    local p="$1" manifest="$2" m
    for m in $(expected "$p" marketplaces); do
        comm -12 <(expected "$p" plugins) \
                 <(printf '%s' "$manifest" | jq -r --arg m "$m" '.marketplaces[]? | select(.id==$m) | .plugin_ids[]' | sort -u) \
            | sed "s/\$/@$m/"
    done | sort -u
}

# ── disk ──────────────────────────────────────────────────────────────────────
disk_claude_code() {
    local p="$1" manifest="$2" plugins="$CLIENT_HOME/.claude/plugins" r=0 m d servers
    wait_for "$plugins/.systemprompt-marketplaces.json" \
        || { printf '[FAIL] no %s after %ss — the bridge never synced Claude Code\n' "$plugins/.systemprompt-marketplaces.json" "$SYNC_WAIT"; return 1; }
    assert_set "marketplace dirs" "$(expected "$p" marketplaces)" \
        "$(ls -1 "$plugins/marketplaces" 2>/dev/null)" || r=1
    assert_set "installed_plugins keys" "$(expected_plugin_keys "$p" "$manifest")" \
        "$(jq -r '.plugins | keys[]' "$plugins/installed_plugins.json" 2>/dev/null)" || r=1
    assert_set "sidecar marketplaces" "$(expected "$p" marketplaces)" \
        "$(jq -r '.marketplaces[]' "$plugins/.systemprompt-marketplaces.json" 2>/dev/null)" || r=1
    servers="$(expected "$p" mcp_servers | map_lines normalize_key)"
    for d in "$plugins"/marketplaces/*/plugins/*/; do
        [ -f "$d.mcp.json" ] || continue
        m="${d#"$plugins"/marketplaces/}"; m="${m%/}"
        assert_subset ".mcp.json servers in $m" "$servers" \
            "$(jq -r '.mcpServers | keys[]' "$d.mcp.json" 2>/dev/null)" || r=1
    done
    return $r
}

opencode_cfg() {
    local f
    for f in "$CLIENT_HOME/.config/opencode/opencode.jsonc" "$CLIENT_HOME/.config/opencode/opencode.json"; do
        [ -f "$f" ] && { printf '%s' "$f"; return 0; }
    done
    printf '%s' "$CLIENT_HOME/.config/opencode/opencode.json"
}
disk_opencode() {
    local p="$1" skills="$CLIENT_HOME/.config/opencode/skills" r=0 cfg
    wait_for "$skills/.systemprompt-managed.json" \
        || { printf '[FAIL] no %s after %ss — the bridge never synced OpenCode\n' "$skills/.systemprompt-managed.json" "$SYNC_WAIT"; return 1; }
    cfg="$(opencode_cfg)"
    assert_set "skill dirs" "$(expected "$p" skills | map_lines kebab)" \
        "$(find "$skills" -mindepth 1 -maxdepth 1 -type d -not -name '.*' -printf '%f\n' 2>/dev/null)" || r=1
    assert_set "opencode.json mcp keys" "$(expected "$p" mcp_servers | map_lines normalize_key)" \
        "$(jq -r '.mcp // {} | keys[]' "$cfg" 2>/dev/null)" || r=1
    assert_set "sidecar marketplaces" "$(expected "$p" marketplaces)" \
        "$(jq -r '.marketplaces[]?' "$skills/.systemprompt-managed.json" 2>/dev/null)" || r=1
    return $r
}

# ── kb ────────────────────────────────────────────────────────────────────────
kb_exec_count() {
    q "select count(*) from mcp_tool_executions
       where user_id = '$1' and tool_name = '$KB_TOOL' and status = 'success'"
}
kb_entitled() {
    local uid="$1" before after result r=0
    before="$(kb_exec_count "$uid")"
    result="$(preflight_mcp "$KB_SERVER" "$KB_TOOL" "$KB_QUERY")"
    case $? in
        0) ok "$KB_TOOL listed and callable over the loopback proxy" ;;
        1) printf '[FAIL] %s does not list %s\n' "$KB_SERVER" "$KB_TOOL"; return 1 ;;
        2) printf '[FAIL] %s call failed: %s\n' "$KB_TOOL" "$(printf '%s' "$result" | tr -d '\n' | cut -c1-300)"; return 1 ;;
        *) printf '[FAIL] the proxy opened no MCP session for %s\n' "$KB_SERVER"; return 1 ;;
    esac
    if printf '%s' "$result" | grep -qi "$KB_SENTINEL"; then
        ok "a hit mentions $KB_SENTINEL"
    else
        printf '[FAIL] no hit mentions %s: %s\n' "$KB_SENTINEL" "$(printf '%s' "$result" | tr -d '\n' | cut -c1-300)"
        printf '       is the sentinel indexed? \`just knowledge-seed\`\n'
        r=1
    fi
    sleep 2
    after="$(kb_exec_count "$uid")"
    if [ "${after:-0}" -gt "${before:-0}" ]; then
        ok "mcp_tool_executions for user $uid: $before -> $after"
    else
        printf '[FAIL] no new successful mcp_tool_executions row for user %s (%s -> %s)\n' "$uid" "$before" "$after"; r=1
    fi
    return $r
}
kb_refused() {
    local host="$1" status r=0 hits
    status="$(e2e_mcp_status "$KB_SERVER")"
    case "$status" in
        2*) printf '[FAIL] the proxy answered %s for /mcp/%s — not entitled\n' "$status" "$KB_SERVER"; r=1 ;;
        *)  ok "the proxy refuses /mcp/$KB_SERVER ($status)" ;;
    esac
    if [ "$host" = "claude-code" ]; then
        # Only the bridge-written .mcp.json files count: a plugin's own
        # descriptor may name the server it asked for and still not get it.
        hits="$(find "$CLIENT_HOME/.claude/plugins" -name .mcp.json -exec grep -l "\"$KB_SERVER\"" {} + 2>/dev/null || true)"
    else
        hits="$(jq -r --arg s "$KB_SERVER" '.mcp[$s] // empty' "$(opencode_cfg)" 2>/dev/null || true)"
    fi
    if [ -z "$hits" ]; then ok "no config entry names $KB_SERVER"; else printf '[FAIL] config names %s:\n%s\n' "$KB_SERVER" "$hits"; r=1; fi
    return $r
}

# ── model ─────────────────────────────────────────────────────────────────────
model_prompt() {
    local tool
    case "$1" in
        claude-code) tool="mcp__${KB_SERVER}__${KB_TOOL}" ;;
        *) tool="${KB_SERVER}_${KB_TOOL}" ;;
    esac
    # Why: a skill run is recorded from the client's own signals — the
    # `/plugin:skill` slash prompt (skill ids hyphenated) or the Skill tool —
    # so the turn is started the way a user starts it, not by describing the
    # skill in prose. See extensions/web/schema/migrations/040_skill_invocation_view.sql.
    # Why: each skill refuses to run without its real input (a plan needs a
    # requirement, an intake needs a story), so the argument is what a user
    # would paste, not a bare client name.
    printf '/%s:%s %s Call the %s tool first and reply with the first hit'"'"'s title only. Do not use Bash and do not answer from memory.' "$3" "${2//_/-}" "$(skill_argument "$2")" "$tool"
}
run_model_turn() {
    local host="$1" model="$2" prompt="$3" raw="$4" cfg
    if [ "$host" = "claude-code" ]; then
        cx "cd \$HOME/work && timeout ${CELL_TIMEOUT} claude -p \"${prompt//\"/\\\"}\" \
              --model '$model' --permission-mode bypassPermissions --output-format json \
              --allowedTools 'Skill,mcp__${KB_SERVER}__${KB_TOOL}' 2>&1" >"$raw" 2>&1
    else
        # No tool policy: OpenCode 0.5 rejects a top-level `tools` key
        # ("Unrecognized key(s) in object: 'tools'") and refuses to start. The
        # prompt names the tool, and the verdict is the server's own rows.
        cx "cd \$HOME/work && timeout ${CELL_TIMEOUT} opencode run --model 'systemprompt/$model' \"${prompt//\"/\\\"}\" 2>&1" >"$raw" 2>&1
    fi
}
model_cell() {
    local p="$1" host="$2" uid="$3" skill model cutoff raw rc r=0 mcp_ok row req said
    skill="$(model_skill "$p")"
    for model in $MODEL_LIST; do
        if ! gateway_up; then printf '[FAIL] the gateway stopped answering before the %s turn\n' "$model"; r=1; continue; fi
        cutoff="$(q "select now()")"
        raw="$LOG_DIR/$p-$host-${model//\//_}.txt"
        run_model_turn "$host" "$model" "$(model_prompt "$host" "$skill" "$(model_plugin "$p")")" "$raw"; rc=$?
        sleep 3
        mcp_ok="$(e2e_count_mcp_exec "$cutoff" "$KB_TOOL" "$KB_SERVER" "$uid")"
        row="$(e2e_tool_call_model_row "$cutoff" "$KB_TOOL" "$model" "$uid")"
        req="$(e2e_count_requests "$cutoff" "$model" "$uid")"
        if [ "${mcp_ok:-0}" -gt 0 ] && [ -n "$row" ]; then
            ok "$model: ai_request_tool_calls names $KB_TOOL for user $uid and mcp_tool_executions succeeded"
        elif [ "${req:-0}" = "0" ]; then
            printf '[FAIL] %s: no ai_requests row for user %s — the model never dispatched (rc=%s)\n' "$model" "$uid" "$rc"; r=1
        elif [ -z "$row" ]; then
            said="$( (jq -r '.result // empty' "$raw" 2>/dev/null | tr '\n' ' ' | cut -c1-160) || true)"
            [ -n "$said" ] || said="$(tr '\n' ' ' <"$raw" | tail -c 160)"
            printf '[FAIL] %s: dispatched but no tool call recorded (rc=%s); said: %s\n' "$model" "$rc" "$said"; r=1
        else
            printf '[FAIL] %s: tool call recorded but no successful MCP execution (rc=%s)\n' "$model" "$rc"; r=1
        fi
    done
    return $r
}

# ── The sweep ─────────────────────────────────────────────────────────────────
say "sweeping principals {${PRINCIPALS// /,}} × hosts {${HOSTS// /,}} (transcripts in $LOG_DIR)"
for p in $PRINCIPALS; do
    PAT="$(jq -r --arg k "$(seed_key "$p")" '.[$k].secret // empty' "$PATS_FILE")"
    UID_="$(jq -r --arg k "$(seed_key "$p")" '.[$k].userId // empty' "$PATS_FILE")"
    for host in $HOSTS; do
        printf '\n\033[1m[%s × %s]\033[0m\n' "$p" "$host"
        DISK=FAIL; KB=FAIL; MODEL=SKIP; NOTE=""
        if [ -z "$PAT" ] || [ -z "$UID_" ]; then
            NOTE="no seeded PAT for $p in $PATS_FILE"
            printf '%s|%s|%s|%s|%s|%s\n' "$p" "$host" FAIL FAIL SKIP "$NOTE" >>"$RESULTS_FILE"
            continue
        fi
        if ! gateway_up; then
            printf '%s|%s|%s|%s|%s|%s\n' "$p" "$host" UNRUN UNRUN UNRUN "the gateway stopped answering" >>"$RESULTS_FILE"
            continue
        fi

        e2e_proxy_port_guard
        e2e_host_client_home "e2e-mkt-$p-$host"
        case "$host" in
            # E2E_THINKING_TOKENS=<n> gives Claude Code an extended-thinking
            # budget so the traced turn carries reasoning tokens as well.
            claude-code) E2E_CLIENT_PATH="$(dirname "$CLAUDE_BIN")"; E2E_CX_ENV=(${E2E_THINKING_TOKENS:+MAX_THINKING_TOKENS=$E2E_THINKING_TOKENS}) ;;
            opencode)    E2E_CLIENT_PATH="$(dirname "$OPENCODE_BIN")"; E2E_CX_ENV=(OPENCODE_DISABLE_AUTOUPDATE=1 TERM=dumb) ;;
        esac

        say "enrol $p on $host under $CLIENT_HOME"
        ENROL_RC=0
        e2e_enrol_host "$host" "$PAT" >"$LOG_DIR/$p-$host-install.txt" 2>&1 || ENROL_RC=$?
        if [ "$ENROL_RC" != "0" ]; then
            NOTE="installer exited $ENROL_RC (see $LOG_DIR/$p-$host-install.txt)"
            printf '[FAIL] %s\n' "$NOTE"
            printf '%s|%s|%s|%s|%s|%s\n' "$p" "$host" FAIL FAIL SKIP "$NOTE" >>"$RESULTS_FILE"
            cleanup_cell; continue
        fi

        say "disk: what the bridge wrote for $p on $host"
        MANIFEST="$(fetch_manifest "$PAT")"
        if [ -z "$MANIFEST" ]; then
            printf '[FAIL] /v1/bridge/manifest answered nothing for this PAT\n'
        elif [ "$host" = "claude-code" ]; then
            disk_claude_code "$p" "$MANIFEST" && DISK=PASS
        else
            disk_opencode "$p" && DISK=PASS
        fi

        say "kb: the knowledge-bank server for $p on $host"
        if expects_kb "$p"; then
            kb_entitled "$UID_" && KB=PASS
        else
            kb_refused "$host" && KB=PASS
        fi

        if [ "$SKIP_MODEL" = "1" ]; then
            NOTE="model skipped (--skip-model)"
        elif ! expects_kb "$p" || [ -z "$(model_skill "$p")" ]; then
            NOTE="model skipped: no knowledge skill for $p"
        else
            say "model: a real $host turn as $p"
            if model_cell "$p" "$host" "$UID_"; then MODEL=PASS; else MODEL=FAIL; fi
        fi

        printf '%s|%s|%s|%s|%s|%s\n' "$p" "$host" "$DISK" "$KB" "$MODEL" "${NOTE:-ok}" >>"$RESULTS_FILE"
        cleanup_cell
    done
done

# ── Report ────────────────────────────────────────────────────────────────────
echo
say "results"
printf '%-12s %-12s %-6s %-6s %-6s %s\n' PRINCIPAL HOST DISK KB MODEL NOTE
printf '%-12s %-12s %-6s %-6s %-6s %s\n' ------------ ------------ ------ ------ ------ ----
while IFS='|' read -r p h d k m n; do
    printf '%-12s %-12s %-6s %-6s %-6s %s\n' "$p" "$h" "$d" "$k" "$m" "$n"
done <"$RESULTS_FILE"
FAILS="$(grep -c 'FAIL\|UNRUN' "$RESULTS_FILE" || true)"
CELLS="$(wc -l <"$RESULTS_FILE" | tr -d ' ')"
echo
info "transcripts and installer logs: $LOG_DIR"
if [ "$FAILS" = "0" ]; then
    info "all $CELLS cells passed"
    exit 0
fi
warn "$FAILS of $CELLS cells failed or never ran"
exit 1
