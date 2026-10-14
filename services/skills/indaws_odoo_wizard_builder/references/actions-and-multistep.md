# Botones de acción y wizards multi-paso

Los botones del wizard son métodos `type="object"` que devuelven un dict. Lo que
devuelvan decide qué pasa al pulsarlos.

## Cerrar el wizard

```python
def action_apply(self):
    self.ensure_one()
    # ... lógica ...
    return {'type': 'ir.actions.act_window_close'}
```

`{'type': 'ir.actions.act_window_close'}` cierra la ventana modal. **Devolver
`False`/`None`** (o no devolver nada) tiene el mismo efecto visual: Odoo cierra
el wizard. Por eso, si quieres que el flujo CONTINÚE, debes devolver una acción
explícita.

## Recargar / redirigir a los registros reales

Tras actuar, abre los registros resultantes en el modelo normal. Patrón real
(`account.move.reversal.reverse_moves`, v18 `account/wizard/account_move_reversal.py:154`, `AccountMoveReversal.reverse_moves`):

```python
action = {
    'name': _('Reverse Moves'),
    'type': 'ir.actions.act_window',
    'res_model': 'account.move',
}
if len(moves_to_redirect) == 1:
    action.update({'view_mode': 'form', 'res_id': moves_to_redirect.id})
else:
    action.update({'view_mode': 'list,form',
                   'domain': [('id', 'in', moves_to_redirect.ids)]})
return action
```

Para refrescar la vista de origen sin redirigir a otro modelo:

```python
return {'type': 'ir.actions.client', 'tag': 'reload'}
```

## Lanzar un report / otra acción

```python
def action_print(self):
    self.ensure_one()
    return self.env.ref('my_module.action_report_x').report_action(self.line_ids)
```

(`report_action` para QWeb PDF; o devuelve la `ir.actions.report` directamente.)

## Multi-paso

Dos patrones equivalentes; elige uno.

### A) Un solo wizard con campo `state`

Un campo selection controla qué se ve; los botones cambian `state` y devuelven
la MISMA acción reabriendo el wizard (mismo `res_id`).

```python
state = fields.Selection([('step1', 'Step 1'), ('step2', 'Step 2')],
                         default='step1')

def action_next(self):
    self.ensure_one()
    self.state = 'step2'
    return {
        'type': 'ir.actions.act_window',
        'res_model': self._name,
        'res_id': self.id,            # reabre el MISMO registro transient
        'view_mode': 'form',
        'target': 'new',
    }
```

La visibilidad por `state` se hace en la vista con `invisible="state != 'step2'"`
(ver `wizard-views.md`).

### B) Encadenar wizards distintos

Cada paso es un TransientModel propio; el botón crea/abre el siguiente y le pasa
contexto. Útil cuando los pasos tienen campos muy distintos.

```python
def action_next(self):
    self.ensure_one()
    return {
        'type': 'ir.actions.act_window',
        'res_model': 'my.module.step2.wizard',
        'view_mode': 'form',
        'target': 'new',
        'context': {'default_step1_id': self.id,   # transient → transient: OK mientras viva la sesión
                    'default_partner_id': self.partner_id.id},
    }
```

> Nota: un m2o de un transient a otro transient es aceptable durante el flujo,
> pero recuerda que ambos son efímeros (vacuum). No los uses como almacenamiento.

## Checklist de botones
- `self.ensure_one()` al principio (salvo proceso multi-registro deliberado).
- Devuelve acción explícita si el flujo continúa; `act_window_close` para cerrar.
- No dependas de que la vista origen se refresque sola: redirige o `reload`.
