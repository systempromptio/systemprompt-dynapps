# Troubleshooting

Common failures, mapped to a plain-language explanation for a non-technical user. When you hit one, explain it to the user **in their language** and in marketing terms — not a raw stack trace.

## Table of contents
- [Build / compile](#build--compile)
- [Validation](#validation)
- [Odoo upload](#odoo-upload)
- [Test send](#test-send)
- [Rendering complaints](#rendering-complaints)

---

## Build / compile
| Technical error | Plain explanation | Fix |
|---|---|---|
| `mjml ... cannot be used inside ...` | A block is in the wrong place in the layout. | Fix nesting in the `.mjml` (e.g. text must live inside a column). |
| `MJML returned empty HTML` / `r.html undefined` | The wrong MJML version (v5) got installed. | Reinstall pinned: `npm i --save-exact mjml@4.18.0`. |
| `juice failed ... reading 'replace'` | The previous step produced nothing. | Check MJML errors first; ensure `juice@8.1.0`. |
| `Missing dependencies` | The skill's packages are not installed. | `npm install --no-audit --no-fund` in the skill dir. |

## Validation
| validate.cjs says | Plain explanation | Fix |
|---|---|---|
| `Weight NN KB exceeds limit` | The email is too heavy; Gmail will cut it off. | Trim CSS/links; keep minify on; reduce content. |
| `Forbidden <script>/<form>/...` | Something not allowed in email slipped in. | Remove it from the source design. |
| `<img src> is not absolute https` | An image points to a local/relative path. | Use a full `https://...` image URL. |
| `Unbalanced MSO conditional comments` | The Outlook fallback code got cut. | Re-run `prepare_odoo.cjs`; check any hand-written `mj-raw` mso block is balanced. |
| `Found N '{{' placeholders` | Template tags that Odoo would delete on send. | Replace with plain UPPERCASE words (CUSTOMER_NAME). |
| `Missing wrapper o_layout` | The fragment was not adapted for Odoo. | Run `prepare_odoo.cjs` on the compiled HTML. |

## Odoo upload
| Symptom | Plain explanation | Fix |
|---|---|---|
| `-32000 Internal server error` | The generic write tool can't handle a large body. | Use `mcp__odoo-indaws__orm_write` (the dedicated tool), not `orm_execute_kw method=write`. |
| Placeholders/styles missing after re-read | The HTML was altered after upload. | Don't reopen the Odoo visual editor; re-upload via `orm_write`. |
| Editor shows old/blank content | Only one body field was written. | Write BOTH `body_html` and `body_arch`. |
| MCP tools not available | Odoo isn't connected to this session. | Tell the user; run only up to validation. |

## Test send
| Symptom | Plain explanation | Fix |
|---|---|---|
| Wrong email got sent / not received | A guessed mail id was used. | Read the real id from `mail.mail(NNNN,)`; never guess. |
| `state` stays `outgoing` | The queue wasn't processed. | Call `process_email_queue([id])`. |

## Rendering complaints
| User says | Likely cause | Fix |
|---|---|---|
| "Top white strip on mobile (Apple Mail)" | Apple Mail's irreducible inset; not a CSS bug. | The `o_layout` reset minimizes it; the residual gap is the client's chrome. |
| "Square button in Outlook" | Word ignores CSS border-radius. | Add the `v:roundrect` VML patch (mjml-mapping.md §5) if the design needs it. |
| "No gradient in Outlook" | Word ignores CSS gradients. | Solid fallback shows; add `v:fill` VML if a real gradient is required. |
| "Custom font not showing in Outlook/Gmail" | Those clients don't load webfonts. | Expected; the Arial fallback applies. |
| "Badge and text too far apart in Apple Mail" | Separate `<tr>` rows get spread out. | Stack them in a SINGLE cell with two `<div>`s (margin between). |
