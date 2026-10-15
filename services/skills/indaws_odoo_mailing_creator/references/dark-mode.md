# Dark mode (defensive)

When a recipient reads in **dark mode**, the mail client may repaint your colors on its own — differently per
client. This is **defensive**: the goal is "it doesn't break", NOT "a second pretty theme". Hard-won recipe
from the DynCloud campaign (mailings 05/06).

## What each client does
- **Apple Mail (iOS/macOS):** *partial* auto-inversion — darkens light backgrounds and lightens dark text,
  but inconsistently. The most dangerous one. **Respects** `@media (prefers-color-scheme: dark)` and the
  `color-scheme` meta.
- **Gmail app (Android/iOS):** aggressive full inversion on many elements. **Ignores** `prefers-color-scheme`.
- **Outlook:** varies (sometimes nothing, sometimes inversion).

Net effect: you design/test in light, but ~half your readers see a version you never saw.

## The classic break
A light card (white bg, dark navy text) + a dark-colored logo image. In a recipient's dark mode Apple Mail
darkens the white bg, but it does NOT invert the logo image → **dark logo on a now-dark bg = invisible**. And
brand colors shift to muddy tones.

## The recipe (what actually survives Odoo)
1. **Declare color-scheme** so clients don't force-invert (goes in `mj-raw` in the head; survives the wrap):
   ```html
   <meta name="color-scheme" content="light dark">
   <meta name="supported-color-schemes" content="light dark">
   ```
   For an intentionally-dark email use `content="dark light"`.
2. **Explicit overrides** in `mj-style` (Apple Mail/iOS honor it; Gmail ignores → must still look OK without):
   ```css
   @media (prefers-color-scheme: dark) {
     .card { background-color:#1A1819 !important; }
     .ink  { color:#F1F1F1 !important; }
   }
   ```
   Use neutral-safe values `#121212`/`#F1F1F1` (not pure black/white) to avoid harsh re-inversion.
3. **Logo for inverted bg:** email can't do `filter:brightness()`. Generate a **recolored** copy (PIL: every
   non-transparent pixel → white, keep alpha), upload as a separate `ir.attachment`, swap it in dark mode (or
   just use a logo that reads on both). See `odoo-rendering-gotchas.md` §16.
4. **Gradient text on dark:** the normal clip-to-text gradient (violet→carmine→tangerine, fallback `#7B00DC`)
   is invisible on a dark card. Use a light-gradient variant (`#C9B8FF→#FF911F`, fallback `#C9B8FF`). See §11.

## Odoo caveats (why this differs from a generic skill)
- The `meta` tags survive because Odoo's layout wrap keeps head metas / `OfficeDocumentSettings`. But do NOT
  rely on a `<html lang>` or `<mj-title>` for anything — Odoo reinjects its own `<head>`.
- `@media` blocks in the folded `<style>` survive the `email_outgoing` sanitizer (verified).
- **Gmail ignores `prefers-color-scheme`** → never depend on the override alone; the *base* (non-dark) design
  must already be legible. The override is a bonus for Apple Mail, not a guarantee.

## Bottom line
Set the `color-scheme` metas on every mailing. Add `@media (prefers-color-scheme: dark)` overrides + a
light-readable logo only when the design has high-contrast elements that the client would mangle (white cards,
dark logos, brand-colored text). Test by toggling the device to dark mode and using the "Send test" button.
