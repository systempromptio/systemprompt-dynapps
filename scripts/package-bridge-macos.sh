#!/usr/bin/env bash
# Build, sign, notarize, and stage the macOS bridge as the single-file DMG the
# homepage and the admin Bridge Setup page link to
# (storage/files/downloads/dynapps-bridge-macos.dmg), plus the zipped .app the
# in-app self-updater installs (dynapps-bridge-macos.zip). The updater cannot
# consume the DMG: it unpacks with `ditto -x -k` and assesses the bundle with
# codesign + spctl before swapping it in (core bin/bridge/src/update/install/macos.rs).
#
# Must run on macOS: the build needs the Apple toolchain, the .app wrap uses
# sips/iconutil (bridge/scripts/make-mac-app.sh), and signing needs a
# "Developer ID Application" identity in the login keychain.
#
# The binary is universal (arm64 + x86_64 via lipo) so one DMG serves both
# Apple Silicon and Intel Macs — which is why the asset has no arch suffix.
#
# Notarization is REQUIRED by default: a Developer-ID-signed but un-notarized
# app downloaded from the web is still blocked by Gatekeeper, so shipping one
# would look identical to shipping nothing. Store credentials once with:
#
#     xcrun notarytool store-credentials dynapps-bridge \
#         --apple-id <apple-id> --team-id <team-id> \
#         --password <app-specific-password>
#
# and this script picks the profile up by name (override with NOTARY_PROFILE).
# SKIP_NOTARIZE=1 exists for local smoke builds only — the staged DMG is then
# not shippable and the script says so loudly.
#
# Two CI-facing modes (.github/workflows/release.yml):
#   UNSIGNED=1   — local smoke build only; never stages a release asset.
#   NOTARY_KEY + NOTARY_KEY_ID + NOTARY_ISSUER — App Store Connect API key
#                  auth for notarytool, instead of a keychain profile; what a
#                  runner can hold as secrets.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BRIDGE_DIR="$REPO_ROOT/bridge"
DOWNLOADS_DIR="$REPO_ROOT/storage/files/downloads"
ASSET="dynapps-bridge-macos.dmg"
UPDATER_ASSET="dynapps-bridge-macos.zip"
NOTARY_PROFILE="${NOTARY_PROFILE:-dynapps-bridge}"
TARGETS=(aarch64-apple-darwin x86_64-apple-darwin)
UNIVERSAL="universal-apple-darwin"

[ "$(uname -s)" = "Darwin" ] || {
    echo "ERROR: must run on macOS (codesign/notarytool/hdiutil)." >&2; exit 1; }

for t in "${TARGETS[@]}"; do
    rustup target list --installed | grep -qx "$t" || {
        echo "ERROR: rust target missing — run: rustup target add $t" >&2; exit 1; }
done

UNSIGNED="${UNSIGNED:-0}"
if [ "$UNSIGNED" = "1" ]; then
    CODESIGN_IDENTITY="-"
    SKIP_NOTARIZE=1
    NOTARY_KEY=""
    echo "==> UNSIGNED=1: local smoke build only; no release asset will be staged." >&2
fi

