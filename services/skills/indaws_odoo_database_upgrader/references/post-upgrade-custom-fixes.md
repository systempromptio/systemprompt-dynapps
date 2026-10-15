# Post-Upgrade: Fixes Comunes en Modulos Custom

Guia de patrones de rotura en modulos custom tras migrar con el Upgrade Service.
Estos NO son fixes SQL pre-envio, sino correcciones Python/XML que aplicar
sobre el codigo fuente de los modulos custom despues de restaurar el dump migrado.

**Cuando leer:** FASE 3.5.1, como catalogo de deteccion en el pre-scan de modulos custom.

> **Integracion con skills:** Este catalogo se usa como INPUT de deteccion
> en FASE 3.5.1 (pre-scan). Los fixes no se aplican manualmente — se delegan
> a `odoo-module-upgrader` (para modulos CODE) o se reemplazan por
> implementaciones Studio (para modulos STUDIO/SPLIT).
> Ver `references/custom-module-orchestration.md` para el flujo completo.

---

## Indice

| # | Categoria | Versiones | Severidad |
|---|-----------|-----------|-----------|
| 1 | [Python: `_search()` sin `access_rights_uid`](#1-python-_search-sin-access_rights_uid) | v17+ | BLOCKING |
| 2 | [Python: `params.get('action')` devuelve string](#2-python-paramsgetaction-devuelve-string) | v17+ | BLOCKING |
| 3 | [Python: SQL report `_table_query` y objetos `SQL()`](#3-python-sql-report-_table_query-y-objetos-sql) | v17+ | CRITICAL |
| 4 | [Python: Columnas duplicadas en `sale.report`](#4-python-columnas-duplicadas-en-salereport) | v17+ | BLOCKING |
| 5 | [Python: `tax_totals_json` → `tax_totals`](#5-python-tax_totals_json--tax_totals) | v17+ | BLOCKING |
| 6 | [QWeb: `<field t-field>` → `<span t-field>` en reports](#6-qweb-field-t-field--span-t-field-en-reports) | v17+ | BLOCKING |
| 7 | [QWeb: Bootstrap 3/4 → Bootstrap 5](#7-qweb-bootstrap-34--bootstrap-5) | v17+ | WARNING |
| 8 | [QWeb: `web.address_layout` duplica datos](#8-qweb-webaddress_layout-duplica-datos) | v17+ | WARNING |
| 9 | [QWeb: Tax totals template calls](#9-qweb-tax-totals-template-calls) | v17+ | BLOCKING |
| 10 | [QWeb: Layout necesita `layout_document_title`](#10-qweb-layout-necesita-layout_document_title) | v17+ | WARNING |
| 11 | [XML: Locale strings sin comillas](#11-xml-locale-strings-sin-comillas) | v17+ | BLOCKING |
| 12 | [XML: `website_id.name` → `website`](#12-xml-website_idname--website) | v16+ | BLOCKING |
| 13 | [Python: Hooks OCA `_select_additional_fields`](#13-python-hooks-oca-_select_additional_fields) | v17+ | CRITICAL |
| 14 | [Python: Dead code en modulos de herencia](#14-python-dead-code-en-modulos-de-herencia) | Todas | INFO |
| 15 | [Glue como vehiculo del kit en odoo.sh](#15-glue-como-vehiculo-del-kit-en-odoosh) | Todas (odoo.sh) | PATRON |
| 16 | [Stock: reservas corruptas + override verbatim de reserva](#16-stock-reservas-corruptas--override-verbatim-de-reserva) | v17+ | CRITICAL |

---

## 1. Python: `_search()` sin `access_rights_uid`

**Versiones afectadas:** v17+ (eliminado del ORM)

**Error:**
```
TypeError: _search() got an unexpected keyword argument 'access_rights_uid'
```

**Causa:** v17 reescribio el ORM y elimino el parametro `access_rights_uid` de `_search()`.
Cualquier modulo que sobrescriba `_search()` con la firma antigua falla.

**Deteccion:**
```bash
grep -rn "access_rights_uid" migrated_modules/ --include="*.py"
```

**Fix:**
```python
# ANTES (v15):
@api.model
def _search(self, domain, offset=0, limit=None, order=None, access_rights_uid=None):
    # ... logica custom ...
    return super()._search(
        domain, offset=offset, limit=limit, order=order,
        access_rights_uid=access_rights_uid,
    )

# DESPUES (v17+):
@api.model
def _search(self, domain, offset=0, limit=None, order=None):
    # ... logica custom ...
    return super()._search(
        domain, offset=offset, limit=limit, order=order,
    )
```

**Fuente:** Pharmactive v15→v18 (crm_sale_sales_team_access_right)

---

## 2. Python: `params.get('action')` devuelve string

**Versiones afectadas:** v17+ (web client refactorizado)

**Error:**
```
ValueError: Expected singleton: ir.actions.act_window('o', 'r', 'd', 'e', 'r', 's')
```

**Causa:** En v17+, `self._context.get('params', {}).get('action')` puede devolver
un string como `'orders'` en vez de un integer. Si se hace `browse('orders')`,
Python itera la string caracter a caracter, creando un recordset con IDs invalidos.

**Deteccion:**
```bash
grep -rn "params.get('action')" migrated_modules/ --include="*.py"
grep -rn "params.get(\"action\")" migrated_modules/ --include="*.py"
```

**Fix:**
```python
# ANTES (v15):
action_id = params.get('action')
if action_id:
    action = self.env['ir.actions.act_window'].browse(action_id)

# DESPUES (v17+):
action_id = params.get('action')
if action_id and isinstance(action_id, int):
    action = self.env['ir.actions.act_window'].browse(action_id)
    active_model = action.res_model
    active_id = self._context.get('default_order_id') or self._context.get('default_invoice_id')
else:
    active_model = params.get('model')
    active_id = params.get('id')
```

**Fuente:** Pharmactive v15→v18 (product_customer_code)

---

## 3. Python: SQL report `_table_query` y objetos `SQL()`

**Versiones afectadas:** v17+ (`_table_query` reemplaza `init()`)

**Error:**
```
TypeError: can only concatenate str (not "SQL") to str
```
o bien el report SQL no devuelve los campos esperados.

**Causa:** En v17+, los modelos de report como `sale.report`, `purchase.report`,
`account.invoice.report` usan la propiedad `_table_query` que llama a
`_select()`, `_from()`, `_where()`, `_group_by()`. Estos metodos retornan
objetos `SQL()` en v17+, NO strings planos. Modulos que concatenan con `+`
sobre strings rompen al recibir `SQL()`.

**Deteccion:**
```bash
# Buscar herencias de modelos report con concatenacion de strings
grep -rn "_select\|_group_by\|_from\|_where" migrated_modules/ --include="*.py" \
  | grep -v "^Binary"
```

**Fix para `sale.report`:**
```python
# ANTES (v15) — concatenacion directa:
def _select_sale(self):
    return super()._select_sale() + ", s.my_field as my_field"

def _group_by_sale(self):
    return super()._group_by_sale() + ", s.my_field"

# DESPUES (v17+) — SIGUE FUNCIONANDO con strings si el core usa _select_sale()
# porque sale.report.py mantiene _select_sale() como string helper.
# PERO los metodos _select(), _group_by() SIN el sufijo _sale retornan SQL().
# Verificar cual usa el core en la version destino.
```

**Fix para `purchase.report`:**
```python
# ANTES (v15):
def _select(self):
    return super()._select() + ", po.my_field as my_field"

# DESPUES (v17+) — usar SQL():
from odoo.tools import SQL

def _select(self):
    return super()._select() + SQL(", po.my_field as my_field")

# O usar el hook _select_additional_fields si disponible:
def _select_additional_fields(self):
    res = super()._select_additional_fields()
    res['my_field'] = SQL("po.my_field")
    return res
```

**Nota:** En `sale.report` v18, los helpers `_select_sale()` y `_group_by_sale()`
todavia devuelven strings. Pero `_select()` y `_group_by()` (sin sufijo)
devuelven `SQL()`. Si el modulo custom hereda los helpers con sufijo, funciona.
Si hereda los sin sufijo, hay que usar `SQL()`.

**Fuente:** Pharmactive v15→v18 (sale_purchase_reports_extension, purchase_report.py)

---

## 4. Python: Columnas duplicadas en `sale.report`

**Versiones afectadas:** v17+ (columnas movidas al core o OCA)

**Error:**
```
psycopg2.errors.AmbiguousColumn: column reference "invoice_status" is ambiguous
psycopg2.errors.AmbiguousColumn: column reference "type_id" is ambiguous
```

**Causa:** Campos que antes solo existian en modulos custom fueron absorbidos por
el core o por OCA en versiones nuevas. Si el modulo custom sigue declarando
el field Y seleccionandolo en SQL, la columna aparece duplicada.

**Campos comunes absorbidos en v17/v18:**
- `invoice_status` → ahora en core `sale.report`
- `type_id` → ahora en OCA `sale_order_type` via `_select_additional_fields`
- `order_id` → ahora se espera en core pero hay que añadirlo si falta

**Deteccion:**
```bash
# Buscar campos que el core ya declara
grep -rn "invoice_status\|type_id" migrated_modules/ --include="*.py" \
  | grep -i "field\|select"
```

**Fix:**
1. Leer el `sale_report.py` del core de la version destino
2. Listar todos los campos que ya estan en SELECT
3. Comparar con lo que añade el modulo custom
4. Eliminar duplicados del SELECT, GROUP BY y declaracion de fields

```python
# Si el core ya tiene invoice_status, BORRAR:
# invoice_status = fields.Selection([...])  # YA NO NECESARIO
# "s.invoice_status as invoice_status"      # YA EN CORE

# Si OCA ya tiene type_id via _select_additional_fields, BORRAR:
# "s.type_id as type_id"                    # YA EN OCA
# y borrar de _group_by_sale() tambien
```

**Fuente:** Pharmactive v15→v18 (sale_purchase_reports_extension, sale_order_type_ext)

---

## 5. Python: `tax_totals_json` → `tax_totals`

**Versiones afectadas:** v17+ (refactoring contable)

**Error:**
```
AttributeError: 'sale.order' object has no attribute 'tax_totals_json'
```
o bien JSON parse error en reports.

**Causa:** En v15/v16, los totales de impuestos se almacenaban como JSON string
en `tax_totals_json`. En v17+, es un campo computed que devuelve un dict
directamente como `tax_totals`.

**Deteccion:**
```bash
grep -rn "tax_totals_json" migrated_modules/ --include="*.py" --include="*.xml"
```

**Fix Python:**
```python
# ANTES (v15):
import json
tax_data = json.loads(doc.tax_totals_json)

# DESPUES (v17+):
tax_data = doc.tax_totals  # ya es un dict
```

**Fix QWeb (ver seccion 9 para template calls):**
```xml
<!-- ANTES (v15): -->
<t t-set="tax_totals" t-value="json.loads(doc.tax_totals_json)"/>

<!-- DESPUES (v17+): -->
<t t-set="tax_totals" t-value="doc.tax_totals"/>
```

**Fuente:** Pharmactive v15→v18 (pharmactive_custom_reports)

---

## 6. QWeb: `<field t-field>` → `<span t-field>` en reports

**Versiones afectadas:** v17+ (QWeb parser mas estricto)

**Error:**
```
lxml.etree.XMLSyntaxError: Unknown element 'field' in report context
```
o bien el campo simplemente no se renderiza en el PDF.

**Causa:** En v17+, QWeb reports solo admiten elementos HTML estandar.
El tag `<field>` es exclusivo de vistas de formulario/arbol. En reports,
siempre usar `<span t-field="..."/>` o `<t t-out="..."/>`.

**Deteccion:**
```bash
grep -rn "<field t-field" migrated_modules/ --include="*.xml" \
  | grep -v "views/" | grep "report"
```

**Fix:**
```xml
<!-- ANTES: -->
<field t-field="doc.partner_id"/>

<!-- DESPUES: -->
<span t-field="doc.partner_id"/>
```

**Nota:** `<field>` en `views/*.xml` (forms, trees) sigue siendo correcto.
Solo los `report/*.xml` y templates QWeb necesitan el cambio.

**Fuente:** Pharmactive v15→v18 (9 archivos de reports)

---

## 7. QWeb: Bootstrap 3/4 → Bootstrap 5

**Versiones afectadas:** v17+ (migracion a BS5)

**Error:** No da error directo, pero los PDFs salen con layout roto
(elementos desalineados, tablas sin formato, columnas desplazadas).

**Causa:** Odoo v17+ usa Bootstrap 5. Las clases de BS3/BS4 ya no existen.

**Tabla de mapeo completa:**

| BS3/BS4 (v15) | BS5 (v17+) | Contexto |
|----------------|------------|----------|
| `text-right` | `text-end` | Alineacion texto derecha |
| `text-left` | `text-start` | Alineacion texto izquierda |
| `pull-right` | `float-end` | Flotar elemento derecha |
| `pull-left` | `float-start` | Flotar elemento izquierda |
| `table-condensed` | `table-sm` | Tablas compactas |
| `ml-*` | `ms-*` | Margin left → margin start |
| `mr-*` | `me-*` | Margin right → margin end |
| `pl-*` | `ps-*` | Padding left → padding start |
| `pr-*` | `pe-*` | Padding right → padding end |
| `font-weight-bold` | `fw-bold` | Negrita |
| `font-italic` | `fst-italic` | Cursiva |
| `sr-only` | `visually-hidden` | Accesibilidad |
| `badge-*` | `bg-*` | Color badges |
| `no-gutters` | `g-0` | Quitar gutters en row |
| `form-group` | `mb-3` | Espaciado formularios |
| `custom-select` | `form-select` | Select nativo |
| `custom-control` | `form-check` | Checkboxes/radios |
| `data-toggle` | `data-bs-toggle` | Atributos JS |
| `data-target` | `data-bs-target` | Atributos JS |
| `data-dismiss` | `data-bs-dismiss` | Atributos JS |

**Deteccion masiva:**
```bash
grep -rn "text-right\|text-left\|pull-right\|pull-left\|table-condensed\|ml-[0-9]\|mr-[0-9]\|pl-[0-9]\|pr-[0-9]" \
  migrated_modules/ --include="*.xml" --include="*.scss"
```

**Fix:** Buscar y reemplazar con `replace_all`. Ejemplo:
```bash
# Con sed (para revision):
grep -rl "text-right" migrated_modules/ --include="*.xml" | head -20
# Con Edit tool: usar replace_all=true por archivo
```

**Fuente:** Pharmactive v15→v18 (10+ archivos XML)

---

## 8. QWeb: `web.address_layout` duplica datos

**Versiones afectadas:** v17+ (nuevo mecanismo de layouts)

**Error:** El PDF muestra la direccion del cliente 2-3 veces:
una de `address_layout`, otra de `information_block`, y otra del bloque
`informations` custom.

**Causa:** En v17+, `web.external_layout` llama al layout seleccionado por
`company.external_report_layout_id`. Si el layout custom incluye
`<t t-call="web.address_layout"/>`, este renderiza las variables `address`
e `information_block` que el template del report tambien define.
Si ademas el template hereda y reemplaza `div#informations` con su propia
tabla de datos del cliente, se triplica la informacion.

**Arquitectura del rendering chain:**
```
report template (sale.report_saleorder_document)
  → web.external_layout
    → company.external_report_layout_id (= layout custom)
      → SI llama web.address_layout → renderiza address + information_block
      → renderiza layout_document_title
      → renderiza t-out="0" (= el body del report template)
        → div#informations (custom) → OTRA VEZ la misma info
```

**Fix:** Si el template custom reemplaza `div#informations` con su propia tabla
de datos del cliente, el layout NO debe llamar a `web.address_layout`.

```xml
<!-- Layout custom - NO llamar address_layout si los templates custom
     ya tienen su propio bloque de informacion del cliente -->
<div class="article ...">
    <h2 t-if="layout_document_title" t-out="layout_document_title"/>
    <t t-out="0"/>
</div>

<!-- SI se quiere usar address_layout, NO reemplazar div#informations
     en los templates hijos -->
```

**Fuente:** Pharmactive v15→v18 (invoice_layout_pharmactive)

---

## 9. QWeb: Tax totals template calls

**Versiones afectadas:** v17+ (unificacion de tax totals)

**Error:**
```
QWebException: template 'sale.sale_order_portal_content_totals_table' not found
```
> **En v19 el traceback dice `QWebError`, no `QWebException`.** La clase se renombró sin alias
> (`odoo/addons/base/models/ir_qweb.py:522`, `QWebException` en v18 → `:534` en v19; **0 ocurrencias** del nombre viejo en CE19/EE19,
> medido 2026-08-17). Si grepeas el log de una migración a 19 buscando `QWebException`, no
> encuentras nada y parece que el problema no está.
o totales de impuestos vacios en el PDF.

**Causa:** En v17+, cada modelo tiene su propia template para tax totals:
- Sale: `sale.document_tax_totals`
- Account: `account.document_tax_totals`
- Purchase: `purchase.document_tax_totals`

Reciben `tax_totals` (dict) y `currency` como parametros.

**Fix:**
```xml
<!-- Sale Order -->
<t t-call="sale.document_tax_totals">
    <t t-set="tax_totals" t-value="doc.tax_totals"/>
    <t t-set="currency" t-value="doc.currency_id"/>
</t>

<!-- Invoice -->
<t t-call="account.document_tax_totals">
    <t t-set="tax_totals" t-value="o.tax_totals"/>
    <t t-set="currency" t-value="o.currency_id"/>
</t>

<!-- Company currency (invoices) -->
<t t-if="o.tax_totals.get('display_in_company_currency')">
    <t t-call="account.document_tax_totals_company_currency_template"/>
</t>

<!-- Purchase Order -->
<t t-call="purchase.document_tax_totals">
    <t t-set="tax_totals" t-value="o.tax_totals"/>
    <t t-set="currency" t-value="o.currency_id"/>
</t>
```

**Fuente:** Pharmactive v15→v18 (sale, invoice, purchase report templates)

---

## 10. QWeb: Layout necesita `layout_document_title`

**Versiones afectadas:** v17+ (refactoring de layouts)

**Error:** El titulo del documento (QUOTATION, INVOICE, etc.) no aparece
en el PDF, o aparece duplicado si se pone manualmente.

**Causa:** En v17+, el titulo se gestiona via la variable `layout_document_title`
que el template del report define con `<t t-set="layout_document_title">`.
El layout custom debe renderizarlo con `<h2 t-if="layout_document_title" t-out="layout_document_title"/>`.

**Fix en el layout:**
```xml
<div class="article ...">
    <h2 t-if="layout_document_title" t-out="layout_document_title"/>
    <t t-out="0"/>
</div>
```

**Fix en el template (si se quiere titulo custom):**
```xml
<xpath expr="//t[@t-set='layout_document_title']" position="replace">
    <t t-set="layout_document_title">
        <t t-if="is_proforma"><span>PROFORMA INVOICE</span></t>
        <t t-elif="doc.state in ['draft','sent']"><span>QUOTATION</span></t>
        <t t-else=""><span>ORDER CONFIRMATION</span></t>
        <span> </span>
        <span t-field="doc.name"/>
    </t>
</xpath>
```

**Fuente:** Pharmactive v15→v18 (layout_reports.xml)

---

## 11. XML: Locale strings sin comillas

**Versiones afectadas:** v17+ (QWeb parser estricto)

**Error:**
```
NameError: name 'en_US' is not defined
```

**Causa:** En v15, QWeb toleraba comparaciones como `doc.lang == en_US`.
En v17+, el parser evalua `en_US` como nombre de variable (no existe).

**Deteccion:**
```bash
grep -rn "== en_US\|== es_ES\|== fr_FR\|== de_DE" migrated_modules/ --include="*.xml"
```

**Fix:**
```xml
<!-- ANTES: -->
<t t-if="doc.partner_id.lang == en_US">

<!-- DESPUES: -->
<t t-if="doc.partner_id.lang == 'en_US'">
```

**Fuente:** Pharmactive v15→v18 (5 archivos XML)

---

## 12. XML: `website_id.name` → `website`

**Versiones afectadas:** v16+ (campo website cambiado)

**Error:**
```
AttributeError: 'res.company' object has no attribute 'website_id'
```
o bien:
```
'NoneType' object has no attribute 'name'
```

**Causa:** En v15, `res.company.website` era un Many2one a `website`.
En v16+, `res.company` tiene `website` como Char field directamente
en `res.partner`. Para acceder al website de la empresa:
`company.partner_id.website` o `res_company.partner_id.website`.

**Deteccion:**
```bash
grep -rn "website_id\.name\|company\.website_id" migrated_modules/ --include="*.xml"
```

**Fix:**
```xml
<!-- ANTES: -->
<span t-field="company.website_id.name"/>

<!-- DESPUES (en layouts que tienen company como variable): -->
<span t-esc="company.partner_id.website"/>

<!-- DESPUES (en templates con res_company): -->
<span t-if="res_company.partner_id.website" t-esc="res_company.partner_id.website"/>
```

**Nota:** Usar `t-esc` en vez de `t-field` porque `website` en `res.partner`
es un Char field simple, no un recordset.

**Fuente:** Pharmactive v15→v18 (6 layouts, todos con el mismo patron)

---

## 13. Python: Hooks OCA `_select_additional_fields`

**Versiones afectadas:** v17+ (OCA adopta hooks del core)

**Error:** Columna duplicada en SQL view (AmbiguousColumn), o bien el report
dashboard no muestra los campos esperados.

**Causa:** En v17+, OCA usa el hook `_select_additional_fields()` del core
para añadir campos a `sale.report`, `purchase.report`, etc.
Si un modulo custom tambien añade el mismo campo via `_select_sale()`,
la columna aparece duplicada.

**Ejemplo:** OCA `sale_order_type` añade `type_id` via:
```python
# OCA sale_order_type/reports/sale_report.py
def _select_additional_fields(self):
    res = super()._select_additional_fields()
    res['type_id'] = SQL("s.type_id")
    return res
```

Si el modulo custom tambien hace:
```python
# Custom module — ROMPE porque type_id ya existe
def _select_sale(self):
    return super()._select_sale() + ", s.type_id as type_id"
```

**Fix:** Verificar que modulos OCA estan instalados y que campos aportan.
Eliminar del modulo custom los campos que OCA ya provee.

```bash
# Buscar que campos añade OCA:
grep -rn "_select_additional_fields\|_select_sale\|_group_by_sale" $OCA_SRC/v18/ --include="*.py" -l
```

**Fuente:** Pharmactive v15→v18 (sale_order_type_ext vs OCA sale_order_type)

---

## 14. Python: Dead code en modulos de herencia

**Versiones afectadas:** Todas

**Severidad:** INFO (no rompe, pero es tech debt)

**Causa:** Tras eliminar campos/metodos duplicados, algunos modulos quedan con
solo `pass` en el body de la clase. Estos modulos siguen funcionando (la
herencia es valida) pero no hacen nada y deberian evaluarse para desinstalacion.

**Deteccion:**
```bash
# Buscar modelos que solo tienen pass
grep -rn "class.*Model" migrated_modules/ --include="*.py" -A 5 \
  | grep -B 1 "^\s*pass$"
```

**Accion:** Marcar para revision del equipo. Si el modulo solo tiene `pass`
en todos sus modelos Y no aporta vistas/datos propios, es candidato
a desinstalacion.

**Fuente:** Pharmactive v15→v18 (sale_order_type_ext)

---

## 15. Glue como vehiculo del kit en odoo.sh

**Versiones afectadas:** Todas (especifico de despliegue en odoo.sh)

**Severidad:** PATRON (arquitectura, no es un fix puntual)

**Problema:** odoo.sh NO importa dumps externos ni permite aplicar SQL del kit
sobre la BD migrada automaticamente. La migracion la dispara un commit en
produccion (`backup → upgrade.odoo.com → restore → -u`). Hace falta un vehiculo
para que los fixes del kit corran DENTRO de ese flujo.

**Patron:** un modulo "glue" custom con **doble canal**:

1. **`post_init_hook`** — se ejecuta cuando el glue se INSTALA (entorno staging
   o import inicial). Sirve para import de datos / bootstrap en una BD que aun
   no viene de un upgrade.
2. **`migrations/<ver>/{pre,post,end}-migration.py`** — se ejecutan cuando Odoo
   trata el glue como UPGRADE (el flujo real de odoo.sh). Aqui van embebidos los
   fixes del kit: AEAT close, `preserve_fiscalyear`, P3 (jsonb cast de
   company_dependent), `drop_orphan_fks`, reactivacion de Studio **CURADA**
   (lista de ids validados, NUNCA bulk — ver gotcha #30), etc.

**Bootstrap `installed <vprev>` (clave):** para que Odoo ejecute los `migrate`
scripts (canal 2) en vez de tratar el glue como instalacion nueva (canal 1),
el glue debe figurar como `installed` en la version ORIGEN antes del upgrade.
Se siembra via `prod_live.sql` (aplicado a mano en el Shell de prod ANTES del
commit disparador):
```sql
-- bootstrap del glue como installed en la version origen
INSERT INTO ir_module_module (name, state, latest_version)
VALUES ('<cliente>_migration', 'installed', '<vprev>.1.0.0')
ON CONFLICT (name) DO UPDATE
   SET state = 'installed', latest_version = '<vprev>.1.0.0';
```
Tras el upgrade, Odoo ve `installed <vprev>` + codigo `<vdest>` → lo trata como
UPGRADE → corren los `pre/post/end-migration.py`.

**Deteccion / verificacion:**
```bash
# El glue declara los dos canales:
grep -rn "post_init_hook" migrated_modules/<cliente>_migration/__manifest__.py
ls migrated_modules/<cliente>_migration/migrations/*/
```

**Fuente:** Pharmactive v15→v18 en odoo.sh (modulo glue `pharmactive_migration`).
Ver tambien `phases/fase-5-go-live.md` (seccion odoo.sh con submódulos) y
`playbooks/go_live/odoosh_cutover_commit.md`.

---

## 16. Stock: reservas corruptas + override verbatim de reserva

**Versiones afectadas:** v17+ (cambio del motor de reservas en v16/17)
**Severidad:** CRITICAL — bloquea traspasos, fabricaciones y ventas

Tras migrar aparece, al reservar/validar movimientos de stock:
> *It is not possible to reserve more products of [PRODUCTO] than you have in stock.*
…incluso con stock disponible y `reserved_quantity = 0`. Son **dos problemas distintos** que suelen coexistir:

### 16a. Datos: reservas fantasma / quants sobre-reservados
La migracion deja `stock.quant` con `reserved_quantity > quantity` (reserva sin stock que la respalde) y `stock.move`/`stock.move.line` con `picked=True` en estados no-done. Esto merma el disponible agregado y dispara el error.

**Gotcha clave:** `stock.move._do_unreserve()` (`stock/models/stock_move.py:884`, `StockMove.action_product_forecast_report`) **salta los movimientos con `move.picked=True`** (y las lineas con `ml.picked=True`). Resetear solo la linea NO basta: hay que resetear `picked` en **`stock_move` Y `stock_move_line`** antes de desreservar.

**Fix (reset de reservas, idempotente):**
```python
# 1) quitar picked en MOVIMIENTOS y LINEAS (si no, _do_unreserve los salta)
env.cr.execute("UPDATE stock_move SET picked=FALSE WHERE state NOT IN ('done','cancel') AND picked=TRUE")
env.cr.execute("UPDATE stock_move_line SET picked=FALSE WHERE state NOT IN ('done','cancel') AND picked=TRUE")
env.invalidate_all()
# 2) desreservar todo lo vivo
moves = env['stock.move'].search([('state','in',('confirmed','partially_available','assigned'))])
moves._do_unreserve()
# 3) limpiar reserva residual de quants (fantasma sin linea)
env.cr.execute("UPDATE stock_quant SET reserved_quantity=0 WHERE reserved_quantity!=0")
env.invalidate_all()
# 4) (opcional) re-reservar contra stock real, POR MOVIMIENTO con savepoint
for mv in moves:
    env.cr.execute("SAVEPOINT sp")
    try:
        mv._action_assign(); env.cr.execute("RELEASE SAVEPOINT sp")
    except Exception:
        env.cr.execute("ROLLBACK TO SAVEPOINT sp")  # mov corrupto: no aborta el resto
```
Para entregarlo al cliente como Server Action (DRY-RUN via `raise UserError`, timeout, savepoints), ver skill `odoo-server-action-creator` (gotchas G29-G31).

### 16b. Codigo: override de `_update_reserved_quantity` copiado verbatim de v14/v15
Si el error PERSISTE tras limpiar datos, busca un modulo custom que sobreescriba `stock.quant._update_reserved_quantity`. En v14/v15 ese metodo de quant hacia la comprobacion de disponibilidad y lanzaba el `raise UserError`. En v18 es un wrapper trivial sin esa comprobacion (la logica se movio a `stock.move._update_reserved_quantity`/`_get_reserve_quantity`). Un override copiado tal cual reintroduce el `raise` y rompe la reserva en v18 (sobre todo con stock en ubicacion **padre**). **Fix: eliminar el override.** Patron generico en skill `odoo-module-upgrader` (Python Patterns) y comportamiento en `odoo-deep-knowledge` stock G73/G74.

**Deteccion:**
```bash
# datos: quants sobre-reservados (SQL, sobre la BD migrada)
# SELECT count(*) FROM stock_quant q JOIN stock_location l ON l.id=q.location_id
#   AND l.usage='internal' WHERE q.reserved_quantity > COALESCE(q.quantity,0)+1e-7;
# codigo: override del metodo de reserva
grep -rn "_update_reserved_quantity\|It is not possible to reserve more products" migrated_modules/ --include="*.py"
```

---

## Procedimiento sistematico de deteccion

Ejecutar estas busquedas sobre `migrated_modules/` al inicio de FASE 3:

```bash
# 1. API _search (BLOCKING)
grep -rn "access_rights_uid" migrated_modules/ --include="*.py"

# 2. Action browse sin guard (BLOCKING)
grep -rn "params.get.*action" migrated_modules/ --include="*.py"

# 3. SQL report herencia (CRITICAL)
grep -rn "_select\|_group_by\|_from\|_where" migrated_modules/ --include="*.py" | grep "def "

# 4. tax_totals_json (BLOCKING)
grep -rn "tax_totals_json" migrated_modules/ --include="*.py" --include="*.xml"

# 5. <field t-field> en reports (BLOCKING)
grep -rn "<field t-field" migrated_modules/ --include="*.xml" | grep -v "views/"

# 6. Bootstrap 3/4 classes (WARNING)
grep -rn "text-right\|text-left\|pull-right\|pull-left\|table-condensed" migrated_modules/ --include="*.xml"

# 7. Locale sin comillas (BLOCKING)
grep -rn "== en_US\|== es_ES" migrated_modules/ --include="*.xml"

# 8. website_id.name (BLOCKING)
grep -rn "website_id" migrated_modules/ --include="*.xml" | grep -v "__manifest__"

# 9. address_layout en layouts custom (WARNING — riesgo de duplicacion)
grep -rn "address_layout" migrated_modules/ --include="*.xml"

# 10. Override de reserva copiado de v14/v15 (CRITICAL) — patron #16b
grep -rn "_update_reserved_quantity\|It is not possible to reserve more products" migrated_modules/ --include="*.py"
```

Resultados con hits → revisar y aplicar el fix correspondiente.
