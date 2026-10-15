# Navigation & Internationalization Reference

## Scorecard Filtered Navigation — The Problem

Scorecards show KPI numbers but **natively have no filtered drill-down**. Unlike `odoo_*` charts (bar/line/pie) which open filtered views when clicking a data point, scorecards use `chartOdooMenusReferences` which only calls `doAction(menu.actionID)` — no domain, no context, no filters.

| Feature | Scorecards | `odoo_*` charts (bar/line/pie) |
|---------|-----------|-------------------------------|
| Click opens view | Via `chartOdooMenusReferences` (action from menu) | Via drill-down (filtered by clicked segment) |
| Global filters applied on click | **No** | **Yes** (domain built from active filters + segment) |
| Custom domain on click | **Depends on solution** (see below) | **Yes** (per data point) |
| Implementation | `navigateToOdooMenu` → `doAction(actionID)` | `getChartDatasetActionCallbacks` → `navigateTo(actionXmlId, {domain})` |

### Three solutions exist (no custom JS required):

## Solution 1: `odoo://view` Links Below Scorecards

**The simplest and most flexible approach.** Place a clickable text link in the cell row immediately below each scorecard. The link contains a custom domain that filters the view.

```
┌─────────────────────────────┐
│  Ahorro anual (€)           │  ← Scorecard figure (visual only)
│       171k                  │
│  ▲ 30k vs periodo anterior  │
└─────────────────────────────┘
  Ver detalle →                  ← odoo://view link with filtered domain
```

**Pros**: Zero development, full domain control, works with any Odoo version
**Contras**: User clicks a text link, not the card itself

**When to use**: Always — this is the MANDATORY baseline navigation for every scorecard.

See the `odoo://view` section below for syntax and examples.

## Solution 2: Cell-Based "Fake Scorecards" With Links (RECOMMENDED)

Replace scorecard figures with **styled merged cells**. Each card = 3 rows (title + number + link), 2 columns wide. The link row contains an `odoo://view` link with filtered domain — the card area is visually cohesive and the link is clickable.

```
┌──────────────────────────────┐
│  Ahorro anual (€)            │  ← Row 1: Title (merged, fillColor, bold)
│       171,811                │  ← Row 2: Number (merged, fontSize 22, =Data!B2)
│  ▶ Ver detalle               │  ← Row 3: odoo://view link (merged, small font)
└──────────────────────────────┘
```

### Layout grid

4 cards per row, each 2 columns wide. 8 columns at 120px = 960px total.

```
Col:  A:B (card 0)  C:D (card 1)  E:F (card 2)  G:H (card 3)
      120+120px     120+120px     120+120px     120+120px
```

Each card occupies 3 consecutive rows:

| Row | Content | Height | Style |
|-----|---------|--------|-------|
| Title | Card label text | 28px | bold, fontSize 10, fillColor, verticalAlign bottom |
| Number | `=Data!B2` (formula ref) | 44px | bold, fontSize 22, fillColor, align center, verticalAlign middle |
| Link | `[▶ Ver detalle](odoo://view/...)` | 22px | fontSize 9, fillColor, verticalAlign top |

### Complete implementation pattern

#### 1. Style definitions (one set per background color)

