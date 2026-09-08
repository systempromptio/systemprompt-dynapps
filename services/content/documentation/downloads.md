---
title: "Downloads"
description: "Download the Astound Bridge for Windows, macOS, and Linux from this gateway, verify the checksum, and run the Linux one-line installer."
author: "Astound Digital"
slug: "downloads"
keywords: "download, bridge, windows, macos, linux, aarch64, x86_64, install.sh, sha256, checksum, cosign, astound-bridge"
kind: "reference"
public: true
tags: ["documentation", "getting-started", "bridge", "downloads"]
published_at: "2026-09-02"
updated_at: "2026-09-02"
after_reading_this:
  - "Download the bridge for your platform from this gateway"
  - "Verify a download against its published checksum"
  - "Run the Linux one-line installer"
related_playbooks:
  - title: "Install the Desktop Bridge"
    url: "/documentation/bridge-install"
  - title: "Connect Claude Code"
    url: "/documentation/connect-claude-code"
  - title: "Authentication"
    url: "/documentation/authentication"
---

# Downloads

Every file below is served by this gateway. The same files are attached to
the matching GitHub Release of the platform, and the copy here is
byte-identical: the release pipeline bakes them into the image, so the
version you download always matches the gateway you connect to.

| Platform | File | Checksum |
|---|---|---|
| Windows x86_64 | [astound-bridge-windows.exe](/files/downloads/astound-bridge-windows.exe) | [.sha256](/files/downloads/astound-bridge-windows.exe.sha256) |
| macOS, universal (Apple Silicon and Intel) | [astound-bridge-macos.dmg](/files/downloads/astound-bridge-macos.dmg) | [.sha256](/files/downloads/astound-bridge-macos.dmg.sha256) |
| Linux x86_64 (glibc 2.35 or newer) | [astound-bridge-linux-x86_64.tar.gz](/files/downloads/astound-bridge-linux-x86_64.tar.gz) | [.sha256](/files/downloads/astound-bridge-linux-x86_64.tar.gz.sha256) |
| Linux aarch64 (glibc 2.35 or newer) | [astound-bridge-linux-aarch64.tar.gz](/files/downloads/astound-bridge-linux-aarch64.tar.gz) | [.sha256](/files/downloads/astound-bridge-linux-aarch64.tar.gz.sha256) |
| Linux installer | [install.sh](/files/downloads/install.sh) | — |

## Linux: one command

The installer downloads the right tarball for your CPU, verifies it, installs
the bridge, links it to your account, and configures Claude Code. Get a code
from your **Profile** page after signing in, then:

```bash
curl -fsSL https://gateway.example.com/files/downloads/install.sh | sh -s -- \
  --download-base https://gateway.example.com/files/downloads --code <code>
```

Replace `gateway.example.com` with this gateway's hostname. The full
walk-through, including WSL, is in
[Install the Desktop Bridge](/documentation/bridge-install).

## Windows and macOS

Download the file, verify it, run it, and approve the device link in your
browser. Step by step: [Install the Desktop Bridge](/documentation/bridge-install).

Until the binaries carry an OS code-signing certificate, the first launch is
challenged: Windows SmartScreen shows *More info → Run anyway*; macOS
Gatekeeper blocks the first open (right-click → Open, or
`xattr -d com.apple.quarantine /Applications/AstoundBridge.app`). Verify the
checksum before you accept either prompt.

## Verify a download

Each binary has a `.sha256` file beside it. Compare before running:

```bash
# Linux
sha256sum -c astound-bridge-linux-x86_64.tar.gz.sha256
# macOS
shasum -a 256 -c astound-bridge-macos.dmg.sha256
# Windows (PowerShell)
(Get-FileHash .\astound-bridge-windows.exe -Algorithm SHA256).Hash
```

Assets published by the release pipeline are also signed with Sigstore, and
carry `.sig` and `.pem` files alongside them in the GitHub Release.

**The 0.44.0 assets are an exception and carry no signatures.** GitHub Actions <!-- pinned-release -->
is unavailable on this account, so the pipeline did not run and the binaries
were built and published by hand; the release holds only the `.sha256`
sidecars above. Use the checksum verification above for 0.44.0 — it is the <!-- pinned-release -->
whole of what is available for this release, and it does confirm the bytes
served match the bytes published. Signing resumes when the pipeline does.

Where `.sig` and `.pem` files are present, verify with
[cosign](https://docs.sigstore.dev/):

```bash
cosign verify-blob --signature astound-bridge-windows.exe.sig \
  --certificate astound-bridge-windows.exe.pem \
  --certificate-identity-regexp='https://github.com/Astound-Digital/systemprompt-astound/' \
  --certificate-oidc-issuer=https://token.actions.githubusercontent.com \
  astound-bridge-windows.exe
```
