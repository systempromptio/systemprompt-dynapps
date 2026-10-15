<!-- verified: v18.0 @ 2026-06 -->
# Tipos de Dependencia en Odoo

Referencia de dónde se esconden las dependencias de un campo, modelo o método.
Usar durante el paso de clasificación del workflow.

---

## 1. Compute / Related / Depends / Inverse / Search (`.py`)

### `compute=`
```python
campo = fields.Char(compute='_compute_campo')
```
- Buscar el nombre del campo en argumentos `compute=`
- Buscar el nombre del campo en `@api.depends('modelo.campo')`
- Un campo computado puede depender del objetivo vía cadena de depends

### `related=`
```python
campo = fields.Char(related='partner_id.name')
```
- El campo relacionado hereda nombre, tipo y store del campo origen
- Renombrar el origen rompe TODOS los related que lo referencian
- Buscar `related='...modelo.campo...'` con cualquier prefijo de ruta

### `@api.depends`
```python
@api.depends('partner_id.name', 'order_id.amount_total')
```
- Puede referenciar el campo directamente o a través de un relacionado

### `inverse=`
```python
campo = fields.Char(compute='_compute', inverse='_inverse_campo')
```
- Buscar `inverse='nombre_método'` si se renombra el método

### `search=`
```python
campo = fields.Char(compute='_compute', search='_search_campo')
```
- Buscar `search='nombre_método'` si se renombra el método

---

## 2. Vistas XML (`.xml`)

### Referencia directa de campo
```xml
<field name="campo_objetivo"/>
<field name="campo_objetivo" widget="..."/>
```

### Atributos condicionales (v14-v16)
```xml
<field name="otro" attrs="{'invisible': [('campo_objetivo', '=', False)]}"/>
```

### Atributos directos (v17+)
```xml
<field name="otro" invisible="not campo_objetivo"/>
<field name="otro" required="campo_objetivo == 'done'"/>
<field name="otro" readonly="estado != 'draft'"/>
```

### Dominios en vistas
```xml
<field name="partner_id" domain="[('campo_objetivo', '!=', False)]"/>
<field name="order_id" domain="[('campo_objetivo', 'in', ['a','b'])]"/>
```

### Herencia de vista
```xml
<field name="campo_objetivo" position="after">...</field>
<xpath expr="//field[@name='campo_objetivo']" position="..."/>
```

---

## 3. Informes QWeb (`.xml`)

```xml
<t t-field="o.campo_objetivo"/>
<t t-if="o.campo_objetivo">...</t>
<t t-esc="o.campo_objetivo"/>
```
- Buscar en ficheros bajo `report/` o con `<template>` / `<report>`

---

## 4. Server Actions / Automatizaciones (`.xml` y BD)

### En XML de datos
```xml
<field name="code">
    record.campo_objetivo = ...
    if record.campo_objetivo:
</field>
```
- Buscar en `<record model="ir.actions.server">` y `<record model="base.automation">`

### En BD (no cubierto por grep)
Los registros creados manualmente via UI no están en XML.
**Avisar al usuario** de que revise `ir.actions.server` y `base.automation` en BD.

---

## 5. Record Rules / ACL (`.xml` / `.csv`)

### Record rules con dominio
```xml
<field name="domain_force">[('campo_objetivo', '=', uid)]</field>
```
- Buscar en `<record model="ir.rule">`

### `ir.model.access.csv`
```
id,name,model_id:id,group_id:id,perm_read,...
```
- Solo referencia el modelo, no campos individuales
- Relevante si se elimina un MODELO (buscar `model_<nombre_modelo>`)

---

## 6. Módulos Externos — Herencia (`.py` / `.xml`)

### Herencia de modelo Python
```python
class ExtendedModel(models.Model):
    _inherit = 'sale.order'
    campo_nuevo = fields.Char(related='campo_objetivo')
```
- `_inherit` sin `_name` extiende el modelo original
- `_inherits` delega campos a otro modelo
- Buscar en TODOS los módulos, no solo el propietario del campo