```python
STYLES = {
    # Card title styles
    "10": {"bold": True, "fontSize": 10, "textColor": "#434343", "fillColor": "#F0F4F7", "verticalAlign": "bottom"},   # gray
    "11": {"bold": True, "fontSize": 10, "textColor": "#434343", "fillColor": "#ECFDF5", "verticalAlign": "bottom"},   # green
    "12": {"bold": True, "fontSize": 10, "textColor": "#434343", "fillColor": "#EFF6FF", "verticalAlign": "bottom"},   # blue
    "13": {"bold": True, "fontSize": 10, "textColor": "#434343", "fillColor": "#FFF7ED", "verticalAlign": "bottom"},   # orange
    "14": {"bold": True, "fontSize": 10, "textColor": "#434343", "fillColor": "#FEF2F2", "verticalAlign": "bottom"},   # pink
    # Card number styles
    "20": {"bold": True, "fontSize": 22, "textColor": "#1F1F1F", "fillColor": "#F0F4F7", "align": "center", "verticalAlign": "middle"},
    "21": {"bold": True, "fontSize": 22, "textColor": "#1F1F1F", "fillColor": "#ECFDF5", "align": "center", "verticalAlign": "middle"},
    "22": {"bold": True, "fontSize": 22, "textColor": "#1F1F1F", "fillColor": "#EFF6FF", "align": "center", "verticalAlign": "middle"},
    "23": {"bold": True, "fontSize": 22, "textColor": "#1F1F1F", "fillColor": "#FFF7ED", "align": "center", "verticalAlign": "middle"},
    "24": {"bold": True, "fontSize": 22, "textColor": "#1F1F1F", "fillColor": "#FEF2F2", "align": "center", "verticalAlign": "middle"},
    # Card link styles
    "30": {"fontSize": 9, "textColor": "#01666b", "fillColor": "#F0F4F7", "verticalAlign": "top"},
    "31": {"fontSize": 9, "textColor": "#01666b", "fillColor": "#ECFDF5", "verticalAlign": "top"},
    "32": {"fontSize": 9, "textColor": "#01666b", "fillColor": "#EFF6FF", "verticalAlign": "top"},
    "33": {"fontSize": 9, "textColor": "#01666b", "fillColor": "#FFF7ED", "verticalAlign": "top"},
    "34": {"fontSize": 9, "textColor": "#01666b", "fillColor": "#FEF2F2", "verticalAlign": "top"},
}
# Map: bg_key -> (title_style_id, number_style_id, link_style_id)
BG_STYLES = {
    "gray": ("10", "20", "30"), "green": ("11", "21", "31"),
    "blue": ("12", "22", "32"), "orange": ("13", "23", "33"), "pink": ("14", "24", "34"),
}
```

#### 2. Link builder (programmatic — avoids manual escaping)

```python
import json

def odoo_link(text, domain_str, model="project.task"):
    config = {
        "viewType": "list",
        "action": {
            "domain": domain_str,
            "modelName": model,
            "views": [[False, "list"], [False, "form"]]
        },
        "name": text
    }
    return f"[{text}](odoo://view/{json.dumps(config)})"
```

#### 3. Card builder function

```python
CARD_COLS = [("A", "B"), ("C", "D"), ("E", "F"), ("G", "H")]

def add_card(title, data_ref, domain, bg_key, col_idx, start_row):
    """Build one fake scorecard card at grid position."""
    c1, c2 = CARD_COLS[col_idx]
    ts, ns, ls = BG_STYLES[bg_key]

    r_title = start_row      # 0-indexed row
    r_number = start_row + 1
    r_link = start_row + 2

    # Title row (merge 2 cols)
    cells[f"{c1}{r_title+1}"] = {"content": f"  {title}"}
    merges.append(f"{c1}{r_title+1}:{c2}{r_title+1}")
    sheet_styles[f"{c1}{r_title+1}:{c2}{r_title+1}"] = int(ts)

    # Number row (formula reference to Data sheet)
    cells[f"{c1}{r_number+1}"] = {"content": f"={data_ref}"}
    merges.append(f"{c1}{r_number+1}:{c2}{r_number+1}")
    sheet_styles[f"{c1}{r_number+1}:{c2}{r_number+1}"] = int(ns)

    # Link row (odoo://view with filtered domain)
    cells[f"{c1}{r_link+1}"] = {"content": odoo_link("  ▶ Ver detalle", domain)}
    merges.append(f"{c1}{r_link+1}:{c2}{r_link+1}")
    sheet_styles[f"{c1}{r_link+1}:{c2}{r_link+1}"] = int(ls)

    # Row heights
    rows[str(r_title)] = {"size": 28}   # title
    rows[str(r_number)] = {"size": 44}  # number
    rows[str(r_link)] = {"size": 22}    # link
```

#### 4. Usage — building card rows

```python
# Row 1: 4 main KPI cards starting at row 2 (0-indexed)
cards_row1 = [
    ("Nº Tareas",           "Data!B8", '[["project_id","=",5]]',                                        "gray"),
    ("Ahorro anual (€)",    "Data!B2", '[["project_id","=",5]]',                                        "green"),
    ("Ahorro energético",   "Data!B3", '[["project_id","=",5]]',                                        "blue"),
    ("Reducción CO2",       "Data!B4", '[["project_id","=",5]]',                                        "gray"),
]
for i, (title, ref, domain, bg) in enumerate(cards_row1):
    add_card(title, ref, domain, bg, col_idx=i, start_row=2)

# Gap + section header after cards
rows["5"] = {"size": 10}  # spacing
cells["A7"] = {"content": "Medidas Recomendadas"}
# ... next card row at start_row=7
```

#### 5. Key JSON properties

