#!/bin/sh
# Astound Bridge client installer for Linux.
#
# Published as storage/files/downloads/install.sh and run as:
#
#     curl -fsSL https://your-gateway/files/downloads/install.sh | sh
#
# This installs the desktop *bridge*, not the gateway server
# server. This one takes a bare Linux box to a working `claude`:
#
#   download + checksum -> install bridge -> install Claude Code -> sign in
#   (SSO by default; --pat / --code for unattended installs) -> write env ->
#   sync -> enrol client applications (--host) -> doctor
#
# POSIX sh only — it runs on whatever /bin/sh the target box has.
set -eu

BIN_NAME="astound-bridge"

say()  { printf '%s\n' "$*"; }
step() { printf '\033[1m==>\033[0m %s\n' "$*"; }
warn() { printf '\033[33mwarn:\033[0m %s\n' "$*" >&2; }
fail() { printf '\033[31mERROR:\033[0m %s\n' "$*" >&2; exit 1; }

# ── Presentation ──────────────────────────────────────────────────────────────
# The closing banner is the only part of this script most people read, so it is
# drawn rather than printed. Styling is gated on stdout being a terminal: under
# `curl | sh` only *stdin* is the pipe, so this stays true for the piped install
# and goes quiet when the output is redirected to a file or a CI log.
if [ -t 1 ]; then
    B=$(printf '\033[1m'); D=$(printf '\033[2m'); R=$(printf '\033[0m')
    GRN=$(printf '\033[32m'); CYN=$(printf '\033[36m'); YEL=$(printf '\033[33m')
else
    B=''; D=''; R=''; GRN=''; CYN=''; YEL=''
fi

# Box drawing needs a UTF-8 locale to measure and to render; under LC_CTYPE=C
# the glyphs mojibake and character counts fall back to bytes, so both the
# charset and the padding degrade together to ASCII.
case "${LC_ALL:-${LC_CTYPE:-${LANG:-}}}" in
    *[Uu][Tt][Ff]8*|*[Uu][Tt][Ff]-8*)
        BOX_H='─'; BOX_V='│'; BOX_TL='┌'; BOX_TR='┐'; BOX_BL='└'; BOX_BR='┘'
        MARK_OK='✔'; MARK_BAD='✖' ;;
    *)
        BOX_H='-'; BOX_V='|'; BOX_TL='+'; BOX_TR='+'; BOX_BL='+'; BOX_BR='+'
        MARK_OK='*'; MARK_BAD='!' ;;
esac

BOX_W=62

# Rows are padded from a *character* count, never a byte count: the check mark
# and the box glyphs are multibyte, and hand-counted spaces silently go ragged
# the moment any of that text is edited.
box_rule() {
    _r=''
    _i=0
    while [ "$_i" -lt "$BOX_W" ]; do _r="$_r$BOX_H"; _i=$((_i + 1)); done
    printf '%s%s%s%s%s\n' "$1$B" "$2" "$_r" "$3" "$R"
}

# $1 = plain text, $2 = border colour, $3 = optional style for the text itself.
box_row() {
    _pad=$(printf "%$((BOX_W - $(printf '%s' "$1" | wc -m | tr -d ' ') ))s" '')
    printf '%s%s%s%s%s%s%s%s%s\n' \
        "$2$B" "$BOX_V" "$R${3:-}" "$1" "$R" "$_pad" "$2$B" "$BOX_V" "$R"
}

