# Workflow detallado — qualification-preflight

Detalle de los pasos que SKILL.md resume. La investigación (Tier 0-3) se rige por `../../qualification-form/references/research.md`.

## Paso 0 — Query de leads elegibles

Fecha de corte: ahora menos 7 días en `YYYY-MM-DD HH:MM:SS` (UTC).

```
orm_search_read(
  model='crm.lead',
  domain=[
    ('type', '=', 'opportunity'),
    ('stage_id.sequence', '=', 1),  # etapa de entrada (id 1, hoy "Lead"). NO usar stage_id.name='New': fue renombrada y el nombre da 0 → no-op silencioso.
    ('create_date', '>=', '<fecha_corte>'),
    ('email_from', '!=', False),
  ],
  fields=['id', 'name', 'partner_id', 'user_id', 'contact_name',
          'email_from', 'phone', 'website', 'description', 'stage_id'],
  limit=50,
  order='create_date desc'
)
```

- **`limit=50`**: cap por ejecución para no saturar `utilities:publish` ni quemar tokens. Si la cola es mayor, el siguiente tick del cron se la come.
- **`dry-run`**: si el argumento es `dry-run`, ejecutar SOLO esta query y reportar el listado (id, name, partner, user, create_date). NO seguir.
- 0 leads → reportar "0 leads elegibles" y terminar éxito.

## Paso 1 — Filtrar ya procesados (idempotencia)

Por cada lead, comprobar si ya tiene la nota:

```
orm_search_read(
  model='mail.message',
  domain=[('model','=','crm.lead'), ('res_id','=',<lead_id>),
          ('subject','=','Form de cualificacion pre-rellenado')],
  fields=['id'], limit=1
)
```

Si ≥1 → **skip**, incrementar `skipped`, loggear razón. **Optimización**: 1 sola query con `('res_id','in',[<ids>])` y construir un set de procesados.

## Paso 2 — Procesar cada lead (loop, `create_date desc`)

### 2.0 Parse del chatter web + reparación de la ficha

Desde ~2026-06-25 los leads de la web entran vía **email entrante → Odoo**: los datos reales del formulario (Company, Name, Email, Phone, Description, "¿Ya usas Odoo?") llegan SOLO en el **primer mensaje del chatter**, y los campos nativos quedan inservibles — `email_from`/contacto = el relay (`bram.vanespen@dynapps.be`), y `partner_name`/`contact_name`/`phone`/`website` vacíos. Este paso **repara la ficha ANTES** de investigar (2.1) y de postear (2.5). Sin él, 2.1 investigaría el dominio del relay (`dynapps.be`) y la ficha llegaría vacía a la llamada.

**2.0.1 — Localizar el mensaje web.**
```
orm_search_read('mail.message',
  [('model','=','crm.lead'), ('res_id','=',<lead_id>)],
  ['body','email_from','author_id','subject','date'], order='date asc')
```
Buscar el mensaje cuyo `body` contenga el marcador `created via the website` / `creado a través del sitio web` (case-insensitive; suele ser el primero).
- **Sin marcador** → lead manual (no-web): **saltar 2.0** y seguir con 2.1 (flujo actual). Log `2.0 sin marcador web → skip parse`.
- Si hay varios con marcador, usar el **más antiguo** (el de creación).

**2.0.2 — Parsear los campos del formulario** del body (strip de HTML). Mapa de etiquetas (ES/EN, case-insensitive, admite `:` o salto de línea como separador):

| Campo form | Sinónimos aceptados | Destino |
|---|---|---|
| Company | `Company`, `Empresa`, `Compañía`, `Compania` | `crm.lead.partner_name` |
| Name | `Name`, `Nombre`, `Contacto`, `Contact` | `crm.lead.contact_name` |
| Email | `Email`, `E-mail`, `Correo` | `crm.lead.email_from` |
| Phone | `Phone`, `Teléfono`, `Telefono`, `Móvil`, `Movil` | `crm.lead.phone` |
| Description | `Description`, `Descripción`, `Mensaje`, `Comentario` | **solo lectura** → alimenta 2.1/form; **NO escribir** |
| ¿Ya usas Odoo? | `Already use Odoo`, `¿Ya usas Odoo?`, `Usa Odoo`, `Currently use Odoo` | **solo** → sugerencia "Sistema actual" del form; sin campo ni tag |

- **Por campo**: saltar si vacío o placeholder (`___`, `-`, `N/A`, `n/a`).
- **Umbral mínimo**: si no se extrae **ni Email ni Company** válidos → no hay datos fiables: log `2.0 chatter sin Email/Company` y seguir a 2.1 **sin escribir**.