```python
# Grid lines OFF for clean card appearance
"areGridLinesVisible": False

# Styles are range-based (sheet-level), not cell-level
"styles": {"A3:B3": 10, "A4:B4": 20, "A5:B5": 30}

# No scorecard figures, no chartOdooMenusReferences needed
"figures": [...]  # only odoo_* charts remain
"chartOdooMenusReferences": {}
```

### Visual result

```
┌────────────┐ ┌────────────┐ ┌────────────┐ ┌────────────┐
│ Nº Tareas  │ │ Ahorro (€) │ │ Ahorro kWh │ │ Reduc. CO2 │
│    245     │ │  170,811   │ │   88,847   │ │   14,411   │
│ ▶ Ver det. │ │ ▶ Ver det. │ │ ▶ Ver det. │ │ ▶ Ver det. │
└────────────┘ └────────────┘ └────────────┘ └────────────┘
```

### Background color palette

| Key | fillColor | Use for |
|-----|-----------|---------|
| gray | `#F0F4F7` | General counts, neutral KPIs |
| green | `#ECFDF5` | Positive KPIs (savings, revenue) |
| blue | `#EFF6FF` | Energy/technical KPIs |
| orange | `#FFF7ED` | Measures, recommendations |
| pink | `#FEF2F2` | Alerts, subscriptions |

**Pros**: Click on the link within the card, full domain control, no menus/actions needed, clean visual appearance with color-coded backgrounds, zero Odoo-side configuration
**Contras**: Loses native scorecard features (humanize, baseline comparison, animated values). Number formatting must be handled in Data sheet formulas. No click on the number itself (only on the link row).

**When to use**: When you want visually appealing KPI cards with clickable filtered navigation and don't need baseline comparisons. This is the **recommended default** for dashboards that prioritize navigation over period comparison aesthetics.

## Solution 3: Custom Menus With Filtered `ir.actions.act_window`

Create **hidden Odoo menus** whose actions have the domain pre-baked. Connect them via `chartOdooMenusReferences`. The scorecard figure is natively clickable and opens a filtered view.

```
┌─────────────────────────────┐
│  Ajuste de potencia         │  ← Click ANYWHERE on card
│       154                   │  → Opens list filtered by
│                             │     project_id=5 AND setting=True
└─────────────────────────────┘
```

### Step-by-step implementation:

#### Step 1: Create `ir.actions.act_window` with domain

```python
# One action per scorecard, each with its specific domain
action_id = call("ir.actions.act_window", "create", [{
    "name": "Factura Luz - Ajuste de potencia",
    "res_model": "project.task",
    "domain": "[('project_id', '=', 5), ('x_studio_setting', '=', True)]",
    "view_mode": "list,form,kanban",
    "context": "{'lang': 'es_ES'}",
    "target": "current",
}])
```

#### Step 2: Create hidden menu pointing to the action

```python
# Hidden parent menu (active=False makes it invisible in navbar)
parent_id = call("ir.ui.menu", "create", [{
    "name": "Dashboard Menus (hidden)",
    "sequence": 999,
    "active": False,
}])

# Child menu linked to the action (active=True but parent is hidden)
menu_id = call("ir.ui.menu", "create", [{
    "name": "Factura Luz - Ajuste de potencia",
    "parent_id": parent_id,
    "action": f"ir.actions.act_window,{action_id}",
    "sequence": 10,
}])
```

#### Step 3: Map scorecard figure UUID to menu ID

```json
"chartOdooMenusReferences": {
    "scorecard-figure-uuid": 654
}
```

**Note**: `chartOdooMenusReferences` accepts both menu XML IDs (string) and menu database IDs (integer).

### Key details:

- **Menu visibility**: Menus MUST be `active=True` and reachable in the menu tree. The frontend menu service (`/web/action/load_menus`) only loads active, visible menus. A menu with `active=False` (or under an inactive parent) will NOT be found by `chartOdooMenusReferences` → the scorecard won't be clickable. **Solution**: Place menus under a deep existing menu (e.g., `Module > Configuration > Dashboard Actions`) with high `sequence` (999) so they're technically visible but practically hidden from users
- **Domain is in the action**: The `ir.actions.act_window.domain` is evaluated server-side and applied to the view. This is a standard Odoo mechanism.
- **One action per scorecard**: Each scorecard with a different filter needs its own action+menu pair
- **Limitation**: Dashboard **global filters** (periodo, comercializadora, etc.) are NOT applied when clicking. Only the action's static domain is used. The user sees filtered data on the dashboard but the click opens a view with only the static domain.

