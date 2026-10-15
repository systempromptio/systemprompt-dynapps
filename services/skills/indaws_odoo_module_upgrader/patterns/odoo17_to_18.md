# Patrones de Migración: Odoo 17 → 18

Cambios breaking del salto v17→v18. Menos destructivo que v16→v17, pero con
trampas sutiles en reportes QWeb, layout y campos.

## XML/Views

### 1. layout_document_title ya no existe en el report layout
```xml
<!-- BEFORE v17 -->
<t t-call="web.external_layout">
    <t t-set="layout_document_title">My Report</t>
    ...
</t>

<!-- AFTER v18 — usar t-set en el header o t-call con title param -->
<t t-call="web.external_layout">
    ...
</t>
```

**Gotcha:** En v18, el título del documento se gestiona de forma diferente.
Si el reporte necesita título, usar el nombre del `ir.actions.report`.

### 2. field tag dentro de t-foreach restringido
```xml
<!-- BEFORE v17 — funcionaba -->
<t t-foreach="doc.line_ids" t-as="line">
    <field name="name" t-field="line.name"/>
</t>

<!-- AFTER v18 — usar t-esc o t-out -->
<t t-foreach="doc.line_ids" t-as="line">
    <span t-out="line.name"/>
</t>
```

**Regla:** Dentro de `t-foreach`, no usar `<field>` ni `t-field` sobre la variable
del iterador. Usar `t-out` (o `t-esc` deprecated).

### 3. t-raw eliminado completamente
```xml
<!-- BEFORE (deprecated desde v15, error en v18) -->
<span t-raw="record.description"/>

<!-- AFTER v18 -->
<span t-out="record.description"/>
```

### 4. Bootstrap 5 cambios adicionales
v18 actualiza algunas clases Bootstrap que v17 ya usaba:

```xml
<!-- Verificar uso de clases deprecated -->
<!-- float-left → float-start (ya desde v16, pero v18 es estricto) -->
<!-- float-right → float-end -->
<!-- text-left → text-start -->
<!-- text-right → text-end -->
<!-- ml-* → ms-* -->
<!-- mr-* → me-* -->
<!-- pl-* → ps-* -->
<!-- pr-* → pe-* -->
```

## Python

### 5. slug/unslug reubicado
```python
# BEFORE v17
from odoo.addons.http_routing.models.ir_http import slug, unslug

# AFTER v18
# http_routing absorbido en core
from odoo.http import request

def slug(value):
    return request.env['ir.http']._slug(value)

def unslug(value):
    return request.env['ir.http']._unslug(value)
```

**Gotcha:** El módulo `http_routing` ya no existe como addon separado en v18.
Si el módulo lo tiene en `depends`, eliminarlo.

### 6. report_action cambios
```python
# BEFORE v17
return self.env.ref('module.report_name').report_action(self)

# AFTER v18 — sigue funcionando, pero:
# Si el reporte usa t-call="web.html_container" el wrapper cambió
```

### 7. _register_hook → init (en algunos casos)
Verificar si `_register_hook` sigue siendo llamado. En v18 algunos hooks
se movieron a `__init__` del módulo.

### 8. Campos company_dependent
```python
# BEFORE v17
field = fields.Char(company_dependent=True)

# AFTER v18 — la declaracion sigue igual, el ALMACENAMIENTO cambia por completo:
# ir.property ESTA ELIMINADO en v18 (odoo/addons/base/models/ir_property.py existe en v17, NO en v18).
# El valor pasa a una columna jsonb de la tabla del propio modelo, con company_id como clave
# (odoo18/odoo/fields.py:180 docstring, :280 "Company-dependent fields are stored as jsonb").
# => Toda query/ORM/SQL directa contra ir.property NO se "actualiza": hay que REESCRIBIRLA.
#    Un search([('property_name','=',...)]) sobre ir.property revienta: el modelo no existe.
```

## QWeb Reports — CRÍTICOS

### 9. External layout CSS classes
```xml
<!-- BEFORE v17 -->
<div class="page">
    <div class="oe_structure"/>

<!-- AFTER v18 — verificar que las clases CSS existan -->
<!-- Algunas clases de layout cambiaron su styling -->
```

### 10. Tax totals rendering
```xml
<!-- BEFORE v17 -->
<t t-set="tax_totals" t-value="doc.tax_totals"/>
<t t-call="account.document_tax_totals"/>

<!-- AFTER v18 -->
<t t-set="tax_totals" t-value="doc.tax_totals"/>
<t t-call="account.document_tax_totals">
    <t t-set="total_label" t-value="'Total'"/>
</t>
```

### 11. Report render API
```python
# BEFORE v17
pdf = self.env['ir.actions.report']._render_qweb_pdf(
    'module.report_template', res_ids=[record.id])

# AFTER v18
pdf = self.env['ir.actions.report']._render_qweb_pdf(
    self.env.ref('module.report_template'), res_ids=[record.id])
# Primer argumento ahora espera un recordset, no un string XML ID
```

## JavaScript

### 12. Tour registration
```javascript
// BEFORE v17
import tour from "web_tour.tour";
tour.register("my_tour", { ... }, [...]);

// AFTER v18
import { registry } from "@web/core/registry";
registry.category("web_tour.tours").add("my_tour", {
    steps: () => [...],
});
```

### 13. AbstractWebClient ya no existe
Si el módulo extiende `AbstractWebClient`, migrar a `WebClient` component (OWL).

## Detección Completa

```bash
# t-raw (debe ser 0)
grep -rn 't-raw=' --include="*.xml" | wc -l

# slug import viejo
grep -rn 'from odoo.addons.http_routing' --include="*.py" | wc -l

# http_routing en depends
grep -rn 'http_routing' --include="__manifest__.py" | wc -l

# layout_document_title
grep -rn 'layout_document_title' --include="*.xml" | wc -l

# Bootstrap clases viejas
grep -rn 'float-left\|float-right\|text-left\|text-right\|ml-\|mr-\|pl-\|pr-' --include="*.xml" | wc -l

# Tour registration vieja
grep -rn 'tour.register\|web_tour.tour' --include="*.js" | wc -l

# field tag en t-foreach
grep -rn 't-field=.*_as' --include="*.xml" | wc -l
```