# ── Signing identity ──────────────────────────────────────────────────────────
# Auto-detect the Developer ID cert; CODESIGN_IDENTITY overrides when the
# keychain holds more than one.
if [ -z "${CODESIGN_IDENTITY:-}" ]; then
    CODESIGN_IDENTITY="$(security find-identity -v -p codesigning \
        | sed -n 's/.*"\(Developer ID Application: [^"]*\)".*/\1/p' | head -1)"
    [ -n "$CODESIGN_IDENTITY" ] || {
        echo "ERROR: no 'Developer ID Application' identity in the keychain and" >&2
        echo "CODESIGN_IDENTITY unset. Install the cert or set the variable." >&2
        exit 1
    }
fi

# ── Notary credentials — checked BEFORE the (slow) build, not after ───────────
# API-key auth when NOTARY_KEY is set, keychain profile otherwise. (Plain
# array assignment: macOS ships bash 3.2, which has no mapfile.)
if [ -n "${NOTARY_KEY:-}" ]; then
    NOTARY=(--key "$NOTARY_KEY" --key-id "${NOTARY_KEY_ID:?NOTARY_KEY_ID required with NOTARY_KEY}" \
            --issuer "${NOTARY_ISSUER:?NOTARY_ISSUER required with NOTARY_KEY}")
else
    NOTARY=(--keychain-profile "$NOTARY_PROFILE")
fi
if [ "${SKIP_NOTARIZE:-0}" != "1" ]; then
    xcrun notarytool history "${NOTARY[@]}" >/dev/null 2>&1 || {
        echo "ERROR: no notarytool credential profile '$NOTARY_PROFILE'." >&2
        echo "Store one (once) with:" >&2
        echo "    xcrun notarytool store-credentials $NOTARY_PROFILE \\" >&2
        echo "        --apple-id <apple-id> --team-id <team-id> --password <app-specific-password>" >&2
        echo "or set NOTARY_PROFILE, or SKIP_NOTARIZE=1 for a local-only build." >&2
        exit 1
    }
fi

# ── Build both slices and fuse them ───────────────────────────────────────────
if [ "${SKIP_BUILD:-0}" != "1" ]; then
    for t in "${TARGETS[@]}"; do
        echo "==> building dynapps-bridge for $t"
        (cd "$BRIDGE_DIR" && cargo build --locked --release --target "$t")
    done
fi
for t in "${TARGETS[@]}"; do
    [ -f "$BRIDGE_DIR/target/$t/release/dynapps-bridge" ] || {
        echo "ERROR: missing $t binary. Run without SKIP_BUILD=1." >&2; exit 1; }
done

UNI_DIR="$BRIDGE_DIR/target/$UNIVERSAL/release"
mkdir -p "$UNI_DIR"
lipo -create -output "$UNI_DIR/dynapps-bridge" \
    "$BRIDGE_DIR/target/aarch64-apple-darwin/release/dynapps-bridge" \
    "$BRIDGE_DIR/target/x86_64-apple-darwin/release/dynapps-bridge"
lipo -archs "$UNI_DIR/dynapps-bridge" | grep -q 'x86_64.*arm64\|arm64.*x86_64' || {
    echo "ERROR: lipo output is not universal:" >&2
    lipo -archs "$UNI_DIR/dynapps-bridge" >&2
    exit 1
}

# ── Wrap, sign, verify ────────────────────────────────────────────────────────
"$BRIDGE_DIR/scripts/make-mac-app.sh" --target "$UNIVERSAL"
APP="$UNI_DIR/DynappsBridge.app"

echo "==> signing with: $CODESIGN_IDENTITY"
# Hardened runtime is a notarization requirement. Inside-out: binary, then
# bundle. No entitlements needed — WKWebView JITs in its own XPC process.
# An ad-hoc identity ("-") cannot be timestamped, so the flag is dropped there.
SIGN_FLAGS=(--force --options runtime)
[ "$CODESIGN_IDENTITY" = "-" ] || SIGN_FLAGS+=(--timestamp)
codesign "${SIGN_FLAGS[@]}" --sign "$CODESIGN_IDENTITY" "$APP/Contents/MacOS/dynapps-bridge"
codesign "${SIGN_FLAGS[@]}" --sign "$CODESIGN_IDENTITY" "$APP"
codesign --verify --strict --deep "$APP"

# Notarize the app itself before imaging it: its ticket must survive copying
# to Applications and ejecting the DMG, including an offline first launch.
if [ "${SKIP_NOTARIZE:-0}" != "1" ]; then
    ditto -c -k --keepParent "$APP" "$UNI_DIR/DynappsBridge.app.zip"
    xcrun notarytool submit "$UNI_DIR/DynappsBridge.app.zip" "${NOTARY[@]}" --wait --timeout 30m
    xcrun stapler staple "$APP"
    xcrun stapler validate "$APP"
    spctl --assess --type exec --verbose=4 "$APP"
fi

# ── DMG ───────────────────────────────────────────────────────────────────────
STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT
ditto "$APP" "$STAGE/DynappsBridge.app"
ln -s /Applications "$STAGE/Applications"
DMG="$BRIDGE_DIR/target/$UNIVERSAL/release/$ASSET"
rm -f "$DMG"
if ! hdiutil create -volname "Dynapps Bridge" -srcfolder "$STAGE" \
    -format UDZO -ov "$DMG"; then
    # macOS privacy controls can deny create's intermediate /Volumes mount.
    # Mount a blank writable image at a private temporary path, then copy
    # the signed files and compress it. makehybrid adds FinderInfo to every
    # entry, invalidating the app's strict signature verification.
    # The signed-app, staple and copy-after-eject checks below still apply.
    echo "==> direct image creation failed; trying a private temporary mount"
    (
        IMAGE_WORK="$(mktemp -d)"
        IMAGE_MOUNT="$IMAGE_WORK/mounted"
        cleanup_image() {
            if mount | grep -Fq " on $IMAGE_MOUNT ("; then
                hdiutil detach "$IMAGE_MOUNT" -quiet || return
            fi
            rm -rf "$IMAGE_WORK"
        }
        trap cleanup_image EXIT
        SIZE_KB="$(du -sk "$STAGE" | awk '{print $1 + 65536}')"
        hdiutil create -size "${SIZE_KB}k" -fs HFS+ \
            -volname "Dynapps Bridge" "$IMAGE_WORK/writable.dmg"
        hdiutil attach "$IMAGE_WORK/writable.dmg" -nobrowse \
            -mountpoint "$IMAGE_MOUNT" -quiet
        ditto "$STAGE" "$IMAGE_MOUNT"
        codesign --verify --strict --deep "$IMAGE_MOUNT/DynappsBridge.app"
        hdiutil detach "$IMAGE_MOUNT" -quiet
        hdiutil convert "$IMAGE_WORK/writable.dmg" -format UDZO -o "$DMG" -ov
    )
fi
DMG_FLAGS=(--force)
[ "$CODESIGN_IDENTITY" = "-" ] || DMG_FLAGS+=(--timestamp)
codesign "${DMG_FLAGS[@]}" --sign "$CODESIGN_IDENTITY" "$DMG"

# ── Notarize + staple ─────────────────────────────────────────────────────────
if [ "${SKIP_NOTARIZE:-0}" = "1" ]; then
    # Deliberately NOT staged: a DMG built with notarization skipped must
    # never be the one `just deploy` picks up and ships, since Gatekeeper
    # blocks it on every other Mac.
    echo "==> SKIP_NOTARIZE=1 — built and signed, but NOT staged for deploy:" >&2
    echo "    $DMG" >&2
    # A nonzero result prevents the coordinator caching this as shippable.
    exit 2
else
    echo "==> notarizing (waits for Apple; typically 1-5 minutes)"
    xcrun notarytool submit "$DMG" "${NOTARY[@]}" --wait --timeout 30m
    xcrun stapler staple "$DMG"
    # The end-to-end check: would Gatekeeper accept this exact download?
    spctl -a -t open --context context:primary-signature "$DMG"
fi

bash "$REPO_ROOT/scripts/verify-bridge-macos.sh" "$DMG"

# ── Stage ─────────────────────────────────────────────────────────────────────
mkdir -p "$DOWNLOADS_DIR"
install -m 0644 "$DMG" "$DOWNLOADS_DIR/$ASSET"
# shasum on macOS, sha256sum on Linux CI — output format is identical.
(cd "$DOWNLOADS_DIR" && shasum -a 256 "$ASSET" > "$ASSET.sha256")

# The stapled bundle, zipped with ditto so the symlinks, xattrs and ticket the
# signature covers survive; a plain `zip` strips them and fails Gatekeeper.
rm -f "$DOWNLOADS_DIR/$UPDATER_ASSET"
ditto -c -k --keepParent "$APP" "$DOWNLOADS_DIR/$UPDATER_ASSET"
chmod 0644 "$DOWNLOADS_DIR/$UPDATER_ASSET"
(cd "$DOWNLOADS_DIR" && shasum -a 256 "$UPDATER_ASSET" > "$UPDATER_ASSET.sha256")
bash "$REPO_ROOT/tests/release/macos-updater-zip.sh" "$DOWNLOADS_DIR/$UPDATER_ASSET"

echo "==> staged $DOWNLOADS_DIR/$ASSET and $DOWNLOADS_DIR/$UPDATER_ASSET"
cat "$DOWNLOADS_DIR/$ASSET.sha256" "$DOWNLOADS_DIR/$UPDATER_ASSET.sha256"
echo "Served from /files/downloads at runtime; cloud deploy bakes storage/ into the image."
