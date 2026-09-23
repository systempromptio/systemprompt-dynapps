#!/usr/bin/env bash
# Assess the self-updater's zip exactly as the installed bridge will before it
# swaps the bundle in: ditto -x -k, then codesign --deep --strict and a
# Gatekeeper execute assessment (core bin/bridge/src/update/install/macos.rs).
set -euo pipefail
ZIP="${1:?usage: macos-updater-zip.sh astound-bridge-macos.zip}"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
/usr/bin/ditto -x -k "$ZIP" "$WORK"
APP="$WORK/AstoundBridge.app"
[ -d "$APP" ] || { echo "FAIL: $ZIP holds no AstoundBridge.app at its root" >&2; exit 1; }
/usr/bin/codesign --verify --deep --strict "$APP"
/usr/sbin/spctl --assess --type execute "$APP"
xcrun stapler validate "$APP"
"$APP/Contents/MacOS/astound-bridge" --version
echo "PASS: updater zip unpacks to a signed, notarized, stapled AstoundBridge.app"
