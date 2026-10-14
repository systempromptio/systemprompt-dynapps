# Charts — v19 New Features

## New Odoo Chart Types (9 new, 12 total)

v18 has 3 types: `odoo_bar`, `odoo_line`, `odoo_pie`. v19 adds **9 more**:

### Odoo Funnel Chart
```json
{
  "type": "odoo_funnel",
  "title": {"text": "Pipeline Funnel", "bold": true, "color": "#01666B"},
  "cumulative": true, "dataSets": [{}],
  "actionXmlId": "crm_enterprise.crm_opportunity_action_dashboard",
  "metaData": {"groupBy": ["stage_id"], "measure": "expected_revenue", "resModel": "crm.lead"},
  "searchParams": {"domain": [], "groupBy": ["stage_id"], "orderBy": []},
  "fieldMatching": {}
}
```

### Odoo Geo Chart (map)
```json
{
  "type": "odoo_geo",
  "title": {"text": "Revenue by Country"},
  "colorScale": "blues", "dataSets": [],
  "actionXmlId": "sale.action_order_report_all",
  "metaData": {"groupBy": ["partner_id.country_id"], "measure": "price_subtotal", "resModel": "sale.report"},
  "searchParams": {"domain": [], "groupBy": ["partner_id.country_id"], "orderBy": []},
  "fieldMatching": {}
}
```
- `colorScale`: `"blues"`, `"greens"`, `"reds"`, etc.
- Requires topojson data (loaded automatically by `geo_json_service`)

### Odoo Treemap Chart
```json
{
  "type": "odoo_treemap", "dataSets": [],
  "metaData": {"groupBy": ["categ_id"], "measure": "price_subtotal", "resModel": "sale.report"},
  "searchParams": {"domain": [], "groupBy": ["categ_id"], "orderBy": []},
  "fieldMatching": {}
}
```

### Odoo Sunburst Chart
```json
{
  "type": "odoo_sunburst", "dataSets": [],
  "metaData": {"groupBy": ["pos_categ_id", "product_id"], "measure": "price_subtotal", "resModel": "pos.order.line"},
  "searchParams": {"domain": [], "groupBy": ["pos_categ_id", "product_id"], "orderBy": []},
  "fieldMatching": {}
}
```
- Multiple `groupBy` creates hierarchical levels

### Odoo Waterfall Chart
```json
{
  "type": "odoo_waterfall", "dataSets": [],
  "metaData": {"groupBy": ["date:month"], "measure": "balance", "resModel": "account.move.line"},
  "searchParams": {"domain": [], "groupBy": ["date:month"], "orderBy": []},
  "fieldMatching": {}
}
```
- Shows incremental positive/negative contributions to a total

### Odoo Pyramid Chart
```json
{
  "type": "odoo_pyramid", "dataSets": [],
  "metaData": {"groupBy": ["age_range"], "measure": "__count", "resModel": "hr.employee"},
  "searchParams": {"domain": [], "groupBy": ["age_range"], "orderBy": []},
  "fieldMatching": {}
}
```
- Horizontal symmetric bars for demographic/comparative data

### Odoo Radar Chart
```json
{
  "type": "odoo_radar", "dataSets": [],
  "metaData": {"groupBy": ["skill_type_id"], "measure": "level_progress", "resModel": "hr.employee.skill"},
  "searchParams": {"domain": [], "groupBy": ["skill_type_id"], "orderBy": []},
  "fieldMatching": {}
}
```
- Spider/radar web for multidimensional comparison

### Odoo Scatter Chart
```json
{
  "type": "odoo_scatter", "dataSets": [],
  "metaData": {"groupBy": ["partner_id"], "measure": "amount_total", "resModel": "sale.order"},
  "searchParams": {"domain": [], "groupBy": ["partner_id"], "orderBy": []},
  "fieldMatching": {}
}
```
- XY correlation plot

### Odoo Combo Chart
```json
{
  "type": "odoo_combo", "dataSets": [],
  "metaData": {"groupBy": ["date:month"], "measure": "price_subtotal", "resModel": "sale.report"},
  "searchParams": {"domain": [], "groupBy": ["date:month"], "orderBy": []},
  "fieldMatching": {}
}
```
- Combines bars + line on the same chart (dual axis)

### Complete Chart Type Reference

