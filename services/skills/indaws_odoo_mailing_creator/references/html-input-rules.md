# Source HTML input rules

Rules the incoming source HTML should follow so the MJML conversion is reliable and deterministic. The skill validates the source against these (step 2), auto-fixes what is safe, and reports the rest in plain language. These rules are brand-agnostic — they constrain STRUCTURE, never design (colors/fonts/copy are the client's).

## Table of contents
- [1. Structure](#1-structure)
- [2. Layout: nothing exotic](#2-layout-nothing-exotic)
- [3. Styles: tokens, not scattered hex](#3-styles-tokens-not-scattered-hex)
- [4. Content variables](#4-content-variables)
- [5. Resources](#5-resources)
- [6. Hygiene](#6-hygiene)

---

## 1. Structure
1. One root wrapper for the body.
2. Each visual band is a clear section; columns are adjacent blocks (they become `mj-section`/`mj-column`).
3. Recognizable block types (hero, text, image, button, divider, spacer, footer) map 1:1 to MJML components. If the source marks blocks with `data-block="hero|text|image|button|divider|spacer|footer"`, mapping is unambiguous — encourage it but do not require it.

## 2. Layout: nothing exotic
4. **No** `display:flex`, `display:grid`, `position:absolute/fixed`, or `float` for layout. Columns = adjacent column blocks (MJML puts them in tables). The skill converts flex/grid columns to stacked MJML columns and reports it.
5. Vertical stacking of sections + MJML columns only. Width 600–640px.

## 3. Styles: tokens, not scattered hex
6. Prefer colors as a small token set declared once (or CSS variables). The skill extracts them into `mj-attributes` and per-component attributes — faithfully to the source, not to any house style.
7. One font family with a web-safe fallback (`Arial, Helvetica, sans-serif`). The design font goes inline (so it beats Odoo's non-`!important` Arial rule); Outlook uses the fallback.
8. No complex `<style>` selectors; per-block styles MJML can inline. `@media` only for simple responsive/dark (MJML/`mj-style` handles it).

## 4. Content variables
9. Fill-later campaign placeholders as **plain UPPERCASE words** (`CUSTOMER_NAME`, `CTA_URL`, `YEAR`) — the team edits them to a fixed value before sending. Never bare `{{ }}` for these. The skill validates none remain. For real **per-recipient** personalization (recipient name, etc.) use Odoo's render syntax — body uses QWeb `<t t-out="object.x">fallback</t>`, preview/subject use `{{ object.x }}`. Full guide: `odoo-personalization.md`.

## 5. Resources
10. All images with **absolute https URL** and `alt`. Relative URLs break in email.
11. Buttons as button blocks with an absolute `href` (MJML emits the Outlook-safe button; rounded corners in classic Outlook need the VML patch — see `mjml-mapping.md` §5).

## 6. Hygiene
12. No JavaScript, no forms, no `<iframe>`, no embedded video/audio (unsupported in email; use an image-link instead).

The more the source follows these, the closer the conversion gets to deterministic 1:1. When something is off, auto-fix if safe (e.g. flex→columns) and tell the user, in their language, what changed.
