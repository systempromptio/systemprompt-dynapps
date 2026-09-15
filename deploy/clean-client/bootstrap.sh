#!/usr/bin/env bash
# Brings a clean client all the way to a working `claude`, asking for exactly
# one thing: the device-link code.
#
# Everything else — install, proxy, sync — is derivable, so it runs unattended.
# The code is not: it is a credential, and proving identity through a browser is
# the point of the flow rather than an obstacle to it.
set -euo pipefail

GATEWAY="${ASTOUND_BRIDGE_GATEWAY_URL:?gateway URL not set}"

# The container reaches the gateway at host.docker.internal; a browser on the
# host reaches the same server at localhost. Printing the container's view would
# send you to a hostname your browser cannot resolve.
BROWSER_GATEWAY="${GATEWAY//host.docker.internal/localhost}"

bold() { printf '\033[1m%s\033[0m\n' "$*"; }
step() { printf '\n\033[1m==>\033[0m %s\n' "$*"; }
warn() { printf '\033[33mwarn:\033[0m %s\n' "$*" >&2; }

if ! command -v astound-bridge >/dev/null 2>&1; then
    printf '\033[31mERROR:\033[0m astound-bridge is not mounted. Run `just bridge-build` first.\n' >&2
    exit 1
fi

# ── 0. Feedback plane ─────────────────────────────────────────────────────────
# The bridge self-enrols a device credential at login, but only over https or
# loopback http — the credential is a bearer token and host.docker.internal is
# neither. So relay the port through 127.0.0.1 and hand the bridge a localhost
# URL; the host's gateway is still what answers. Any other gateway is left
# exactly as given. See README "Feedback plane".
case "$GATEWAY" in
    http://host.docker.internal:*|http://host.docker.internal|http://host.docker.internal/*)
        FWD_PORT="${GATEWAY#http://host.docker.internal}"
        FWD_PORT="${FWD_PORT#:}"; FWD_PORT="${FWD_PORT%%/*}"
        FWD_PORT="${FWD_PORT:-80}"
        if ! command -v gateway-forward >/dev/null 2>&1; then
            printf '\033[31mERROR:\033[0m gateway-forward is missing from this image. Run `just clean-client-build`.\n' >&2
            exit 1
        fi
        step "relaying 127.0.0.1:$FWD_PORT to host.docker.internal for the feedback plane"
        gateway-forward "$FWD_PORT" host.docker.internal
        GATEWAY="http://localhost:$FWD_PORT"
        ;;
esac

# ── 1. Credentials ────────────────────────────────────────────────────────────
# Skip the prompt when the state volume already carries a PAT that still works;
# re-authenticating a working client would just burn a code.
if [ -f "$HOME/.config/astound/astound-bridge.pat" ] && astound-bridge whoami >/dev/null 2>&1; then
    step "already signed in — reusing the stored PAT"
    # A credential saved by an earlier bootstrap still names the Docker alias
    # as its gateway; whoami works over it, but no device is enrolled and the
    # feedback plane stays dark until the sign-in is redone through the relay.
    if grep -qs 'host\.docker\.internal' "$HOME/.config/astound/astound-bridge.toml"; then
        warn "the stored config still points at host.docker.internal, so the bridge
      never self-enrolled a device. Run 'just clean-client-reset' and sign in
      again to route through 127.0.0.1 (see README, \"Feedback plane\")."
    fi
elif [ -n "${ASTOUND_BRIDGE_CODE:-}" ]; then
    # The caller already has the code (it came off the profile page as an
    # argument), so there is nothing to ask for.
    step "signing in with the supplied code"
    astound-bridge login --code "$ASTOUND_BRIDGE_CODE" \
        --gateway "$GATEWAY" --device-name clean-client
else
    step "sign in"
    bold "  Open this in your browser:"
    printf '\n      %s/bridge-auth/device-link\n\n' "$BROWSER_GATEWAY"
    echo "  Sign in, click Allow, then copy the code (the 'Just the code' section)."
    echo
    printf '  code: '
    read -r CODE

    # Terminals with bracketed paste wrap a paste in ESC[200~ … ESC[201~.
    # Readline strips those at an interactive prompt; `read` in a script does
    # not, so without this the code arrives with an invisible prefix and the
    # gateway rejects a string the screen shows as correct.
    CODE=$(printf '%s' "$CODE" | sed $'s/\033\\[[0-9;]*[~A-Za-z]//g' | tr -d '\000-\037')
    # Then the surrounding whitespace a paste often carries. `read` strips its
    # own, but stripping an escape can expose more, so trim after rather than
    # relying on what came before.
    CODE="${CODE#"${CODE%%[![:space:]]*}"}"
    CODE="${CODE%"${CODE##*[![:space:]]}"}"

    # Tolerant on purpose: the page shows a command as well as a bare code, and
    # the CLI accepts either. Refusing a paste here would be gratuitous.
    astound-bridge login --code "$CODE" --gateway "$GATEWAY" --device-name clean-client
fi

# ── 2. Policy + plugins ───────────────────────────────────────────────────────
step "installing integration"
astound-bridge install --apply --apply-schedule

step "starting the loopback proxy"
# No systemd in this container, which is why --apply-schedule warned above.
if astound-bridge doctor 2>/dev/null | grep -q '^\[OK  \] inference proxy'; then
    echo "    already running"
else
    astound-bridge proxy >"$HOME/.cache/bridge-proxy.log" 2>&1 &
    for _ in $(seq 1 20); do
        astound-bridge doctor 2>/dev/null | grep -q '^\[OK  \] inference proxy' && break
        sleep 0.5
    done
fi

step "syncing plugins, skills, agents, MCP"
astound-bridge sync --allow-tofu

step "verifying"
astound-bridge doctor || warn "doctor reported problems — read them above before trusting the result"

if [ -n "${CLEAN_CLIENT_EXEC_CLAUDE:-}" ]; then
    step "starting Claude Code"
    # A login shell so the managed block in ~/.profile is sourced — that
    # sourcing is what puts ANTHROPIC_BASE_URL in front of claude.
    exec bash -lc claude
fi

cat <<'DONE'

  Ready. Type:

      claude

  This shell is a login shell, so ANTHROPIC_BASE_URL and ANTHROPIC_AUTH_TOKEN
  are already set from ~/.config/astound/env.sh. Proxy log: ~/.cache/bridge-proxy.log
  Gateway relay log (host.docker.internal gateways only): ~/.cache/gateway-forward.<port>.log

DONE

# A login shell, so the managed block in ~/.profile that install just wrote is
# actually sourced — that sourcing is the thing being tested.
exec bash -l
