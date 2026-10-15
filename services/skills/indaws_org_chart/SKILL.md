---
name: indaws-org-chart
version: 1.0.4
effort: medium
model: sonnet
description: >
  Generate organizational chart visualizations from Odoo HR data.
  Use this skill when you need to visualize company hierarchy,
  department structure, reporting lines, or employee organization.
  Fetches hr.employee and hr.department data via odoo-pilot and
  produces Mermaid diagrams and interactive HTML org charts.
---

# Org Chart

Generate visual organizational charts from your Odoo HR data. Produces both Mermaid diagrams (markdown-friendly) and interactive HTML pages.

## Prerequisites

See `references/shared/odoo-connection.md` for connection pattern and `references/shared/batch-limits.md` for pagination rules.

- **odoo-pilot** skill installed (same repository)
- Environment variables: `ODOO_URL`, `ODOO_DB`, `ODOO_KEY`, `ODOO_USER`

## Quick Start

```bash
SCRIPT_DIR="skills/org-chart/scripts"
eval $(skills/odoo-pilot/scripts/auth.sh)
$SCRIPT_DIR/fetch_org_data.sh | python3 $SCRIPT_DIR/generate_mermaid.py > output/org-chart.md
$SCRIPT_DIR/fetch_org_data.sh | python3 $SCRIPT_DIR/generate_html.py > output/org-chart.html
$SCRIPT_DIR/fetch_org_data.sh --include-photos | python3 $SCRIPT_DIR/generate_html.py > output/org-chart.html
```

## Scripts

### `fetch_org_data.sh`

Fetches all active employees, departments, and companies from Odoo.

```bash
./scripts/fetch_org_data.sh [OPTIONS]
```

**Options:**
- `--include-photos` — Include employee photos (base64 `image_128`), makes output larger
- `--department-id ID` — Filter to employees in a specific department
- `--company-id ID` — Filter by company

**Output:** JSON to stdout with structure:
```json
{
  "timestamp": "2026-03-08T09:00:00",
  "employees": [...],
  "departments": [...],
  "companies": [...]
}
```

### `generate_mermaid.py`

Converts org data JSON to a Mermaid flowchart diagram.

```bash
cat org-data.json | python3 generate_mermaid.py [--by-department] [--max-depth N]
```

**Options:**
- `--by-department` — Group nodes into department subgraphs
- `--max-depth N` — Limit tree depth (default: unlimited)

**Output:** Markdown file with Mermaid code fence to stdout.

### `generate_html.py`

Converts org data JSON to a self-contained interactive HTML org chart.

```bash
cat org-data.json | python3 generate_html.py > org-chart.html
```

**Features:**
- Expandable/collapsible nodes
- Employee cards with name, job title, department, email, phone, photo
- Department color coding
- Search/filter by name
- Print-friendly CSS

## Output Files

Generated files are saved to `skills/org-chart/output/`:

| File | Description |
|------|-------------|
| `org-chart.md` | Mermaid diagram in markdown |
| `org-chart.html` | Interactive HTML org chart |

## Customization

Filter options: `--department-id ID`, `--company-id ID` on `fetch_org_data.sh`. Limit tree depth with `--max-depth N` on `generate_mermaid.py`. See [references/integrations.md](references/integrations.md) for 12+ skill integrations. Weekly cron regenerates every Monday 9:00 AM.

## GOTCHAS
1. **hr.employee.parent_id can be circular** — always check for cycles before rendering the tree.
2. **Departments without manager show as orphaned nodes** — filter or mark them explicitly.
3. **Mermaid has a node limit** — charts with >100 employees may fail to render. Use department-level view for large orgs.
4. **Employee photos are base64 in Odoo** — too large for Mermaid. Use text-only nodes or HTML output for photos.
5. **Los inactivos NO aparecen**: el ORM añade `active=True` al dominio salvo `active_test=False` en contexto (`odoo/models.py`, `BaseModel._where_calc`). Para incluir bajas, pasa `active_test=False`.

## Workflow
1. Fetch hr.employee and hr.department data via odoo-pilot
2. Build hierarchy tree from parent_id relationships
3. Detect and handle cycles/orphans
4. Generate Mermaid diagram and/or interactive HTML
5. Present to user with navigation options

## Delegates to
| Skill | Suite | When |
|-------|-------|------|
| odoo-pilot | common | HR data fetch via API |

## Consumed by
| Skill | Suite | For what |
|-------|-------|----------|
| people-report | hr | Organizational hierarchy visualization |

## RULES
### NEVER
- Render more than 100 nodes in a single Mermaid diagram
- Include inactive employees without explicit request
### ALWAYS
- Filter by active=True si quieres ser explícito (el ORM ya lo hace: ver gotcha 5); para INCLUIR bajas hace falta `active_test=False` en contexto, no basta con quitar el filtro
- Offer both Mermaid (markdown) and HTML (interactive) output
