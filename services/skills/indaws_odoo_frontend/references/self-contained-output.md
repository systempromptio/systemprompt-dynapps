# Self-Contained Output Format (Website Pages / Landing Pages)

All website pages and landing pages MUST be generated as **self-contained HTML** ready to paste inside an Odoo QWeb template. The output file contains everything in a single block:

```html
<!-- Instructions for pasting into QWeb template -->

<style>
/* Full contents of assets/css/indaws-frontend.css inlined here */
@import url('https://fonts.googleapis.com/css2?family=Montserrat:ital,wght@0,400;0,600;0,700;1,400;1,600&amp;display=swap');
/* ... all iw-* class definitions ... */
</style>

<section class="iw-hero iw-section-wave">
  <!-- Page content using Bootstrap 5 + iw-* classes -->
</section>
<!-- More sections... -->
```

## Rules

1. **Inline the full CSS** from `assets/css/indaws-frontend.css` inside a `<style>` block at the top of the file. Do NOT reference external CSS files.
2. **No `<!DOCTYPE>`, `<html>`, `<head>`, `<body>` wrappers** -- Odoo's `website.layout` provides those.
3. **No `<script>` tags** -- Odoo Editor strips them from website pages.
4. **No `iw-reveal` animation classes** -- They require JS that cannot be included. Do not use `iw-reveal`, `iw-reveal-left`, `iw-reveal-right`, or `iw-reveal-scale`.
5. **XML-safe URLs** -- Use `&amp;` instead of `&` in `@import` and any other URL (the file is pasted inside a QWeb/XML template).
6. **Image placeholders** -- Use `/web/image/placeholder.png` for all images. Add an HTML comment above each `<img>` indicating the intended image: `<!-- IMAGEN: description_or_filename.png -->`.
7. **Include a header comment** explaining how to paste the content into the QWeb template structure:
   ```
   <t t-name="website.page_name">
     <t t-call="website.layout">
       ... PASTE HERE ...
     </t>
   </t>
   ```
8. **Preserve all accents and special characters** -- Spanish text must include proper tildes and accents (a, e, i, o, u, n, u). Never strip diacritics.

## Page Composition Pattern

Compose pages by combining components from `references/components-catalog.md`. Typical structure:

```
Hero section (iw-hero)
  |
Feature section (iw-section + cards or text+image)
  |
Stats section (iw-stat)
  |
Testimonial section (iw-testimonial)
  |
CTA section (iw-bg-gradient-space)
```