**Pros**: Click directly on the card, native scorecard aesthetics preserved (humanize, baseline, colors), domain filtering works
**Contras**: Global dashboard filters not applied on click (static domain only), requires creating actions+menus in Odoo

**When to use**: When you want the best of both worlds — native scorecards that are clickable with filtered navigation.

### Comparison table

| | Static domain | Global filters on click | Click target | Native scorecard | Effort | Odoo-side setup |
|---|---|---|---|---|---|---|
| **Solution 1**: odoo://view links | YES | NO* | Link below card | YES | Zero | None |
| **Solution 2**: Fake scorecards (RECOMMENDED) | YES | NO* | Link row in card | NO | Low | None |
| **Solution 3**: Custom menus | YES | NO | Card itself | YES | Low | Actions + menus |

*\* `odoo://view` links contain a static domain at build time. To include dynamic filters, the link would need to be rebuilt when filters change — which Odoo does not support natively.*

### Recommended approach

**Use Solution 2 as the default** for dashboards where navigation is important. It provides:
- Clean visual cards with color-coded backgrounds
- Clickable filtered links within each card
- Zero Odoo-side configuration (no menus, no actions)
- Full control over layout and styling

Use Solution 1 (links below native scorecards) when baseline comparison is critical and you need `humanize`, `baselineMode`, animated values.

Use Solution 3 only when you specifically need the native scorecard click behavior AND can guarantee all menus in the chain are `active=True` (see caveats below).

**Solution 3 caveat — menu visibility trap**: Menus with `active=False` or under an inactive parent are NOT loaded by the frontend menu service → `chartOdooMenusReferences` silently fails → scorecards are not clickable. Menus must be `active=True` and under active parents, which means they'll be visible in the navbar (even if deeply nested with `sequence: 999`). This makes Solution 3 fragile in practice.

## Clickable Figures (`chartOdooMenusReferences`)

Maps figure UUIDs to Odoo menus. Clicking the figure navigates to the menu's action.

### Setup

In JSON root, map figure UUIDs to menu IDs (integer) or menu XML IDs (string):

```json
"chartOdooMenusReferences": {
    "scorecard-uuid-1": 654,
    "scorecard-uuid-2": "project.menu_project_management_all_tasks",
    "chart-uuid-3": "sale.menu_sale_report"
}
```

### How it works

1. User clicks on the figure (scorecard, chart, gauge)
2. Odoo resolves the menu → gets the menu's `actionID`
3. Calls `actionService.doAction(actionID)` → opens the action's view with its domain

### Finding menu XML IDs

```python
# Search for menus related to a module
env['ir.model.data'].search_read(
    [('model', '=', 'ir.ui.menu'), ('module', '=', 'project')],
    ['name', 'module', 'res_id']
)
```

Common menus: `project.menu_project_management_all_tasks`, `crm.crm_menu_root`, `crm.crm_opportunity_report_menu`, `sale.menu_sale_report`, `account.menu_finance_reports`

**For odoo_* charts**: Drill-down already provides filtered navigation per data point. `chartOdooMenusReferences` adds little value — use only if you want a fallback "module homepage" click.

## Clickable View Links (`odoo://view`)

Dashboard cells can contain markdown-style links that navigate to Odoo views when clicked. This is how Odoo standard dashboards create "Top Opportunities", "Top Salespeople" section headers that open the corresponding list/pivot/graph view.

### Syntax

```
[Link Text](odoo://view/JSON_CONFIG)
```

### JSON Config Structure

```json
{
  "viewType": "list",
  "action": {
    "domain": "[[\"type\", \"=\", \"opportunity\"]]",
    "context": {
      "group_by": ["user_id"],
      "pivot_measures": ["__count", "prorated_revenue"],
      "pivot_column_groupby": [],
      "pivot_row_groupby": ["user_id"]
    },
    "modelName": "crm.lead",
    "views": [
      [false, "kanban"],
      [false, "list"],
      [false, "calendar"],
      [false, "pivot"],
      [false, "graph"],
      [false, "form"]
    ]
  },
  "threshold": 0,
  "name": "Pipeline",
  "positional": true
}
```

### Properties

