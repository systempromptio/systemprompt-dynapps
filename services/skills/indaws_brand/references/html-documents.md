# HTML Document Generation Reference

## Table of Contents
1. [Base Template](#base-template)
2. [CSS Custom Properties](#css-custom-properties)
3. [Font Embedding](#font-embedding)
4. [Document Structure](#document-structure)
5. [Component Patterns](#component-patterns)
6. [Dark Background Variant](#dark-background-variant)
7. [Responsive Guidelines](#responsive-guidelines)

---

## Base Template

Start from `assets/templates/base.html` for all HTML documents. It includes:
- Font declarations pointing to `assets/fonts/`
- CSS custom properties for all brand colors
- Header with white logo on Blue Space background
- Main content area
- Footer with logo and copyright

Replace `{{DOCUMENT_TITLE}}`, `{{CONTENT}}`, and `{{YEAR}}` placeholders.

## CSS Custom Properties

```css
:root {
  --indaws-warm-yellow: #E5B92B;
  --indaws-blue-lilac: #6B68FA;
  --indaws-blue-space: #1C265D;
  --indaws-light-sky: #8AC2DB;
  --indaws-white: #FFFFFF;
  --indaws-gradient-blue: linear-gradient(135deg, #6B68FA 0%, #8AC2DB 100%);
  --indaws-gradient-yellow: linear-gradient(135deg, #8AC2DB 0%, #E5B92B 100%);
}
```

Always reference these variables in CSS. Never hardcode hex values outside of the `:root` block.

## Font Embedding

```css
@font-face {
  font-family: 'Montserrat';
  src: url('../fonts/Montserrat-VariableFont_wght.ttf') format('truetype');
  font-weight: 100 900;
  font-style: normal;
  font-display: swap;
}
@font-face {
  font-family: 'Montserrat';
  src: url('../fonts/Montserrat-Italic-VariableFont_wght.ttf') format('truetype');
  font-weight: 100 900;
  font-style: italic;
  font-display: swap;
}
```

Font stack: `font-family: 'Montserrat', Verdana, sans-serif;`

## Document Structure

```html
<header class="header">         <!-- Blue Space bg, white logo top-left -->
  <div class="container">
    <img src="../logos/logotipo-indaws-color-white-gradient_blue.svg" alt="Indaws">
  </div>
</header>

<main class="main-content">    <!-- White bg by default -->
  <div class="container">
    <h1>DOCUMENT TITLE</h1>    <!-- Uppercase, Bold, Blue Lilac -->
    <p>Body content...</p>     <!-- Regular, Blue Space -->
  </div>
</main>

<footer class="footer">        <!-- Blue Space bg, white logo bottom-left -->
  <div class="container">
    <img src="../logos/logotipo-indaws-color-white-gradient_blue.svg" alt="Indaws">
    <span>&copy; 2026 Indaws. Todos los derechos reservados.</span>
  </div>
</footer>
```

## Component Patterns

### CTA Button
```html
<a href="#" class="cta-button">Solicita tu consulta</a>
```
Warm Yellow background, Blue Space text, Bold uppercase.

### Section with Blue Space Background
```html
<section class="bg-blue-space">
  <div class="container">
    <h2>Section Title</h2>  <!-- White, uppercase -->
    <p>White text content on dark background.</p>
  </div>
</section>
```

### Gradient Background Section
```html
<section class="bg-gradient">
  <div class="container">
    <h2>Innovation Section</h2>
    <p>White text on blue-to-sky gradient.</p>
  </div>
</section>
```

### Highlighted Text
```html
<span class="text-accent">Warm Yellow accent text</span>
<span class="text-lilac">Blue Lilac emphasis</span>
```

### Data Table
```html
<table style="width:100%; border-collapse:collapse;">
  <thead>
    <tr style="background-color:var(--indaws-blue-space); color:var(--indaws-white);">
      <th style="padding:0.75rem; text-align:left; font-weight:700; text-transform:uppercase;">Column</th>
    </tr>
  </thead>
  <tbody>
    <tr style="border-bottom:1px solid var(--indaws-light-sky);">
      <td style="padding:0.75rem;">Data</td>
    </tr>
  </tbody>
</table>
```

## Dark Background Variant

For full dark-themed pages, add `class="dark-theme"` to `<body>`:
- Background: Blue Space
- Body text: White
- Headings H1/H2: White
- Headings H3/H4: Light Sky
- Use `logotipo-indaws-color-white-gradient_blue.svg` for header and footer

## Responsive Guidelines

- Max container width: 1200px
- Side padding: 2rem (mobile: 1rem)
- Font sizes scale down on mobile:
  - H1: 2rem (from 2.5rem)
  - H2: 1.5rem (from 1.75rem)
  - Body: 1rem (unchanged)
- Logo minimum width: 70px (never smaller)
- Stack footer elements vertically on mobile

```css
@media (max-width: 768px) {
  .container { padding: 0 1rem; }
  h1 { font-size: 2rem; }
  h2 { font-size: 1.5rem; }
  .header .container,
  .footer .container { flex-direction: column; gap: 1rem; }
}
```
