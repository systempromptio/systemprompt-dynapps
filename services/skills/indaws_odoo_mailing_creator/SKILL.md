---
name: indaws-odoo-mailing-creator
version: 1.0.3
effort: medium
model: sonnet
description: >-
  Converts source HTML into cross-client, Outlook-safe email and ships it through Odoo 19 Mass Mailing
  with a real test send. Use when the user needs to create, render, or upload a newsletter or email
  campaign to Odoo.
  Triggers: "sube este email a Odoo", "newsletter no se ve bien en Outlook", "crea un mailing",
  "mailing masivo", "plantilla de email", "boletín", "email campaign", "upload email to Odoo",
  "mass mailing", "send test email".
---

# odoo-mailing-creator

Convert a source HTML email into a cross-client, Outlook-safe email and ship it through Odoo 19 Mass Mailing.

## Language

Write all code and docs in English. **Reply to the user in the language they use** (Spanish, English, etc.).

## What this skill does

Source HTML → **MJML** → compile to email-safe HTML → validate (no browser) → upload to Odoo
`mailing.mailing` → send a real test. Brand- and client-agnostic: replicates the incoming design
faithfully. Artifacts live in `forge/src/` (`.mjml`), `forge/dist/` (`.html` + `.odoo.html`),
`forge/tests/` (reports), outside the skill directory. See `references/requirements.md`.

## Workflow (guided, with gates)

Keep this checklist visible and update it as you go:

```
Mailing progress:
- [ ] 1. Source HTML received
- [ ] 2. Validated against input rules
- [ ] 3. Converted to MJML (design preserved faithfully)
- [ ] 4. Compiled to email-safe HTML
- [ ] 5. Adapted to an Odoo fragment
- [ ] 6. Static validation passed (exit 0)
- [ ] 7. Preview shown and CONFIRMED by the user   <-- GATE
- [ ] 8. Uploaded to Odoo (draft) and verified
- [ ] 9. Real test sent to the given address       <-- GATE
- [ ] 10. Human visual QA checklist delivered
```

**Step 1** — Ask for the HTML (file path or pasted); save to `forge/src/<slug>.source.html`.

**Step 2** — Read `references/html-input-rules.md`. Auto-fix safe issues; report changes; block only on
constructs that cannot render at all (scripts, forms, unresolvable relative URLs).

**Step 3** — Read `references/mjml-mapping.md`. Map design to MJML components. Extract colors/fonts from
source into `mj-attributes`/`mj-all`. Apply Outlook `mj-raw` patches only where the source needs them.
SVG icons: convert to PNG by default (`scripts/svg_to_png.cjs`), upload as `ir.attachment`, use real
`/web/image/<id>` URL. Fall back to emoji only if `rsvg-convert` absent. See `references/svg-assets.md`.

**Step 4** — `node scripts/compile.cjs forge/src/<slug>.mjml forge/dist/<slug>.html`. MJML syntax errors:
fix and retry. Self-install runs on first call; pinned: `mjml@4.18.0`, `juice@8.1.0`.

**Step 5** — `node scripts/prepare_odoo.cjs forge/dist/<slug>.html forge/dist/<slug>.odoo.html`.
Produces `<div class="o_layout o_basic_theme">…</div>` fragment. See `references/odoo-injection.md`.

**Step 6** — `node scripts/validate.cjs forge/dist/<slug>.odoo.html`. Must exit 0.

**Step 7 (GATE)** — Show summary (subject, size KB, sections, image count) and `.odoo.html` path.
Ask for explicit confirmation before touching Odoo.

**Step 8** — Prefer `scripts/odoo_push.cjs` (JSON-RPC, reads from disk); MCP `orm_create` fails `-32000`
for large payloads. Upload SVG→PNG attachments first, verify checksums, substitute real URLs, re-run
compile/prepare/validate. Preheader: set native `preview` field via `odoo_push.cjs set-preview` — it is
NOT a hidden div in the body. Do NOT reopen the visual editor after upload.

