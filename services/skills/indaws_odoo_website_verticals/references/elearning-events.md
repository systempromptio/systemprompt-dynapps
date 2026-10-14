# eLearning (slides) & Events

Verificado contra `$ODOO_SRC/odoo18/addons/website_slides` y `website_event` /
`event`. v17/v19: rutas JSON `type='json'`→`type='jsonrpc'` (v19).
**Las rutas de fichero de esta referencia son de v18**; v19 reorganiza `website_slides/models/`
(ver la tabla de `slide.channel.partner`). Los nombres de modelo y campo son estables.

## website_slides (eLearning)

### Modelos clave

**`slide.channel`** (curso) — `website_slides/models/slide_channel.py:274`, `Channel`, _inherit
`website.published.multi.mixin` (284).
- `enroll` selection 'public'/'invite' (379-383): inscripcion libre vs por invitacion.
- `visibility` selection 'public'/'connected'/'members' (388-393): quién ve el curso.
- `channel_partner_ids` One2many a `slide.channel.partner` (399-404); `members`,
  `completion` (405-410).
- `_action_add_members()` (908-972): da de alta partners con `member_status='joined'`
  e invoca `message_subscribe()` (968-971). `_filter_add_members()` (974-982) respeta
  la politica `enroll`.

**`slide.channel.partner`** (inscripcion) — **el fichero se PARTE en v19**:

| Version | Fichero | `_name` | `member_status` |
|---|---|---|---|
| 17.0 / 18.0 | `models/slide_channel.py` (rangos `:20-272` en v18) | `:21` | `:28-33` (v18), `:29` (v17) |
| 19.0 | **`models/slide_channel_partner.py`** (nuevo) | `:6` | `:13` |

Verificado 2026-08-07. El **campo sigue existiendo en las tres** versiones con los mismos cuatro
valores; lo que cambia es donde vive. Buscar `slide.channel.partner` en `slide_channel.py` de v19 no
encuentra nada y parece que el modelo desaparecio — no: se movio a su propio fichero (v19 anade
tambien `slide_slide_partner.py`, misma reorganizacion).

- `member_status` selection 'invited'/'joined'/'ongoing'/'completed' — este es
  el estado real de avance, NO un booleano `completed`.
- `completion` Integer (34); `_recompute_completion()` (102-152) recalcula avance.

**`slide.slide`** (contenido) — `website_slides/models/slide_slide.py:93`, `Slide`.
- `is_published` (239, tracking=1) y `website_published` (240, tracking=False):
  ambos controlan visibilidad.
- `slide_category` selection 'infographic'/'article'/'document'/'video'/'quiz' (156-163).
- `slide_type` computado (414-441): 'pdf'/'youtube_video'/'quiz'/… (186-198).
- `can_publish` computado (608-611) delega en `channel.can_publish`.
- `write()` (658-695) publica notificacion y recomputa completion del canal;
  `_post_publication()` (790-809) envia email con `publish_template_id`.
- `_action_mark_completed()` (907-925) marca completado; `_action_set_quiz_done()`
  (941-977) suma/resta karma del quiz.

### Controller (`controllers/main.py`, clase `WebsiteSlides` extiende `WebsiteProfile`, línea 30)

| Ruta | método | línea |
|---|---|---|
| `/slides` | `slides_channel_home()` | 339 |
| `/slides/<channel>` | `channel()` | 498-508 |
| `/slides/slide/<slide>` | `slide_view()` | 947-991 |
| `/slides/slide/set_completed` (json) | `slide_set_completed()` | 1054-1066 |
| `/slides/slide/<slide>/set_completed` (http redirect) | `slide_set_completed_and_redirect()` | 1045-1052 |
| `/slides/slide/set_uncompleted` (json) | `slide_set_uncompleted()` | 1074-1085 |
| `/slides/channel/join` (post) | `slide_channel_join()` | 849-861 |
| `/slides/channel/leave` (post) | `slide_channel_leave()` | 863-868 |

- `slide_view()` auto-marca como completado los slides NO-quiz en la primera vista si
  `can_self_mark_completed` (956-958). Los quiz se completan al aprobar.

### Cómo extender slides

- **Nuevo tipo de contenido**: extiende `slide_type`/`slide_category` (selection) en
  `_inherit = 'slide.slide'` y ajusta `_compute_slide_type`.
- **Logica de inscripcion**: hereda `_action_add_members()` / `_filter_add_members()`
  para reglas custom (p.ej. inscribir desde otro modulo).
- **Completar desde código**: llama `slide._action_mark_completed()`, no escribas el
  estado a mano (recompute de completion del canal depende de ello).

## website_event (Eventos)

### Modelos clave

