# Translation Mechanics — Odoo Python

## Importación correcta

```python
# v17 y anteriores
from odoo.tools.translate import _

# v18+ (también disponible en el entorno)
# env._("string") — lleva el contexto lang del usuario automáticamente
```

## `_()` — traducción inmediata

Usar para strings evaluados en tiempo de ejecución (mensajes de error, labels dinámicos).

```python
# CORRECTO
raise UserError(_("El pedido %s no puede confirmarse.") % order.name)
raise UserError(_("El pedido %(name)s tiene %(qty)d líneas.") % {"name": order.name, "qty": qty})

# INCORRECTO — f-string impide extracción del término
raise UserError(_(f"El pedido {order.name} no puede confirmarse."))

# INCORRECTO — .format() dentro de _() mismo problema
raise UserError(_("El pedido {name}".format(name=order.name)))
```

**Regla:** `_()` recibe SIEMPRE un literal string. Las variables van FUERA.

## `_lt` — lazy translation (campos de clase)

Evalúa el string solo cuando se accede, no en el import. Obligatorio en:
- `fields.Char(string=_lt("Nombre"))`
- `_description = _lt("Mi Modelo")`
- Atributos de clase que se evalúan antes de que el entorno esté disponible

```python
from odoo.tools.translate import _, _lt

class SaleOrder(models.Model):
    _description = _lt("Pedido de venta")  # lazy — correcto en clase
    
    name = fields.Char(string=_lt("Referencia"))  # lazy — correcto en field
    
    def action_confirm(self):
        raise UserError(_("No se puede confirmar"))  # inmediato — correcto en método
```

**Truco:** Si ves `_()` en un atributo de clase → cambiar a `_lt()`.

## `env._()` — v18+ con contexto de idioma

En Odoo v18, los modelos exponen `env._()` que usa automáticamente el idioma del usuario actual (`self.env.context.get('lang')`).

```python
# v18+ — usa el idioma del usuario en context
message = self.env._("El registro ha sido procesado")

# v17 — equivalente manual
lang = self.env.context.get('lang', 'en_US')
message = _("El registro ha sido procesado")  # sigue siendo válido en v18
```

**NUNCA usar `env._()` en código que debe funcionar en v17** — `AttributeError`.

## Campos con `translate=True`

Los valores almacenados en BD se traducen automáticamente según el idioma del usuario:

```python
name = fields.Char(string="Nombre", translate=True)
description = fields.Html(string="Descripción", translate=True)
```

El valor base se guarda en el idioma de instalación. Las traducciones adicionales se importan vía `.po` con `type=model`.

### Cómo se comporta de verdad, medido

Banco `tools/bench-orm`, 2026-08-15, con `es_ES` activado y un campo `translate=True`:

| Operación | `en_US` | `es_ES` |
|---|---|---|
| `create({'label_i18n': 'hello'})` | `hello` | `hello` |
| `with_context(lang='es_ES').label_i18n = 'hola'` | `hello` | `hola` |
| `rec.label_i18n = 'goodbye'` — **sin contexto de idioma** | `goodbye` | **`hola`** |

Y en la propia fila de Postgres:

```
SELECT label_i18n FROM dyn_sec  ->  {'en_US': 'hello', 'es_ES': 'hola'}
```

Un campo traducible **no es una columna de texto**: es un `jsonb` con una clave por idioma, en la
misma tabla del registro (desde v16; antes vivía en `ir.translation`).

> ⚠️ **Una escritura sin contexto de idioma solo toca el idioma actual, y las demás traducciones
> sobreviven en silencio.** Es el error que más caro sale: corriges una errata en inglés y el
> español sigue diciendo lo de antes, para siempre, sin que nada lo señale. Si el cambio debe
> afectar a todos los idiomas, hay que escribirlo idioma por idioma —o vaciar los demás a
> propósito—, porque un `write` normal no los limpia.

## Contextos de traducción

Para forzar un idioma específico en una operación:

```python
# Forzar traducción al español independientemente del usuario
record.with_context(lang='es_ES').name
```

## Placeholders — resumen

| Patrón | Ejemplo | ¿Correcto? |
|--------|---------|------------|
| `%s` / `%d` con `%` fuera | `_("Hola %s") % name` | ✓ |
| `%(key)s` con `%` fuera | `_("Hola %(n)s") % {"n": name}` | ✓ |
| f-string dentro | `_(f"Hola {name}")` | ✗ |
| `.format()` dentro | `_("Hola {}".format(name))` | ✗ |
| concatenación dentro | `_("Hola " + name)` | ✗ |
