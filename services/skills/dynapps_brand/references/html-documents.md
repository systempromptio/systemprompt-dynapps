# HTML Documents — Dynapps Brand Implementation

This reference covers the proven patterns for building on-brand Dynapps HTML documents. These patterns come from two successfully delivered documents (Website Strategy 2026, Marketing Onboarding 2026) and encode the visual identity into reusable CSS and structural conventions.

## Document Structure

Every Dynapps HTML document follows this skeleton:

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Dynapps — {{DOCUMENT_TITLE}}</title>
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=Readex+Pro:wght@200;300;400;500;600;700&display=swap" rel="stylesheet">
  <style>
    /* Brand CSS custom properties (see SKILL.md for full set) */
    :root { ... }
    /* Reset, then component styles */
  </style>
</head>
<body>
  <nav>...</nav>
  <section class="hero">...</section>
  <!-- Content sections alternating dark/light -->
  <div class="footer-grad"></div>
  <footer>...</footer>
  <script>/* Scroll reveal observer */</script>
</body>
</html>
```

Everything is in a **single file** — no external CSS or JS files. This makes documents self-contained and portable.

---

## CSS Foundation

### Reset and Base

```css
*, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }
html { scroll-behavior: smooth; font-size: 16px; }
body {
  font-family: 'Readex Pro', system-ui, sans-serif;
  background: var(--midnight);
  color: var(--tw);
  overflow-x: hidden;
  -webkit-font-smoothing: antialiased;
}
```

### Content Width

Use a wrapper class to constrain content width while allowing full-bleed backgrounds:

```css
.wrap { max-width: 1020px; margin: 0 auto; padding: 0 3.5rem; }
```

For wider layouts (presentations, dashboards): `max-width: 1200px`.

---

## Navigation Bar

Fixed at top with glass morphism effect and brand gradient underline:

```css
nav {
  position: fixed; top: 0; left: 0; right: 0; z-index: 100;
  display: flex; align-items: center; justify-content: space-between;
  padding: 0 3.5rem; height: 54px;
  background: rgba(5,5,67, 0.88);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border-bottom: 1px solid rgba(255,255,255,0.06);
}

/* Gradient accent line under nav */
nav::after {
  content: ''; position: absolute; bottom: 0; left: 0; right: 0; height: 1px;
  background: var(--grad-h); opacity: 0.4;
}
```

Nav typically contains: logo on the left, document label or nav links on the right. Use pill-shaped links with `border-radius: 100px`.

**Alternative nav style** (dot navigation for presentation-style docs):
```css
.nav-dot {
  width: 8px; height: 8px; border-radius: 50%;
  background: var(--grey-600); cursor: pointer;
}
.nav-dot.active { background: var(--carmine); width: 24px; border-radius: 4px; }
```

---

## Hero Section

The hero sets the tone. It should be full-viewport height with the title anchored toward the bottom or centered:

```css
.hero {
  min-height: 100vh;
  display: flex; flex-direction: column; justify-content: flex-end; /* or center */
  padding: 120px 48px 80px;
  position: relative; overflow: hidden;
  background: var(--midnight);
}
```

### Ambient Background Effects

Add depth with very subtle brand-colored glows:

```css
/* Violet glow — top area */
.hero::before {
  content: '';
  position: absolute; top: -40%; right: -20%;
  width: 80vw; height: 80vw; border-radius: 50%;
  background: radial-gradient(circle, rgba(149,28,243,0.10) 0%, transparent 70%);
  pointer-events: none;
}

/* Carmine glow — bottom area */
.hero::after {
  content: '';
  position: absolute; bottom: -20%; left: -10%;
  width: 60vw; height: 60vw; border-radius: 50%;
  background: radial-gradient(circle, rgba(255,87,93,0.06) 0%, transparent 70%);
  pointer-events: none;
}
```

### Icon Pattern Background

For more textured heroes, use a geometric pattern at very low opacity:

```css
.hero-pattern {
  position: absolute; inset: 0; pointer-events: none; opacity: 0.04;
  background-image:
    radial-gradient(circle at 25% 25%, rgba(255,255,255,.8) 0%, rgba(255,255,255,.8) 18%, transparent 19%),
    radial-gradient(circle at 75% 75%, rgba(255,255,255,.8) 0%, rgba(255,255,255,.8) 18%, transparent 19%);
  background-size: 120px 120px;
  mask-image: radial-gradient(ellipse 80% 80% at 60% 50%, black 0%, transparent 100%);
}
```

### Hero Content

- **Badge/label**: Small pill at top — `font-size: 13px; font-weight: 500; letter-spacing: 0.5px; text-transform: uppercase;`
- **Title**: Large Readex Pro Bold, with accent word highlighted
- **Subtitle**: Regular weight, dimmed text (`var(--td)`), 1-2 lines max

---

## Content Sections

### Padding and Spacing

```css
section { padding: 120px 48px; position: relative; }
/* or for more compact documents: */
.section { padding: 7rem 0; }
```

### Section Number Pattern

For numbered sections (common in strategy/onboarding docs):
```css
.section-num {
  font-size: 13px; font-weight: 300; color: var(--grey-400);
  letter-spacing: 2px; margin-bottom: 16px;
}
```

Usage: `<div class="section-num">01</div>`

### Background Alternation

Alternate section backgrounds for visual rhythm. The three proven patterns:

1. **Dark** (default): `background: var(--midnight);` — white text
2. **Light**: `background: var(--snow);` or `var(--off-white);` — dark text, great for data-heavy sections
3. **Deep**: `background: var(--deep);` — creates subtle depth contrast against standard dark

On light sections, adjust accent colors: use `var(--violet)` instead of `var(--tangerine)` for highlights.

---

## Cards and Grids

### Cards on Dark Background

```css
.card {
  background: rgba(255,255,255,0.04);
  border: 1px solid rgba(255,255,255,0.06);
  border-radius: 16px;
  padding: 2rem;
}
.card:hover {
  background: rgba(255,255,255,0.06);
  border-color: rgba(255,87,93,0.15);
}
```

### Cards on Light Background

```css
.card-light {
  background: var(--white);
  border: 1px solid var(--grey-100);
  border-radius: 16px;
  padding: 2rem;
  box-shadow: 0 2px 12px rgba(0,0,0,0.04);
}
```

### Grid Layout

```css
.grid-2 { display: grid; grid-template-columns: repeat(2, 1fr); gap: 1.5rem; }
.grid-3 { display: grid; grid-template-columns: repeat(3, 1fr); gap: 1.5rem; }
.grid-4 { display: grid; grid-template-columns: repeat(4, 1fr); gap: 1.5rem; }

