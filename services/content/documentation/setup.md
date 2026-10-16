---
title: "Set Up Dynapps on Your Laptop"
description: "The business-user setup: install Dynapps Bridge on Windows or Mac, sign in with your Microsoft account, connect Dynapps Odoo and DynBrain, and open Claude Cowork with the Dynapps skills and tools."
author: "Dynapps"
slug: "setup"
kind: "guide"
public: true
tags: ["documentation", "getting-started", "bridge"]
published_at: "2026-10-16"
updated_at: "2026-10-16"
keywords: "setup, install, dynapps bridge, cowork, claude desktop, windows, mac, odoo, dynbrain, connectors, business users"
after_reading_this:
  - "Install and open Dynapps Bridge on Windows or Mac"
  - "Sign in, connect Dynapps Odoo and DynBrain, and sync your skills"
  - "Open Claude Cowork with the Dynapps skills and tools ready"
related_playbooks:
  - title: "Use Your Skills and Run Them on the Agent Farm"
    url: "/documentation/run-skills"
  - title: "Downloads"
    url: "/documentation/downloads"
  - title: "Connect Cowork"
    url: "/documentation/connect-cowork"
---

# Set Up Dynapps on Your Laptop

About 10 minutes. You need admin rights on your computer and your Dynapps
Microsoft account. When you finish, Claude Cowork has the Dynapps skills and
your Dynapps Odoo and DynBrain tools.

> **Before you start: check your access.** Ask your Dynapps administrator to
> confirm your account is in the **staff** group (or has an admin role).
> Without it you can sign in, but Cowork won't show the Dynapps skills and
> tools. Every user must also connect Dynapps Odoo and DynBrain (step 6 on
> Windows, step 7 on Mac) before syncing.

