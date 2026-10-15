# Diferencias Semanticas: Codigo Fuente vs Server Action

## Regla Fundamental

> En una server action NO estas dentro de un metodo de modelo.
> No hay `self`. No hay `@api.depends`. No hay `return`.
> Estas en un sandbox con variables inyectadas.

---

## Tabla de Traduccion Completa

### 1. Acceso al Entorno

| Codigo Fuente | Server Action | Notas |
|---------------|---------------|-------|
| `self.env` | `env` | Directo, sin self |
| `self.env.cr` | `env.cr` | Cursor de BD |
| `self.env.uid` | `uid` o `env.uid` | UID disponible directamente |
| `self.env.user` | `user` o `env.user` | Usuario como variable directa |
| `self.env.context` | `env.context` | Contexto actual |
| `self.env.company` | `env.company` | Compania actual |
| `self.env.companies` | `env.companies` | Companias permitidas |
| `self.env.lang` | `env.lang` | Idioma actual |
| `self.env.ref('xml_id')` | `env.ref('xml_id')` | Buscar por XML ID |

### 2. Acceso al Modelo y Registros

| Codigo Fuente | Server Action | Notas |
|---------------|---------------|-------|
| `self` | `record` o `records` | **CAMBIO CRITICO** |
| `self.browse(ids)` | `model.browse(ids)` | `model` es recordset vacio del modelo |
| `self.search(domain)` | `model.search(domain)` | Busqueda sobre el modelo |
| `self.create(vals)` | `model.create(vals)` o `env['model'].create(vals)` | Crear registros |
| `self.ensure_one()` | `record.ensure_one()` | Solo si record tiene 1 elemento |
| `for rec in self:` | `for rec in records:` | Iterar sobre registros |
| `self.mapped('field')` | `records.mapped('field')` | Mapear campo |
| `self.filtered(lambda)` | `records.filtered(lambda)` | Filtrar registros |
| `self.sorted(key)` | `records.sorted(key)` | Ordenar registros |

### 3. Escritura de Campos (DIFERENCIA CRITICA)

| Codigo Fuente | Server Action | Por que |
|---------------|---------------|---------|
| `self.name = 'valor'` | `record.write({'name': 'valor'})` | **STORE_ATTR bloqueado** |
| `self.field = value` | `record.write({'field': value})` | **STORE_ATTR bloqueado** |
| `rec.name = 'x'` | `rec.write({'name': 'x'})` | Incluso en loops |
| `self.state = 'done'` | `record.write({'state': 'done'})` | Siempre write() |

**ESTA ES LA DIFERENCIA MAS IMPORTANTE Y LA FUENTE #1 DE ERRORES**

```python
# CODIGO FUENTE (funciona)
for rec in self:
    rec.name = rec.partner_id.name

# SERVER ACTION (FALLA - STORE_ATTR)
for rec in records:
    rec.name = rec.partner_id.name  # ERROR!

# SERVER ACTION (correcto)
for rec in records:
    rec.write({'name': rec.partner_id.name})

# SERVER ACTION (optimizado - batch)
for rec in records:
    rec.write({'name': rec.partner_id.name})
# O mejor aun si es el mismo valor para todos:
records.write({'name': 'valor_comun'})
```

### 4. Campos Relacionales x2many

| Codigo Fuente | Server Action | Notas |
|---------------|---------------|-------|
| `self.line_ids = [(0, 0, vals)]` | `record.write({'line_ids': [Command.create(vals)]})` | Usar Command namespace |
| `self.tag_ids = [(4, id)]` | `record.write({'tag_ids': [Command.link(id)]})` | Linkear existente |
| `self.tag_ids = [(6, 0, ids)]` | `record.write({'tag_ids': [Command.set(ids)]})` | Reemplazar todos |
| `self.line_ids = [(2, id)]` | `record.write({'line_ids': [Command.delete(id)]})` | Eliminar |
| `self.line_ids = [(3, id)]` | `record.write({'line_ids': [Command.unlink(id)]})` | Desvincular |
| `self.line_ids = [(1, id, vals)]` | `record.write({'line_ids': [Command.update(id, vals)]})` | Actualizar existente |
| `self.line_ids = [(5,)]` | `record.write({'line_ids': [Command.clear()]})` | Limpiar todos |

**Equivalencia de tuplas clasicas vs Command:**

| Tupla clasica | Command | Efecto |
|---------------|---------|--------|
| `(0, 0, vals)` | `Command.create(vals)` | Crear y vincular |
| `(1, id, vals)` | `Command.update(id, vals)` | Actualizar vinculado |
| `(2, id)` | `Command.delete(id)` | Eliminar de BD |
| `(3, id)` | `Command.unlink(id)` | Desvincular sin borrar |
| `(4, id)` | `Command.link(id)` | Vincular existente |
| `(5,)` | `Command.clear()` | Desvincular todos |
| `(6, 0, ids)` | `Command.set(ids)` | Reemplazar vinculados |

### 5. Imports y Modulos

#### Odoo Estandar (sin indaws_base)

| Codigo Fuente | Server Action | Alternativa |
|---------------|---------------|-------------|
| `import json` | PROHIBIDO | `json` ya disponible (via mail/base_automation) |
| `from datetime import date` | PROHIBIDO | `datetime.date` ya disponible |
| `from dateutil.relativedelta import relativedelta` | PROHIBIDO | `dateutil.relativedelta` ya disponible |
| `import pytz` | PROHIBIDO | `timezone` (funcion) ya disponible |
| `import base64` | PROHIBIDO | `b64encode`, `b64decode` ya disponibles |
| `import logging` | PROHIBIDO | `log()` y `_logger` ya disponibles |
| `from odoo.exceptions import UserError` | PROHIBIDO | `UserError` ya disponible |
| `from odoo import Command` | PROHIBIDO | `Command` ya disponible |
| `import re` | PROHIBIDO | **NO HAY ALTERNATIVA** - ver Gotcha G02 |
| `import os` | PROHIBIDO | **NO HAY ALTERNATIVA** (ni deberia) |
| `import requests` | PROHIBIDO | **NO HAY ALTERNATIVA** directa - ver Gotcha G02 |