**2.0.3 — Gate de relay (decide si se PISA).** Calcular `es_relay` sobre el `email_from` **actual** del lead: es relay si (a) coincide con el remitente del mensaje web (`author_id`/`email_from` del mensaje), **O** (b) su dominio está en la lista de relays (`dynapps.be`, `indaws.es` — ampliable), **O** (c) está vacío.
- `es_relay = True` → **PISAR** los 4 campos nativos con lo parseado (overwrite).
- `es_relay = False` (email real distinto del relay) → **RESPETAR** el `email_from`/contacto actuales (log `2.0 email real <x> conservado`); a esos campos solo se les escribe lo que esté **vacío**.

**2.0.4 — Escribir en la ficha** (con `orm_write`/`orm_execute_kw method=write`, **antes** de 2.1 y 2.5):
- `crm.lead`: `partner_name`←Company, `contact_name`←Name, `email_from`←Email, `phone`←Phone — pisar-si-relay (2.0.3) o solo-si-vacío si el email es real.
- `partner_id`: si el partner enlazado es el relay (su email/dominio es relay) → `partner_id = False` (desvincula del contacto relay). **NUNCA crear partners** ni reescribir un partner legítimo existente.
- `source_id` (utm.source = origen web): resolver `orm_search_read('utm.source', ['|',('name','=','Website'),('name','=','Sitio web')], ['id'], limit=1)`. Si hay match **y** `source_id` está vacío → escribir. Sin match → log + omitir (**NO crear**).
- **EXCLUIDOS** (igual que 2.1b, por el bug de lectura del connector): `vat`/CIF, `is_company`, `industry_id` — no se pueden verificar vacíos con fiabilidad. Ver [[project_crm_lead_vat_field_bug]] y [[project_preflight_mcp_field_domain_bugs]].
- **NO tocar `description`** (lo rellena el SDR con el resumen; 2.1 lo lee tal cual).
- Guardar lo parseado en `web_parse` para 2.1 y 2.2: el `domain` de research sale ahora del **Email/website reparado** (NO del relay), y "¿Ya usas Odoo?" → `suggestions.sistema`/`notas_sistema` como **dato declarado**.

**Idempotencia**: re-parsear en reintentos es inocuo (escritura idempotente; mismo resultado). La nota `subject="Form de cualificacion pre-rellenado"` sigue siendo la **única** llave anti-duplicados (Paso 1).

### 2.1 Investigación pre-form (jerárquica)

**Aplicar el mismo flujo que `qualification-form/references/research.md`** (Tier 0 → Tier 1 → stop check → Tier 2 → Tier 3 scout-odoo → consolidar `suggestions`). Resumen:

1. **Tier 0** — Leer lead+partner: `description, vat, website, country_id, email_from, contact_name, phone`. Extraer `domain` (descartar genéricos: `gmail.com, hotmail.com, outlook.com, yahoo.com, icloud.com, protonmail.com, live.com, msn.com, aol.com, me.com`) y `cif` (quitar prefijo "ES"). **El `email`/`domain` vienen REPARADOS de 2.0 (`web_parse`): investigar el dominio del prospecto, NUNCA el del relay (`dynapps.be`/`indaws.es`).**
2. **Tier 1** — UNA call dirigida: `cif`→`WebFetch einforma/{cif}` (si 404, fallback a homepage + SERP en paralelo, ver gotchas.md); solo `domain`→`WebFetch https://{domain}`; solo `nombre`→`WebSearch "{empresa}" facturacion empleados site:einforma.com OR site:axesor.es OR ...`.
3. **Stop check** — si actividad + facturación + empleados → skip Tier 2.
4. **Tier 2** (solo si gap, máx 1 call): falta sector→homepage; falta facturación→SERP prensa; falta empleados→LinkedIn.
5. **Tier 3 scout-odoo** — siempre `Skill('scout-odoo', "Busca '{empresa}' en clientes Odoo. Devuelve partner actual y switcher history. Si no esta, responde 'no encontrada'.")`. Match→`notas_sistema`; error/no-match→continuar. scout-odoo es MCP-only (`odoo-scraper-mcp` / "Odoo References Scraper", `scraper_run_query`); sin CLI.
6. **Leer `description`** del lead — pain/módulos/timeline/decisores declarados. Strip de HTML si viene de email reply.
7. **Construir `suggestions`** (mapeo conservador de `research.md`): `actividades`/`notas_*` de fuentes verificables; `facturacion`/`empleados` solo con fuente+año; `decisores` SIEMPRE con `contact_name`+`email_from`+`phone`; `pain_points`/`modulos`/`presupuesto`/`usuarios` NUNCA desde web; `timeline` solo si está en `description`.
8. Si nada relevante → `suggestions = {}`.

(Sin caché entre leads: en Indaws es muy raro que entren 2 leads del mismo dominio — coste de caché > beneficio.)

### 2.1b Escribir campos FRÍOS en Odoo (conservador, solo si vacío)

