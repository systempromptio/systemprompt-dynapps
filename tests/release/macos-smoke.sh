#!/usr/bin/env bash
# Exercise a packaged bridge without enrolling or modifying the developer's
# clients. Run on Intel and Apple Silicon; no local compilation is permitted.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"

# install.sh is Linux-only: on Darwin it must stop and name the dmg rather than
# install the Linux aarch64 binary an arm64 Mac would otherwise map to. A PATH
# shim fakes `uname`, so this half runs on any host, and `--install-sh-only`
# runs nothing else (Linux CI has no hdiutil and no dmg to pass).
install_sh_refuses_darwin() {
    local work shim base out status
    work="$(mktemp -d)"
    shim="$work/shim"
    base="file://$work/downloads"
    mkdir -p "$shim" "$work/home"
    for os in Darwin Linux; do
        # shellcheck disable=SC2016  # $1 belongs to the shim, not to this shell
        printf '#!/bin/sh\ncase "$1" in -m) echo arm64 ;; *) echo %s ;; esac\n' "$os" > "$shim/uname"
        chmod +x "$shim/uname"
        status=0
        out="$(HOME="$work/home" PATH="$shim:$PATH" sh "$ROOT/scripts/install-bridge.sh" \
            --download-base "$base" --no-claude-code 2>&1 < /dev/null)" || status=$?
        if [ "$os" = Darwin ]; then
            [ "$status" -eq 1 ] || { echo "FAIL: install.sh on Darwin exited $status, want 1" >&2; exit 1; }
            grep -Fq "$base/dynapps-bridge-macos.dmg" <<< "$out" \
                || { echo "FAIL: install.sh on Darwin does not name the dmg: $out" >&2; exit 1; }
            if grep -Fq 'downloading' <<< "$out" || [ -e "$work/home/.local" ]; then
                echo "FAIL: install.sh on Darwin went on to download or install" >&2; exit 1
            fi
        else
            grep -Fq 'downloading dynapps-bridge-linux-aarch64.tar.gz' <<< "$out" \
                || { echo "FAIL: install.sh on Linux arm64 no longer reaches the aarch64 download: $out" >&2; exit 1; }
        fi
    done
    rm -rf "$work"
    echo "PASS: install.sh refuses Darwin and names the dmg; Linux arm64 still maps to aarch64"
}
install_sh_refuses_darwin
[ "${1:-}" != "--install-sh-only" ] || exit 0

DMG="${1:?usage: macos-smoke.sh downloaded.dmg expected-sha256}"
EXPECTED="${2:?expected SHA-256 is required}"
[[ "$EXPECTED" =~ ^[0-9a-f]{64}$ ]]
[ "$(shasum -a 256 "$DMG" | cut -d' ' -f1)" = "$EXPECTED" ]
bash "$ROOT/scripts/verify-bridge-macos.sh" "$DMG"
WORK="$(mktemp -d)"
MOUNT="$WORK/mounted"
cleanup() {
    if mount | grep -Fq " on $MOUNT ("; then hdiutil detach "$MOUNT" -quiet; fi
    rm -rf "$WORK"
}
trap cleanup EXIT
hdiutil attach "$DMG" -readonly -nobrowse -mountpoint "$MOUNT" -quiet
ditto "$MOUNT/DynappsBridge.app" "$WORK/DynappsBridge.app"
hdiutil detach "$MOUNT" -quiet
BIN="$WORK/DynappsBridge.app/Contents/MacOS/dynapps-bridge"
export DYNAPPS_BRIDGE_CONFIG="$WORK/config/dynapps-bridge.toml"
export XDG_CONFIG_HOME="$WORK/config" XDG_DATA_HOME="$WORK/data" XDG_CACHE_HOME="$WORK/cache"
unset DYNAPPS_BRIDGE_PAT
mkdir -p "$XDG_CONFIG_HOME" "$XDG_DATA_HOME" "$XDG_CACHE_HOME"
"$BIN" --version
"$BIN" --help > "$WORK/help.txt"
grep -Fq 'login' "$WORK/help.txt"

# Login requires a terminal even with --no-browser. Supply one with script,
# then an empty code, verifying endpoint selection without enrolling an account.
login_link() {
    local expected_url="$1"; shift
    if printf '\n' | script -q /dev/null "$BIN" login --no-browser "$@" > "$WORK/login.txt" 2>&1; then
        echo 'FAIL: an empty login unexpectedly succeeded' >&2
        exit 1
    fi
    grep -Fq "$expected_url/bridge-auth/device-link" "$WORK/login.txt"
    grep -Fq 'nothing pasted' "$WORK/login.txt"
}
login_link https://dynapps.es
login_link http://127.0.0.1:18081 --gateway http://127.0.0.1:18081
printf 'gateway_url = "http://127.0.0.1:18082"\n' > "$DYNAPPS_BRIDGE_CONFIG"
login_link http://127.0.0.1:18082
login_link http://127.0.0.1:18083 --gateway http://127.0.0.1:18083
grep -Fq 'http://127.0.0.1:18082' "$DYNAPPS_BRIDGE_CONFIG"
echo "PASS: $(uname -m) packaged CLI, Dynapps default, explicit and saved gateway overrides, empty-login rejection"