# ── Arguments ─────────────────────────────────────────────────────────────────
# The publish step (scripts/package-bridge-linux.sh) substitutes the instance's
# own download URL into DEFAULT_DOWNLOAD_BASE, so a piped install needs no
# arguments beyond the credential. Running the raw checkout copy leaves the
# placeholder in place, and then --download-base is required as before.
DEFAULT_DOWNLOAD_BASE="@DOWNLOAD_BASE@"
DOWNLOAD_BASE="${ASTOUND_DOWNLOAD_BASE:-}"
GATEWAY_URL="${ASTOUND_GATEWAY_URL:-}"
PAT="${ASTOUND_BRIDGE_PAT:-}"
CODE="${ASTOUND_BRIDGE_CODE:-}"
PUBKEY="${ASTOUND_BRIDGE_PUBKEY:-}"
# Which client applications to enrol: write each one's provider profile against
# this machine's proxy port and loopback secret. Left empty, the default below
# is worked out from what is on PATH. `sync` alone does NOT do this — it writes
# MCP connectors and no credential, and the client then gets 403 from the proxy.
HOSTS="${ASTOUND_HOSTS:-}"
SKIP_CLAUDE=0
while [ $# -gt 0 ]; do
    case "$1" in
        --download-base) DOWNLOAD_BASE="$2"; shift 2 ;;
        --gateway)       GATEWAY_URL="$2"; shift 2 ;;
        --pat)           PAT="$2"; shift 2 ;;
        --code)          CODE="$2"; shift 2 ;;
        --pubkey)        PUBKEY="$2"; shift 2 ;;
        --host)          HOSTS="${HOSTS:+$HOSTS,}$2"; shift 2 ;;
        --no-claude-code) SKIP_CLAUDE=1; shift ;;
        -h|--help)
            say "usage: install.sh --download-base URL [--gateway URL]"
            say "                  [--code <exchange-code> | --pat sp-live-...]"
            say "                  [--pubkey <base64>] [--host <id>] [--no-claude-code]"
            say ""
            say "With neither --pat nor --code you are signed in interactively"
            say "against your organisation's identity provider, and the token is"
            say "bound to the identity you sign in as. That is the normal case."
            say ""
            say "--pat and --code exist for unattended installs, and may also come"
            say "from \$ASTOUND_BRIDGE_PAT / \$ASTOUND_BRIDGE_CODE. An administrator"
            say "issues a code with:"
            say "    systemprompt admin bridge issue-code --user-id <uuid>"
            say "Codes are one-shot, short-lived, and assert an identity rather"
            say "than proving it."
            say ""
            say "--pubkey pins the manifest signing key out of band. Without it the"
            say "first sync trusts the key it is served (TOFU) and says so."
            say ""
            say "--host enrols a client application: it writes that client's provider"
            say "profile against this machine's proxy port and loopback secret."
            say "Repeatable, comma-separated, and also read from \$ASTOUND_HOSTS."
            say "Ids: claude-code, opencode. Default: claude-code, plus opencode"
            say "when the opencode binary is on PATH. Sync alone writes MCP servers"
            say "and no credential, which the proxy answers with 403."
            exit 0 ;;
        *) fail "unknown argument: $1" ;;
    esac
done
if [ -z "$DOWNLOAD_BASE" ]; then
    case "$DEFAULT_DOWNLOAD_BASE" in
        @*) fail "no download base — re-run with --download-base https://your-gateway/files/downloads" ;;
        *)  DOWNLOAD_BASE="$DEFAULT_DOWNLOAD_BASE" ;;
    esac
fi
DOWNLOAD_BASE="${DOWNLOAD_BASE%/}"
# The tarball lives under the gateway that serves this script, so the gateway
# URL is derivable: strip the /files/downloads suffix.
[ -n "$GATEWAY_URL" ] || GATEWAY_URL="${DOWNLOAD_BASE%/files/downloads}"

# ── Preconditions ─────────────────────────────────────────────────────────────
command -v curl >/dev/null 2>&1 || fail "curl is required"
command -v tar  >/dev/null 2>&1 || fail "tar is required"

# BSD and GNU disagree on the checksum tool, and busybox ships neither name.
if command -v sha256sum >/dev/null 2>&1; then
    sha256_of() { sha256sum "$1" | cut -d' ' -f1; }
elif command -v shasum >/dev/null 2>&1; then
    sha256_of() { shasum -a 256 "$1" | cut -d' ' -f1; }
else
    fail "no sha256sum or shasum available — cannot verify the download"
fi

ARCH="$(uname -m)"
case "$ARCH" in
    x86_64|amd64)   ASSET_ARCH="x86_64" ;;
    aarch64|arm64)  ASSET_ARCH="aarch64" ;;
    *) fail "unsupported architecture '$ARCH' — only x86_64 and aarch64 are published" ;;
esac
ASSET="${BIN_NAME}-linux-${ASSET_ARCH}.tar.gz"

if [ "$(id -u)" = "0" ]; then
    INSTALL_DIR="/usr/local/bin"
else
    INSTALL_DIR="$HOME/.local/bin"
fi

# ── Download + verify ─────────────────────────────────────────────────────────
WORK="$(mktemp -d)"
# shellcheck disable=SC2064  # expand WORK now: it is fixed for the run
trap "rm -rf '$WORK'" EXIT INT TERM

step "downloading $ASSET"
curl -fsSL -o "$WORK/$ASSET" "$DOWNLOAD_BASE/$ASSET" \
    || fail "download failed: $DOWNLOAD_BASE/$ASSET"
