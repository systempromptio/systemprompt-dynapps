# Web Forms & Web Marketing

Form builder, newsletter/popups (mass mailing) y short URLs con tracking UTM
(link.tracker). Verificado contra `$ODOO_SRC/odoo18/addons/`. Diffs v17/v19 al final.

## website_form / website_crm (Form Builder)

El form builder (snippet `s_website_form`) permite que un visitante cree registros
de un modelo arbitrario. La seguridad es por **whitelist** a nivel de `ir.model` e
`ir.model.fields`.

### Controller (`website/controllers/form.py`, clase `WebsiteForm`)

| Ruta | método | línea (v18) |
|---|---|---|
| `/website/form/<string:model_name>` (POST, csrf=False) | `website_form()` | 31-32 |
| (interno) | `_handle_website_form()` | 65 |
| (interno) | `extract_data()` validacion+categorizacion | 167 |
| (interno) | `insert_record()` crea con SUPERUSER_ID | 268 |
| (interno) | `insert_attachment()` | 302 |
| (interno) | `_get_form_writable_fields()` | 32-52 |

- La verificacion reCAPTCHA se invoca dentro del controller:
  `request.env['ir.http']._verify_request_recaptcha_token('website_form')`
  (`website/controllers/form.py:47`, `WebsiteForm.website_form`). Implementacion en
  `google_recaptcha/models/ir_http.py:34`, `Http._verify_request_recaptcha_token` (`_verify_request_recaptcha_token`) y
  `_verify_recaptcha_token()` (56) que llama a la API de Google y devuelve
  is_human/is_bot. Claves en `ir.config_parameter`: `recaptcha_public_key`,
  `recaptcha_private_key`, score minimo `recaptcha_min_score`.

### Whitelist (`website/models/website_form.py`)

- `ir.model.website_form_access = Boolean` (27): habilita el modelo en el form builder.
- `ir.model.website_form_label` (29), `website_form_key` (30),
  `website_form_default_field_id` (28): metadatos del form (acción, registro en el
  Registry, campo donde guardar metadatos custom).
- `ir.model.fields.website_form_blacklisted = Boolean(default=True)` (202): por defecto
  TODOS los campos están vetados. `init()` (140-151) los inicializa como blacklisted.
  `_get_form_writable_fields()` (32-52) devuelve solo los que tienen
  `website_form_blacklisted = False` → es un modelo **whitelist** (hay que abrir campos
  explícitamente).

### Cómo extender el form builder

- **Habilitar un modelo custom**: data XML que ponga `website_form_access=True` en su
  `ir.model` y `website_form_blacklisted=False` en los `ir.model.fields` que quieras
  permitir. Define `website_form_key` y `website_form_label`.
- **Lógica post-creación**: hereda `_handle_website_form()` o el método de inserción,
  o usa el campo `website_form_default_field_id` para volcar metadatos.
- `website_crm` es justo eso: abre `crm.lead` al form builder (ver su data).

## website_mass_mailing (Newsletter & Popups)

### Controller (`website_mass_mailing/controllers/main.py`)

| Ruta | método | línea |
|---|---|---|
| `/website_mass_mailing/is_subscriber` (json) | check | 11 |
| `/website_mass_mailing/subscribe` (json, public) | `subscribe()` | 35-36 |
| (estático) | `subscribe_to_newsletter()` | 51-72 |

- `subscribe()` verifica reCAPTCHA: `_verify_request_recaptcha_token(
  'website_mass_mailing_subscribe')` (37).
- `subscribe_to_newsletter()` (51-72): crea `mailing.contact` si no existe (67) y la
  `mailing.subscription` (68); si ya existía con `opt_out=True`, la reactiva poniendo
  `opt_out=False` (69-70) en vez de crear duplicado.

### Modelos

- `mailing.contact` (`mass_mailing/models/mailing_contact.py:14`, `MassMailingContact._name`): el contacto.
- `mailing.subscription` (`mass_mailing/models/mailing_subscription.py:11`, `MailingSubscription._name`): tabla
  intermedia contact_id ↔ list_id con `opt_out`.

### Snippets newsletter (autoría HTML → odoo-frontend)

- `views/snippets_templates.xml`: `s_newsletter_block` (9), `s_newsletter_subscribe_popup`
  (14), `s_newsletter_subscribe_form` (163).
- `views/snippets/s_popup.xml`: popup de captacion.

## website_links + link.tracker (Short URLs & UTM)

El modelo base vive en el módulo `link_tracker`; `website_links` solo anade la UI/rutas
de creacion y estadisticas.

### Modelo `link.tracker` (`link_tracker/models/link_tracker.py:23`, `LinkTracker`)

- `_inherit = ['utm.mixin']` (35): hereda campos UTM.
- `url` (38) destino; `short_url` computado `_compute_short_url` (40).
- UTM: `campaign_id` (51), `medium_id` (52), `source_id` (53), todos `ondelete='set null'`.
- Unicidad: `LINK_TRACKER_UNIQUE_FIELDS = ('url','campaign_id','medium_id','source_id','label')`
  (16) → mismo url+utm+label = un único tracker; los códigos cortos son 1→N
  (`link.tracker.code`, 299; `_get_random_code_strings`, 312).