Persistir los campos fríos verificables en la ficha (sustituye el enriquecimiento del antiguo crm-pre-qualification; mismo mapeo). **Solo lo que tenga fuente citable Y solo si el campo está vacío.**

**Campos que SÍ se escriben:**

| Modelo | Campo | De dónde | Notas |
|---|---|---|---|
| `res.partner` | `vat` | CIF | con prefijo `ES` |
| `res.partner` | `company_registry` | CIF | sin prefijo |
| `res.partner` | `is_company` | research | `True` si es empresa |
| `res.partner` | `industry_id` | sector/CNAE | resolver nombre→id (`res.partner.industry`); si no hay match claro, SKIP |
| `res.partner` | `city`/`state_id`/`country_id` | research | `state_id`→`res.country.state`, `country_id`→`res.country` |
| `res.partner` | `comment` | resumen research | breve, con fuentes |
| `crm.lead` | `partner_name` | nombre empresa | |
| `crm.lead` | `city`/`state_id`/`country_id` | research | |

**NUNCA se escriben aquí:** `crm.lead.description` (texto del cliente; además el 2.1 lo lee), `expected_revenue` (es el valor del proyecto Indaws, no la facturación del prospecto → esa va en `comment`/nota), ni campos calientes/especulativos (pain, módulos, presupuesto, usuarios, timeline).

**Procedimiento:**
1. Releer valores actuales (Tier 0 ya leyó el partner; leer también los del lead).
2. Resolver relacionales con `orm_search_read`: `country_id` (`res.country`), `state_id` (`res.country.state` + país), `industry_id` (`res.partner.industry` por nombre/CNAE; si no hay match confiable, omitir).
3. Construir el dict **solo con claves cuyo valor actual sea vacío/falsy Y con dato verificable**.
4. Escribir con `orm_execute_kw` (method `write`): una llamada a `res.partner` (`partner_id`) y otra a `crm.lead` (`lead_id`), solo si el dict no está vacío. Ej: `orm_execute_kw(model='res.partner', method='write', args=[[partner_id], {dict_solo_vacios}])`.
5. Guardar `campos_escritos` (nombre + fuente) SOLO para el reporte final (Paso 3). **No** se muestran en la nota del chatter.
6. Si la escritura MCP falla → log y continuar (no bloquea form ni nota).

### 2.2 Generar HTML del formulario

`Read` de `../qualification-form/templates/qualification-form.html` y sustituir (simple replace, no Jinja):
- `{{lead_id}}` → `str(lead.id)`
- `{{lead_name}}` → `lead.name` HTML-escaped (`& < > "`)
- `{{partner_name}}` → el `partner_name` **reparado en 2.0** (Company del chatter); fallback `lead.partner_id[1]` → `lead.contact_name`. HTML-escaped.
- `{{sdr_name}}` → **`lead.user_id[1]`** (comercial asignado, no quien dispara el cron)
- `{{fecha}}` → `YYYY-MM-DD`
- `{{lead_name_json}}` → `json.dumps(lead.name)`; `{{sdr_name_json}}` → `json.dumps(lead.user_id[1])`
- `{{suggestions_json}}` → `json.dumps(suggestions)` (o `{}`)

Verificar que no queda ningún `{{...}}` sin sustituir (salvo el comentario literal ~línea 181 del template).

### 2.3 Publicar HTML

`utilities:publish` con el HTML completo. Esperar la URL (puede tardar; NO reintentar antes de 30s). Si falla para este lead → log, continuar.

### 2.4 Resolver `partner_id` del `user_id` (para el mention)

El mention usa `res.partner.id`, no `res.users.id`:

```
orm_search_read(model='res.users', domain=[('id','=',<lead.user_id[0]>)], fields=['partner_id'], limit=1)
```
Guardar `user_partner_id = result[0]['partner_id'][0]` y `user_partner_name`. Si `user_id` vacío → postear sin mention, log warning.

### 2.5 Postear nota chatter con mention

**La nota del chatter contiene SOLO el mention + el link al formulario. Nada más.** Toda la investigación (actividad, sistema, pain, decisores) y la traza de campos escritos viven DENTRO del formulario y en la ficha — repetirlas en el chatter lo ensucia. No incluyas bullets de "Investigación", ni "Datos pre-rellenados", ni "Campos fríos escritos en la ficha".

Body exacto (sustituir `{USER_PARTNER_ID}`, `{USER_PARTNER_NAME}`, `{URL}`):

