# Plantillas de correo (mail.template)

Base v18. Citas en `$ODOO_SRC/odoo18/addons/mail/models/mail_template.py` y
`mail_render_mixin.py`.

## Declarar la plantilla (data record)

Delega la creacion en `odoo-data-files`. Estructura tipica:

```xml
<record id="email_order_confirmation" model="mail.template">
    <field name="name">Order: Confirmation</field>
    <field name="model_id" ref="my_module.model_business_order"/>
    <field name="subject">Pedido {{ object.name }}</field>
    <field name="email_from">{{ (object.user_id.email_formatted or user.email_formatted) }}</field>
    <field name="email_to">{{ object.partner_id.email_formatted }}</field>
    <field name="body_html" type="html">
        <div>Hola <t t-out="object.partner_id.name or ''"/>, tu pedido esta confirmado.</div>
    </field>
    <field name="auto_delete" eval="True"/>
    <field name="lang">{{ object.partner_id.lang }}</field>
</record>
```

Campos clave:
- `model_id`: Many2one a `ir.model` (referencia `model_<modelo>` con `.` -> `_`).
- `subject`, `email_from`, `email_to`, `email_cc`, `reply_to`, `partner_to`.
- `body_html` (`type="html"`).
- `auto_delete` (default `True`): borra el `mail.mail` tras enviarlo.
- `lang`: expresion para traducir el render por idioma del destinatario.
- `report_template_ids` / `attachment_ids`: adjuntos.

## Motor de render: placeholders

- Motor por defecto: **`inline_template`** -> placeholders `{{ expr }}` (estilo Jinja-like
  inline). Usado en `subject`, `email_from`, `email_to`...
- `body_html` se renderiza como **QWeb** -> `<t t-out="..."/>`, `<t t-if="...">`.

NO mezcles: `{{ }}` en campos de texto plano, `t-out` en el cuerpo HTML.
Variables disponibles en el contexto: `object` (el record), `user`, `ctx`, `format_date`,
`format_amount`, etc.

## Render programatico: `_render_template` / `_render_field`

`mail/models/mail_render_mixin.py:541`, `MailRenderMixin._render_template_inline_template` (v18):

```python
def _render_template(self, template_src, model, res_ids,
                     engine='inline_template', add_context=None, options=None):
```

- `template_src`: el texto a renderizar (o XML id de una qweb view si `engine='qweb_view'`).
- `model`: nombre del modelo (str, p.ej. `'business.order'`).
- `res_ids`: LISTA de ids.
- `engine`: `'inline_template'` (default), `'qweb'` o `'qweb_view'`.
- **Devuelve un dict `{res_id: valor_renderizado}`**, NO un string suelto.

```python
rendered = self.env["mail.render.mixin"]._render_template(
    "Hola {{ object.partner_id.name }}", "business.order", record.ids,
)
texto = rendered[record.id]
```

`_render_field(self, field, res_ids, engine='inline_template', ...)`
(`mail/models/mail_render_mixin.py:648`, `MailRenderMixin._render_template`, v18): renderiza un campo concreto de la plantilla sobre
varios records.

## Envio: `send_mail` / `send_mail_batch`

`mail/models/mail_template.py:620`, `MailTemplate.send_mail` (v18):

```python
def send_mail(self, res_id, force_send=False, raise_exception=False,
              email_values=None, email_layout_xmlid=False):
```

- `res_id`: id (int) del record sobre el que renderizar.
- `force_send`: `True` envia ya; `False` (recomendado) usa la cola de correo. **Y "recomendado" no es
  opinion nuestra**: es el default de la firma y el propio docstring de Odoo dice *"otherwise use the
  mail queue (recommended)"* (`mail/models/mail_template.py:620`, `MailTemplate.send_mail` v18 / `:675` v19, docstring en
  `:682` v19). La cola la drena un cron nativo —`mail/data/ir_cron_data.xml:4-8`, `ir_cron_mail_scheduler_action` ejecuta
  `model.process_email_queue(batch_size=1000)`, definido en `mail/models/mail_mail.py:218`, `MailMail.process_email_queue` (v18) /
  `:194` (v19)—, asi que no hay que montar nada: dejar el default YA da envio asincrono y
  reintentable. Verificado 2026-08-07.
  **Ojo con la firma del metodo del cron entre versiones**: v18 `process_email_queue(self, ids=None,
  batch_size=1000)`, v19 `process_email_queue(self, email_ids=(), batch_size=1000)` — el primer
  parametro se **renombra** (`ids` → `email_ids`) y cambia su default; si lo llamas por nombre desde
  codigo propio, en la otra version peta.
- `email_values`: dict para sobrescribir valores del `mail.mail` generado.
- `email_layout_xmlid`: layout de notificacion que envuelve el email.
- **Devuelve el id (int) del `mail.mail` creado.**

```python
template = self.env.ref("my_module.email_order_confirmation")
template.send_mail(record.id, force_send=False)
```

Batch: `send_mail_batch(self, res_ids, force_send=False, ...)` — version por lotes,
devuelve recordset de `mail.mail`. Existe desde v17 (`mail/models/mail_template.py:620`, `MailTemplate.send_mail`; v18 `:645`, v19 `:699`).

## Diffs v17 -> v18 -> v19

- **`send_mail`**: firma IDENTICA en v17 (`mail/models/mail_template.py:595`, `MailTemplate._generate_template`), v18 (`:620`) y
  v19 (`:675`). En v18 pasa a delegar internamente en `send_mail_batch`.
- **`send_mail_batch`**: existe desde v17 (`mail/models/mail_template.py:620`, `MailTemplate.send_mail_batch`, v17), v18 (`mail/models/mail_template.py:645`, `MailTemplate.send_mail_batch`), v19 (`mail/models/mail_template.py:699`, `MailTemplate.send_mail_batch`).
  Solo cambian los nº de linea por version.
- **`_render_template`** (`mail_render_mixin.py`): firma estable —
  v17 (`:486`), v18 (`:600`), v19 (`:657`). Mismos engines (`inline_template`/`qweb`/`qweb_view`).
- **`_render_field`**: estable — v17 (`:593`), v18 (`:707`), v19 (`:775`).
  (Corregido 2026-08-17, medido con `grep -n`: estas dos filas daban `:541`/`:650` y `:648`/`:768`.
  **Los dos numeros de v18 eran exactamente v17+55**, o sea extrapolados con aritmetica, no medidos
  — y los de v19 fallaban por 7 en las dos, misma firma. Las lineas de `send_mail`/`send_mail_batch`
  de arriba **si** estaban medidas y salen exactas. Moraleja: en un mismo fichero convivian citas
  medidas y citas deducidas, indistinguibles desde fuera.)