| Type | Category | Best for |
|------|----------|----------|
| `odoo_bar` | Comparison | Categorical comparisons, rankings |
| `odoo_line` | Trend | Time series, evolution |
| `odoo_pie` | Distribution | Part-of-whole, proportions |
| `odoo_funnel` | Process | Pipeline stages, conversion rates |
| `odoo_geo` | Geographic | Revenue by country/region |
| `odoo_treemap` | Hierarchy | Proportional categories (flat) |
| `odoo_sunburst` | Hierarchy | Multi-level nested categories |
| `odoo_waterfall` | Flow | Incremental contributions (+/-) |
| `odoo_pyramid` | Comparison | Demographic/symmetric data |
| `odoo_radar` | Multi-axis | Skill assessment, multidimensional |
| `odoo_scatter` | Correlation | Variable relationships |
| `odoo_combo` | Combined | Bars + trend line together |

## New Chart Properties (on ALL chart types)

| Property | Type | Description |
|----------|------|-------------|
| `chartId` | UUID | **Required in v19**, same as figure `id` |
| `dataSets` | array | **Required** on all `odoo_*` types (can be `[]` or `[{}]`) |
| `actionXmlId` | string | Links chart → Odoo action for click-through |
| `humanize` | boolean | Format numbers as 1.2k, 3.5M |
| `axesDesign` | object | `{"x": {"title": {"text": ""}}, "y": {"title": {"text": "Revenue"}}}` |

## Enriched Titles (v19)

```json
"title": {"text": "Pipeline", "bold": true, "fontSize": 21, "color": "#01666B"}
```

Applies to chart titles and carousel titles.

## Carousel Figure

**New figure type** — multi-view container users can flip between:

```json
{
  "id": "carousel-uuid",
  "col": 0, "row": 6, "offset": {"x": 0, "y": 11},
  "width": 1000, "height": 362,
  "tag": "carousel",
  "data": {
    "title": {"text": "Sales Analysis", "fontSize": 21, "color": "#01666B", "bold": true},
    "chartDefinitions": {
      "chart-uuid-1": {"type": "odoo_bar", "metaData": {...}, "searchParams": {...}},
      "chart-uuid-2": {"type": "odoo_geo", "metaData": {...}, "searchParams": {...}}
    },
    "items": [
      {"type": "chart", "chartId": "chart-uuid-1", "title": "Bar Chart"},
      {"type": "chart", "chartId": "chart-uuid-2", "title": "Map"},
      {"type": "carouselDataView", "title": "Top 10"}
    ],
    "fieldMatching": {}
  }
}
```

| Property | Description |
|----------|-------------|
| `tag` | `"carousel"` |
| `chartDefinitions` | Dict of chart definitions by UUID |
| `items` | Ordered views: `"chart"` (refs chartDefinitions) or `"carouselDataView"` (auto data table) |

**All v19 new charts can live INSIDE carousels** or as standalone figures.

## Zoomable Time-Based Charts (NEW in v19)

Charts with temporal X-axis support interactive zoom and navigation:

```json
{
  "type": "odoo_line",
  "zoomable": true,
  "metaData": {"groupBy": ["date:month"], "measure": "amount_total", "resModel": "sale.report"},
  ...
}
```

| Property | Type | Description |
|----------|------|-------------|
| `zoomable` | boolean | Enables zoom/pan on temporal axis. Default `false` |

- Applies to: `odoo_bar`, `odoo_line`, `odoo_combo`, `odoo_waterfall` (charts with time X-axis)
- Enabled **by default in fullscreen Dashboard view**
- Can be manually activated in spreadsheets
- Uses ChartJS zoom plugin (`zoomWindowPlugin`) — drag to zoom, scroll to pan

## Fullscreen Figures (NEW in v19)

Any figure can be expanded to fullscreen view:

- `FullScreenFigureStore` manages the active fullscreen figure
- `toggleFullScreenFigure(figureId)` — click to expand, click again to collapse
- In fullscreen: `width` and `height` become viewport-sized, `x: 0, y: 0`
- Especially useful in dashboards with many small charts

## Scorecard Changes (v18 → v19)

- `baselineDescr` format: `"text"` (string) → `{"text": "text"}` (object with text property)
- Migration 18.4.2 handles this automatically for existing data
