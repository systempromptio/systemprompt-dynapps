# Apertura del wizard, context y default_get

## Abrir el wizard: ir.actions.act_window con target="new"

Un wizard se abre devolviendo (desde un botón/acción de servidor) un dict
`ir.actions.act_window` con `target="new"` (popup modal):

```python
def action_open_wizard(self):
    return {
        'name': _('Do Something'),
        'type': 'ir.actions.act_window',
        'res_model': 'my.module.do.something',
        'view_mode': 'form',
        'target': 'new',                       # modal; 'current' lo abriría a pantalla completa
        'context': {
            'default_partner_id': self.partner_id.id,   # precarga directa de un campo
            'active_id': self.id,
            'active_ids': self.ids,
            'active_model': self._name,
        },
    }
```

También se declara como acción XML lanzada desde un botón de la vista o un menú:

```xml
<record id="action_do_something" model="ir.actions.act_window">
    <field name="name">Do Something</field>
    <field name="res_model">my.module.do.something</field>
    <field name="view_mode">form</field>
    <field name="target">new</field>
</record>
```

### Cómo se pasa el contexto

- `default_<campo>`: Odoo precarga ese campo con el valor dado. Es la vía más
  simple cuando ya tienes el valor en el momento de abrir.
- `active_id` (int): id del registro de origen (acción sobre 1 registro).
- `active_ids` (list): ids seleccionados (acción de lista multi-selección).
- `active_model` (str): modelo de origen (`'account.move'`, etc.).
- `active_domain` (list): dominio de la selección/filtro de origen.

Cuando el botón se declara en una vista de lista/formulario, Odoo rellena
`active_*` automáticamente con la selección; no hace falta ponerlos a mano.

## default_get: precarga desde el contexto

`default_get(self, fields_list)` se llama al crear el formulario para inicializar
campos. Es el sitio canónico para leer `active_ids`/`active_model` y precargar el
wizard desde la selección de origen.

**No es cosa del cliente web.** Medido (`tools/bench-orm`, v18 y v19): un
`create()` por ORM **también** pasa por tu `default_get`, y el contexto llega
igual. Tres consecuencias:

- `Wiz.with_context(active_ids=[7, 9]).create({})` rellena los campos desde el
  contexto sin que haya navegador de por medio — es la forma de probar un wizard
  en un test sin tocar la vista.
- Lo que venga en `vals` **gana**: `create({'from_ctx': 'lo mío'})` no lo pisa el
  `default_get`.
- Y por lo mismo: si tu `default_get` lanza `UserError` cuando la selección no le
  gusta, ese error salta también en cualquier `create()` interno del wizard. Un
  test o un cron que cree el wizard sin contexto se lo come.

`fields_list` **no trae todos los campos del modelo: trae los que faltan en
`vals`** (medido: creando con `{'name': ...}`, `name` no aparece en la lista y sí
el resto). Por eso el `if 'campo' in fields_list` no es cosmético — sin él
asignas defaults a campos que el llamante ya había puesto.

Patrón real (`account.move.reversal`, v18
`odoo18/addons/account/wizard/account_move_reversal.py::_check_journal_type`):

```python
@api.model
def default_get(self, fields):
    res = super().default_get(fields)
    move_ids = (
        self.env['account.move'].browse(self.env.context['active_ids'])
        if self.env.context.get('active_model') == 'account.move'
        else self.env['account.move']
    )
    if len(move_ids.company_id) > 1:
        raise UserError(_("All selected moves for reversal must belong to the same company."))
    if any(move.state != "posted" for move in move_ids):
        raise UserError(_('You can only reverse posted moves.'))
    if 'company_id' in fields:                       # respeta fields_list
        res['company_id'] = move_ids.company_id.id or self.env.company.id
    if 'move_ids' in fields:
        res['move_ids'] = [(6, 0, move_ids.ids)]     # comando m2m
    return res
```

Puntos clave del patrón:
1. Llamar a `super().default_get(fields)` y partir de su `res`.
2. Comprobar `active_model` ANTES de hacer `browse(active_ids)` — el wizard puede
   abrirse desde sitios distintos.
3. Validar pronto (`UserError`) si la selección no es apta.
4. Comprobar `if 'campo' in fields` (el parámetro `fields_list`) antes de asignar.
5. Para m2m/o2m usar comandos ORM (`(6, 0, ids)`, `Command.set(...)`).

`active_id` vs `active_ids`: si la acción viene de un solo registro tendrás
`active_id`; las acciones de lista pasan `active_ids`. Lee siempre `active_ids`
(lista) en default_get; si necesitas el singular usa `context.get('active_id')`.

## Wizard AGNÓSTICO de modelo (reutilizable por cualquier consumidor)

El patrón de arriba lee `active_model` pero luego hace `browse` sobre UN modelo fijo
(`account.move`). Si el wizard debe servir a **varios modelos** (o quieres que sea un motor
reutilizable), NO hardcodees `self.env['x']`: lee el modelo del contexto.

```python
@api.model
def default_get(self, fields_list):
    res = super().default_get(fields_list)
    model = self.env.context.get('active_model')
    ids = self.env.context.get('active_ids', [])
    if model and ids:
        records = self.env[model].browse(ids)          # el modelo lo decide el consumidor
        # ...lógica genérica sobre `records` (sin asumir un modelo concreto)
    return res
```

Guarda `active_model` en un `Char` del wizard si necesitas reusarlo en el método de acción
(el contexto no siempre se propaga al `button` del propio wizard).

### Los bindings van POR CONSUMIDOR, no en el módulo del wizard

Que un wizard aparezca en el menú **Acción ▸** de un modelo lo decide el **binding** de su acción,
no el `TransientModel`. Declara el binding **en el módulo de cada consumidor** (o en el propio si
solo hay uno), apuntando `binding_model_id` al modelo de ese consumidor:

```xml
<record id="action_generic_wizard_partner" model="ir.actions.act_window">
    <field name="name">Do Something</field>
    <field name="res_model">my.generic.wizard</field>
    <field name="view_mode">form</field>
    <field name="target">new</field>
    <field name="binding_model_id" ref="base.model_res_partner"/>   <!-- por consumidor -->
    <field name="binding_view_types">list,form</field>
</record>
```
Campos verificados v19: `binding_model_id` / `binding_type` / `binding_view_types`
(`odoo/addons/base/models/ir_actions.py:74-79`, `IrActions`); el cliente los resuelve con
`_get_bindings(model_name)` (`:186`). Así el módulo del wizard queda agnóstico y cada consumidor
declara dónde aparece.