Jump to: [Windows](#windows) · [Mac](#mac) · [If something doesn't look right](#if-something-doesnt-look-right)

## Windows

### 1. Turn on Virtual Machine Platform (required)

Claude Cowork needs this Windows feature, and it is often switched off. Do this
first, because it needs a restart.

Press the **Windows key**, type **Turn Windows features on or off** and open
it. Tick **Virtual Machine Platform** and select **OK**. When Windows asks,
select **Restart now**.

If the box is already ticked, skip this step. If you can't change it, ask IT to
enable Virtual Machine Platform for you.

### 2. Download the app

Go to [dynapps.systemprompt.io](https://dynapps.systemprompt.io/) and select
**Download for Windows**. The file is called `dynapps-bridge-windows.exe`. It
is also listed on the [Downloads](/documentation/downloads) page.

If Edge says the file "isn't commonly downloaded", open the menu (…) next to
the download and select **Keep**, then **Keep anyway**.

### 3. Trust the app and run it anyway

The Windows app is **not yet signed**, so Windows will warn you. This is
expected.

Right-click the file and choose **Run as administrator**. On the blue
**"Windows protected your PC"** screen, select **More info**, then **Run
anyway**. When Windows asks whether to allow an app from an unknown publisher
to make changes, select **Yes**.

This approves only this one file. Leave Windows security switched on, and only
trust the file you downloaded from dynapps.systemprompt.io.

### 4. Always open it as administrator

Whenever you start Dynapps Bridge, right-click it, choose **Run as
administrator** and select **Yes**. Admin rights let the bridge set up Claude
for you. If you get no Yes button, ask IT for a one-off admin approval.

### 5. Sign in with your Microsoft account

Select **Sign in**. Your browser opens the Dynapps sign-in, which sends you to
Microsoft. Use your normal Dynapps work account, then go back to the app. It
shows you as signed in.

### 6. Connect Dynapps Odoo and DynBrain (required)

Do this **before** you use or sync anything. In your browser, go to
[dynapps.systemprompt.io/admin/connectors](https://dynapps.systemprompt.io/admin/connectors).

Select **Connect** on the **Dynapps Odoo** card and sign in with your
portal.dynapps.es account. Do the same on the **DynBrain** card. Continue until
every card shows a green **Connected** badge.

If a card says "Needs attention", select **Reconnect**. Use **Test connection**
to check a tool works.

### 7. Sync from the Marketplace tab, then open Claude Cowork

Go back to Dynapps Bridge and open the **Marketplace** tab. Sync from there so
the bridge picks up your connected tools and skills.

Fully quit Claude Desktop if it is open, including from the system tray. Open
it again and go to **Cowork**. The Dynapps skills and tools appear, and you are
ready. Next: [use your skills](/documentation/run-skills).

## Mac

### 1. Download the app

Go to [dynapps.systemprompt.io](https://dynapps.systemprompt.io/) and select
**Download for macOS**. The file is called `dynapps-bridge-macos.dmg`. It works
on all Macs.

**Always download a fresh copy of the latest version.** Older copies may not
work with Cowork. If you installed Dynapps Bridge before, quit it, download it
again and replace the old app in Applications when asked.

### 2. Install it

Open the downloaded file and drag **Dynapps Bridge** into the **Applications**
folder. Then eject the disk image from Finder's sidebar.

### 3. Open it past the security prompt

Open Dynapps Bridge from Applications. If macOS asks whether to open an app
downloaded from the internet, select **Open**.

If it says the app "cannot be opened", go to **System Settings → Privacy &
Security**, scroll down, select **Open Anyway** and enter your Mac password.
This only approves this one app. Leave Mac security settings switched on.

### 4. Approve admin access

The bridge asks for your Mac password so it can set up Claude. Enter it.

### 5. Sign in with your Microsoft account

Select **Sign in**. Your browser opens the Dynapps sign-in, which sends you to
Microsoft. Use your normal Dynapps work account, then go back to the app. It
shows you as signed in.

### 6. Install the Claude profile by hand (required)

After you sign in, the bridge downloads a settings **profile** for Claude.
macOS does **not** install it for you, and Cowork won't connect to Dynapps
until you do.

Open **System Settings → General → Device Management** (on some Macs,
**Privacy & Security → Profiles**). Under **Downloaded**, double-click the
Dynapps / Claude profile, select **Install** and enter your Mac password.

Opening the profile is not the same as installing it. Once installed, it moves
out of Downloaded. If you can't find it, sign in to the bridge again to
download it again.

### 7. Connect Dynapps Odoo and DynBrain (required)

Do this **before** you use or sync anything. In your browser, go to
[dynapps.systemprompt.io/admin/connectors](https://dynapps.systemprompt.io/admin/connectors).

Select **Connect** on the **Dynapps Odoo** card and sign in with your
portal.dynapps.es account. Do the same on the **DynBrain** card. Continue until
every card shows a green **Connected** badge.

If a card says "Needs attention", select **Reconnect**. Use **Test connection**
to check a tool works.

### 8. Sync from the Marketplace tab, then open Claude Cowork

Go back to Dynapps Bridge and open the **Marketplace** tab. Sync from there so
the bridge picks up your connected tools and skills.

Fully quit Claude Desktop with **⌘Q**, open it again and go to **Cowork**. The
Dynapps skills and tools appear, and you are ready. Next:
[use your skills](/documentation/run-skills).

## If something doesn't look right

| What you see | What to do |
|---|---|
| Cowork shows no Dynapps skills or tools | Check that both cards on the Connectors page say Connected, then sync again from the **Marketplace** tab in Dynapps Bridge |
| Still missing after that | Quit Claude fully and reopen it. If still missing, ask your Dynapps administrator to check you are in the **staff** group |
| Cowork on Mac shows no Dynapps tools | Make sure you downloaded the latest Dynapps Bridge, then check the Claude profile is installed (System Settings → General → Device Management) |
| Odoo asks you to confirm every action | Expected: Dynapps Odoo confirms each tool call before it runs |
| The sign-in page won't load | Check you are online and not on a guest network, then select Sign in again. You can also sign in at [dynapps.systemprompt.io/admin/login](https://dynapps.systemprompt.io/admin/login) |
| Cowork won't start on Windows | Check Virtual Machine Platform is turned on (Windows step 1) and that you restarted |
| Windows won't let you run as administrator | Ask IT for a one-off admin approval for Dynapps Bridge |
| Security software blocks the file completely | Stop and contact IT. Don't turn off antivirus or security settings |

Using Claude Code in a terminal instead of Cowork? Follow
[Connect Claude Code](/documentation/connect-claude-code). For checksums and
the technical bridge reference, see [Downloads](/documentation/downloads) and
[Install the Desktop Bridge](/documentation/bridge-install).
