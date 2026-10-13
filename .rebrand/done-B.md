# done-B (bridge client)

## Files changed
- bridge/: Cargo.toml, .cargo/config.toml, build.rs, src/main.rs (Brand: Dynapps Bridge / dynapps-bridge / config dir dynapps / env DYNAPPS_BRIDGE / com.dynapps.bridge[-sync|-gui] / default gateway https://dynapps.systemprompt.io / sign-in hint "Microsoft Entra ID"), src/registry.rs, macos/Info.plist, scripts/make-mac-app.sh (DynappsBridge.app), scripts/render-icons.py, README.md, assets/{icon.svg,logo.svg,theme.css,app-icon.ico,tray-icon.png,window-icon-1024.png}, web/css/{brand-overrides,setup}.css
- extensions/web/shared/admin-core/src/bridge_client.rs (BRIDGE_BINARY = dynapps-bridge)
- services/ai/gateway.yaml lines 155-163 only (repo systempromptio/systemprompt-dynapps, assets dynapps-bridge-*, token_secret DYNAPPS_RELEASE_TOKEN)
- scripts/package-bridge-{linux,macos,windows}.sh, install-bridge.sh, clean-client-install.sh, verify-bridge-macos.sh, repair-release.sh, lib/e2e-client.sh
- deploy/clean-client/** ; tests/unit/bridge/**, tests/bridge/*.test.mjs, tests/release/**, tests/contract/admin/src/bridge_device_link_contract.rs, tests/unit/web/src/account_pages.rs
- Re-grep of the whole set for "astound": zero (excluding Cargo.lock/CORE_REF).

## Icons
cairosvg / rsvg are not installed. brand/icon.svg wraps one embedded 512px PNG, so render-icons.py now decodes that PNG with Pillow and resizes (cairosvg only used if installed AND no embedded image). window-icon-1024.png is therefore a 2x LANCZOS upscale of the 512 source (slightly soft; replace with a vector master if Dynapps supplies one). ICO has 16/32/48/256 frames. logo.svg = brand/dynapps-wordmark.svg with currentColor text set to #FAFAFA for the dark theme.

## Theme
Accent #F13C42, secondary #951CF3 (--sp-secondary, info chips #B66BF7), --sp-brand-gradient #7B00DC->#F13C42->#FF911F used as a top rule on the sign-in panel. Fonts: core bundles fonts locally (fonts.css), no network loads; "Readex Pro" leads the stack (used if installed) with system sans fallback. No font file bundled.

## Notes / deviations
- token_secret kept upper-case to match the existing convention (DYNAPPS_RELEASE_TOKEN), not the lower-case name in the task message. Secret must exist under that name.
- storage/files/downloads/ does not exist in this tree; nothing to git rm.
- tests/unit/bridge/src/desktop_policy.rs expects deploymentDisplayName "Dynapps"; services/external_agents/claude_desktop.yaml still says "Astound Digital" (agent S owns it).

## Blockers (requirements)
- Signing: new identifiers com.dynapps.bridge* need an Apple Developer ID cert + notarization profile and a Windows Authenticode cert for "Dynapps"; release secrets DYNAPPS_BRIDGE_* and DYNAPPS_RELEASE_TOKEN must be created in systempromptio/systemprompt-dynapps. Existing Astound certs/keys must not be reused.
- Bridge binaries must be rebuilt and released before install.sh/update assets resolve.
