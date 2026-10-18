# Keyword Strategy — Dynapps

## Keyword Categories

### Transactional (BOFU — highest conversion intent)

Target on service pages and homepage.

| Category | Example Keywords | Target Page Type |
|----------|-----------------|------------------|
| Brand + Service | "odoo partner espana", "implementador odoo" | Homepage, about |
| Service-specific | "consultoria odoo", "implantacion erp", "migracion odoo 19" | Service pages |
| Location-based | "odoo valencia", "erp valencia", "consultora odoo madrid" | Service pages + local |
| Comparison | "odoo vs sap", "odoo vs holded", "mejor erp pymes espana" | Blog (comparison posts) |
| Pricing | "odoo precio", "coste implementacion erp", "odoo enterprise precio" | Service / FAQ page |

### Informational (TOFU — attract organic traffic)

Target on blog posts and educational content.

| Category | Example Keywords | Target Page Type |
|----------|-----------------|------------------|
| What-is | "que es odoo", "que es un erp", "que es crm" | Blog (pillar content) |
| How-to | "como implementar odoo", "como migrar erp" | Blog (guides) |
| Tutorials | "crear producto odoo", "configurar inventario odoo" | Blog (tutorials) |
| Industry | "erp para manufactura", "software gestion restaurantes" | Blog (industry pages) |
| Trends | "tendencias erp 2026", "ia en erp", "digitalizacion pymes" | Blog (thought leadership) |

### Navigational (brand searches)

Protect and optimize.

| Category | Example Keywords |
|----------|-----------------|
| Brand | "dynapps", "dynapps odoo", "dynapps erp" |
| Brand + intent | "dynapps contacto", "dynapps precios", "dynapps casos exito" |

---

## Keyword Map Template

Assign primary + secondary keywords to each page:

```
PAGE: [URL]
├── Primary keyword: [keyword] (vol: X, diff: Y, pos: Z)
├── Secondary keywords: [kw1], [kw2], [kw3]
├── Search intent: informational / transactional / navigational
├── Current title: "[current title tag]"
├── Proposed title: "[optimized title tag]"
├── Current meta: "[current meta description]"
├── Proposed meta: "[optimized meta description]"
└── Content status: OK / Needs update / Needs creation
```

---

## Topic Clusters

Organize content into pillar + satellite structure for topical authority.

### Cluster 1: "ERP para PYMES"
- **Pillar page**: /que-es-odoo (or dedicated pillar)
- **Satellites**:
  - que es un erp (blog)
  - erp vs excel (blog)
  - como elegir erp (blog)
  - odoo vs competidores comparativa (existing blog)
  - caso exito [PYME] con odoo (case studies)

### Cluster 2: "Implementacion Odoo"
- **Pillar page**: /odoo-implementacion (service page)
- **Satellites**:
  - fases implementacion erp (blog)
  - errores implementacion erp (blog)
  - cuanto tarda implementar odoo (blog)
  - caso exito [implementacion rapida] (case study)
  - guia migracion odoo (blog)

### Cluster 3: "Modulos Odoo"
- **Pillar page**: /modulos-odoo (or existing page)
- **Satellites**:
  - odoo inventario (blog/service)
  - odoo contabilidad (blog/service)
  - odoo punto de venta (blog/service)
  - odoo ecommerce (blog/service)
  - odoo fabricacion (blog/service)

### Cluster 4: "Digitalizacion Empresas"
- **Pillar page**: /transformacion-digital
- **Satellites**:
  - digitalizacion pymes espana (blog)
  - subvenciones digitalizacion (blog — Digitaliza CV)
  - kit digital odoo (blog)
  - caso exito [transformacion] (case study)

---

## Keyword Research Process

### Finding New Keywords

1. **Seed keywords** → DataForSEO related_keywords endpoint
2. **Competitor gap** → keywords competitors rank for, we don't
3. **Search Console** → queries with impressions but low CTR (optimize existing pages)
4. **Content-collection** → match case studies/modules to untargeted keywords
5. **Reddit/forums** → questions people ask about Odoo/ERP

### Evaluating Keywords

Score each keyword:

| Factor | Weight | Measurement |
|--------|--------|-------------|
| Search volume | 30% | Monthly searches (DataForSEO) |
| Keyword difficulty | 25% | Competition score 0-100 |
| Business relevance | 25% | How close to Dynapps services |
| Current position | 20% | Already ranking = easier to improve |

**Priority formula**: `(Volume * 0.3) + ((100 - Difficulty) * 0.25) + (Relevance * 0.25) + (Position_bonus * 0.20)`

Position bonus: TOP 20 = 5, TOP 50 = 3, Not ranking = 1

### Keyword Refresh Schedule

- **Weekly**: Track target keywords rankings
- **Monthly**: Check for new keyword opportunities from competitors
- **Quarterly**: Full keyword research refresh, update strategy

---

## Cannibalization Detection

Two pages targeting the same keyword = both lose.

### Detection Method

1. Export all indexed pages with their ranking keywords (DataForSEO)
2. Find keywords where 2+ Dynapps pages appear
3. Choose the winner page (most relevant, best content)
4. Redirect or differentiate the loser page

### Resolution Options

| Situation | Solution |
|-----------|----------|
| 2 similar blog posts | Merge into one comprehensive post, 301 redirect |
| Blog post vs service page | Differentiate intent: blog = informational, service = transactional |
| Thin pages competing | Delete thin one, redirect to strong one |
| Different angles on same topic | Add canonical to preferred page, or internal link strategy |

---

## Geographic Strategy

Current traffic: MX 32%, ES 30%, PE 11% — significant LATAM audience.

### Considerations
- Primary target: Spain (ES) — main business market
- Secondary: LATAM — valuable for brand awareness and lead gen
- Hreflang tags should be correct (es-ES, es-MX if needed, en)
- Content in Spanish serves both markets naturally
- Consider Spain-specific keywords vs generic Spanish keywords
