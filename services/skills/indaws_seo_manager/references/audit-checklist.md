# SEO Audit Checklist — dynapps.es

## Technical SEO

### Crawlability & Indexation
- [ ] robots.txt — check directives, no accidental blocks
- [ ] sitemap.xml — present, submitted, all important pages included
- [ ] Indexation ratio — indexed pages vs submitted (Search Console / DataForSEO)
- [ ] Orphan pages — pages not linked from anywhere
- [ ] Crawl depth — important pages within 3 clicks from homepage
- [ ] Canonical tags — correct on all pages, no self-referencing loops
- [ ] Noindex tags — verify only intended pages are noindexed
- [ ] Redirect chains — max 1 hop, no loops
- [ ] 404 errors — identify and fix/redirect broken pages
- [ ] 5xx errors — server issues

### Performance
- [ ] Core Web Vitals (per PageSpeed Insights / DataForSEO):
  - LCP (Largest Contentful Paint) < 2.5s
  - FID (First Input Delay) < 100ms
  - CLS (Cumulative Layout Shift) < 0.1
- [ ] Page speed mobile (target: >70 score)
- [ ] Page speed desktop (target: >85 score)
- [ ] Image optimization (WebP, lazy loading, proper sizing)
- [ ] CSS/JS minification
- [ ] Server response time (TTFB < 200ms)
- [ ] Gzip/Brotli compression

### Security & Structure
- [ ] HTTPS on all pages
- [ ] HTTP → HTTPS redirects working
- [ ] www → non-www (or vice versa) canonical redirect
- [ ] Hreflang tags (ES/EN) correct
- [ ] Structured data / Schema markup (Organization, BreadcrumbList, Article, FAQ)
- [ ] Mobile-friendliness (responsive, no horizontal scroll)
- [ ] Favicon present

---

## On-Page SEO

### Title Tags
- [ ] Every page has a unique title
- [ ] Title length: 50-60 characters
- [ ] Primary keyword in title (preferably at beginning)
- [ ] Brand name at end: "... | Dynapps"
- [ ] No duplicate titles across site
- [ ] Compelling for CTR (not just keyword stuffing)

### Meta Descriptions
- [ ] Every page has a unique meta description
- [ ] Length: 150-160 characters
- [ ] Primary keyword included naturally
- [ ] Call to action or value proposition
- [ ] No duplicate descriptions

### Headings
- [ ] Only one H1 per page
- [ ] H1 includes primary keyword
- [ ] Logical H1 → H2 → H3 hierarchy
- [ ] No skipped heading levels
- [ ] Headings describe content (not generic "Read more")

### URLs
- [ ] Short, descriptive slugs
- [ ] Keywords in URL where natural
- [ ] No special characters, underscores, or uppercase
- [ ] Consistent structure: /blog/[slug], /odoo-[service]

### Images
- [ ] All images have descriptive alt text
- [ ] Alt text includes keywords where relevant (not forced)
- [ ] File names descriptive (not IMG_001.jpg)
- [ ] Images properly sized (not oversized then CSS-resized)

### Internal Linking
- [ ] Key pages linked from homepage
- [ ] Blog posts link to related service pages
- [ ] Case studies link to relevant modules/services
- [ ] Descriptive anchor text (not "click here")
- [ ] No excessive links per page (< 100)
- [ ] Breadcrumb navigation present

---

## Content SEO

### Quality & Depth
- [ ] No thin content (all indexed pages > 300 words minimum, ideally > 800)
- [ ] Content matches search intent for target keyword
- [ ] Content more comprehensive than top-ranking competitors
- [ ] Up-to-date information (no references to "Odoo 15" as "latest")
- [ ] Unique content (no duplicate/near-duplicate pages)

### Keyword Strategy
- [ ] Each important page targets a specific primary keyword
- [ ] No keyword cannibalization (multiple pages targeting same keyword)
- [ ] Long-tail keywords covered in blog posts
- [ ] Topic clusters defined (pillar page + supporting content)
- [ ] Content gap vs competitors filled

### Content Freshness
- [ ] Last update date visible on blog posts
- [ ] Outdated posts updated or redirected
- [ ] Regular publishing schedule (minimum 2 posts/month for SEO)
- [ ] Seasonal content refreshed annually

---

## Off-Page SEO

### Backlink Profile
- [ ] Total referring domains (current: ~202)
- [ ] Domain authority / AS score (current: 21)
- [ ] Toxic links identified and disavowed
- [ ] Anchor text distribution natural
- [ ] Follow vs nofollow ratio healthy (~99% follow is fine)
- [ ] Referring domain growth trend

### Competitor Gap
- [ ] Domains linking to competitors but not Dynapps identified
- [ ] High-value link opportunities prioritized
- [ ] Partner directories and Odoo ecosystem links claimed

### Local SEO (if applicable)
- [ ] Google Business Profile optimized
- [ ] NAP (Name, Address, Phone) consistent across web
- [ ] Local keywords targeted ("odoo valencia", "erp valencia")

---

## Audit Data Collection Scripts

```bash
# Run full automated audit
python scripts/seo_audit.py --domain dynapps.es --full

# Run specific sections
python scripts/seo_audit.py --domain dynapps.es --technical
python scripts/seo_audit.py --domain dynapps.es --onpage
python scripts/seo_audit.py --domain dynapps.es --content
python scripts/seo_audit.py --domain dynapps.es --backlinks
python scripts/seo_audit.py --domain dynapps.es --competitors
```

## Audit Frequency

| Section | Frequency | Trigger |
|---------|-----------|---------|
| Technical | Monthly | + after site updates |
| On-page | Monthly | + after content batch |
| Content | Quarterly | + when strategy changes |
| Backlinks | Monthly | Standard |
| Competitors | Quarterly | Standard |
| Full audit | Quarterly | Standard |
