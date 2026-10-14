> **LEGACY (julio 2026):** Este documento describe el blog Odoo del antiguo www.indaws.es. El sitio actual www.dynapps.es usa otro CMS: valida estructura, URLs y CTAs contra el sitio nuevo antes de aplicar nada. Los principios SEO de contenido siguen vigentes.

# Blog Content Creation Guide — indaws.es

## Overview

This guide covers how to create high-quality blog posts for indaws.es that follow the site's design system, brand guidelines, and SEO best practices. Blog posts on indaws.es use Odoo's blog engine (`blog.post` model) with custom HTML content that leverages Tailwind CSS + Bootstrap for professional layouts.

---

## Tech Stack for Blog Content

### Required Header Patch

Every blog post that uses custom styling MUST include this hidden header at the start of the `content` field:

```html
<div class="o_not_editable" contenteditable="false" style="display: none;">
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Montserrat:wght@400;500;700;900&display=swap" rel="stylesheet">
    <script src="https://cdn.tailwindcss.com"></script>
    <script>
        tailwind.config = {
            corePlugins: { preflight: false },
            theme: {
                extend: {
                    colors: {
                        'brand-lilac': '#6B68FA',
                        'brand-space': '#1C265D',
                        'brand-warm': '#E5B92B',
                        'brand-sky': '#8AC2DB',
                    },
                    fontFamily: { sans: ['Montserrat', 'sans-serif'] },
                    boxShadow: {
                        'glass': '0 8px 32px 0 rgba(31, 38, 135, 0.37)',
                        'neon': '0 0 20px rgba(107, 104, 250, 0.5)',
                    }
                }
            }
        }
    </script>
    <style>
        .text-brand-space { color: #1C265D !important; }
        .bg-brand-space { background-color: #1C265D !important; }
        .font-black { font-weight: 900 !important; }
    </style>
</div>
```

### CSS Framework

- **Tailwind CSS** via CDN with `preflight: false` (critical — avoids breaking Odoo/Bootstrap base styles)
- **Bootstrap** grid system (container, row, col-*) already loaded by Odoo
- **Montserrat** font via Google Fonts
- Brand colors via Tailwind config

### Section Pattern

Each content section uses this pattern:

```html
<section class="s_text_block py-5 bg-white" data-snippet="s_section_name">
    <div class="container">
        <div class="row justify-content-center">
            <div class="col-lg-10">
                <!-- Content here -->
            </div>
        </div>
    </div>
</section>
```

Alternate between `bg-white` and `bg-gray-50` for visual separation.

---

## Blog Post Structure

Every comparison or long-form blog post should include ALL of these sections:

### 1. Hero Section
Dark background (brand-space), decorative gradient blurs, main title, subtitle.

```html
<section class="s_text_block py-5 bg-brand-space position-relative overflow-hidden">
    <div class="absolute inset-0 pointer-events-none o_not_editable" contenteditable="false">
        <div class="absolute top-[-15%] right-[-10%] w-[600px] h-[600px] bg-brand-lilac rounded-full blur-[150px] opacity-25"></div>
    </div>
    <div class="container position-relative z-10">
        <div class="row justify-content-center">
            <div class="col-lg-10 text-center">
                <span class="inline-block px-4 py-2 bg-brand-lilac/20 text-brand-lilac rounded-full text-sm font-bold uppercase tracking-wider border border-brand-lilac/30">Category Badge</span>
                <h1 class="font-sans font-black text-4xl lg:text-6xl text-white mb-4 uppercase leading-tight">Title</h1>
                <p class="font-sans text-xl text-gray-300 leading-relaxed mb-5">Subtitle/hook</p>
            </div>
        </div>
    </div>
</section>
```

### 2. Table of Contents
Numbered navigation card with anchor links.

### 3. Content Sections
Two-column layouts where appropriate (text + visual element).

### 4. Comparison Table
Styled with branded header, badges for values, hover effects:

```html
<div class="table-responsive rounded-3xl border border-gray-200 shadow-2xl">
    <table class="table mb-0 bg-white">
        <thead class="bg-brand-space text-white">
            <tr class="uppercase text-xs tracking-wider">
                <th class="p-4 fw-bold border-0">Factor</th>
                <th class="p-4 fw-bold text-center bg-brand-lilac border-0">Odoo</th>
                <th class="p-4 fw-bold text-center border-0">Competitor</th>
            </tr>
        </thead>
        <tbody class="text-gray-700 text-sm">
            <tr class="hover:bg-gray-50 transition">
                <td class="p-4 fw-bold">Feature</td>
                <td class="p-4 text-center bg-brand-lilac/5">
                    <span class="badge bg-green-100 text-green-800 px-3 py-1 rounded-full border border-green-200">Value</span>
                </td>
                <td class="p-4 text-center">Value</td>
            </tr>
        </tbody>
    </table>
</div>
```

### 5. YouTube Video Embed
Always include at least one relevant video:

```html
<div style="border-radius: 20px; overflow: hidden; box-shadow: 0 10px 30px rgba(0,0,0,0.1); border: 1px solid #e5e5e5;">
    <div class="ratio ratio-16x9">
        <iframe src="https://www.youtube.com/embed/VIDEO_ID" title="Video Title" allowfullscreen style="border:0;"></iframe>
    </div>
    <div style="padding: 20px; background: white;">
        <p style="font-size: 0.75rem; text-transform: uppercase; color: #8AC2DB; font-weight: bold; letter-spacing: 1px; margin-bottom: 5px;">INDAWS ACADEMY</p>
        <h4 style="font-size: 1rem; color: #1C265D; margin: 0;">Video caption</h4>
    </div>
</div>
```