```html
<div style="font-family:'Readex Pro',system-ui,Verdana,sans-serif;color:#050543;background:#F5F5FA;padding:18px;border-radius:10px;border-left:4px solid #951CF3;max-width:680px">
  <p style="margin:0 0 12px 0;font-size:13px;color:#050543"><a href="#" class="o_mail_redirect" data-oe-model="res.partner" data-oe-id="{USER_PARTNER_ID}">@{USER_PARTNER_NAME}</a>, he pre-cualificado este lead automaticamente.</p>
  <p style="margin:0 0 14px 0;font-size:13px;line-height:1.5;color:#050543">Formulario de cualificacion pre-rellenado (revisalo, completalo en la llamada y pega el resumen aqui en el chatter):</p>
  <p style="margin:0;font-size:15px"><a href="{URL}" target="_blank" style="color:#951CF3;font-weight:700;text-decoration:underline">Abrir formulario de cualificacion &rarr;</a></p>
</div>
```

**Dos cosas que TIENEN que ir como `<a>` (causa nº1 del bug Anna/Toldos Transol 2026-06-17 — link muerto + mention en texto plano):**

1. **El link SIEMPRE es `<a href="{URL}">…texto…</a>`, NUNCA la URL cruda.** Odoo **NO autolinka** una URL pelada dentro de un body HTML (ni dentro de un `<p>`); queda como texto no clicable. El único link clicable es un ancla `<a href>` explícita. No pegues nunca `https://...` suelto en el body.
2. **El mention SIEMPRE es `<a href="#" class="o_mail_redirect" data-oe-model="res.partner" data-oe-id="{USER_PARTNER_ID}">@{USER_PARTNER_NAME}</a>`.** Escribir `@Nombre` como texto plano NO genera el chip clicable (aunque pases `partner_ids`). La clase `o_mail_redirect` + `data-oe-id` es lo que Odoo convierte en tag.

- **NUNCA un botón con `color:#fff`** (texto blanco): Odoo limpia el `background` inline en el chatter → texto blanco invisible. Link de texto color visible (`#951CF3`) + `text-decoration:underline`.
- Si `user_id` está vacío → postear el mismo body **sin** el `<a ...o_mail_redirect...>@...</a>` (sin mention), log warning.
- No añadir footer, heading, ni párrafos extra: el form ya explica qué hacer.

Postear con `orm_execute_kw` method `message_post` (el MCP Dynapps `88fb5415-...` es un passthrough ORM directo → `message_post` recibe el HTML nativo, conservando links y chips clicables; NO existe ya el tool dedicado `post_message`):

```
orm_execute_kw(
  model='crm.lead',
  method='message_post',
  ids=[<lead.id>],
  kwargs={
    'body': <html>,
    'subject': 'Form de cualificacion pre-rellenado',
    'subtype_xmlid': 'mail.mt_note',
    'partner_ids': [<user_partner_id>],   # notifica al comercial; omitir si user_id vacío
    'message_type': 'comment',
  },
)
```
(A diferencia del wrapper antiguo, `message_post` vía proxy SÍ acepta `partner_ids`: úsalo para la notificación, además del ancla `o_mail_redirect` que crea el chip visible. Pasa el id del lead en `ids`, no en `args`.)

**Verificación OBLIGATORIA tras postear** (`orm_search_read` sobre `mail.message`, campo `body`): la nota es válida SOLO si el body contiene a la vez `<a href="https` (link clicable) y `class="o_mail_redirect"` (mention chip). Si en su lugar aparece la URL pelada fuera de un `<a>`, o `&lt;`/`&gt;` (HTML escapado), la nota salió DEGRADADA → `orm_unlink` y reintenta una vez.

**Si el post falla** (devuelve `Internal server error`) o sale ESCAPADO tras el reintento: **NO dejes una nota degradada**. Como el batch es idempotente y este lead no quedará marcado como procesado (sin nota con el `subject`), **SKIP del lead** (log en `errors` como `2.5 message_post fallido/escapado`) y el siguiente tick del cron lo reintenta. JAMÁS postear texto plano con URL cruda (no es clicable) ni dejar HTML escapado. (El escapado de `message_post` era un bug del conector antiguo `97489e40`; en el proxy Dynapps se espera HTML limpio, pero la verificación post-post es obligatoria igualmente.)

### 2.6 Manejo de errores por lead

Cualquier error en 2.1-2.6 → capturar excepción, loggear `{lead_id, lead_name, step, error_msg}` en `errors`, **continuar con el siguiente lead**. NO abortar el batch. La escritura 2.1b es no-bloqueante.

## Paso 3 — Reporte final

```
Batch pre-qualification completado.
Total leads en query:    {total}
Skipped (ya procesados): {skipped}
Procesados con exito:    {processed}
Fichas reparadas (2.0):  {repaired}
Errores:                 {errors_count}

Procesados (lead_id - name - sdr):
  - 1234 - Construcciones Lopez SL - Ana Garcia   [web]
Errores:
  - 1236 - Talleres XYZ - paso 2.3 (utilities:publish timeout)
```

- `total=0` → "No hay leads nuevos elegibles para cualificar."
- `errors_count > 0` → sugerir acción ("Reintenta con /qualify-preflight tras revisar el connector MCP").