**`event.event`** base (`event/models/event_event.py`, _inherit `mail.thread`,
`mail.activity.mixin`). Extension web en `website_event/models/event_event.py:20`, `Event`,
_inherit `['event.event', 'website.seo.metadata', 'website.published.multi.mixin',
'website.cover_properties.mixin', 'website.searchable.mixin']`, **y en v19 uno mas**:
`'website.page_visibility_options.mixin'` (`website_event/models/event_event.py:27`, `Event._inherit` v19; 0 ocurrencias en v17/v18):
- `website_published` (52, tracking=True), `website_visibility` 'public'/'link'/
  'logged_users' (47-51), `website_menu` (53-56), `is_participating` (42-43).

**`event.registration`** (`event/models/event_registration.py:13`, `EventRegistration`):
- `state` selection 'draft'/'open'/'done'/'cancel' (64-74): draft=sin confirmar,
  open=registrado, done=asistio, cancel=cancelado.
- `partner_id` (44), `event_ticket_id` (35-36), `name`/`email`/`phone`/`company_name`
  computados del partner (45-51), `registration_answer_ids` (76).
- `create()` (208-228) y `write()` (230-238) llaman `_update_mail_schedulers()`.
- Extension web (`website_event/models/event_registration.py:6`, `EventRegistration`): `visitor_id` (9) y
  `_get_website_registration_allowed_fields()` (11-12) → set de campos aceptados desde
  web: `{'name','phone','email','company_name','event_id','partner_id','event_ticket_id'}`.

### Controller (`controllers/main.py`, clase `WebsiteEventController`, línea 18)

| Ruta | método | línea |
|---|---|---|
| `/event`, `/events` | `events()` | 42 |
| `/event/<event>/register` | `event_register()` | 183-186 |
| `/event/<event>/registration/new` (json/jsonrpc, POST) | `registration_new()` | 265-270 |
| `/event/<event>/registration/confirm` (http, POST) | `registration_confirm()` | 377-392 |
| `/event/<event>/registration/success` | `event_registration_success()` | 394+ |

Flujo de registro web:
1. `_process_tickets_form()` (201-229): extrae cantidades de entradas del POST.
2. `_process_attendees_form()` (272-353): extrae valores de cada asistente y respuestas,
   filtrando por `_get_website_registration_allowed_fields()` (282-283).
3. `registration_confirm()` (377-392): valida aforo (388) y crea registros vía
   `_create_attendees_from_registration_post()` (355-375), que setea `visitor_id`,
   `event_id`, `partner_id`, `registration_answer_ids`.

### Cómo extender eventos

- **Campos custom en el registro web**: para que un campo llegue desde el formulario,
  hereda `_get_website_registration_allowed_fields()` y anádelo al set (sin esto el
  controller lo descarta por seguridad).
- **Logica al confirmar**: hereda `registration_confirm()` o
  `_create_attendees_from_registration_post()`.
- **Sub-apps**: `website_event_track` (ponencias), `website_event_booth` (stands),
  `website_event_sale` (entradas de pago) extienden estos modelos; revisa su manifest
  antes de duplicar lógica.

## Gotchas (slides & events)

1. **`member_status`, no `completed`**: el avance del alumno es `slide.channel.partner.
   member_status` ('invited'/'joined'/'ongoing'/'completed', `website_slides/models/slide_channel.py:28-33`, `ChannelUsersRelation.member_status`),
   recomputado por `_recompute_completion()` (102-152). No inventes un booleano.
2. **Doble flag de publicacion de slide**: `is_published` (tracked) y `website_published`
   (no tracked) coexisten (`website_slides/models/slide_slide.py:239-240`, `class Slide`); publica via `is_published` para
   que dispare la notificacion de `write()`/`_post_publication()` (658-695, 790-809).
3. **Auto-completado en primera vista**: `slide_view()` marca completados los slides
   no-quiz nada más verlos si `can_self_mark_completed` (`website_slides/controllers/main.py:956-958`, `WebsiteSlides.slide_view`);
   si testeas progreso, ten en cuenta que ver ≠ pendiente.
4. **Completar via método**: usa `slide._action_mark_completed()` (907-925); escribir el
   estado del partner a mano no recomputa la `completion` del canal.
5. **Registro web filtra campos**: `_process_attendees_form()` solo acepta los campos de
   `_get_website_registration_allowed_fields()` (`website_event/models/event_registration.py:11-12`, `EventRegistration._get_website_registration_allowed_fields`);
   un campo no whitelisteado se ignora silenciosamente en el POST.
6. **Estado del registro**: el registro web nace según el flujo (draft/open) y los
   cambios de `state` reprograman emails vía `_update_mail_schedulers()`
   (`event/models/event_registration.py:230-238`, `EventRegistration.write`); no fuerces `state` saltándote `write`.