### Herencia de vista XML
```xml
<record model="ir.ui.view">
    <field name="inherit_id" ref="sale.view_order_form"/>
    ...
    <xpath expr="//field[@name='campo_objetivo']"/>
</record>
```

---

## 7. Manifiestos y Dependencias (`.py`)

### `__manifest__.py`
```python
'depends': ['sale', 'account']
```
- Si se mueve un campo a otro módulo, los dependientes deben actualizar `depends`

---

## 8. Migraciones Existentes (`migrations/`)

```python
# migrations/16.0.1.0.0/post-migrate.py
cr.execute("UPDATE sale_order SET campo_nuevo = campo_objetivo")
```
- Si el campo ya fue renombrado antes, habrá scripts de migración que lo referencian
- Buscar en `migrations/**/*.py`

---

## 9. Tests (`tests/`)

```python
self.env['sale.order'].create({'campo_objetivo': 'valor'})
order.campo_objetivo
```
- Riesgo BAJO pero deben actualizarse para evitar falsos negativos en CI

---

## 10. Controladores y Wizards (`.py`)

```python
# controllers/main.py
vals = request.env['sale.order'].search([('campo_objetivo', '=', val)])

# wizards/my_wizard.py
campo = fields.Char(related='order_id.campo_objetivo')
```
---

## Lo que pasa de verdad cuando una referencia se rompe

**Medido el 2026-08-15 en v18 y v19**, un caso por módulo, con un campo que no existe:
`tools/bench-orm/probe-broken-refs.sh` (instalación) y `test_broken_refs.py` (uso posterior).
Esta tabla es la que ordena el riesgo del motor, y contradice la intuición de partida.

### Revientan al INSTALAR — o sea, no llegan a producción

| Dependencia rota | Lo que dice Odoo |
|---|---|
| `@api.depends("inexistente")` | `ValueError: Wrong @depends on '_compute_x'. Dependency field 'inexistente' not found` |
| `related="partner_id.inexistente"` | `KeyError: 'Field inexistente referenced in related field definition … does not exist.'` |
| `<field name="inexistente"/>` en una vista | `ValidationError: Error while validating view` |
| `invisible="inexistente"` y demás modificadores | `ValidationError` al validar la vista |
| `ir.rule` con el campo en `domain_force` | `ValidationError: Invalid domain: Invalid field …` (v18 dice `in leaf`, v19 `in condition`) |

Son las **buenas noticias**: el módulo no entra, la base se queda a medias y te enteras el mismo
día. Un análisis de impacto que las marque como lo más grave está mirando al sitio equivocado.

### Instalan en SILENCIO y revientan al usarlas

| Dependencia rota | Cuándo salta |
|---|---|
| `_order = "inexistente desc"` | `ValueError` en el primer `search()` |
| `compute="_metodo_que_no_existe"` | `AttributeError` al leer el campo |
| `search="_metodo_que_no_existe"` | `AttributeError` al filtrar por él |
| `domain` de un `ir.actions.act_window` | al abrir la acción |
| QWeb `t-esc="doc.inexistente"` | al renderizar el informe |
| código de una `ir.actions.server` | al ejecutarla — un botón, un cron, una automatización |

El código de las server actions **no se valida contra el modelo** al instalar: sólo pasa por
`safe_eval`, que mira los opcodes. `record.write({"inexistente": 1})` entra sin una queja.

### No saltan NUNCA

| Dependencia rota | Qué pasa |
|---|---|
| `@api.onchange("inexistente")` | el método **no se dispara jamás**. Silencio absoluto. |
| `@api.constrains("inexistente")` | el método **no se ejecuta** y el registro prohibido **entra**. |

Los dos instalan sin un aviso, y ningún test de negocio los caza: nadie escribe un test que
compruebe que una validación *sigue* saltando. Al renombrar un campo, estos dos decoradores son lo
primero que hay que buscar a mano — el motor los marca CRÍTICO por esto.
