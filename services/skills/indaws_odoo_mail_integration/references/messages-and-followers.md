# Mensajes y followers

Base v18. Citas en `$ODOO_SRC/odoo18/addons/mail/models/mail_thread.py`.

## `message_post` vs `_message_log`

| | `message_post` | `_message_log` |
|---|---|---|
| Visibilidad | publico | privado (prefijo `_`) |
| Notifica followers | SI (segun subtype) | NO |
| Dispara emails | SI | NO |
| Permisos | comprueba ACL | corre como sudo |
| Uso tipico | comentario, notificacion al cliente | nota interna / log de sistema |

Regla: nota interna silenciosa -> `_message_log`. Notificar seguidores / mandar correo -> `message_post`.

## `message_post` (firma real)

`mail/models/mail_thread.py:2132`, `MailThread._get_customer_information` (v18). **Keyword-only** (`self, *, ...`):

```python
def message_post(self, *,
                 body='', subject=None, message_type='notification',
                 email_from=None, author_id=None, parent_id=False,
                 subtype_xmlid=None, subtype_id=False, partner_ids=None,
                 attachments=None, attachment_ids=None, body_is_html=False,
                 **kwargs):
```

Parametros clave:
- `body`: `str` (se escapa) o `Markup` (HTML). Para HTML usa `Markup(...)` o `body_is_html=True` (RPC).
- `subject`, `message_type` (cualquiera salvo `user_notification`, reservado a `message_notify`).
- `subtype_xmlid`: XML id del subtype, p.ej. `'mail.mt_comment'` / `'mail.mt_note'`; fuerza `subtype_id`.
- `partner_ids`: partners a notificar ademas de los followers que matchean el subtype.
- `attachments`: lista de tuplas `(name, content)` o `(name, content, info)`, content NO en base64.
- `attachment_ids`: ids de `ir.attachment` ya existentes (no commands).
- `author_id` / `email_from`: ver `_message_compute_author`.

Ejemplo:

```python
from markupsafe import Markup
record.message_post(
    body=Markup("<p>Pedido <b>confirmado</b></p>"),
    subject="Confirmacion",
    subtype_xmlid="mail.mt_comment",
    partner_ids=[partner.id],
)
```

## `_message_log` / `_message_log_batch` (firma real)

`mail/models/mail_thread.py:2800`, `MailThread._message_log_with_view` y `:2818` (v18). Keyword-only:

```python
def _message_log(self, *, body='', subject=False, author_id=None,
                 email_from=None, message_type='notification',
                 partner_ids=False, attachment_ids=False,
                 tracking_value_ids=False):
```

`_message_log` hace `ensure_one()` y delega en `_message_log_batch({self.id: body}, ...)`.
Para muchos records de golpe usa `_message_log_batch(bodies, ...)` donde `bodies` es
`{res_id: body}` — mas rapido, no notifica, crea como sudo.

```python
record._message_log(body="Sincronizado con sistema externo")
```

## Notificacion (`_notify_*`)

Hooks de override (NO suelen llamarse directamente). Firmas en v18:
- `_notify_thread(self, message, msg_vals=False, **kwargs)` (`:3169`) — orquesta el envio.
- `_notify_thread_by_inbox` (`:3225`), `_notify_thread_by_email` (`:3284`),
  `_notify_thread_by_web_push` (`:3744`).
- `_notify_get_recipients(self, message, msg_vals, **kwargs)` (`:3865`) — calcula destinatarios.
- `_notify_get_recipients_groups` (`:3949`) — agrupa destinatarios (botones, layout por grupo).

## Followers

### `message_subscribe` / `message_unsubscribe`

`mail/models/mail_thread.py:4341`, `MailThread._truncate_payload_get_max_payload_length` y `:4394` (v18):

```python
def message_subscribe(self, partner_ids=None, subtype_ids=None):
def message_unsubscribe(self, partner_ids=None):
```

- `partner_ids`: partners a suscribir como followers.
- `subtype_ids`: subtypes a los que se suscriben; si `None`, usa los `default=True`.

```python
record.message_subscribe(partner_ids=[partner.id])
```

### Auto-suscripcion

- `_message_auto_subscribe(self, updated_values, followers_existing_policy='skip')` (`:4481`):
  punto de entrada; suscribe automaticamente a partners derivados de los valores escritos.
- `_message_auto_subscribe_followers(self, updated_values, default_subtype_ids)` (`:4412`):
  override para definir QUE partners auto-suscribir (p.ej. el comercial del pedido) y con
  que subtypes. Devuelve lista de tuplas `(partner_id, subtype_ids, ...)`.
- `_message_auto_subscribe_notify(self, partner_ids, template)` (`:4446`).

```python
def _message_auto_subscribe_followers(self, updated_values, default_subtype_ids):
    res = super()._message_auto_subscribe_followers(updated_values, default_subtype_ids)
    if updated_values.get("user_id"):
        user = self.env["res.users"].browse(updated_values["user_id"])
        if user.partner_id:
            res.append((user.partner_id.id, default_subtype_ids, False))
    return res
```

## Diffs v17 -> v18 -> v19

- **`message_post`**: firma IDENTICA en v17 (`mail/models/mail_thread.py:2087`, `_mail_find_partner_from_emails.sort_key`) y v18 (`:2132`).
  En **v19** (`mail/models/mail_thread.py:2182`, `MailThread.message_post`) se anaden tres kwargs nuevos:
  `outgoing_email_to`, `incoming_email_to`, `incoming_email_cc`. Codigo que los use NO es
  portable hacia atras; el resto de kwargs no cambia.
- **`_message_log` / `_message_log_batch`**: estables en v17/v18/v19.
- **`message_subscribe` / `message_unsubscribe`**: estables en las tres versiones.