| Property | Description |
|----------|-------------|
| `viewType` | Default view to open: `"list"`, `"pivot"`, `"graph"`, `"kanban"`, `"form"` |
| `action.domain` | Domain filter for the view (Python string or JSON array) |
| `action.context` | View context: group_by, pivot measures, graph settings |
| `action.modelName` | Odoo model to query |
| `action.views` | Available views as `[[view_id, "type"], ...]` — use `false` for auto |
| `name` | Action display name |
| `positional` | Use positional action loading |

### Examples from Odoo Standard Dashboards

#### Link to List View
```json
{
  "A24": {
    "content": "[Top Opportunities](odoo://view/{\"viewType\":\"list\",\"action\":{\"domain\":\"[[\\\"type\\\", \\\"=\\\", \\\"opportunity\\\"]]\",\"modelName\":\"crm.lead\",\"views\":[[false,\"list\"],[false,\"form\"]]},\"name\":\"Pipeline\"})"
  }
}
```

#### Link to Pivot View
```json
{
  "A37": {
    "content": "[Top Salespeople](odoo://view/{\"viewType\":\"pivot\",\"action\":{\"domain\":[\"&\",[\"type\",\"=\",\"opportunity\"],[\"user_id\",\"!=\",false]],\"context\":{\"group_by\":[\"user_id\"],\"pivot_measures\":[\"__count\",\"prorated_revenue\"]},\"modelName\":\"crm.lead\",\"views\":[[false,\"pivot\"],[false,\"list\"]]},\"name\":\"Pipeline\"})"
  }
}
```

#### Link to Graph View
```json
{
  "A7": {
    "content": "[Pipeline Stages](odoo://view/{\"viewType\":\"graph\",\"action\":{\"domain\":[[\"type\",\"=\",\"opportunity\"]],\"context\":{\"group_by\":[\"stage_id\",\"team_id\"],\"graph_measure\":\"prorated_revenue\",\"graph_mode\":\"bar\"},\"modelName\":\"crm.lead\",\"views\":[[false,\"graph\"]]},\"name\":\"Pipeline\"})"
  }
}
```

### Escaping Rules

The JSON inside the URL must be properly escaped:
- Double quotes inside the JSON become `\"`
- When inside a JSON string value (like domain), they become `\\\"`
- Best practice: build the link programmatically, not manually

```python
import json

view_config = {
    "viewType": "list",
    "action": {
        "domain": '[["type", "=", "opportunity"]]',
        "modelName": "crm.lead",
        "views": [[False, "list"], [False, "form"]]
    },
    "name": "Pipeline"
}
link = f"[Top Opportunities](odoo://view/{json.dumps(view_config)})"
```

## Translation Function (`_t()`)

The `=_t()` formula wraps text for automatic translation based on the user's language.

### Usage

```
A25: =_t("Opportunity")     → "Oportunidad" (in es_ES)
B25: =_t("Stage")           → "Etapa"
C25: =_t("Salesperson")     → "Comercial"
```

### Where Odoo Uses `_t()`

- Column headers in PIVOT.HEADER/VALUE tables
- Row labels in Data sheets (KPI names)
- Section dividers and category labels
- Any user-facing text that should be translated

### Pattern: Data Sheet with Translated Labels

```
A1: =_t("KPI")
A2: =_t("Expected count")
A3: =_t("Closed count")
A4: =_t("Expected revenue")
A5: =_t("Closed revenue")
A6: =_t("Percentage closed")
B1: =_t("Current")
C1: =_t("Previous")
```

### When to Use `_t()`

| Use `_t()` | Don't use `_t()` |
|------------|-------------------|
| Column/row headers | Dashboard title (use plain text) |
| KPI labels | Filter labels (have their own `label` property) |
| Section titles on Data sheet | Formulas (`=PIVOT.VALUE(...)`) |
| Static descriptive text | Numbers, dates, cell references |

### Important Notes

- `_t()` only works in cell content (not in figure titles or filter labels)
- Translations come from Odoo's `ir.translation` / `ir.model.data`
- The function returns the source text if no translation exists
- For Spanish dashboards with `locale: es_ES`, Odoo auto-translates standard terms
- Custom terms need manual translation entries in Odoo

## Combining Links and Translations

Section headers often combine both:

```json
{
  "A24": {
    "content": "[Top Opportunities](odoo://view/{...})"
  },
  "A25": {
    "content": "=_t(\"Opportunity\")"
  },
  "B25": {
    "content": "=_t(\"Stage\")"
  },
  "C25": {
    "content": "=_t(\"Revenue\")"
  }
}
```

Row 24 = Clickable section header (navigates to view)
Row 25 = Translated column headers for the ranking table below