@media (max-width: 768px) {
  .grid-2, .grid-3, .grid-4 { grid-template-columns: 1fr; }
}
```

---

## Tags and Pills

For categorization and labeling:

```css
.tag {
  display: inline-block;
  padding: 0.25rem 0.7rem;
  border-radius: 100px;
  font-size: 0.7rem; font-weight: 600;
  letter-spacing: 0.04em;
}

/* Variants */
.tag-accent { background: rgba(255,87,93,.12); color: var(--carmine); border: 1px solid rgba(255,87,93,.22); }
.tag-violet { background: rgba(149,28,243,.10); color: var(--violet); border: 1px solid rgba(149,28,243,.16); }
.tag-muted  { background: rgba(255,255,255,.06); color: var(--td); border: 1px solid rgba(255,255,255,.1); }
```

---

## Footer

```css
.footer-grad { height: 3px; background: var(--grad-h); }

footer {
  padding: 1.8rem 3.5rem;
  display: flex; align-items: center; justify-content: space-between;
  font-size: 0.65rem; color: var(--lm);
  background: var(--deep);
}
```

Always include the gradient line (`footer-grad`) directly above the footer — it's a brand signature element.

Footer content typically: brand name on left, document meta or copyright on right.

---

## Scroll Reveal Animation

All Dynapps HTML documents use a scroll-triggered reveal animation. Elements start invisible and slide up as they enter the viewport:

```css
.reveal {
  opacity: 0;
  transform: translateY(40px);
  transition: opacity 0.7s ease, transform 0.7s ease;
}
.reveal.visible { opacity: 1; transform: translateY(0); }

/* Staggered delays for groups */
.reveal-delay-1 { transition-delay: 0.1s; }
.reveal-delay-2 { transition-delay: 0.2s; }
.reveal-delay-3 { transition-delay: 0.3s; }
```

JavaScript (place before `</body>`):

```javascript
const observer = new IntersectionObserver((entries) => {
  entries.forEach(e => { if (e.isIntersecting) e.target.classList.add('visible'); });
}, { threshold: 0.15 });

document.querySelectorAll('.reveal').forEach(el => observer.observe(el));
```

Apply `.reveal` to any element that should animate in. Use delay classes for staggered grid items.

---

## Responsive Breakpoints

```css
@media (max-width: 768px) {
  nav { padding: 0 1.5rem; }
  .hero { padding: 80px 24px 60px; }
  section { padding: 80px 24px; }
  .wrap { padding: 0 1.5rem; }
  footer { padding: 1.25rem 1.5rem; flex-direction: column; gap: 0.5rem; text-align: center; }
}
```

---

## Embedding Logo SVGs

For the nav logo, inline the SVG or reference it. When inlining, the logo should be ~20px height in the nav bar:

```css
.nav-logo img { height: 20px; display: block; }
```

For hero or splash usage, the vertical logo with Odoo baseline works well at larger sizes.

---

## Checklist for HTML Documents

Before delivering, verify:

- [ ] Single-file HTML (no external CSS/JS)
- [ ] Readex Pro loaded via Google Fonts with preconnect
- [ ] CSS custom properties using extended palette values
- [ ] Glass nav bar with gradient underline
- [ ] Hero section with ambient glow effects
- [ ] Sections alternate dark/light for rhythm
- [ ] Scroll reveal animation on content elements
- [ ] Gradient accent line above footer
- [ ] Responsive styles for mobile
- [ ] Logo uses correct variant for background
- [ ] Accent words highlighted in headlines
