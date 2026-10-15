# Odoo + Apple Mail rendering gotchas (battle-tested)

Hard-won lessons from converting a complex light-theme mailing (cards, gradients, webfonts, full-bleed bg)
and validating it in Apple Mail iOS + Odoo. These are NON-OBVIOUS and cost many test cycles. Read before
fighting any of these symptoms.

## Table of contents
- [0. The #1 rule: how to test](#0-the-1-rule-how-to-test)
- [1. Webfonts](#1-webfonts)
- [2. font-family must be inline everywhere](#2-font-family-must-be-inline-everywhere)
- [3. Equal-height cards side by side](#3-equal-height-cards-side-by-side)
- [4. mj-raw tables break the body wrapper](#4-mj-raw-tables-break-the-body-wrapper)
- [5. Default paddings cause misalignment](#5-default-paddings-cause-misalignment)
- [6. Odoo stretches :only-child images to 100%](#6-odoo-stretches-only-child-images-to-100)
- [7. Image sizing: width not height, and pre-size the PNG](#7-image-sizing-width-not-height-and-pre-size-the-png)
- [8. Full-bleed background on mobile](#8-full-bleed-background-on-mobile)
- [9. Stacking mj-table cards on mobile](#9-stacking-mj-table-cards-on-mobile)
- [10. Equal horizontal & vertical gaps](#10-equal-horizontal--vertical-gaps)
- [11. Gradient text for brand words](#11-gradient-text-for-brand-words)
- [12. juice drops rules that match nothing](#12-juice-drops-rules-that-match-nothing)
- [13. border+radius on a td → square border (use a div)](#13-border--border-radius-on-a-td--square-border-in-apple-mail)
- [14. pills wrap vertically in Outlook OWA](#14-pillsbadges-wrap-vertically-in-outlook-owa)
- [15. bake icon+box into one PNG](#15-icons-inside-a-colored-box-bake-iconbox-into-one-png)
- [16. recolor a logo for an inverted background](#16-recolor-a-logo-for-an-inverted-background)

---

## 0. The #1 rule: how to test
**Validate with the mailing.mailing "Send test" button in Odoo, NOT with a `mail.mail` you create+process_email_queue.**
A `mail.mail` sent directly does NOT get wrapped in Odoo's `mass_mailing_mail_layout` (no `<head>`/reset),
so it renders differently from the real send — it will mislead you (looks broken when it's fine, or vice
versa). Every "it looks the same / it looks wrong" confusion in this project traced back to testing via
`mail.mail`. Use `mail.mail` only as a last resort and know it under-represents the real render.

## 1. Webfonts
- Odoo's `email_outgoing` sanitizer **strips `<link rel=stylesheet>` AND mangles `@import`** (it leaves a
  bare ` url(...)` and double-escapes `&`). The ONLY webfont mechanism that survives is **`@font-face`
  inside `<style>`**.
- **Use `.ttf` with `format('truetype')`, NOT `.woff2`.** Apple Mail iOS does not reliably load woff2 from
  a body `<style>`; it does load TTF. (Mailing 01 worked precisely because it used TTF.)
- Use the correct **`latin` subset** URL for each weight — each weight has a DIFFERENT gstatic URL. Pulling
  the wrong subset (or repeating one URL for all weights) = font silently doesn't apply. Extract per weight:
  `curl -A "<modern UA>" "https://fonts.googleapis.com/css2?family=X:wght@400&display=swap"` and take the
  url under the `/* latin */` comment.
- Do NOT use `mj-font` (it emits the gated `<link>`, which Odoo strips). Hand-write `@font-face` in `mj-style`.

## 2. font-family must be inline everywhere
Apple Mail applies `@font-face` only when the element's `font-family` is **inline** (`style="font-family:…"`),
not when it comes from a class rule in `<style>`. The compile pipeline runs juice which inlines most of it,
BUT verify it reached **every** text element — especially:
- `<strong>` / `<b>` bold runs (they often lack inline font-family → render in the serif system fallback).
- inline `<span style="font-weight:700">` highlights.
Add `font-family:'…',Arial,sans-serif` inline to every bold/highlight span.

## 3. Equal-height cards side by side
Two cards in a row that must match height: **use `mj-table` with sibling `<td>` cells in one `<tr>`.**
- `min-height` on inline-block MJML columns does NOT equalize (each measures its own content). Tried — fails.
- `mj-group` does NOT equalize either (columns are still inline-block inside). Tried — fails.
- Real sibling `<td>` cells auto-equalize height in every client, including Apple Mail. This is the only
  reliable way.

## 4. mj-raw tables break the body wrapper
Do NOT put a full `<table>` inside `mj-raw`. It desyncs MJML's div balancing and **closes the body/background
wrapper early**, dropping later sections OUTSIDE the page-gradient div (you'll see "3 divs outside the global
div" in devtools, and the bg stops partway). Use the native **`mj-table`** component instead — it integrates
into MJML's structure and does not break the wrapper.

## 5. Default paddings cause misalignment
MJML applies a default padding to several components that silently indents content:
- **`mj-text`**: default `padding:10px 25px` → a title/paragraph without explicit `padding="0"` sits 25px
  further in than its sibling label → visible left-misalignment. Always set `padding` explicitly.
- **`mj-table`**: same default `10px 25px` → add `padding="0"` or the whole table is inset 25px.
Symptom: "the title is indented vs the eyebrow above it" — it's the padding, not the monospaced font.

## 6. Odoo stretches :only-child images to 100%
`mass_mailing_mail.scss` ships: `@media (max-width:1135px){ img:only-child:not(.img-fluid){min-width:100%!important} }`.
Any image that is the **only child of its container** gets blown up to 100% width with `!important`, ignoring
your `width:78px`. Devtools shows the width change "not taking effect". **Fix:** make the image NOT an only
child — add an invisible sibling next to it, e.g.
`<img …><span style="display:none;font-size:0;line-height:0;max-height:0;">&zwnj;</span>`.
(Alternatively add class `img-fluid`, but the sibling trick is surgical and predictable.)

## 7. Image sizing: width not height, and pre-size the PNG
- Email clients often **ignore `height` on `<img>`** and use the natural width → a 1920px logo overflows.
  Size by **`width`** (with `height:auto`), like `mj-image` does.
- Better: **rasterize the PNG to the real display size ×3** (retina) and upload that as the attachment, so
  the asset can't render huge regardless of how the client treats attributes, and stays sharp. ×2 looked
  soft on retina; ×3 is crisp. (`sips -z <h> <w> in.png --out out.png` on macOS.)

## 8. Full-bleed background on mobile
A `mj-body width="660px"` makes a centered 660px table; on mobile its background doesn't reach the screen
edges (white at the sides). Fixes, combined:
- Put the page gradient on the **`o_layout` wrapper** (full width) inline — NOT only on the 660px body. Keep
  the mj-body transparent so the wrapper shows through. (Inject it as an inline `style` on the `o_layout` div
  in a post-step; see §12 for why a CSS rule won't survive.)
- Add `@media (max-width:480px){ .o_layout table[align="center"]{width:100%!important;max-width:100%!important} … }`
  forcing the 660px tables/columns to 100% so content uses full width and the bg reaches the edges.

## 9. Stacking mj-table cards on mobile
`mj-table` sibling `<td>` cells stay side by side by default. To stack on mobile, give the card cells a class
and: `@media (max-width:480px){ td.cardcell{display:block!important;width:100%!important;box-sizing:border-box!important;margin-bottom:12px!important} td.cardgap{display:none!important;width:0!important} }`.

## 10. Equal horizontal & vertical gaps
- Horizontal gap = a spacer cell BETWEEN the card cells. Make widths sum to **exactly 100%**
  (`49% + 2% + 49%`), else the leftover % inflates the spacer and the gap looks bigger than the number says.
- Vertical gap = the next section's `padding-top`, in px equal to the horizontal % (e.g. 2% of ~592px ≈ 12px).
- Tune ONE side at a time; changing both at once makes it impossible to converge.

## 11. Gradient text for brand words
For brand words (e.g. the product name) use clip-to-text gradient on non-Outlook, solid fallback on Outlook:
`@media screen{ .grad-text{ background-image:linear-gradient(...)!important; -webkit-background-clip:text!important; background-clip:text!important; -webkit-text-fill-color:transparent!important; color:transparent!important } }`
and inline `color:<solid>` as the Outlook/Gmail fallback. Apply the class to **every** occurrence of the
brand word, not just the headline. (Gmail ignores background-clip → shows the solid color, which is fine.)

## 12. juice drops rules that match nothing
juice runs during compile, BEFORE `prepare_odoo` adds the `<div class="o_layout">` wrapper. A `.o_layout{…}`
rule in `mj-style` matches no element at juice time, so **juice deletes it**. To style the o_layout wrapper
(e.g. full-bleed bg), inject it as an **inline `style`** on the wrapper div in a post-prepare step, not as a
CSS rule.

## 13. border + border-radius on a `<td>` → square border in Apple Mail
A `<td>` with BOTH `border` and `border-radius` renders the **fill rounded but the border square** in Apple
Mail (the border doesn't follow the radius). Very visible when the border is high-contrast (e.g. a light
border on a dark card, or a colored tip box). **Fix: move `background`+`border`+`border-radius` onto an inner
`<div>`** — divs clip border+radius together reliably. Pattern: `<td style="padding:0"><div style="bg;border;
border-radius;padding">…</div></td>`. A `<td>` with bg+radius but NO border is fine (no square border to show).
If the card also needs a `@media` gradient, put the class on the **div** and target `.cls` (not `td.cls`).

## 14. Pills/badges wrap vertically in Outlook OWA
`white-space:nowrap` on the **cell** (`<td>`) is ignored by Outlook web (OWA), which breaks short words
character-by-character to fit a narrow reading pane (`EN CURSO` → `EN`/`CURSO`, `OFFLINE` → `OFF`/`LIN`/`E`).
**Fix: put `white-space:nowrap` on the pill's own `<span>`** (inline), not just the surrounding cell.

## 15. Icons inside a colored box: bake icon+box into ONE PNG
Don't build "small icon centered in a styled `<td>` box" — it's fragile (the only-child stretch of §6, box
size drift between clients, centering quirks). **Rasterize the icon ONTO its rounded box as a single PNG** at
display-size ×3 and drop it in as one `<img width=40 height=40>`. No box `<td>`, no centering, no stretch —
identical in every client. Build a composite SVG: `<rect rx fill>` (the box) + `<g transform="translate(m,m)
scale(displayIcon/24)" stroke=…>…lucide paths…</g>` (the icon, in its 24-viewBox), then `rsvg-convert -w
<3×> -h <3×>`. For a **translucent** box over a gradient, use `fill="#FFF" fill-opacity="0.16"` — the PNG's
alpha lets the gradient show through, so it composites correctly over any background. Inspect with a PIL
montage on the REAL background color before uploading. (Mailings 04/05/06 do this for calendar/clock/status/
key/docs icons.)

## 16. Recolor a logo for an inverted background
When a logo (dark wordmark) must sit on a dark background and the source uses `filter:brightness()` (not
supported in email → the dark logo stays invisible), generate a **recolored** copy: with PIL, set every
non-transparent pixel to white and keep the alpha, upload as a separate `ir.attachment`, use it at low opacity.
(Mailing 05 footer uses a white-recolored Indaws logo.)
