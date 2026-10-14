# Weekly SEO Report — Template

Generate every Tuesday at 6:00 AM CET. Data collected Monday night.

## Quick Generation

```bash
# 1. Collect fresh data (Monday night)
python scripts/rank_tracker.py --keywords target --save
python scripts/seo_audit.py --keywords

# 2. Generate HTML report (Tuesday morning)
python scripts/generate_html_reports.py --type weekly

# Output: data/reports/weekly-seo-report.html
```

---

## Visual Dashboard Structure

The HTML template (`templates/weekly-seo-report.html`) generates a bento-grid dashboard:

```
┌─────────────────────────┬─────────────────┐
│       SEO SCORE         │    Keywords     │
│       81/100            │   Rankings      │
│    (hero card)          │  TOP 3/10/50    │
├─────────────────────────┼─────────────────┤
│    Keyword Movements    │   Movimientos   │
│    ▲12  ▼3  ★2 new      │   Destacados    │
├──────────┬──────────────┼─────────────────┤
│ Indexed  │  Backlinks   │  Domain Score   │
│ Pages    │              │                 │
├──────────┴──────────────┼─────────────────┤
│   Top 5 Acciones        │ Technical Issues│
│   Esta Semana           │ 4xx, 5xx, etc   │
├─────────────────────────┴─────────────────┤
│         Top Pages por Rendimiento         │
├─────────────────────────┬─────────────────┤
│    Core Web Vitals      │   Competidores  │
│    LCP / FID / CLS      │  Keywords Share │
└─────────────────────────┴─────────────────┘
```

---

## Report Structure (Telegram Summary)

```markdown
# Reporte SEO Semanal — Dynapps
## Semana [fecha lunes] al [fecha viernes]

### Dashboard
| Metrica | Esta semana | Anterior | Cambio |
|---------|-------------|----------|--------|
| Keywords TOP 3 | 8 | 6 | +2 |
| Keywords TOP 10 | 24 | 22 | +2 |
| Keywords TOP 50 | 89 | 85 | +4 |
| Trafico organico | — | — | — |
| Paginas indexadas | 120 | 118 | +2 |
| Errores tecnicos | 3 | 5 | -2 |

### Movimientos Destacados
**Subidas:** "odoo partner" +4, "consultoria erp" +3
**Bajadas:** "implementar odoo" -2
**Nuevas TOP 10:** "odoo modulos"

### Top 5 Acciones
1. [ALTA] Añadir meta description a 12 paginas
2. [ALTA] Corregir 3 errores 404
3. [MEDIA] Expandir contenido en 5 paginas thin
4. [MEDIA] Añadir alt text a 45 imagenes
5. [BAJA] Optimizar titulos demasiado largos

### Para Aprobacion
- Actualizar meta description en /odoo-consultoria
- Redirigir /old-page a /new-page
- Publicar nuevo post sobre [tema]

Ver dashboard completo: [link a HTML]
```

---

## Data Collection Checklist (Monday Night)

- [ ] `rank_tracker.py --keywords target --save` — Rankings
- [ ] `seo_audit.py --keywords` — Keyword inventory
- [ ] Check Search Console for new issues
- [ ] Review competitor changes
- [ ] Check changelog.json for pending actions

## Report Quality Checklist

- [ ] Every action has clear "what to do"
- [ ] Impact estimates backed by data
- [ ] No action without target keyword and URL
- [ ] Competitor data is current
- [ ] Changelog complete for executed changes
- [ ] Brand compliance noted for content recommendations

## Delivery Checklist

- [ ] HTML report generated and verified
- [ ] Telegram summary sent at 6:00 AM CET
- [ ] Victor reviews and approves actions
- [ ] Approved actions logged for execution
- [ ] Results tracked for next week

---

## Metrics Tracked

### Primary Metrics
- **Keywords TOP 3/10/50**: Position distribution
- **Keyword movements**: Subidas vs bajadas
- **New entries**: Keywords entering TOP 10/20

### Secondary Metrics
- **Domain Score**: DataForSEO rank metric
- **Referring Domains**: Backlink quality indicator
- **Technical Issues**: 4xx, 5xx, missing metas

### Core Web Vitals
- **LCP** (Largest Contentful Paint): Target < 2.5s
- **FID** (First Input Delay): Target < 100ms
- **CLS** (Cumulative Layout Shift): Target < 0.1

---

## Action Categories

| Type | Examples | Approval Needed |
|------|----------|-----------------|
| **Technical** | Fix 404s, add canonical tags | No (< 10 changes) |
| **On-Page** | Update meta descriptions, titles | No (batch < 20) |
| **Content** | New posts, major rewrites | Yes |
| **Structure** | URL changes, redirects | Yes |
| **Backlinks** | Outreach, disavow | Yes |
