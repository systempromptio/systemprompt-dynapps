---
title: "Install on Windows"
description: "Install Astound Bridge for Windows Cowork, or set up Claude Code and its Linux bridge inside WSL."
author: "Astound Digital"
slug: "install-windows"
kind: "guide"
public: true
tags: ["documentation", "getting-started", "bridge"]
published_at: "2026-09-09"
updated_at: "2026-09-09"
related_playbooks:
  - title: "Connect Claude Code"
    url: "/documentation/connect-claude-code"
  - title: "Connect Cowork"
    url: "/documentation/connect-cowork"
  - title: "Downloads"
    url: "/documentation/downloads"
---

# Install on Windows

Choose the setup that matches your client:

| Client | Where it runs | Bridge to install |
|---|---|---|
| Claude Cowork in Claude Desktop | Windows | Native Windows executable |
| Claude Code in a terminal | Inside WSL | Linux bridge inside the same WSL distribution |

Windows Desktop policy configures Claude Desktop, not native Windows Claude Code inference. Installing the Windows bridge does not create the Unix Claude Code settings file.

## Cowork on Windows

### 1. Download and verify

Download the [Windows executable](/files/downloads/astound-bridge-windows.exe) and its [checksum](/files/downloads/astound-bridge-windows.exe.sha256) from your gateway. Follow the PowerShell comparison in [Downloads](/documentation/downloads#windows-powershell) before running it.

Use a stable folder for the executable, following your organization's software-installation policy. Do not move or delete an executable that a registered background job uses. In PowerShell, set the full path to your verified copy:

```powershell
$Bridge = "C:\Users\YOUR_USERNAME\Downloads\astound-bridge-windows.exe"
& $Bridge --version
```

Replace the example path if you saved it elsewhere. If Windows or your organization's endpoint policy blocks the executable, ask IT to approve the verified package. Do not disable SmartScreen or antivirus protection.

### 2. Configure your gateway and sign in

Use the HTTPS address and trusted manifest public key supplied by your administrator:

```powershell
$GatewayUrl = "https://gateway.example.com"
& $Bridge install --gateway $GatewayUrl --pubkey "ADMINISTRATOR_PROVIDED_BASE64_PUBLIC_KEY"
& $Bridge login --gateway $GatewayUrl
```

If IT already provisioned the trusted key, do not replace it. Approve the browser's device link with your work account.

For code-based sign-in, open your gateway Profile, click **Generate a connect code**, then run:

```powershell
& $Bridge login --gateway $GatewayUrl --code "YOUR_CONNECT_CODE"
```

The code is single-use and expires after ten minutes. Generate another if necessary; reloading Profile does not create one.

### 3. Connect Claude Desktop

Install your organization's approved Claude Desktop app, then follow [Connect Cowork](/documentation/connect-cowork). Open your verified Astound Bridge executable from File Explorer and keep the app running while using Cowork. Approve expected Windows elevation prompts only after checking their source and purpose; Desktop policy installation can require administrator rights.

The default bridge configuration directory is `%APPDATA%\astound\`. Paste that path into File Explorer's address bar to locate it, or run `& $Bridge status` for the actual location. It contains credentials; do not share its contents.

## Claude Code in WSL

Install or open a WSL distribution approved by your organization. See [Microsoft's WSL installation guide](https://learn.microsoft.com/en-us/windows/wsl/install) if WSL is not available.

Inside the WSL terminal:

1. Follow [Linux and WSL bridge installation](/documentation/bridge-install#linux-and-wsl).
2. Confirm `astound-bridge --version` and `claude --version` both run inside WSL.
3. Complete [Connect Claude Code](/documentation/connect-claude-code), including enrolment and your choice of routing mode.
4. Keep the WSL proxy or its systemd user service running.
5. Send a real request and confirm it in gateway History.

The default standalone Claude Code file is `~/.config/astound/claude-code-settings.json` **inside WSL**. It is created during Claude Code enrolment, not by installing the Windows executable. Use the Linux command examples from the Claude Code guide, including for Gemini.

Windows and WSL have separate bridge credentials, settings, and process lifecycles. A running Windows bridge does not replace the WSL proxy, and a WSL installation does not configure native Windows Cowork.