#### Con indaws_base instalado (DESBLOQUEA imports)

`indaws_base` inyecta `wrap_module()` e `import_module()` en el eval context,
lo que permite importar CUALQUIER modulo Python instalado en el servidor.

| Codigo Fuente | Server Action (con indaws_base) | Notas |
|---------------|--------------------------------|-------|
| `import re` | `re = wrap_module('re', ['findall', 'sub', 'match', 'search', 'split', 'compile'])` | Regex completo |
| `import requests` | `requests = import_module('requests', ['get', 'post', 'put', 'delete'])` | HTTP client |
| `import csv` | `csv = wrap_module('csv', ['reader', 'writer', 'DictReader', 'DictWriter'])` | CSV read/write |
| `import hashlib` | `hashlib = import_module('hashlib', ['md5', 'sha256'])` | Hashing |
| `import io` | `io = import_module('io', ['StringIO', 'BytesIO'])` | Streams |
| `import json` | `json` (ya inyectado directamente) | Garantizado |
| `import xml.etree.ElementTree as ET` | `ET = import_module('xml.etree.ElementTree', ['parse', 'fromstring'])` | XML parsing |
| `import urllib.parse` | `urllib_parse = import_module('urllib.parse', ['urlencode', 'quote'])` | URL encoding |
| `open('/tmp/file.csv')` | `open('/tmp/file.csv')` | `open` inyectado |

**Regla con indaws_base**: Si el modulo Python esta instalado en el servidor, puedes importarlo.
La sintaxis cambia (`import X` -> `wrap_module/import_module`), pero la funcionalidad esta completa.

### 6. Retorno de Valores

| Codigo Fuente | Server Action | Notas |
|---------------|---------------|-------|
| `return action_dict` | `action = action_dict` | Asignar a variable `action` |
| `return {'type': 'ir.actions.act_window', ...}` | `action = {'type': 'ir.actions.act_window', ...}` | Abrir ventana |
| `return True` | (no hacer nada) | No necesario |
| `return` | (no hacer nada) | Ejecucion termina sola |

```python
# SERVER ACTION - Retornar accion de ventana
action = {
    'type': 'ir.actions.act_window',
    'name': 'Facturas',
    'res_model': 'account.move',
    'view_mode': 'list,form',
    'domain': [('partner_id', '=', record.partner_id.id)],
    'context': {'default_partner_id': record.partner_id.id},
}
```

### 7. Decoradores y Estructura

| Codigo Fuente | Server Action | Notas |
|---------------|---------------|-------|
| `@api.model` | No existe | El codigo se ejecuta directamente |
| `@api.depends(...)` | No existe | No hay campos computados |
| `@api.onchange(...)` | No existe | No hay onchanges |
| `@api.constrains(...)` | No existe | No hay constraints |
| `def mi_metodo(self):` | No existe | No se definen metodos |
| `class MiModelo(models.Model):` | No existe | No se definen clases |
| `super().metodo()` | No existe | No hay herencia |

### 8. Logging

| Codigo Fuente | Server Action | Destino |
|---------------|---------------|---------|
| `_logger = logging.getLogger(__name__)` | Ya disponible como `_logger` | Server logs |
| `_logger.info('msg')` | `_logger.info('msg')` | Server logs |
| `_logger.warning('msg')` | `_logger.warning('msg')` | Server logs |
| `_logger.error('msg')` | `_logger.error('msg')` | Server logs |
| N/A | `log('msg', level='info')` | Tabla `ir.logging` (UI) |

### 9. Excepciones

| Codigo Fuente | Server Action | Notas |
|---------------|---------------|-------|
| `raise UserError(_("Mensaje"))` | `raise UserError("Mensaje")` | **Sin _() en server actions** |
| `raise ValidationError(...)` | `raise UserError(...)` | Solo UserError disponible |
| `raise AccessError(...)` | `raise UserError(...)` | Solo UserError disponible |

**IMPORTANTE**: `ValidationError` y `AccessError` NO estan en el contexto. Solo `UserError`.

### 10. Transacciones y SQL

| Codigo Fuente | Server Action | Notas |
|---------------|---------------|-------|
| `self.env.cr.execute(sql)` | `env.cr.execute(sql)` | Funciona pero CUIDADO |
| `self.env.cr.commit()` | **PROHIBIDO** | Nunca en server actions |
| `self.env.cr.rollback()` | **PROHIBIDO** | Nunca en server actions |
| `self.flush_recordset()` | `record.flush_recordset()` | Funciona |
| `self.invalidate_recordset()` | `record.invalidate_recordset()` | Funciona |

### 11. Sudo y Seguridad

| Codigo Fuente | Server Action | Notas |
|---------------|---------------|-------|
| `self.sudo()` | `record.sudo()` | Funciona |
| `self.with_user(user)` | `record.with_user(user)` | Funciona |
| `self.with_company(company)` | `record.with_company(company)` | Funciona |
| `self.with_context(key=val)` | `record.with_context(key=val)` | Funciona |

### 12. Traducciones

| Codigo Fuente | Server Action | Notas |
|---------------|---------------|-------|
| `_("Texto traducible")` | `"Texto directo"` | **_() NO disponible** |

La funcion de traduccion `_()` no esta en el eval context. Los textos en server actions no son traducibles via el mecanismo estandar de Odoo.
