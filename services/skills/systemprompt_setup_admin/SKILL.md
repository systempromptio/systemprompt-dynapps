---
name: systemprompt-setup-admin
description: Install and reconcile Astound Admin dashboards in the Cowork Artifacts library.
---

# Set up Astound Admin dashboards

Adapted from systemprompt-internal's control-plane installer. This is the only skill
that installs dashboards. It is delivered only to Cowork. Use Read, Glob, Write,
list_artifacts, create_artifact, and MCP; no shell is needed.

1. Read `~/Systemprompt/systemprompt/artifacts/manifest.json` (macOS) or
   `%USERPROFILE%\Systemprompt\systemprompt\artifacts\manifest.json` (Windows),
   resolving the actual home directory from the connected workspace. On a miss,
   Glob inside that directory, then the connected Systemprompt folder. Only after
   discovery fails inspect whether the folder is unconnected, unsynced, or incomplete;
   report the observed state rather than claiming a single glob miss proves sync failed.
2. Read the manifest and call `list_artifacts`. Match by ID, never title. The manifest
   is authoritative: reconcile every record, without assuming a fixed count or plugin.
   Astound Admin currently supplies activity, costs, users, projects, and operations.
3. For each missing record call `create_artifact` sequentially, using its ID,
   description, `<manifest-directory>/<id>.html` as `html_path`, and `mcpTools`
   verbatim as `mcp_tools`. Include name/starred only if the tool schema supports them.
   Do not rewrite HTML, change tool allowlists, or write into the bridge bundle.
4. For stale versions or allowlist mismatches, preserve user edits and report the
   specific replacement needed. Replace when the user's request authorizes updating
   those dashboards; use the host's actual update/delete schema. Do not remove
   `admin-users-directory`: it remains supported here. Leave unrelated artifacts alone.
5. Verify with `list_artifacts`: each expected ID and tool allowlist must match.
   Probe `mcp__systemprompt__systemprompt` with `{"command":"core skills list"}`.
   Report explicit errors as returned; a generic failure alone does not establish
   an authorization or governance cause. Do not loop retries.
6. Write `outputs/setup-receipt.json` with checkedAt, bundled, installed, created,
   alreadyPresent, stale, and failed IDs. If file writing is unavailable, return the
   receipt inline. State N of M verified dashboards installed and the connection result.

The pages fetch structured reports directly. Setup neither fetches project data nor
requires Atlassian connection to install them. A project dashboard prompts for the
user's own connected site/project. No approval controls or Odoo dashboards ship here.
