---
name: indaws-odoo-mail-integration
version: 0.6.13
effort: high
model: opus
description: >-
  Integra el sistema mail de Odoo en un modelo: chatter, tracking, message_post, seguidores, actividades y mail.template. Usala cuando anadas chatter o envies correo. También la mitad JS de mail (store mail.store, extender Discuss, el composer o Thread/Message) y enviar correo con plantilla desde un modelo.
allowed-tools: [Read, Grep, Glob, Edit, Write, Bash]
---

# Odoo Mail Integration

Integra chatter, tracking de campos, subtypes, mensajes, followers, actividades y
plantillas de correo en un modelo de negocio Odoo. Reference practico anclado en
fuente, base **v18**, con diffs confirmados **v17** y **v19**.

Estandares compartidos: `references/shared/coding-standards.md` (codigo y external IDs en ingles SIEMPRE; strings de usuario traducibles).

## Workflow

1. **Heredar mixins** en el modelo de negocio:
   `_inherit = ["mail.thread", "mail.activity.mixin"]`. Ver `references/thread-tracking-subtypes.md`
   para que aporta cada uno y como activar `tracking=True`.
2. **Tracking + subtypes**: marca campos con `tracking=True` (orden con `tracking=<int>`),
   override `_track_subtype` si necesitas subtype condicional, define `mail.message.subtype`
   como data record. Ver `references/thread-tracking-subtypes.md`.
3. **Mensajes / log**: elige `message_post` (notifica seguidores, RPC-safe) o `_message_log`
   (nota silenciosa, sudo, sin notificacion). Firmas reales en `references/messages-and-followers.md`.
4. **Followers**: `message_subscribe` / `message_unsubscribe`, auto-suscripcion via
   `_message_auto_subscribe*`. Ver `references/messages-and-followers.md`.
5. **Actividades**: `activity_schedule` / `activity_feedback` / `activity_unlink`,
   tipos `mail.activity.type` como data, campos del mixin. Ver `references/activities.md`.
6. **Plantillas de correo**: `mail.template` (`model_id`, `email_from`, `auto_delete`),
   render con `_render_template` / `_render_field`, envio con `send_mail` /
   `send_mail_batch`. Ver `references/mail-templates.md`.
7. **Lado vista**: el `<chatter/>` en la vista form lo cubre `odoo-view-designer`. **Es v18+, NO v17+**: el compiler registra `selector: "div.oe_chatter"` en v17 (`mail/static/src/views/web/form/form_compiler.js:74`, `chatter_compiler` (la entrada `chatter_compiler` del registro `form_compilers`)) y `selector: "chatter"` en v18/v19 (`mail/static/src/chatter/web/form_compiler.js:46` / `:47`). En v17 la etiqueta `<chatter/>` no casa con ningún compiler y **el chatter no se renderiza, sin error**.
   NO lo dupliques aqui; referencia esa skill.
8. **¿El encargo es en JS?** (una acción en el menú de un mensaje, un campo nuevo en `Thread` o
   `Message`, tocar el composer, extender Discuss) → `references/discuss-js.md`. Nada de los
   pasos 1-6 sirve ahí: mail tiene **su propio ORM en el cliente**
   (`@mail/core/common/record`), distinto del de las vistas, y los registros se comparan con
   `.eq()`, no con `===`.

## GOTCHAS

1. **`message_post` es keyword-only** — la firma es `message_post(self, *, body='', ...)`.
   NUNCA pases argumentos posicionales; todo va por nombre (`mail/models/mail_thread.py:2132`, `MailThread._get_customer_information`, v18).
2. **`_message_log` tambien es keyword-only y privado** — usa sudo, NO notifica seguidores
   ni dispara emails. Para una nota interna silenciosa usa `_message_log`; para notificar
   usa `message_post` (`mail/models/mail_thread.py:2800`, `MailThread._message_log_with_view`, v18).
3. **`body` quiere `Markup`, no str con HTML** — un str se escapa. Usa
   `from markupsafe import Markup` o pasa `body_is_html=True` (solo RPC).
4. **`tracking=True` sin `mail.thread` no hace nada — pero NO es silencioso (MEDIDO).** Cada carga
   del registro suelta un WARNING **por campo**: `Field <modelo>.<campo>: unknown parameter
   'tracking'` (`_valid_field_parameter`). Se pueden sacar todos con
   `grep "unknown parameter 'tracking'"` en el log de arranque. El modulo instala igual.
4b. **El seguimiento NO se escribe en el `write`: se encola en `env.cr.precommit` (MEDIDO v18+v19).**
   Justo despues del `write`, `message_ids` no ha cambiado; el mensaje con los valores viejo/nuevo
   aparece cuando corre la cola de precommit —o sea, al hacer commit—. En un `TransactionCase` no hay
   commit: hay que llamar a `self.env.cr.precommit.run()` (o `self.flush_tracking()` donde exista)
   antes de afirmar nada. Sin eso, un test concluye que el tracking no funciona; peor aun, un test
   que afirme «no se registro nada» **pasa por el motivo equivocado**.
