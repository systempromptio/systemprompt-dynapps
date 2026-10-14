# SVG assets: emoji (default) vs PNG (brand only)

How to handle SVG/icon assets from the source design in an email-safe way. Emails cannot rely on inline `<svg>` (most clients strip or ignore it), so every SVG must become either an emoji or a hosted PNG.

## Table of contents
- [1. Decision: emoji vs PNG](#1-decision-emoji-vs-png)
- [2. Render SVG -> retina PNG](#2-render-svg---retina-png)
- [3. Upload PNG to Odoo (ir.attachment) and get a URL](#3-upload-png-to-odoo-irattachment-and-get-a-url)
- [4. Put the image in the MJML/HTML](#4-put-the-image-in-the-mjmlhtml)

---

## 1. Decision: emoji vs PNG

| Use | When | Why |
|---|---|---|
| **PNG** (default when source has SVG) | Any inline `<svg>` in the source design | Pixel-faithful to the design. Rasterized at 3x and hosted as an Odoo `ir.attachment`. Costs: external images can be blocked by Outlook/Gmail until "show images". |
| **Emoji** (fallback) | When `rsvg-convert` is missing OR the attachment upload fails, OR a purely decorative icon maps cleanly to an emoji | Never blocked (it's a glyph), no hosting, adapts to dark mode, zero weight. |

**Default: if the source contains SVG, convert to PNG.** Fall back to emoji only when PNG can't be produced/hosted. Always set `alt` so blocked images still communicate.

## 2. Render SVG -> retina PNG

`svg_to_png.cjs` does the deterministic local part (no Odoo). Render at 2x/3x and display at the logical width so it stays crisp on retina:

```bash
node scripts/svg_to_png.cjs <input.svg> <output.png> --width=<px> --scale=3 --emit-base64
```
- `--width` = the logical px width you'll use in `<img width>`.
- `--scale` = retina multiplier (3 recommended). A 24px icon renders at 72x72.
- `--emit-base64` also writes `<output>.b64.txt` with the PNG base64, ready for the attachment upload.

Requires `rsvg-convert` (librsvg). macOS: `brew install librsvg`. If absent, the script aborts with that instruction.

## 3. Upload PNG to Odoo (ir.attachment) and get a URL

Claude does this via MCP (scripts can't call MCP). For each PNG:

1. Create a public attachment with the base64 from step 2:
   ```
   mcp__odoo-indaws__orm_create
     model: "ir.attachment"
     values: {
       "name": "<slug>-<icon>.png",
       "datas": <base64 string from the .b64.txt>,
       "mimetype": "image/png",
       "res_model": "mailing.mailing",   // optional: link to the mailing
       "res_id": <MAILING_ID>,           // optional
       "public": true                    // required so the email can load it without auth
     }
   ```
   The result is the attachment id (read it like any create result; never guess).

2. Build the absolute URL: `<ODOO_BASE_URL>/web/image/<ATTACHMENT_ID>` (e.g. `https://erp.example.com/web/image/12345`). `/web/image/<id>` serves the binary; `public: true` makes it reachable from the recipient's client.

3. Keep a small map `{ icon-name -> URL }` to substitute into the HTML.

> Weight note: base64 only matters for the upload payload, not the email (the email references the URL). Still, keep PNGs small (a 72px icon is ~2 KB).

## 4. Put the image in the MJML/HTML

Use `mj-image` with the absolute URL and the logical width (NOT the rendered px width):
```xml
<mj-image src="<ODOO_BASE_URL>/web/image/<ATTACHMENT_ID>" alt="<meaning>" width="24px" />
```
Or raw HTML inside `mj-text`/`mj-raw`:
```html
<img src="<ODOO_BASE_URL>/web/image/<ATTACHMENT_ID>" width="24" alt="<meaning>" style="display:block;border:0;">
```
After substitution, `validate.cjs` will pass (absolute https src + alt). If you leave an SVG or a relative path, validate.cjs flags it.
