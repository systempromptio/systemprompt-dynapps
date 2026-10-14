# Workflow Detail — Visual Brand Strategist

Full descriptions for each of the 6 workflow steps and the HTML moodboard generation procedure.

---

## Step 1 — Analyse Visual Attributes

Extract **exactly three adjectives** that describe how the brand should be perceived. They must not be synonyms: each must add a distinct dimension (e.g. one emotional/personality, one sectorial character, one competitive positioning). Avoid hollow adjectives like "modern", "innovative", "professional" unless qualified with a concrete nuance ("modern-kinetic", "professional-serene").

Format: `Adjective 1, Adjective 2, Adjective 3` followed by a short sentence explaining why these three and not others.

---

## Step 2 — Define Style Direction

Define the overall design direction. Canonical options are described in `references/style-directions.md` (Minimalist, Maximalist, Retro, Tech, Brutalist, Editorial, Organic, Luxury, Playful, etc.). Choose one primary and, if appropriate, a secondary hint ("Minimalist with an Editorial hint"). Justify in one sentence: why this direction serves the three attributes from step 1 and the specific sector.

---

## Step 3 — Propose Logo Morphology

Describe the **suggested morphology** of the logotype in 2–3 sentences. Do not draw or describe the final logo, but the **constructive principle**: what type of mark it is (logotype only, isotipo, imagotipo, isologo), what geometry or gesturality defines it, what visual metaphor sustains it. Examples:

- "Imagotype with a geometric isotipo based on converging kinetic lines suggesting speed and trajectory; logotype in extra-bold sans-serif to reinforce physical impact."
- "Pure typographic logotype built on a transitional serif with sharp serifs; no isotipo to reinforce the editorial and authoritative feel."
- "Circular isologo with an abstract symbol evoking a knot or tie (continuity, family bond) typographically integrated with the name."

The description must allow a designer to start sketching without ambiguity.

---

## Step 4 — Build Color Palette

Propose **3 or 4 HEX codes**. For each indicate:

- Descriptive name (non-technical): "Vinyl Black", "Sage Green", "Volt Magenta".
- Exact HEX code.
- Functional role: primary, secondary, accent, neutral.
- Sector-specific justification: why this color (not an adjacent one) serves this business. Reference `references/color-psychology.md` to anchor chromatic psychology to the specific sector, not generic psychology.

Rules:
- Do not propose a color without a sector justification.
- Check basic accessibility: primary and accent must have sufficient contrast on the neutral.
- If sector anti-patterns forbid certain tones (see `references/sector-antipatterns.md`), do not use them even as an accent.

---

## Step 5 — Define Typography Pairing

Propose a **font pair**: one for `Heading` (titles, brand, navigation) and one for `Body` (running text, UI, labels). For each indicate:

- Exact font name (ideally available on Google Fonts or a known foundry).
- Typographic classification (Transitional Serif, Geometric Sans-serif, Slab, Display, Monospace, Humanist, etc.).
- Why it fits the style direction and the attributes from step 1.

The pair must have **typographic contrast** (do not propose two neutral sans-serifs). Reference `references/typography-pairings.md` for patterns that work and combinations to avoid. When tempted to pick a "safe" font like Inter + Inter, ask whether that choice truly serves the sector or is laziness disguised as minimalism.

If you propose a paid foundry font (Söhne, GT Sectra, Druk), include in the JSON a close Google Fonts alternative in `google_font_url` / `css_family` so the moodboard renders correctly, and mention the substitution in the justification.

---

## Step 6 — Write Textual Moodboard → Generate HTML Moodboard

Write a paragraph (4–6 sentences) describing the **visual world** surrounding the brand: what type of photography (angles, lighting, subjects), what textures (paper, metal, glass, organic), what auxiliary graphic elements (lines, patterns, shapes), what cultural or aesthetic references to mobilise (cinema, art, decoration). Be specific: instead of "aspirational photography", write "photography with hard lateral light, close-up shots of worked hands, desaturated palette with a single warm colour accent point".

### HTML Moodboard Generation

1. Build a JSON object with the proposal using the schema documented in `references/moodboard-spec.md`.
2. Run `python scripts/render_moodboard.py --spec <path-to-json> --output <path-html>` to generate the file.
3. If Python is not available, use `assets/moodboard_template.html` directly and substitute placeholders via the Write/Edit tool. Placeholders are documented in the template header. If using only 3 colours, remove the entire block between `<!-- BEGIN_OPTIONAL_COLOR_4 -->` and `<!-- END_OPTIONAL_COLOR_4 -->`.
4. Ensure both Google Fonts URLs are distinct and non-empty. If combining heading and body in a single Google Fonts request, duplicate the complete URL in both placeholders (do not leave either empty). Include the weights used in the URL (minimum `wght@400;600` so h1/h2 and `.logo-mark` render correctly).
5. Save the HTML to `outputs/visual-brand-strategist/<timestamp>-<slug>.html` (create the folder if it does not exist) and tell the user the absolute file path at the end of the Markdown response.
6. If the user asks to share the moodboard externally, delegate to the `utilities:publish` skill to obtain a signed public URL.

---

## Deliverable A — Markdown Structure in Chat

```markdown
# Propuesta de identidad visual — [Business name or short description]

## 1. Atributos visuales
[3 adjectives + justification sentence]

## 2. Dirección de estilo
[Direction + justification sentence]

## 3. Logotipo
[2-3 morphology sentences]

## 4. Paleta cromática
- **[Name]** `#HEX` — [Role]. [Sector justification].

## 5. Sistema tipográfico
- **Heading** — [Font] ([classification]). [Justification].
- **Body** — [Font] ([classification]). [Justification].

## 6. Moodboard textual
[Paragraph of 4-6 sentences]
```

---

## Handling Edge Cases

- **Partial requests**: if the user only asks for one section (e.g. "just give me the palette for my specialty café"), deliver only that section but with the same depth of justification. Do not abbreviate.
- **Inherited constraints**: if the user provides fixed colours, mandatory typography, or prohibited elements, treat them as fixed input and build the rest of the proposal around them.
- **Ambiguous or contradictory briefs**: do not force a flat answer. Identify the tension, propose the most coherent reading, and note the decision in a brief line before the deliverable. Internal consistency overrides literal brief.
- **Final validation**: before closing the proposal, validate against the cross-sectional heuristics and sector anti-patterns documented in `references/sector-antipatterns.md`.