Available video IDs (known Indaws videos):
- `CP96yVnXNrY` — Main Indaws video
- `crX-Y_1j4bc` — Used in multiple posts
- `5C0L6ObKjd0` — Odoo v19 features
- `gUxcjyFVeU0` — Odoo career guide

### 6. Callout/Insight Boxes

```html
<div class="bg-white rounded-2xl p-4 border-l-4 border-brand-warm shadow-sm">
    <h4 class="fw-bold text-sm text-brand-space mb-2">Key Insight Title</h4>
    <p class="text-gray-500 text-sm mb-0">Insight content here.</p>
</div>
```

### 7. CTA Section
Dark background with compelling copy:

```html
<section class="s_text_block py-5 bg-brand-space">
    <div class="container">
        <div class="row justify-content-center">
            <div class="col-lg-9 text-center">
                <h2 class="font-sans font-black text-3xl lg:text-5xl text-white mb-6">CTA Title</h2>
                <p class="text-xl text-gray-300 mb-8 leading-relaxed">Supporting copy.</p>
                <a href="/contacto-indaws" class="btn btn-lg px-8 py-4 rounded-full bg-brand-lilac text-white fw-bold shadow-neon border-0 text-lg hover:scale-105 transition-transform">CTA Button Text</a>
            </div>
        </div>
    </div>
</section>
```

### 8. Related Articles Section
Grid of 3-4 related posts with internal links.

### 9. JSON-LD Schema
Article + FAQ structured data at the end:

```html
<script type="application/ld+json">
{
    "@context": "https://schema.org",
    "@type": "Article",
    "headline": "Post Title",
    "description": "Meta description",
    "author": { "@type": "Organization", "name": "Indaws" },
    "publisher": { "@type": "Organization", "name": "Indaws", "logo": { "@type": "ImageObject", "url": "https://www.indaws.es/web/image/website/1/logo" } },
    "datePublished": "2026-02-02",
    "dateModified": "2026-02-02"
}
</script>
```

---

## Internal Linking Strategy

### Mandatory Links per Post Type

**Comparison posts ("Odoo vs X"):**
- Link to pillar page: `/blog/blog-indaws-1/odoo-vs-competidores-comparativa-definitiva-de-erps-2026-778`
- Link to `/que-es-odoo` (first mention of "qué es Odoo")
- Link to `/contacto-indaws` for CTAs (at least 2: mid-article + CTA section)
- Link to relevant app pages when mentioning features:
  - CRM → `/odoo-crm`
  - Inventario → `/odoo-inventario`
  - Fabricación → `/odoo-fabricacion`
  - Contabilidad → `/odoo-contabilidad`
  - Facturación → `/odoo-facturacion`
  - eCommerce → `/odoo-ecommerce`
  - VeriFactu → `/verifactu` or `/blog/blog-indaws-1/odoo-verifactu-597`
- Link to 2-3 related blog posts

**Link styling:**
```html
<a href="/url" style="color: #6B68FA; text-decoration: none; font-weight: 500;">anchor text</a>
```

### Bidirectional Linking
When creating a new cluster post, also update the pillar page to link back to it. This creates the hub-and-spoke model that Google rewards.

---

## Pre-Publication Checklist

### Before creating a new blog post, ALWAYS:

1. **Search for existing posts** on the same topic:
   ```bash
   curl -s -X POST "${ODOO_URL}/json/2/blog.post/search_read" \
     -H "Authorization: bearer ${ODOO_KEY}" \
     -d '{"domain": [["name","ilike","TOPIC"],["is_published","=",true]], "fields": ["id","name","visits","seo_name"]}'
   ```
2. **If an existing post exists with the same topic:**
   - UPDATE the existing post instead of creating a new one (preserves SEO authority/visits)
   - Only create a new post if the angle/intent is clearly different
3. **If creating new:** Check that the keyword intent doesn't overlap with existing posts

### Content Quality
- [ ] All Spanish accents/tildes correct (facturación, gestión, módulo, etc.)
- [ ] Tailwind + Bootstrap header patch included
- [ ] At least 1 YouTube video embedded
- [ ] At least 5 internal links to relevant pages
- [ ] Link to pillar page (if cluster post)
- [ ] CTA with link to /demo-odoo
- [ ] JSON-LD structured data
- [ ] Hero section with branded styling
- [ ] Styled comparison table (if comparison post)
- [ ] Related articles section at bottom
- [ ] Zero emojis in running text (star ratings in tables OK)
- [ ] Professional "tu" address
- [ ] Value vocabulary (no "coste", "problema", "precio")
- [ ] Meta title ≤ 60 chars with primary keyword
- [ ] Meta description ≤ 155 chars with primary keyword + CTA

### After Publishing
- [ ] Verify page loads correctly (visit URL)
- [ ] Update pillar page with link to new post (if cluster)
- [ ] Log change in `data/changelog.json`
- [ ] Check for cannibalization with existing posts

---

## Common Mistakes to Avoid

1. **Creating duplicate posts** instead of updating existing ones — always search first
2. **Plain HTML without CSS** — every post needs the Tailwind header patch and branded styling
3. **No internal links** — minimum 5 per post, including app pages and related posts
4. **No video** — always embed at least one relevant YouTube video
5. **Missing accents** — run accent check before publishing (common: facturación, gestión, módulo, código, versión, integración)
6. **No JSON-LD** — always include Article schema (helps rich results)
7. **Publishing immediately** — always create as unpublished (is_published: false) for review