5. **`subtype_xmlid` fuerza `subtype_id`** — pasa el XML id (`'mail.mt_comment'`,
   `'mail.mt_note'`), no busques el record a mano.
6. **`_track_subtype(self, initial_values)` debe devolver un subtype o llamar a super** —
   recibe los valores ANTES del write (`mail/models/mail_thread.py:596`, `MailThread._track_subtype`, v18).
7. **`activity_schedule` usa el XML id del tipo**, no el id numerico:
   `activity_schedule('mail.mail_activity_data_todo', ...)` (`mail/models/mail_activity_mixin.py:345`, `MailActivityMixin.activity_schedule`, v18).
8. **`_render_template` necesita `res_ids` (lista) y devuelve un dict `{res_id: valor}`** —
   no devuelve un string suelto (`mail/models/mail_render_mixin.py:541`, `MailRenderMixin._render_template_inline_template`, v18).
9. **El motor de render por defecto es `inline_template`** (placeholders `{{ }}`), NO QWeb.
   Para QWeb usa `engine='qweb'` o `_render_template_qweb` (`mail_render_mixin.py`, v18).
   **Y equivocarse no da error (MEDIDO):** una plantilla con `t-out` pasada al motor por defecto
   **sale cruda**, con el `t-out` dentro, y se envia asi. Sonda: `tools/bench-orm/addons/dyn_mail`.
10. **v19 anade kwargs a `message_post`** (`outgoing_email_to`, `incoming_email_to`,
    `incoming_email_cc`) — codigo que los use NO es portable a v17/v18 (`mail/models/mail_thread.py:2182`, `MailThread.message_post`, v19).

## NEVER

- NUNCA llamar a `message_post` / `_message_log` con argumentos posicionales.
- NUNCA pasar HTML como `str` a `body` esperando que se renderice; usa `Markup`.
- NUNCA crear subtypes / activity types / templates a mano con `create`: declaralos como
  data records (delega en `odoo-data-files`).
- NUNCA duplicar el markup del `<chatter/>` aqui: es responsabilidad de `odoo-view-designer`.
- NUNCA usar `tracking=True` sin heredar `mail.thread`.

## ALWAYS

- SIEMPRE heredar `mail.thread` (chatter/tracking) y, si hay actividades, `mail.activity.mixin`.
- SIEMPRE verificar la firma contra la version objetivo (v17/v18/v19) antes de usar kwargs nuevos.
- SIEMPRE referenciar `mail.message.subtype`, `mail.activity.type` y `mail.template` por
  su XML id estandar cuando exista (`mail.mt_comment`, `mail.mail_activity_data_todo`...).
- SIEMPRE preferir `_message_log` para notas internas sin notificacion y `message_post`
  cuando hay que notificar seguidores.
- **Provenance marker (ALWAYS):** every generated file (except `.csv`/`.json` — no comment syntax) ends with `Made with odoo-mail-integration · v<version del frontmatter de esta skill>`, wrapped in the file-type comment, idempotent. Full rule: `references/shared/skill-provenance-marker.md`.

## Delegates to

| Skill | Para que |
|-------|----------|
| odoo-version-api-reference | Firmas exactas y diffs de API entre versiones |
| odoo-view-designer | Insertar el chatter en la vista form: `<chatter/>` en **v18+**, `<div class="oe_chatter">` en v17 |
| odoo-data-files | Declarar subtypes, activity types y mail.template como data records |
| odoo-deep-knowledge | Comportamientos ocultos de mail.thread/tracking/actividades |
| odoo-owl-component-builder | Construir el componente OWL cuando el encargo de `discuss-js.md` pide uno propio (dialog, systray, plantilla) |

## Consumed by

| Skill | Para que |
|-------|----------|
| odoo-module-creator | Anadir mensajeria/tracking/actividades a un modulo nuevo |

## REFERENCES

| Fichero | Contenido |
|---------|-----------|
| references/thread-tracking-subtypes.md | Mixins, `tracking=True`, `_track_subtype`/`_track_changes`, subtypes y `_creation_subtype` |
| references/messages-and-followers.md | `message_post` vs `_message_log` (firmas), `_notify_*`, `message_subscribe`/`_message_auto_subscribe*` |
| references/activities.md | `activity_schedule`/`activity_feedback`/`activity_unlink`, `mail.activity.type`, campos del mixin |
| references/mail-templates.md | `mail.template`, `_render_template`/`_render_field`, `send_mail`/`send_mail_batch`, motores de render |
| references/discuss-js.md | **La mitad JS** (v18): el ORM propio de mail (`Record.one/many/attr`, `.eq()`), el service `mail.store`, extender un modelo con `patch` + campos en `setup()`, y los tres puntos de extensión — `messageActionsRegistry`, `discussComponentRegistry` y el form compiler del `<chatter/>` |
