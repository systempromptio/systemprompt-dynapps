# Reporting / BI — Integraciones cross-módulo (Odoo 18.0, + v19 diffs)

Cómo otros subsistemas CONSUMEN los cuatro motores de reporting descritos en `core.md`. La idea
central: **casi todo el reporting "avanzado" de Odoo reutiliza los mismos cuatro motores base**, no
inventa nada nuevo. Saber qué motor está debajo te dice dónde está el bug real.

Mapa de quién consume qué:

| Consumidor | Motor base que reutiliza | Sección |
|---|---|---|
| `account.report` (EE) export PDF | QWeb/wkhtmltopdf (`_run_wkhtmltopdf`) | §1 |
| `account.report` (EE) export XLSX | xlsxwriter directo (NO QWeb) | §1 |
| Spreadsheet `=ODOO.PIVOT(...)` | `read_group`/`web_read_group` (vía OdooPivotModel) | §2 |
| Spreadsheet `=ODOO.LIST(...)` | `web_search_read` | §2 |
| Dashboards (spreadsheet_dashboard) | Spreadsheet pre-construido (pivots/listas) | §3 |
| Dashboards Odoo "ligeros" (board / cohort) | `read_group` + reports SQL `_auto=False` | §3 |
| Email con PDF adjunto | `_render_qweb_pdf` (sin request → cookie gotcha) | §4 |
| Vistas pivot/graph de cualquier app | reports SQL `_auto=False` + `read_group` | §5 |

---

## 1. account.report (EE) → reutiliza QWeb PDF y xlsxwriter

El motor account.report (P&L, Balance, 303…) **NO tiene su propio renderizador de PDF**: cuando el
usuario pulsa *Export → PDF*, EE construye el HTML del informe y lo pasa al MISMO wkhtmltopdf de la
Parte A.

Fuente EE `odoo_enterprise/18/account_reports/models/account_report.py`:
- Los botones de export se definen en `_init_options_buttons` (`account_reports/models/account_report.py:1751`, `AccountReport._build_columns_from_column_group_vals`) con
  `action_param='export_to_pdf'` / `'export_to_xlsx'` (`account_reports/models/account_report.py:1753-1754`, `AccountReport._build_columns_from_column_group_vals`).
- `export_to_pdf(options)` (`:5908`) compone los `bodies` HTML (con
  `_get_pdf_export_html`, `:5945-5947`) y llama **`action_report._run_wkhtmltopdf(...)`**
  (`:5954`) — exactamente el método de `ir.actions.report` documentado en `core.md` Parte A §2.

**Implicación**: TODOS los gotchas RPTG1-RPTG20 (WebKit viejo, header_spacing, `workers=1`,
returncode -11 por memoria, `unpatched QT` en v19) **aplican también al PDF de los informes
financieros**. Un Balance que sale roto en PDF pero bien en pantalla es el mismo problema de
flex/grid de RPTG1.

- `export_to_xlsx` (`:1754`, `export_file` `:5889`) usa **xlsxwriter** directamente
  (`from odoo.tools.misc import xlsxwriter`, `:26`) — esto NO pasa por QWeb ni wkhtmltopdf. Por eso
  el XLSX nunca tiene los problemas de CSS del PDF, pero tampoco respeta el `external_layout`.

> Para los engines del cálculo (domain/tax_tags/aggregation/account_codes/external/custom) y sus
> gotchas ARG, ver `cross-module/account_reports/`. Aquí solo el puente con QWeb PDF.

---

## 2. Spreadsheet → reutiliza read_group y search_read

Las celdas `=ODOO.PIVOT(...)` y `=ODOO.LIST(...)` de un spreadsheet Odoo NO ejecutan SQL propio:
acaban llamando a la familia `read_group` / `web_search_read` del servidor.

Fuente `addons/spreadsheet/static/src/pivot/odoo_pivot.js`:
- `OdooPivot.createModelAndDefinition` (`:182-202`) instancia un **`OdooPivotModel`** pasándole
  `orm` y `serverData`.
- `OdooPivotModel` **extiende el `pivot_model.js` de la vista pivot web**
  (`spreadsheet/static/src/pivot/pivot_model.js:593`, `OdooPivotModel._getSubGroups` "override to add the order by clause to the **read_group** kwargs"). Es
  decir, el pivot del spreadsheet usa el MISMO motor que el pivot del backend → mismas reglas de
  `aggregator`, `lazy`, medidas (RPTG-BI1..8 aplican).
- Las llamadas se cachean en `server_data.js` (`asyncCache`, `:168`, `this.orm.call(...)`).

**Implicaciones cross-módulo**:
- `=ODOO.PIVOT` sobre un campo computed sin `store=True` falla igual que en el pivot backend
  (RPTG-BI3): no se puede usar como medida.
- Un measure que suma un precio unitario da el mismo número absurdo (RPTG-BI5) en el spreadsheet.
- Si el modelo subyacente es un report SQL `_auto=False` (ej. `sale.report`), el spreadsheet hereda
  su `_depends`/granularidad: datos stale (RPTG-SQL4) o `id` colisionado (RPTG-SQL3) se propagan a
  las celdas.
