# Claude Design brief (for the marketing team)

A paste-ready brief the marketing team gives to Claude (or any HTML generator) to produce a **source HTML** that converts cleanly through this skill. It encodes the input rules so the team does not have to write technical markers by hand. The team only describes the campaign; the brief makes the output conformant.

Hand this block to the team. They fill the `<<...>>` parts and paste it into Claude.

---

```
You are designing a marketing email as a SINGLE self-contained HTML file. It will be converted to MJML
and sent from Odoo, so it MUST follow these constraints exactly:

DESIGN (free — this is the brand of the campaign):
- Brand/colors: <<describe palette or paste hex codes>>
- Font: <<font family>> (with Arial/Helvetica fallback)
- Tone & content: <<what the email says, sections, CTA text and link>>

STRUCTURE (mandatory):
- One body wrapper. Stack the email as full-width horizontal bands (sections), top to bottom.
- For side-by-side content, use adjacent column blocks. Mark each band with data-block when it is one of:
  hero, text, image, button, divider, spacer, footer.
- Target width 600px.

LAYOUT RULES (mandatory — do NOT break):
- NO display:flex, NO display:grid, NO position:absolute/fixed, NO float for layout.
- NO <script>, NO <form>, NO <iframe>, NO <video>/<audio>.

STYLE RULES (mandatory):
- Put colors as a small set of values reused consistently (or CSS variables at the top).
- One font-family with a web-safe fallback. Keep styles simple and per-block.
- Gradients/shadows/rounded corners are fine — provide a SOLID fallback color for each gradient.

IMAGES (mandatory):
- Every image uses an ABSOLUTE https URL and has an alt text.

VARIABLES (mandatory):
- Any personalization placeholder must be a PLAIN UPPERCASE WORD, e.g. CUSTOMER_NAME, CTA_URL, YEAR.
- Do NOT use {{ }} or any template syntax.

OUTPUT:
- Return ONE complete HTML file, nothing else.
```

---

After the team pastes the generated HTML to the skill, the skill validates it against `html-input-rules.md`, converts it to MJML faithfully, compiles, validates, uploads to Odoo and sends a test — with confirmation gates along the way.
