# Mixins, tracking y subtypes

Base v18. Citas verificadas en `$ODOO_SRC/odoo18/addons/mail/models/`.

## Mixins

```python
class BusinessOrder(models.Model):
    _name = "business.order"
    _inherit = ["mail.thread", "mail.activity.mixin"]
```

- **`mail.thread`** (`mail_thread.py`): aporta el chatter (mensajes), el tracking de
  campos (`tracking=True`), los followers (`message_subscribe`) y la API de mensajes
  (`message_post`, `_message_log`). Engancha el tracking en `write` via
  `_compute_field_value` / `_track_prepare` (`mail/models/mail_thread.py:440`, `MailThread._compute_field_value`).
- **`mail.activity.mixin`** (`mail_activity_mixin.py`): aporta el "planificador" de
  actividades: campos `activity_ids`, `activity_state`, `activity_user_id`,
  `activity_date_deadline`... (`mail/models/mail_activity_mixin.py:74-78`, `MailActivityMixin.activity_date_deadline`) y los metodos
  `activity_schedule` / `activity_feedback` / `activity_unlink`.

Hereda solo `mail.thread` si no necesitas actividades. `mail.activity.mixin` NO implica
`mail.thread` (aunque casi siempre se usan juntos).

## Tracking de campos

Marca el campo con `tracking=True` (o un entero para fijar el orden en el mensaje de
seguimiento):

```python
name  = fields.Char(tracking=True)         # tracking=1 implicito
state = fields.Selection(..., tracking=10) # orden explicito
email = fields.Char(tracking=1)
```

Confirmado en `mail/models/res_partner.py:19-27`, `Partner.name` (v18): `name = fields.Char(tracking=1)`,
`... tracking=True`. El entero ordena los cambios dentro del mensaje de tracking.

Cuando un campo trackeado cambia en `write`, `mail.thread` genera automaticamente un
mensaje de seguimiento con los `mail.tracking.value` (modelo `mail_tracking_value.py`),
mostrando valor antiguo -> nuevo.

### CUANDO ocurre eso, exactamente: en el precommit — MEDIDO (v18 y v19)

No en el `write`. El `write` **encola**; lo que crea el mensaje es la cola
`env.cr.precommit`, que corre al hacer **commit**. Medido en un `TransactionCase`
(`tools/bench-orm/addons/dyn_mail`):

| Momento | `len(rec.message_ids)` | valores de tracking |
|---|---|---|
| tras `create` + precommit | 1 | 0 |
| **justo despues del `write`** | **1** | **0** |
| tras `env.cr.precommit.run()` | **2** | **1** |

Dos consecuencias, y la segunda es la que hace dano:

1. **En tests**: un `TransactionCase` no hace commit nunca, asi que hay que llamar a
   `self.env.cr.precommit.run()` antes de afirmar. Sin eso el test concluye que el
   tracking no funciona — y un test que afirme «no se registro nada» **pasa por el
   motivo equivocado**, que es un verde falso de manual.
2. **En codigo**: cualquier logica que lea `message_ids` o `tracking_value_ids`
   **dentro de la misma transaccion**, justo despues del `write` —un compute, un
   `@api.constrains`, un metodo que encadena— no ve el mensaje todavia.

`_track_subtype` tambien se evalua ahi, no en el `write`.

### Sin `mail.thread`, `tracking=True` avisa — no es silencioso

El campo se declara igual (`_fields['name'].tracking` es `True`), pero **cada carga del
registro** suelta un WARNING por campo:

```
Field dyn.mail.untracked.name: unknown parameter 'tracking', if this is an actual
parameter you may want to override the method _valid_field_parameter ...
```

Se sacan todos con `grep "unknown parameter 'tracking'"` sobre el log de arranque. El
modulo instala y funciona; simplemente no hay chatter donde guardar nada.

### `_track_subtype(self, initial_values)`

Override para elegir un subtype condicional segun el cambio. Recibe los valores ANTES
del write. Devuelve un `mail.message.subtype` o llama a `super()`:

```python
def _track_subtype(self, initial_values):
    self.ensure_one()
    if "state" in initial_values and self.state == "done":
        return self.env.ref("my_module.mt_order_done")
    return super()._track_subtype(initial_values)
```

Firma real: `mail/models/mail_thread.py:596`, `MailThread._track_subtype` (v18).

### `_track_template(self, changes)`

Override para enviar un `mail.template` cuando ciertos campos cambian. Devuelve un dict
`{field: (template, options)}`. Firma real: `mail/models/mail_thread.py:704`, `MailThread._track_template` (v18).

### Desactivar tracking puntualmente

Contexto `tracking_disable=True` o `mail_notrack=True` evita la generacion de tracking
en ese `write`/`create` (`mail/models/mail_thread.py:440`, `MailThread._compute_field_value`).

## Subtypes (`mail.message.subtype`)

Los subtypes clasifican mensajes/seguimientos y permiten que cada follower elija que
notificaciones recibe. Se declaran como **data records** (delega en `odoo-data-files`):

```xml
<record id="mt_order_done" model="mail.message.subtype">
    <field name="name">Order Done</field>
    <field name="res_model">business.order</field>
    <field name="default" eval="True"/>
    <field name="description">Order marked as done</field>
</record>
```

Campos clave (`mail_message_subtype.py`, v18):
- `res_model` (`:39`): modelo al que aplica; `False` = todos los modelos.
- `default` (`:40`, default `True`): activado al suscribir un follower.
- `internal` (`:26`): solo visible para empleados internos.
- `hidden` (`:42`): oculto en las opciones del follower.
- `track_recipients` (`:43`).

### Subtypes estandar reutilizables

- `mail.mt_comment` — comentario (notifica).
- `mail.mt_note` — nota interna (log silencioso).

Usalos via `subtype_xmlid` en `message_post` en lugar de crear los tuyos cuando encajen.

### `_creation_subtype(self)`

Devuelve el subtype que se dispara al crear el record (vacio por defecto). Override para
loguear un subtype concreto en la creacion:

```python
def _creation_subtype(self):
    return self.env.ref("my_module.mt_order_created")
```

Firma real: `mail/models/mail_thread.py:445`, `MailThread._creation_subtype` (v18). Relacionado: `_creation_message` (`mail/models/mail_thread.py:452`, `MailThread._creation_message`)
devuelve el body del mensaje de creacion.

## Diffs v17 -> v18 -> v19

- `_track_subtype`, `_track_template`, `_creation_subtype` y el modelo
  `mail.message.subtype` mantienen firma/estructura estable en v17/v18/v19.
- `tracking=True` se declara igual en las tres versiones.