**Step 9 (GATE)** — Tell the user exactly what will happen. Ask for confirmation. Mass send is out of scope.

**Step 10** — Prefer the Odoo "Send test" button on the mailing record; `mail.mail` created directly does
NOT get the `mass_mailing_mail_layout` wrapper and will mislead. See `references/odoo-rendering-gotchas.md`.

**Step 11** — Deliver human visual QA checklist from `references/validation.md`.

## GOTCHAS

- **mjml@v5 is async/incompatible** — ALWAYS pin to `4.18.0`; v5 breaks the compile pipeline silently.
- **MCP `orm_create` fails `-32000`** for large bodies or attachments — use `scripts/odoo_push.cjs` instead.
- **Odoo visual editor regenerates HTML** if you reopen the mailing after upload — never reopen it.
- **SVG does not render in email clients** — convert to PNG via `svg_to_png.cjs`; fallback to emoji only
  if `rsvg-convert` is unavailable.
- **Preheader is a native field** (`preview` on `mailing.mailing`), not a hidden `<div>` in the body.
- **`mail.mail` created directly misses the layout wrapper** — the user sees a different render than
  what recipients will get; always use "Send test" from the mailing record.
- **`css-class` attribute goes on the outermost MJML tag**, not the inner text div — check compiled output.

## NEVER

- NEVER trigger a mass send — this skill only uploads drafts and sends test emails.
- NEVER bump `mjml` to v5 — pin to `4.18.0`.
- NEVER reopen the Odoo visual editor after upload.
- NEVER guess a `mail.mail` id — read the real one from the create response.
- NEVER use `orm_create` for large bodies or attachments — use `odoo_push.cjs`.

## ALWAYS

- ALWAYS get explicit user confirmation before step 8 (upload) and before step 9 (test send).
- ALWAYS verify uploaded body contains `o_layout`, `@media`, `[if mso]` before confirming success.
- ALWAYS confirm each `ir.attachment` URL returns `200 image/png` before substituting into MJML.

## Delegates to

| Skill | Suite | When |
|-------|-------|------|
| `odoo-pilot` | `common-skills` | Odoo authentication and CRUD when MCP is unavailable |

## Consumed by

| Skill | Suite | When |
|-------|-------|------|
| `email-campaign-builder` | `ai-claude-marketing` | Complex HTML email rendering for campaigns |
| `campaign-planning` | `ai-claude-marketing` | Email channel execution within a campaign plan |

## REFERENCES

> Shared connection patterns: `references/shared/odoo-connection.md`

- `references/requirements.md` — environment, deps, MCP tools, troubleshooting matrix.
- `references/html-input-rules.md` — source HTML rules; `claude-design-brief.md` for marketing briefs.
- `references/mjml-mapping.md` — HTML pattern → MJML component + Outlook `mj-raw` patches.
- `references/dark-mode.md` — color-scheme metas, `@media` overrides, recolored logo.
- `references/odoo-personalization.md` — QWeb `<t t-out>` (body) vs `{{ }}` (preview/subject).
- `references/svg-assets.md` — PNG default vs emoji fallback; `svg_to_png.cjs` + `ir.attachment`.
- `scripts/odoo_push.cjs` — JSON-RPC uploader (attach/create-from-template/write-body/send-test).
- `references/odoo-injection.md` — upload + test-send procedure; fragment format.
- `references/odoo-rendering-gotchas.md` — Apple Mail + Odoo gotchas (webfonts, fonts inline, cards,
  mj-raw tables, image stretch, full-bleed bg, gap math, gradient text).
- `references/validation.md` — static checks + human visual QA checklist.
- `references/troubleshooting.md` — common failures in marketing-friendly language.

## If something fails

See `references/troubleshooting.md`. Explain errors in the user's language, in plain terms.