curl -fsSL -o "$WORK/$ASSET.sha256" "$DOWNLOAD_BASE/$ASSET.sha256" \
    || fail "checksum file missing: $DOWNLOAD_BASE/$ASSET.sha256"

step "verifying checksum"
EXPECTED="$(cut -d' ' -f1 < "$WORK/$ASSET.sha256")"
ACTUAL="$(sha256_of "$WORK/$ASSET")"
if [ -z "$EXPECTED" ] || [ "$EXPECTED" != "$ACTUAL" ]; then
    fail "checksum mismatch — refusing to install.
       expected $EXPECTED
       actual   $ACTUAL"
fi
say "    ok  $ACTUAL"

# ── Extract + install ─────────────────────────────────────────────────────────
step "installing to $INSTALL_DIR"
tar -xzf "$WORK/$ASSET" -C "$WORK" || fail "extract failed — the archive is corrupt"
EXTRACTED="$(find "$WORK" -type f -name "$BIN_NAME" -perm -u+x | head -n 1)"
[ -n "$EXTRACTED" ] || fail "the archive does not contain a $BIN_NAME executable"

mkdir -p "$INSTALL_DIR"
# Replace via a temp name + mv: overwriting a running binary in place fails with
# ETXTBSY, which is exactly the upgrade case.
cp "$EXTRACTED" "$INSTALL_DIR/.$BIN_NAME.new"
chmod 0755 "$INSTALL_DIR/.$BIN_NAME.new"
mv -f "$INSTALL_DIR/.$BIN_NAME.new" "$INSTALL_DIR/$BIN_NAME"
BRIDGE="$INSTALL_DIR/$BIN_NAME"
PATH="$INSTALL_DIR:$PATH"
export PATH
say "    $BRIDGE"

# ── Claude Code ───────────────────────────────────────────────────────────────
# Must precede `sync`: the marketplace emitter skips silently when the CLI is
# absent, leaving `claude plugin list` empty with everything else reporting fine.
install_claude_code() {
    if command -v claude >/dev/null 2>&1; then
        say "    already installed: $(command -v claude)"
        return 0
    fi
    if command -v npm >/dev/null 2>&1; then
        npm install -g @anthropic-ai/claude-code >/dev/null 2>&1 && return 0
        warn "npm install of @anthropic-ai/claude-code failed; trying the native installer"
    fi
    curl -fsSL https://claude.ai/install.sh | bash >/dev/null 2>&1 || return 1
    # The native installer drops the binary in ~/.local/bin.
    command -v claude >/dev/null 2>&1
}

if [ "$SKIP_CLAUDE" = "1" ]; then
    say "skipping Claude Code installation (--no-claude-code)"
else
    step "installing Claude Code"
    if install_claude_code; then
        say "    ok"
    else
        warn "could not install Claude Code automatically. Install it yourself
      (npm i -g @anthropic-ai/claude-code), then re-run '$BIN_NAME sync' — until
      then the org marketplace will not be registered."
    fi
fi

# ── Authenticate ──────────────────────────────────────────────────────────────
# Single sign-on is the default: the user authenticates against the org's
# identity provider and the token is bound to that identity. An admin-issued
# code or a pre-minted PAT are the non-interactive alternatives, for images and
# fleet provisioning where nobody is at the keyboard.
#
# Piped into sh, stdin is the script itself, so anything interactive must read
# the terminal directly. --no-browser prints the URL and takes the redirect back
# by paste, which is the only thing that works over SSH or in a container.
step "authenticating against $GATEWAY_URL"
if [ -n "$CODE" ]; then
    "$BRIDGE" login --code "$CODE" --gateway "$GATEWAY_URL" \
        || fail "could not redeem the code. They are one-shot and expire in minutes —
       ask your administrator for a fresh one:
       systemprompt admin bridge issue-code --user-id <your-uuid>"
elif [ -n "$PAT" ]; then
    "$BRIDGE" login "$PAT" --gateway "$GATEWAY_URL" \
        || fail "login failed — check the token and gateway URL"
elif [ -r /dev/tty ]; then
    say ""
    say "Signing you in. A browser will open against $GATEWAY_URL;"
    say "sign in there and approve the request for this machine."
    say ""
    if [ -n "${DISPLAY:-}${WAYLAND_DISPLAY:-}" ]; then
        "$BRIDGE" login --gateway "$GATEWAY_URL" < /dev/tty \
            || fail "sign-in failed or was not approved. Retry with:
       $BIN_NAME login --gateway $GATEWAY_URL"
    else
        # No display server, so no browser to launch: print the URL for the
        # user's own machine and take the redirect back by paste.
        "$BRIDGE" login --no-browser --gateway "$GATEWAY_URL" < /dev/tty \
            || fail "sign-in failed or was not approved. Retry with:
       $BIN_NAME login --no-browser --gateway $GATEWAY_URL"
    fi