- Clics: `link.tracker.click` (326) con `link_id` (334), `ip` (337), `country_id` (338);
  `add_click(code, **route_values)` (347) registra el clic.

### Controllers

- Redirect base: `link_tracker/controller/main.py:12`, `LinkTracker.full_url_redirect` ruta `/r/<string:code>` (public,
  website=True) → `full_url_redirect()` (13) resuelve `get_url_from_code()` (20) y
  llama `add_click()` solo si no es bot.
- `_compute_redirected_url()` (98-127) anade `utm_campaign`/`utm_medium`/`utm_source`
  a la URL destino; el param de sistema `link_tracker.no_external_tracking` (105) omite
  UTM en URLs externas.
- Creacion UI: `website_links/controller/main.py:9`, `WebsiteUrl.create_shorten_url` `/website_links/new` (json, user),
  `/website_links/add_code` (23), `/r/<code>+` página de estadisticas (36).

### Cómo extender marketing web

- **Generar short URL desde código**: `request.env['link.tracker'].create({'url':…,
  'campaign_id':…, 'medium_id':…, 'source_id':…})` y lee `short_url`. Respeta la
  unicidad (mismos UTM → reusa tracker).
- **Newsletter desde código**: usa `subscribe_to_newsletter()` o crea
  `mailing.subscription` directamente, gestionando `opt_out`.

## Diffs de versión

- **v19 captcha como flag**: la ruta del form usa `captcha='website_form'` en el
  `@http.route` (`website/controllers/form.py:31`, `WebsiteForm.website_form`, v19) en vez de la llamada explícita
  a `_verify_request_recaptcha_token` del cuerpo (v18, `website/controllers/form.py:47`, `WebsiteForm.website_form`).
- **v19 JSON-RPC**: `subscribe()` pasa de `type='json'` (v17/v18) a `type='jsonrpc'`
  (`website_mass_mailing/controllers/main.py:36`, `MassMailController.subscribe`, v19). Mismo cambio aplica a las rutas
  json de event y forum.

## Gotchas (forms & marketing)

1. **Form builder es whitelist, no blacklist util**: `website_form_blacklisted`
   default `True` (`website/models/website_form.py:202`, `website_form_model_fields.formbuilder_whitelist`); aunque pongas `website_form_access=True` en
   el modelo, ningun campo es escribible hasta poner `website_form_blacklisted=False`
   en cada `ir.model.fields` (whitelist, `_get_form_writable_fields` 32-52).
2. **Registros del form se crean con SUPERUSER_ID**: `insert_record()`
   (`website/controllers/form.py:268`, `WebsiteForm.insert_record`) crea con permisos de superusuario; la unica barrera real es la
   whitelist. Abrir un modelo sensible al form builder = riesgo de escritura anónima.
3. **reCAPTCHA falla silencioso si no hay claves**: sin `recaptcha_public_key`/
   `private_key` en `ir.config_parameter`, la verificación
   (`google_recaptcha/models/ir_http.py:34`, `Http._verify_request_recaptcha_token`) no protege; confíguralas en producción.
4. **Newsletter no duplica, re-opta**: `subscribe_to_newsletter()` reactiva una
   suscripcion con `opt_out=True` poniendo `opt_out=False` (`website_mass_mailing/
   controllers/main.py:69-70`) en vez de crear otra `mailing.subscription`; al testear
   conteos, ten esto en cuenta.
5. **Opt-out es por lista, no global**: `opt_out` vive en `mailing.subscription`
   (`mass_mailing/models/mailing_subscription.py:11`, `MailingSubscription._name`, `class MailingSubscription`), tabla por lista; un contacto puede estar opt-in en
   una lista y opt-out en otra.
6. **UTM solo si NO es bot**: `add_click()` (`link_tracker/models/link_tracker.py:347`, `LinkTrackerClick.add_click`) y el redirect
   (`link_tracker/controller/main.py:13`, `LinkTracker.full_url_redirect`) solo cuentan clic/aplican tracking si no es bot; los
   contadores no suben para crawlers.
7. **Unicidad del tracker por UTM**: crear un `link.tracker` con misma combinacion
   url+campaign+medium+source+label devuelve/colisiona con el existente
   (`LINK_TRACKER_UNIQUE_FIELDS`, `link_tracker/models/link_tracker.py:16`, `LINK_TRACKER_UNIQUE_FIELDS`); para A/B usa varios
   `link.tracker.code` sobre el mismo tracker.
8. **`no_external_tracking`**: el param `link_tracker.no_external_tracking`
   (`link_tracker/models/link_tracker.py:105`, `LinkTracker._compute_redirected_url`) desactiva la inyeccion de UTM en URLs externas; si esperas
   ver utm_* en la URL destino y no aparecen, revisa ese parametro.