- `=ODOO.LIST` usa `web_search_read` (no read_group) → no agrega; respeta el `aggregator` solo en
  los totales de columna.

> Detalle profundo del subsistema spreadsheet (plugins, migraciones JS, filtros globales, share
> tokens) en `functional/spreadsheet/` (prefijo SPRG).

---

## 3. Dashboards → spreadsheets pre-construidos + read_group

Dos familias de "dashboard" en Odoo, ambas apoyadas en motores ya vistos:

**a) `spreadsheet_dashboard*` (EE)** — Son **spreadsheets Odoo guardados como XML data** y servidos
en modo solo-lectura. Módulos: `spreadsheet_dashboard_sale_renting`, `_crm`, `_stock`, `_hr_payroll`,
`_account_accountant`, etc. (lista en `odoo_enterprise/18/`). Cada dashboard es un spreadsheet
con pivots/gráficos → consume read_group (§2). Gotcha de despliegue: al ser **XML data, un `-u` del
módulo SOBRESCRIBE el dashboard** (igual que cualquier `noupdate=0` data); las personalizaciones del
usuario sobre el dashboard base se pierden.

**b) Dashboards "ligeros" del backend (board, cohort, vistas dashboard)** — Se construyen embebiendo
vistas pivot/graph/list de reports SQL `_auto=False`. Consumen read_group directamente sobre el
report. Por eso un KPI lento en un dashboard suele ser un read_group pesado sobre un report SQL con
JOINs caros (§5), no un problema del dashboard.

**Regla de diagnóstico**: "el dashboard tarda / da números raros" → identifica el report SQL o el
modelo subyacente y reproduce el read_group/pivot a mano; el bug casi nunca está en la capa
dashboard.

---

## 4. Email / cron con PDF adjunto → `_render_qweb_pdf` sin request

Las plantillas de correo que adjuntan un PDF (factura, presupuesto) y los crons que generan PDFs
llaman a `_render_qweb_pdf` **fuera de un request HTTP**. Esto reactiva dos gotchas de la Parte A:

- **RPTG19 (cookie de sesión)**: sin `request` activo no hay `--cookie-jar`
  (`odoo/addons/base/models/ir_actions_report.py:539-558`, `IrActionsReport._run_wkhtmltopdf`, `if request and request.db`). Imágenes/QR que se resuelven por
  URL autenticada (`/web/image/...`) pueden salir **en blanco** en el PDF adjunto al email o
  generado por cron. Fix: recursos `auth='public'` o imágenes embebidas base64.
- **RPTG18**: el check previo de wkhtmltopdf lo hace el controlador; una llamada manual desde la
  plantilla de email lo salta y puede **petar silenciosamente en el envío del correo** si el binario
  falta (comentario explícito en `ir_actions_report.py`).
- **RPTG23 (idioma)**: el cron/email debe pasar el `lang` del partner en el contexto del registro o
  el PDF sale en el idioma del usuario del cron, no del cliente.

---

## 5. Vistas pivot/graph de cualquier app → reports SQL + read_group

El patrón general de "menú Reporting" de cada app (Ventas, Compras, Inventario, Proyecto, CRM,
Timesheet…) es siempre el mismo apilamiento:

```
<menú Reporting> → action sobre un modelo _auto=False (sale.report, purchase.report,
                   account.invoice.report, stock.report.*, timesheets.analysis.report, ...)
                 → vistas pivot/graph/list
                 → web_read_group sobre la subquery/vista SQL del report
```

- La **agregación rápida** la hace el GROUP BY del report SQL (Parte B); el **cruce
  interactivo** (filas×columnas, medidas) lo hace read_group encima (Parte C).
- Por eso para AÑADIR una dimensión a un informe de análisis hay DOS sitios posibles: (a) si el dato
  ya existe como columna del report, basta exponerlo en el arch del pivot; (b) si no, hay que
  ampliar el report SQL (`_select` + `_group_by`, RPTG-SQL2) y declararlo como `fields.X` con el
  `aggregator` correcto.
- Módulos puente extienden el report heredando los hooks: ej. `sale_stock/report/sale_report.py`
  sobreescribe `_select_additional_fields()`/`_from_sale()`/`_group_by_sale()` para añadir campos de
  stock al análisis de ventas. Patrón de extensión documentado en `core.md` §11.

**Diagnóstico unificado**: ante "el informe de análisis X da mal un número", la cadena de sospecha
es: arch del pivot (medida/invisible, RPTG-BI8/5) → read_group (lazy/aggregator, RPTG-BI1/4) →
report SQL (`_select`/`_group_by`/`id`, RPTG-SQL2/3) → datos reales.

---

## 6. v19 — qué cambia en las integraciones

- account.report sigue exportando PDF vía wkhtmltopdf (no Chromium) y XLSX vía xlsxwriter; aplica
  el nuevo error "unpatched QT" multi-doc (RPTG22) también a informes financieros multi-página.
- Spreadsheet pivot pasa a apoyarse en `formatted_read_group` (read_group deprecada en v19); ver
  `v19_changes.md`. La semántica de medidas/aggregator no cambia.
- Reports SQL: el atributo `aggregator` se consolida (group_operator ya solo shim); ver
  `v19_changes.md`.