else
    fail "no credential and no terminal to sign in from.
       Re-run with --pat sp-live-... or --code <exchange-code>, or run
       '$BIN_NAME login --no-browser --gateway $GATEWAY_URL' yourself."
fi

# ── Configure ─────────────────────────────────────────────────────────────────
# Schedule activation requires a working user bus. In a container or WSL
# without user systemd, configure the client and use the detached proxy below.
# The bridge reports a failed schedule activation as a nonzero install result.
SCHEDULE_FLAG=""
if command -v systemctl >/dev/null 2>&1 && systemctl --user daemon-reload >/dev/null 2>&1; then
    SCHEDULE_FLAG="--apply-schedule"
else
    say "    user systemd unavailable; using a session proxy without a sync timer"
fi
step "writing environment configuration"
if [ -n "$PUBKEY" ]; then
    "$BRIDGE" install --apply ${SCHEDULE_FLAG:+"$SCHEDULE_FLAG"} --gateway "$GATEWAY_URL" --pubkey "$PUBKEY" \
        || fail "install --apply failed"
else
    "$BRIDGE" install --apply ${SCHEDULE_FLAG:+"$SCHEDULE_FLAG"} --gateway "$GATEWAY_URL" \
        || fail "install --apply failed"
fi

# ── Proxy ─────────────────────────────────────────────────────────────────────
# The proxy mints the loopback key that env.sh reads, so it must be running
# before doctor — and before `claude` is any use. Where systemd took the unit,
# it is already up and this is a no-op; where it did not, start it detached.
proxy_listening() {
    "$BRIDGE" doctor 2>/dev/null | grep -q '^\[OK  \] inference proxy'
}

step "starting the loopback proxy"
if proxy_listening; then
    say "    already running"
else
    if command -v systemctl >/dev/null 2>&1 && systemctl --user daemon-reload >/dev/null 2>&1; then
        systemctl --user enable --now "${BIN_NAME}-proxy.service" >/dev/null 2>&1 || true
    fi
    if ! proxy_listening; then
        # No systemd here. nohup so it survives this script exiting; the proxy
        # single-instances on its own port, so a double start is harmless.
        nohup "$BRIDGE" proxy >"${TMPDIR:-/tmp}/${BIN_NAME}-proxy.log" 2>&1 &
        sleep 5
    fi
    if proxy_listening; then
        say "    listening on 127.0.0.1:48217"
    else
        warn "the proxy did not come up. Start it yourself with '$BIN_NAME proxy'
      and check ${TMPDIR:-/tmp}/${BIN_NAME}-proxy.log."
    fi
fi

# ── Sync ──────────────────────────────────────────────────────────────────────
# A virgin install has no pinned manifest signing key, so the first sync must
# either be given one (--pubkey, above) or trust the key it is served. Say which
# happened rather than silently trusting.
step "syncing plugins, skills, agents, and MCP servers"
if [ -n "$PUBKEY" ]; then
    "$BRIDGE" sync || warn "sync failed — run '$BIN_NAME sync' after resolving the error above"
else
    say "    no --pubkey given: trusting the manifest signing key served on first sync"
    "$BRIDGE" sync --allow-tofu \
        || warn "sync failed — run '$BIN_NAME sync --allow-tofu' after resolving the error above"
fi

# ── Enrol client applications ─────────────────────────────────────────────────
# This is the step whose absence produced "403 no loopback credential presented"
# in OpenCode: sync writes the MCP connectors into the user's opencode.json, but
# the provider block and the auth.json key are host-profile work, and nothing on
# this path ever asked for them. It runs after sync so the instance's enabled
# hosts are known, and after the proxy so the port and secret it writes are live.
if [ -z "$HOSTS" ]; then
    HOSTS="claude-code"
    command -v opencode >/dev/null 2>&1 && HOSTS="$HOSTS,opencode"
fi
step "enrolling client applications ($HOSTS)"
"$BRIDGE" install --gateway "$GATEWAY_URL" --host "$HOSTS" \
    || warn "host enrolment reported a problem. Re-run it on its own with
      '$BIN_NAME install --host $HOSTS'; until it succeeds that client reaches
      the proxy with no credential and is refused with 403."

