# Install Sales Artifacts

The bridge stages every library artifact this user is entitled to into the
Astound workspace folder as a bundle; Cowork's library only ingests through
its own `create_artifact` tool. This skill is the one-time hand-off between
the two. Run it when a report skill says the *UK Client Report* artifact is
missing, or when the bridge has synced a newer version.

## Use when

- "Install the sales dashboards."
- "The UK Client Report isn't in my library."
- A report skill's hand-off step pointed here.

## Where the bundle is

`<home>/Astound/systemprompt/artifacts/` — the bridge names the workspace
after the brand, and Cowork connects that folder by default. Inside:

- `manifest.json` — one record per artifact: `id`, `name`, `description`,
  `version`, `mcp_tools`, and the file name;
- `<id>.html` — the page itself, for example `uk-client-report.html`.

If the folder or `manifest.json` is missing, the bridge has not synced on this
machine yet: tell the user to open the Astound bridge and sync, then stop. Do
not build a page by hand and do not fetch one from anywhere else.

## Workflow

1. **Read the manifest** with file tools (Cowork may grant no shell). List the
   ids and versions it carries.
2. **List what the library already holds** with `list_artifacts`. Match on
   name (Cowork titles an entry from its id, kebab → Title Case, so
   `uk-client-report` appears as *Uk Client Report* or *UK Client Report*).
3. **For each manifest record not already present**, call `create_artifact`
   with:
   - `html_path`: the staged file's full path from step 1 — never inline HTML;
   - `mcp_tools`: the record's list, verbatim and complete — omitting it
     silently produces a dashboard that can never call a tool;
   - the record's `id`, `name` and `description`; star it if the manifest says
     so.
   One create at a time. Wait for each result before the next.
4. **Verify** with `list_artifacts` again. Report what was created, what was
   already there, and anything that failed with the exact error.
5. **Updating**: if a present artifact's version in the manifest is newer than
   what was installed (the user says the dashboard looks old, or the manifest
   version changed), tell the user to delete the library entry and run this
   skill again. Never create a duplicate beside it.

## Result

A short receipt:

```
Bundle: <path>  (manifest version <n>)
Installed: uk-client-report v1.0.0 (new)
Already present: —
Failed: —
Next: open "UK Client Report" from the library, pick an account.
```

## Guardrails

- **Only what the manifest lists.** No artifact is created from any other
  source, and `mcp_tools` is never edited by hand.
- **Idempotent.** Present means skip; never delete or overwrite on the user's
  behalf.
- **Serial.** One `create_artifact` at a time; parallel creates have produced
  duplicates.
- If `create_artifact` or `list_artifacts` is not available, this is not a
  Cowork session: say so and stop. Claude Code renders these reports as
  Markdown instead.
