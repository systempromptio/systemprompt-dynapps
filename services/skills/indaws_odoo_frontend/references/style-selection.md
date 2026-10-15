# Style Selection (Phase 2 - Presentations Only)

Generate 3 mini HTML previews in `/tmp/iw-preview/slide-previews/` (style-a.html, style-b.html, style-c.html). Create the directory with `mkdir -p /tmp/iw-preview/slide-previews`. Each preview is a single title slide showing the aesthetic.

## Mood-to-Style Mapping

| Mood | Style A | Style B | Style C |
|------|---------|---------|---------|
| Impressed/Confident | Corporate Elegant | Dark Executive | Clean Minimal |
| Excited/Energized | Bold Gradients | Kinetic Motion | Vibrant Contrast |
| Calm/Focused | Swiss Minimal | Soft Muted | Paper Clean |
| Inspired/Moved | Cinematic Dark | Warm Editorial | Atmospheric |

All styles use exclusively Indaws brand colors and Montserrat font.

## User Selection

Present the previews, then ask user to pick via AskUserQuestion:
- Header: "Style"
- "Which style preview do you prefer?"
- Options: "Style A: [Name]", "Style B: [Name]", "Style C: [Name]", "Mix elements from multiple"

See `references/presentations.md` for slide type templates, navigation JS, and full structure.