# ── Verify ────────────────────────────────────────────────────────────────────
# doctor is the acceptance test: credentials, loopback secret, proxy, org
# marketplace, and filesystem layout. Exit 11 means at least one hard failure.
step "verifying the installation"
DOCTOR_RC=0
"$BRIDGE" doctor || DOCTOR_RC=$?

CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/astound"

# The optional device-certificate note is long, so it prints BEFORE the verdict.
# Printed after, it pushed the one command the user actually needs off-screen —
# which is precisely how a successful install came to look like a failed one.
if [ "$DOCTOR_RC" = "0" ]; then
    cat <<NEXT

${D}Unattended renewal (optional). A PAT is fine for a workstation; a long-lived
headless box wants a device certificate, which renews with no browser:

    mkdir -p $CONFIG_DIR
    openssl req -x509 -newkey rsa:2048 -nodes -days 730 \\
      -keyout $CONFIG_DIR/device.key \\
      -out    $CONFIG_DIR/device.pem -subj "/CN=\$(hostname)"
    openssl x509 -in $CONFIG_DIR/device.pem -outform der | sha256sum   # send to your admin

    printf '\\n[mtls]\\ncert_keystore_ref = "%s/device.pem"\\n' "$CONFIG_DIR" \\
      >> $CONFIG_DIR/astound-bridge.toml

Your administrator enrols the fingerprint with:

    systemprompt admin bridge enroll-cert --user-id <UUID> --fingerprint <sha256> --label <machine>${R}

NEXT
fi

# ── Verdict ───────────────────────────────────────────────────────────────────
# Last thing on screen, always. Success states the next command and nothing
# else; failure states the one command that diagnoses it.
say ""
if [ "$DOCTOR_RC" = "0" ] && proxy_listening; then
    box_rule "$GRN" "$BOX_TL" "$BOX_TR"
    box_row  ""                                              "$GRN"
    box_row  "   $MARK_OK  INSTALL SUCCESSFUL"               "$GRN" "$B$GRN"
    box_row  "      Astound Bridge is installed and running." "$GRN"
    box_row  ""                                              "$GRN"
    box_row  "   Start Claude Code:"                         "$GRN"
    box_row  ""                                              "$GRN"
    box_row  "       claude"                                 "$GRN" "$B$CYN"
    box_row  ""                                              "$GRN"
    box_rule "$GRN" "$BOX_BL" "$BOX_BR"
    say ""
    printf '%sNo new shell and nothing to source: Claude Code reads the settings\n' "$D"
    printf 'file written above on every run, in any terminal. env.sh additionally\n'
    printf 'exports the same values for other Anthropic-API clients.%s\n' "$R"
    say ""
    printf '%sgateway%s  %s\n' "$D" "$R" "${GATEWAY_URL:-$DOWNLOAD_BASE}"
    printf '%sconfig%s   %s/env.sh\n' "$D" "$R" "$CONFIG_DIR"
elif [ "$DOCTOR_RC" != "0" ]; then
    box_rule "$YEL" "$BOX_TL" "$BOX_TR"
    box_row  ""                                              "$YEL"
    box_row  "   $MARK_BAD  INSTALL INCOMPLETE"              "$YEL" "$B$YEL"
    box_row  "      doctor reported problems (exit $DOCTOR_RC)." "$YEL"
    box_row  ""                                              "$YEL"
    box_row  "   Fix the FAIL lines above, then re-check with:" "$YEL"
    box_row  ""                                              "$YEL"
    box_row  "       $BIN_NAME doctor"                       "$YEL" "$B$CYN"
    box_row  ""                                              "$YEL"
    box_rule "$YEL" "$BOX_BL" "$BOX_BR"
else
    box_rule "$YEL" "$BOX_TL" "$BOX_TR"
    box_row  ""                                              "$YEL"
    box_row  "   $MARK_BAD  INSTALL INCOMPLETE"              "$YEL" "$B$YEL"
    box_row  "      The proxy is not listening, so Claude Code" "$YEL"
    box_row  "      has nothing to talk to."                 "$YEL"
    box_row  ""                                              "$YEL"
    box_row  "   Start it, then re-check with:"              "$YEL"
    box_row  ""                                              "$YEL"
    box_row  "       $BIN_NAME proxy"                        "$YEL" "$B$CYN"
    box_row  "       $BIN_NAME doctor"                       "$YEL" "$B$CYN"
    box_row  ""                                              "$YEL"
    box_rule "$YEL" "$BOX_BL" "$BOX_BR"
fi
say ""

exit "$DOCTOR_RC"
