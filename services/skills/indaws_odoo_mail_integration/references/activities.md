# Actividades (mail.activity.mixin)

Base v18. Citas en `$ODOO_SRC/odoo18/addons/mail/models/mail_activity_mixin.py`.

Requiere `_inherit = [..., "mail.activity.mixin"]`.

## Campos del mixin (computados)

`mail/models/mail_activity_mixin.py:50-79`, `MailActivityMixin.activity_ids` (v18):
- `activity_ids` (One2many a `mail.activity`).
- `activity_state` (Selection: `overdue` / `today` / `planned`), computado + searchable.
- `activity_user_id` (Many2one a `res.users`), computado + searchable.
- `activity_type_id` (Many2one a `mail.activity.type`), related, searchable.
- `activity_type_icon`, `activity_date_deadline`, `my_activity_date_deadline`,
  `activity_summary`, `activity_exception_decoration`, `activity_exception_icon`.

Estos campos alimentan el widget de actividades en kanban/list/form.

## `activity_schedule` (firma real)

`mail/models/mail_activity_mixin.py:345`, `MailActivityMixin.activity_schedule` (v18):

```python
def activity_schedule(self, act_type_xmlid='', date_deadline=None,
                      summary='', note='', **act_values):
```

- `act_type_xmlid`: XML id del `mail.activity.type` (NO id numerico),
  p.ej. `'mail.mail_activity_data_todo'`, `'mail.mail_activity_data_call'`.
- `date_deadline`: fecha limite (default: hoy).
- `summary`, `note`: texto.
- `**act_values`: campos extra de `mail.activity`, p.ej. `user_id` (responsable).

```python
record.activity_schedule(
    "mail.mail_activity_data_todo",
    date_deadline=fields.Date.today(),
    summary="Revisar pedido",
    user_id=self.env.user.id,
)
```

Variante: `activity_schedule_with_view` (renderiza una vista QWeb como nota) cuando exista
en la version. Para reprogramar: `activity_reschedule(act_type_xmlids, user_id=None,
date_deadline=None, new_user_id=None)` (`mail/models/mail_activity_mixin.py:421`, `MailActivityMixin.activity_reschedule`, v18).

## `activity_feedback` (firma real)

`mail/models/mail_activity_mixin.py:447`, `MailActivityMixin.activity_feedback` (v18):

```python
def activity_feedback(self, act_type_xmlids, user_id=None,
                      feedback=None, attachment_ids=None):
```

Marca como hecha(s) la(s) actividad(es) del/los tipo(s) indicado(s), registrando feedback.
`act_type_xmlids` es una LISTA de XML ids.

```python
record.activity_feedback(["mail.mail_activity_data_todo"], feedback="Revisado OK")
```

## `activity_unlink` (firma real)

`mail/models/mail_activity_mixin.py:463`, `MailActivityMixin.activity_unlink` (v18):

```python
def activity_unlink(self, act_type_xmlids, user_id=None):
```

Cancela/borra actividades del/los tipo(s) sin marcarlas como hechas.

```python
record.activity_unlink(["mail.mail_activity_data_todo"])
```

## Tipos de actividad (`mail.activity.type`)

Se declaran como **data records** (delega en `odoo-data-files`):

```xml
<record id="activity_review_order" model="mail.activity.type">
    <field name="name">Review Order</field>
    <field name="icon">fa-check</field>
    <field name="delay_count">2</field>
    <field name="delay_unit">days</field>
    <field name="res_model">business.order</field>
</record>
```

Tipos estandar reutilizables: `mail.mail_activity_data_todo`, `mail.mail_activity_data_call`,
`mail.mail_activity_data_meeting`, `mail.mail_activity_data_email`,
`mail.mail_activity_data_upload_document`.

## Diffs v17 -> v18 -> v19

- **`activity_schedule`**: firma IDENTICA en v17 (`mail/models/mail_activity_mixin.py:354`, `MailActivityMixin.activity_schedule`),
  v18 (`:345`) y v19 (verificada por grep, misma firma
  `act_type_xmlid='', date_deadline=None, summary='', note='', **act_values`).
- **`activity_feedback`** (v17 `:456`) y **`activity_unlink`** (v17 `:472`): estables en las tres versiones.
- Campos del mixin estables en v17/v18/v19.
