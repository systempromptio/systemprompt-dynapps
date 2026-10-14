# Account Followup — Integrations

Bridges del módulo Enterprise `account_followup` con account (overdue/AML), partner (followup levels), mail (templates multi-language), portal, CRM (escalation responsables), SMS, multi-company y cron. Ubicación: `odoo_enterprise/18/account_followup/`.

Para gotchas del core ver `gotchas.md` (AFG1-34). Aquí solo bridges. Continúa: **AFG35+**.

Convención IDs: **AFG<N><suf>** (A=account/overdue, L=line/workflow, P=partner, M=mail, PT=portal, CRM=crm, MC=multi_company, C=cron, S=sms).

---

## 1. account_followup ↔ account (overdue invoices)

### Modelos y campos

- `res.partner` (extended) [`account_followup/models/res_partner.py:15`, `ResPartner`]:
  - `followup_status` (Selection: `no_action_needed`/`in_need_of_action`/`with_overdue_invoices`) — computed + search.
  - `followup_line_id` (M2O `account_followup.followup.line`) — computed + inverse + search; nivel "alcanzado" en función del AML más antiguo.
  - `total_due`, `total_overdue` — Monetary, restringido a `asset_receivable`.
  - `total_all_due`, `total_all_overdue` — Monetary, incluye `liability_payable` (vendors). Pendientes de renombrar en master.
  - `unreconciled_aml_ids` (O2M `account.move.line`, `readonly=False`) — editable directamente desde la vista para excluir/incluir líneas.
  - `unpaid_invoice_ids`, `unpaid_invoices_count` — solo `out_invoice/out_refund` posted con `payment_state in ('not_paid','partial')`.
  - `has_moves` — boolean basado en EXISTS(account_move).
- `account.move.line` (extended) [`account_followup/models/account_move_line.py:11`, `class AccountMoveLine`]:
  - `followup_line_id` (M2O) — nivel "aplicado" a esta línea concreta tras envío.
  - `invoice_origin` (related `move_id.invoice_origin`) — usado en plantilla del informe.
  - Custom `_read_group_groupby` para grupo `followup_overdue` (date_maturity < current_date).

### Pipeline

```
invoice.post() → AML residual queda en partner.unreconciled_aml_ids
  → res.partner._compute_total_due() filtro: reconciled=False, account_type IN (asset_receivable,liability_payable), parent_state=posted, company_id child_of env.company
  → _query_followup_data SQL LATERAL-join:
     - subquery agregada: para cada partner, MAX(COALESCE(next_ful.delay, ful.delay)) = followup_delay alcanzado
     - LATERAL in_need_of_action_aml: existe AML con date_maturity + ful.delay < current_date Y delay < partner.followup_delay
     - LATERAL exceeded_unreconciled_aml: existe AML overdue genérico (date_maturity < current_date)
  → CASE final asigna followup_status:
       amount_residual <= 0          → no_action_needed
       in_need_of_action_aml found + next_action_date NULL o <= today → in_need_of_action
       exceeded_unreconciled_aml found → with_overdue_invoices
       else → no_action_needed
```

### Configuración

- Cuentas con `account_type='asset_receivable'` y `deprecated=False`.
- `aml.date_maturity` obligatorio para entrar en el SQL (NULL → ignorado en lateral joins).
- `aml.parent_state='posted'` y `aml.reconciled=False`.
- Multi-currency: residual en `amount_residual_currency` para presentación; SQL agrega `amount_residual` (company currency).

### Gotchas

**AFG35A.** `_get_unreconciled_aml_domain` incluye `liability_payable` (vendor bills). Si tu localización mete payables en la misma vista, salen mezclados en `unreconciled_aml_ids`. `_compute_total_due` los suma a `total_all_due` (no a `total_due`), pero la O2M expone ambos. Ref `account_followup/models/res_partner.py:205-212`, `ResPartner._get_unreconciled_aml_domain`.

**AFG36A.** El SQL filtra `aml.company_id = ANY(%(company_ids)s)` con `company_ids = child_of env.company`, no `parent_of`. Si trabajas en company hija, no ves AMLs de la matriz aunque la línea de followup sea `parent_of`. Asimetría: lines visibles ≠ AMLs computados. Ref `account_followup/models/res_partner.py:470`, `ResPartner._get_followup_data_query`.

**AFG37A.** `_set_followup_line_on_unreconciled_amls` (inverse del campo computed): si el usuario edita `followup_line_id` en form, propaga el nivel **anterior** a todas las AMLs unreconciled (no el seleccionado). Lógica intencional pero contraintuitiva: simula "estamos en el nivel N, así que las líneas están a N-1". Ref `account_followup/models/res_partner.py:197-203`, `ResPartner._set_followup_line_on_unreconciled_amls`.

**AFG38A.** El grupo `followup_overdue` en `_read_group` SOLO funciona si `date_maturity IS NOT NULL`. AML sin fecha vencimiento (asientos manuales mal grabados) cuenta como `overdue=False`. Ref `account_followup/models/account_move_line.py:14-21`, `AccountMoveLine._read_group_groupby`.

**AFG39A.** `_compute_unpaid_invoices` usa `commercial_partner_id`, mientras `_compute_total_due` usa `partner_id`. Si un contacto hijo recibe una factura directamente (sin commercial), el contador y la lista divergen. Ref `account_followup/models/res_partner.py:124-131`, `ResPartner._compute_unpaid_invoices` vs `account_followup/models/res_partner.py:175-194`, `ResPartner._compute_total_due`.

**AFG40A.** `invoice_origin` related en AML añade JOIN extra en cualquier read masivo de `account.move.line` aunque no uses followup. Coste invisible. Ref `account_followup/models/account_move_line.py:12`, `class AccountMoveLine`.

### v19 changes

- Nuevo campo `aml.no_followup` (boolean): excluye AML específica del chain — sustituye al patrón "exclude credit notes" vía override.
- `_compute_total_due` añade `no_followup` al `_read_group` y a un nuevo bucket `receivable_overdue_followup_data`.
- Nuevo campo `partner.total_overdue_followup` = overdue **excluyendo** `no_followup`.
- SQL queries añaden `AND line.no_followup IS NOT TRUE`.
- `_get_unreconciled_aml_domain` devuelve `Domain.AND([...])`.

---

## 2. followup.line workflow

### Modelos y campos

- `account_followup.followup.line` [`account_followup/models/account_followup.py:8`, `FollowupLine`]:
  - `name` (Char required translate, único por company).
  - `delay` (Integer, **puede ser negativo** = recordatorio ANTES del vencimiento).
  - `company_id` (M2O required, `_check_company_auto=True`).
  - `mail_template_id` (M2O `mail.template` con domain `model='res.partner'`), `send_email` (default True).
  - `sms_template_id` (M2O `sms.template`), `send_sms` (default False).
  - `join_invoices` (Boolean, default True) — adjunta PDFs de facturas al email.
  - `additional_follower_ids` (M2M `res.users`) — subscritos a `partner.message_follow` tras envío.
  - `create_activity`, `activity_summary`, `activity_note`, `activity_type_id`, `activity_default_responsible_type` (Selection: `followup`/`salesperson`/`account_manager`).
  - `auto_execute` (Boolean) — habilita cron. Mutuamente excluyente con `create_activity` vía onchange.
- `_sql_constraints`:
  - `(company_id, delay)` UNIQUE — no dos niveles con el mismo delay.
  - `(company_id, name)` UNIQUE — no dos niveles con el mismo nombre.
- `_order = 'delay asc'`.

### Pipeline

```
followup.line._get_next_date():
  next_followup = search(delay > self.delay, company=env.company, order=delay asc, limit=1)
  if next_followup:
      delta = next_followup.delay - self.delay
  else:
      previous_followup = search(delay < self.delay, ...)
      if previous_followup: delta = self.delay - previous_followup.delay
      else: delta = self.delay  # único nivel
  return today + timedelta(days=delta)

State transition partner:
  in_need_of_action (status) → trigger _execute_followup_partner
    → if line.create_activity: activity_schedule(activity_type_id || default, user=_get_followup_responsible)
    → _update_next_followup_action_date(line):
       - partner.followup_next_action_date = line._get_next_date()
       - aml.followup_line_id = max(eligible_levels matching delay)
    → _get_followup_attachments + _send_followup
```

### Configuración

- Niveles típicos: 15/30/60/90 días.
- `delay=-7` para recordatorio pre-vencimiento (avisar 7 días antes).
- `auto_execute=True` solo en niveles que el cron debe ejecutar sin intervención.

### Gotchas

**AFG41L.** `copy_data` añade **+15 días al delay más alto** de la company por cada copia. Si copias 3 niveles en batch, sus delays se desplazan 15/30/45 días respectivamente del máximo. Evita colisión `(company,delay)` pero crea niveles inesperados. Ref `account_followup/models/account_followup.py:47-73`, `FollowupLine.copy_data`.

**AFG42L.** Onchange `_onchange_auto_execute` solo se dispara en UI form. Crear vía ORM con `auto_execute=True` y `create_activity=True` simultáneamente NO levanta error — el cron creará actividades duplicadas. Ref `account_followup/models/account_followup.py:75-78`, `class FollowupLine`.

**AFG43L.** `_get_next_date()` con un solo nivel devuelve `today + line.delay` (siempre el mismo intervalo). Si delay=30 y solo hay un nivel, `next_action_date` se reseteará 30 días cada envío → loop infinito de recordatorios. Diseño esperado pero a menudo no entendido. Ref `account_followup/models/account_followup.py:80-99`, `FollowupLine._get_next_date`.

**AFG44L.** `_get_next_followup` y `_get_previous_followup` filtran por `env.company.id` (no `parent_of`). En entornos multi-company anidados, el nivel "siguiente" puede no encontrarse aunque exista en una company padre. Ref `account_followup/models/account_followup.py:101-107`, `FollowupLine._get_next_followup`.

**AFG45L.** `additional_follower_ids` añade usuarios como followers del partner **permanentemente** (`message_subscribe`). Después del envío quedan suscritos a cualquier mensaje futuro, no solo a respuestas al recordatorio. Ref `account_followup/models/account_followup_report.py:422-423`, `AccountFollowupReport._send_email`.

**AFG46L.** `_check_company_auto=True` en la línea valida company contra `mail_template_id`/`sms_template_id`. Plantillas globales (`company_id=False`) son OK; plantillas atadas a otra company → ValidationError silencioso en save desde XML. Ref `account_followup/models/account_followup.py:12`, `class FollowupLine`.

**AFG47L.** `name` field es `translate=True` pero `(company_id, name)` UNIQUE constraint es **case-sensitive** y se aplica sobre el valor base, no la traducción. Mismos nombres traducidos a otro idioma no colisionan vía UI. Ref `account_followup/models/account_followup.py:14`, `FollowupLine.name` y `account_followup/models/account_followup.py:44`, `FollowupLine._sql_constraints`.

### v19 changes

- `_sql_constraints` migrados a `models.Constraint` (PEP8-style).
- Clase renombrada `FollowupLine` → `Account_FollowupFollowupLine`.
- `copy_data` usa `_read_group` con tuple unpacking.
- `send_email` label: "Send Email" → "Email"; `send_sms`: "Send SMS Message" → "SMS".

---

## 3. account_followup ↔ partner (followup levels)

### Modelos y campos

- `res.partner` campos clave:
  - `followup_next_action_date` (Date, **company_dependent=True**) — bloqueo manual: no se envía followup hasta esa fecha.
  - `followup_responsible_id` (M2O `res.users`, **company_dependent**, tracking) — responsable manual asignado.
  - `followup_reminder_type` (Selection: `automatic`/`manual`, default `automatic`) — switch global por partner.
  - `type` Selection extended: `+('followup', 'Follow-up Address')` — tipo de child contact "para recordatorios".
- Computed con `@api.depends_context('company', 'allowed_company_ids')` — clave para multi-company.

### Pipeline

```
_get_followup_responsible() priority resolution:
  1. line.activity_default_responsible_type == 'account_manager' AND partner.user_id → user_id
  2. responsible_type == 'salesperson' AND most_delayed_aml.move_id.invoice_user_id → ese
  3. partner.followup_responsible_id (manual override)
  4. partner.user_id (salesperson general)
  5. most_delayed_aml.move_id.invoice_user_id (fallback)
  6. super()._get_followup_responsible() (en partner sin custom: None)

next((u for u in candidates if u and u.active), super())
```

### Configuración

- `followup_reminder_type='manual'`: partner excluido del cron auto. Solo wizard manual.
- `followup_next_action_date`: editable inline desde form para "pausar" un partner concreto.
- `followup_responsible_id` company_dependent: cada company puede tener su propio responsable para el mismo partner.

### Gotchas

**AFG48P.** `_get_followup_responsible` filtra por `u.active`. Si todos los candidatos están archivados → cae a `super()` que retorna `None` (no hay padre). Email se envía con `email_from=False` → fallback a `mail.outgoing_mail_server.smtp_user`. Ref `account_followup/models/res_partner.py:214-229`, `ResPartner._get_followup_responsible`.

**AFG49P.** `followup_responsible_id` con `company_dependent=True` Y `tracking=True`: tracking dispara mensajes en chatter solo cuando cambia en la company actual; cambios en otras companies no logean en current. Riesgo de auditoría incompleta. Ref `account_followup/models/res_partner.py:64-72`, `class ResPartner`.

**AFG50P.** `_compute_followup_status` depende SOLO de `unreconciled_aml_ids` y `followup_next_action_date`. Si pagas una factura **vía reconciliación manual** sin disparar invalidación del cache `res_partner_all_followup`, el estado puede quedar stale hasta siguiente transacción. Ref `account_followup/models/res_partner.py:114-121`, `class ResPartner`.

**AFG51P.** `type='followup'` en child contacts: ese tipo NO se valida contra ningún esquema. Es un valor "marcador" que solo lee `_get_all_followup_contacts`. Si lo activas en un partner padre por error, aparece duplicado en la lista de contactos. Ref `account_followup/models/res_partner.py:63`, `ResPartner.type`, y `account_followup/models/res_partner.py:231-240`, `ResPartner._get_all_followup_contacts`.

**AFG52P.** `_get_all_followup_contacts` fallback: si NO hay child con `type='followup'` y `address_get(['invoice'])` devuelve el propio partner padre, el email se envía a sí mismo (cuando partner.email == empresa.email genérica). Ref `account_followup/models/res_partner.py:236-240`, `ResPartner._get_all_followup_contacts`.

**AFG53P.** Inverse `_set_followup_line_on_unreconciled_amls` aplica el nivel **anterior** a las AMLs (no el seleccionado). Si lo seteas manualmente al nivel más bajo, `previous_followup_line` queda vacío → todas las AMLs quedan con `followup_line_id=False`, perdiendo el "alto historial". Ref `account_followup/models/res_partner.py:197-203`, `ResPartner._set_followup_line_on_unreconciled_amls`.

**AFG54P.** `_query_followup_data(all_partners=True)` cachea en `env.cr.cache['res_partner_all_followup']`. La caché NO discrimina por company. Iterar companies en un mismo cursor sin `cache.pop(...)` produce datos cruzados. Ref `account_followup/models/res_partner.py:369-380`, `ResPartner._get_all_followup_data` y `account_followup/models/res_partner.py:657-659`, `ResPartner._cron_execute_followup`.

### v19 changes

- `type` Selection ya NO añade `'followup'`. `_complete_name_displayed_types` override eliminado.
- `_get_all_followup_contacts` retorna SOLO billing address (`address_get(['invoice'])`).
- `_get_followup_responsible(multiple_responsible=False)` — nuevo kwarg.
- `_included_unreconciled_aml_max_followup` eliminado; lógica inlined.
- **Upgrade impact**: partners con `child_ids.type='followup'` quedan huérfanos.

---

## 4. account_followup ↔ mail (templates, multi-language)

### Modelos y campos

- `mail.template` (`email_template_followup_1`, `demo_followup_email_template_2`):
  - `model_id` → `res.partner` (no `account.move`).
  - `email_from`: expresión `{{ object._get_followup_responsible().email_formatted }}`.
  - `subject`: `{{ (object.company_id or object._get_followup_responsible().company_id).name }} Payment Reminder - {{ object.commercial_company_name }}`.
  - `lang`: `{{ object.lang }}` — idioma del partner, fallback `env.user.lang`.
  - `auto_delete=False` — preserva mensajes para auditoría.
- `account_followup.manual_reminder` hereda `mail.composer.mixin`.

### Pipeline

```
_send_email():
  partner = browse(options['partner_id'])
  followup_contacts = partner._get_all_followup_contacts() or partner
  for to_send_partner in _get_email_recipients(options):
    if to_send_partner.email:
       self.with_context(lang=partner.lang or env.user.lang)
       body_html = get_followup_report_html(options)  # rendered QWeb
       attachment_ids = options['attachment_ids']  # report PDF + invoices
       author_id = options.get('author_id', partner._get_followup_responsible().partner_id.id)
       partner.with_context(mail_post_autofollow=True, mail_notify_author=True).message_post(
           partner_ids=[to_send_partner.id],
           email_from=_get_email_from(options),       # rendered template.email_from
           body=body_html,
           subject=_get_email_subject(options),
           reply_to=_get_email_reply_to(options),
           email_layout_xmlid='mail.mail_notification_light',
           attachment_ids=attachment_ids,
           subtype_id=mail.mt_note,
       )
       if line.additional_follower_ids:
           partner.message_subscribe(line.additional_follower_ids.partner_id.ids)
```

### Configuración

- `mail.template.lang={{ object.lang }}`: si `partner.lang=False` → caída a `env.user.lang` (NO `partner.company_id.partner_id.lang`).
- `subtype_id=mail.mt_note`: por defecto NO notifica followers genéricos, solo `partner_ids` explícitos.
- `email_layout_xmlid='mail.mail_notification_light'`: layout minimalista sin chrome de notificación corporativa.

### Gotchas

**AFG55M.** `subtype_id=mail.mt_note` (note interna) en lugar de `mt_comment`: los recordatorios NO disparan auto-follow de partner si la company tiene `mail.mt_comment` configurado. Followers genéricos (sales team, accounting) NO reciben copia. Comportamiento intencional para evitar spam pero suele sorprender. Ref `account_followup/models/account_followup_report.py:417`, `AccountFollowupReport._send_email`.

**AFG56M.** `_get_email_from` resuelve `{{ object._get_followup_responsible().email_formatted }}` en runtime. Si el responsible no tiene email → render devuelve `''` → email_from final es `None` → fallback al MTA configurado. Riesgo: el cliente ve emails desde `notifications@odoo.com` o similar. Ref `account_followup/models/account_followup_report.py:250-262`, `AccountFollowupReport._get_email_from`.

**AFG57M.** `lang` context se setea ANTES de leer template.body_html: `template.with_context(lang=partner.lang or env.user.lang).body_html`. Si la traducción no existe → `body_html` devuelve el master string (English). NO hay fallback a `partner.company_id.partner_id.lang`. Ref `account_followup/models/account_followup_report.py:288`, `AccountFollowupReport._get_main_body`.

**AFG58M.** `mail_notify_author=True` (v18 vía context) hace que el responsible reciba copia del propio email enviado. Útil para auditoría pero duplica buzón si el responsible Y el partner comparten dominio corporativo. En v19 se mueve a kwarg `notify_author=True`. Ref `account_followup/models/account_followup_report.py:407`, `AccountFollowupReport._send_email`.

**AFG59M.** `_get_email_reply_to`: si `mail_template.reply_to=False` → cae a `_notify_get_reply_to()[False]` = catchall de la company. Reply del cliente NO va al responsible sino al alias genérico, perdiendo trazabilidad. Ref `account_followup/models/account_followup_report.py:264-275`, `class AccountFollowupReport`.

**AFG60M.** `_get_main_body` fallback a `default_body` (plaintext con `\n`) si no hay `mail_template`. Concatena `plaintext2html(default_body) + responsible.signature` (signature ya es HTML). Si signature contiene `<style>` global → estilos escapan al contenedor del email. Ref `account_followup/models/account_followup_report.py:293-305`, `AccountFollowupReport._get_main_body`.

**AFG61M.** Adjuntos: `attachment_ids` mezcla report PDF (siempre) + facturas (si `join_invoices=True`) + template attachments (si `mail_template_id._generate_template_attachments`). Sin deduplicación → si el report es ALSO un template attachment, se adjunta dos veces. Ref `account_followup/models/res_partner.py:508-542`, `ResPartner._get_followup_attachments`.

### v19 changes

- `mail_template.xml` externalizado en `data/mail_template.xml` separado de `account_followup_data.xml`.
- `_render_template` kwargs-only: `_render_template(template_src, model, res_ids=[...], render_fields=set(), allow_suggested=False, find_or_create_partners=True)`.
- `message_post`: `mail_notify_author` (context) → `notify_author` (kwarg directo).

---

## 5. account_followup ↔ portal

### Modelos y campos

- `res.partner._show_pay_now_button()` — boolean helper:
  - `ir.config_parameter.account_payment.enable_portal_payment` (sudo, str→bool).
  - `payment.method.search_count([('active','=','True')])` — **NOTA**: bug latente, compara con string `'True'` no booleano (ver gotcha).
- Plantilla mail incluye `<a href="/my/invoices/overdue">Pay now</a>` condicional a `_show_pay_now_button()`.
- Bridge: `account_payment` (CE) define `/my/invoices/overdue` route en `account_payment/controllers/portal.py:53`, `class PortalAccount` con `auth='public'` + tx route `/invoice/transaction/overdue`.

### Pipeline

```
Email rendered con button "Pay now":
  → click → GET /my/invoices/overdue (auth=public, redirige a login si no portal user)
  → portal renderiza lista overdue invoices del commercial_partner_id del usuario portal
  → user selecciona "Pay" → POST /invoice/transaction/overdue (JSON)
    → payment.transaction.create + redirect a payment provider
  → tx done → account.move._post_payment_state → AML reconciled
  → next cron run: partner.followup_status recalcula a 'no_action_needed' si saldo=0
```

### Configuración

- `account_payment.enable_portal_payment=True` en `ir.config_parameter` (System Parameter).
- Al menos un `payment.method.active=True`.
- Partner debe ser portal user (`base.group_portal`) para acceder a `/my/invoices/overdue` autenticado; con `auth='public'` puede ver versión "pay by token" para emails con `access_token`.

### Gotchas

**AFG62PT.** Bug latente en `_show_pay_now_button`: `payment.method.search_count([('active','=','True')])` compara con **string** `'True'` en lugar de boolean `True`. Active es Boolean → comparación incorrecta. En la práctica funciona por coerción PostgreSQL pero es semánticamente incorrecto y rompería en backend con type-checking estricto. Ref `account_followup/models/res_partner.py:661-664`, `ResPartner._show_pay_now_button`.

**AFG63PT.** Botón "Pay now" se renderiza en TODOS los emails (todos los niveles) si `_show_pay_now_button()=True`. No hay flag por followup.line para deshabilitar en ciertos niveles (ej. "primera carta no muestra pagar online, solo recordatorio amable"). Hay que custom override del template. Ref `account_followup/data/account_followup_data.xml:22-24`, `email_template_followup_1`.

**AFG64PT.** El link `/my/invoices/overdue` NO incluye partner-specific access token. Si el partner NO está logueado, ve la página de login estándar (no la versión "pay anonymous con token"). Diferencia respecto a links de factura individuales que sí incluyen `access_token`. Ref template XML `account_followup/data/account_followup_data.xml:23`, `email_template_followup_1`.

**AFG65PT.** `account_payment` es CE, no Enterprise. Si la instalación NO tiene `account_payment` instalado (raro pero posible en CE puro sin esta dependencia), el método `_show_pay_now_button` siempre devuelve `False` por ausencia de `payment.method`, y el botón nunca aparece — sin warning al admin. Ref `__manifest__.py` no declara `account_payment` como hard depend.

**AFG66PT.** `enable_portal_payment` se lee con `sudo()`. Si un usuario portal ha perdido permisos al system parameter (caso edge multi-tenancy), igual ve el botón porque el chequeo se hace al renderizar el email del responsable interno. Ref `account_followup/models/res_partner.py:662`, `ResPartner._show_pay_now_button`.

---

## 6. account_followup ↔ CRM (escalation actions)

### Modelos y campos

- `account_followup.followup.line`:
  - `create_activity` (Boolean).
  - `activity_type_id` (M2O `mail.activity.type`) — opcional; si vacío, usa `_default_activity_type()` del partner.
  - `activity_default_responsible_type` (Selection: `followup`/`salesperson`/`account_manager`).
  - `activity_summary`, `activity_note`.
- `mail.activity` creada vía `partner.activity_schedule(...)`.

### Pipeline

```
_execute_followup_partner():
  if followup_line.create_activity:
    partner.activity_schedule(
        activity_type_id = line.activity_type_id.id or partner._default_activity_type().id,
        note             = line.activity_note,
        summary          = line.activity_summary,
        user_id          = partner._get_followup_responsible().id
    )

_get_followup_responsible() (priority):
  responsible_type == 'account_manager' + partner.user_id → partner.user_id  (commercial salesperson)
  responsible_type == 'salesperson' + most_delayed_aml.invoice_user_id → invoice salesperson
  else: followup_responsible_id > user_id > most_delayed_aml.invoice_user_id
```

### Configuración

- "Follow-up Responsible" (default): usa cadena de fallback completa.
- "Salesperson": prioriza invoice_user_id de la factura más impagada (escalada por venta concreta).
- "Account Manager": prioriza partner.user_id (relación comercial global).

### Gotchas

**AFG67CRM.** `activity_default_responsible_type='salesperson'` resuelve a `most_delayed_aml.move_id.invoice_user_id`. "Most delayed" se define como **mayor `amount_residual`** (NO mayor delay temporal). La actividad se asigna al vendedor de la factura más grande, no a la más antigua. Ref `account_followup/models/res_partner.py:266`, `ResPartner._included_unreconciled_aml_max_followup`.

**AFG68CRM.** `auto_execute=True` + `create_activity=True` están excluidos por `onchange`, pero solo en UI. Crear vía ORM/XML sin onchange genera actividades duplicadas: cron ejecuta envío Y crea activity. Ref `account_followup/models/account_followup.py:75-78`, `class FollowupLine`.

**AFG69CRM.** Activity creada NO se cierra automáticamente al pagar la factura. Si followup pasa de `in_need_of_action` a `no_action_needed`, las activities huérfanas quedan abiertas en el bandeja del responsible. Limpieza manual o vía cron custom. Ref `account_followup/models/res_partner.py:558-565`, `ResPartner._execute_followup_partner`.

**AFG70CRM.** No existe `mail.activity_type` predefinido para followup. `activity_type_id=False` cae a `_default_activity_type_for_model('res.partner')` = primer activity type del modelo res.partner (usualmente "To Do" genérico). En reporting de actividades, los followups se mezclan con todas las tareas pendientes. Hay que crear manualmente un activity type "Payment Follow-up". Ref `mail/models/mail_activity.py:39`, `MailActivity._default_activity_type`.

**AFG71CRM.** `partner.user_id` se usa como "salesperson general". Es el campo `user_id` estándar de `res.partner`, NO un campo específico de account. Cualquier cambio (CRM, sales) impacta a la cadena de responsable. No hay desacoplamiento. Ref `account_followup/models/res_partner.py:225`, `ResPartner._get_followup_responsible`.

**AFG72CRM.** `_get_followup_responsible` no devuelve `False`/empty: cae a `super()` (que en partner base NO existe → AttributeError). En la práctica funciona porque siempre hay user_id o admin como último fallback, pero si TODOS los candidatos están archivados → exception silenciada. Ref `account_followup/models/res_partner.py:229`, `ResPartner._get_followup_responsible`.

### v19 changes

- `_get_followup_responsible(multiple_responsible=False)`: nuevo kwarg.
- "Most delayed AML" lógica inlined (sin `_included_unreconciled_aml_max_followup`).
- En `multiple_responsible=True`, devuelve M2M de todos los invoice_user_id de unreconciled AMLs (para activities en lote o group followup actions).

---

## 7. account_followup ↔ multi-company

### Modelos y campos

- `followup.line.company_id` (required, default `env.company`).
- `partner.followup_next_action_date`, `followup_responsible_id` con `company_dependent=True`.
- SQL query: `company_ids = env.company.search([('id', 'child_of', env.company.id)]).ids`.
- Búsquedas de líneas: `[('company_id', 'parent_of', env.company.id)]`.

### Pipeline

```
_cron_execute_followup():
  for company in env['res.company'].search([]):
      env.cr.cache.pop('res_partner_all_followup', None)  # CRÍTICO
      self.with_context(allowed_company_ids=company.ids)._cron_execute_followup_company()

_cron_execute_followup_company():
  followup_data = _query_followup_data(all_partners=True)  # filtrado por env.company actual
  partners = filter status='in_need_of_action' + auto_execute + reminder_type='automatic'
  for partner in partners[:1000]:
      partner._execute_followup_partner()
```

### Configuración

- Estructura jerárquica: company padre + branches hijos.
- Líneas definidas en company padre se aplican a todos los hijos (`parent_of`).
- AMLs evaluados como `child_of env.company` (todas las hijas vistas desde padre).

### Gotchas

**AFG73MC.** Asimetría direccional **crítica**: `followup.line` se busca como `parent_of env.company` (hijos heredan del padre), pero `aml.company_id` se filtra como `child_of env.company` (padre ve hijos). Si trabajas en company hija sin padre encima, no encuentras líneas. Si trabajas en padre, evalúas AMLs de todas las hijas. Ref `account_followup/models/res_partner.py:98`, `ResPartner._search_followup_line` y `account_followup/models/res_partner.py:211`, `ResPartner._get_unreconciled_aml_domain` y `account_followup/models/res_partner.py:296`, `ResPartner._get_first_followup_level` y `account_followup/models/res_partner.py:351`, `ResPartner._get_followup_lines_info` y `account_followup/models/res_partner.py:470`, `ResPartner._get_followup_data_query`.

**AFG74MC.** Cache `res_partner_all_followup` NO se invalida automáticamente entre companies. `_cron_execute_followup` hace `cache.pop()` MANUAL entre iteraciones. Cualquier custom code que itere companies fuera del cron debe replicar el `pop`. Ref `account_followup/models/res_partner.py:657-659`, `ResPartner._cron_execute_followup`.

**AFG75MC.** `followup_next_action_date` company_dependent: pausar followup en company A NO pausa en B. Si el partner factura en ambas, recibe recordatorios de B mientras "está en pausa" en A. UX confuso para el cliente final que recibe doble email. Ref `account_followup/models/res_partner.py:18-24`, `class ResPartner`.

**AFG76MC.** `followup_responsible_id` company_dependent: cambio en company A se logea en chatter del partner pero el partner es global. El chatter mezcla cambios de todas las companies en un solo timeline. Auditoría no segregada por company. Ref `account_followup/models/res_partner.py:64-72`, `class ResPartner`.

**AFG77MC.** El cron itera `res.company.search([])` SIN filtro. Companies archivadas (`active=False`) NO entran (Odoo filtra `active` por defecto), pero companies sin `followup.line` definidas se procesan igual: query devuelve `followup_line_id=False` → ningún partner cumple → iteración vacía pero con coste SQL. Ref `account_followup/models/res_partner.py:654-659`, `ResPartner._cron_execute_followup`.

**AFG78MC.** `_get_followup_report` usa `forced_companies = child_of allowed_company_ids`. Si no hay `allowed_company_ids` en contexto (llamada custom directa), fallback a `env.company.id` y propaga down: posible inclusión de companies hijas no deseadas en el report. Ref `account_followup/models/res_partner.py:497-505`, `ResPartner._get_followup_report`.

**AFG79MC.** `(company_id, delay)` UNIQUE constraint es por company directa, no `parent_of`. Padre y hijo pueden tener líneas con `delay=15` ambos → `_get_first_followup_level` busca con `parent_of` y devuelve el más antiguo por orden de creación. Comportamiento NO determinista en upgrades. Ref `account_followup/models/account_followup.py:43`, `class FollowupLine`.

---

## 8. account_followup ↔ cron (`_cron_execute_followup`)

### Modelos y campos

- `ir.cron` `account_followup.ir_cron_auto_post_draft_entry` (xml_id heredado mal-nombrado):
  - `name`: "Account Report Followup; Execute followup".
  - `interval_number=1`, `interval_type='days'`.
  - `nextcall`: `(now().replace(hour=2, minute=0) + timedelta(days=1))`.
  - `model_id=model_res_partner`, `code='model._cron_execute_followup()'`.

### Pipeline

```
ir.cron tick (daily 02:00):
  _cron_execute_followup():
    for company in res.company.search([]):
        env.cr.cache.pop('res_partner_all_followup', None)
        self.with_context(allowed_company_ids=company.ids)._cron_execute_followup_company()

  _cron_execute_followup_company():
    followup_data = _query_followup_data(all_partners=True)
    partners_to_process = browse(
        d['partner_id'] for d in followup_data.values()
        if d['followup_status'] == 'in_need_of_action'
    ).filtered(
        lambda p: p.followup_line_id.auto_execute and p.followup_reminder_type == 'automatic'
    )[:1000]

    for partner in partners_to_process:
        try:
            partner._execute_followup_partner()  # crea activity, send email/sms, update aml.followup_line_id
        except UserError as e:
            partner._message_log(body=e)  # NO raise — sigue al siguiente
            _logger.warning(e, exc_info=True)
```

### Configuración

- `nextcall` recalculada cada noche a 02:00 (no exacto si el server estuvo down).
- Batch limit: **1000 partners por company por run**. Excedentes esperan al día siguiente.
- Solo procesa `auto_execute=True` Y `followup_reminder_type='automatic'`.

### Gotchas

**AFG80C.** `xml_id="ir_cron_auto_post_draft_entry"` es copy-paste leftover, NO tiene relación con post de asientos draft. Buscar el cron por nombre legible o por code, no por xml_id. Ref `account_followup/data/cron.xml:5`, `ir_cron_auto_post_draft_entry`.

**AFG81C.** Hard limit `[:1000]` por company por run. Empresa con 5000 partners overdue → solo se procesan 1000 el día N, 1000 el día N+1, etc. Sin notificación. Backlog silencioso. Hay que monitorizar `partners_to_process` length custom. Ref `account_followup/models/res_partner.py:645`, `ResPartner._cron_execute_followup_company`.

**AFG82C.** `UserError` capturada Y logged como warning (NO error). Errores recurrentes (partner sin email persistente) no escalan en monitoring estándar. Configurar log handler específico para warnings de `odoo.addons.account_followup`. Ref `account_followup/models/res_partner.py:648-652`, `ResPartner.e`.

**AFG83C.** Otras excepciones (no UserError) NO capturadas: `AccessError`, `ValidationError`, `KeyError` en template render → cron abortado completo, partners siguientes NO procesados. Sin retry. Ref `account_followup/models/res_partner.py:646-652`, `ResPartner._cron_execute_followup_company`.

**AFG84C.** `_cron_execute_followup_company` NO commitea per-partner. Toda la batch va en una sola transacción. Failure parcial = rollback total. Si el partner 999 falla con non-UserError, los 998 anteriores se revierten (mensajes enviados quedan, pero followup_line_id en AMLs y next_action_date partner se revierten). Inconsistencia: cliente recibe email pero next_action_date no avanza → mismo email mañana. Ref ausencia de `cr.commit()` en `account_followup/models/res_partner.py:644-652`, `ResPartner._cron_execute_followup_company`.

**AFG85C.** `_query_followup_data` se ejecuta dentro del cron CON el cache helper, pero `_execute_followup_partner` puede invalidar AMLs (al setear `aml.followup_line_id`). La caché queda **stale** durante la iteración. Partner 500 puede leer estado del partner 100 ya procesado. Mitigado por filtro inicial pero edge cases con dependencias cruzadas existen. Ref `account_followup/models/res_partner.py:641-643`, `ResPartner._cron_execute_followup_company`.

**AFG86C.** `nextcall` con hora hardcoded 02:00 server TZ. Empresa con servidor en UTC pero usuarios en America/Argentina → cron a las 23:00 hora local del día anterior. Si tu plantilla de email tiene timestamps formatted con user TZ → la fecha del email puede aparecer "ayer". Ref `account_followup/data/cron.xml:7`, `ir_cron_auto_post_draft_entry`.

---

## 9. account_followup ↔ SMS (`sms`)

### Modelos y campos

- `account_followup.followup.line.sms_template_id` (M2O `sms.template`, domain `[('model','=','res.partner')]`).
- `account_followup.followup.line.send_sms` (Boolean, default False).
- `sms.template.body` — texto plano (no HTML).
- Recipient lookup: `to_send_partner.mobile or to_send_partner.phone` (v18); `to_send_partner.phone` (v19).

### Pipeline

```
_send_sms():
  partner = browse(options['partner_id'])
  followup_contacts = partner._get_all_followup_contacts() or partner
  sent_at_least_once = False
  for to_send_partner in followup_contacts:
      sms_number = to_send_partner.mobile or to_send_partner.phone
      if sms_number:
          sms_body = with_context(lang=partner.lang or env.user.lang)._get_sms_body(options)
          partner._message_sms(
              body=sms_body,
              partner_ids=partner.ids,
              sms_pid_to_number={partner.id: sms_number}
          )
          sent_at_least_once = True
  if not sent_at_least_once:
      raise UserError("no follow-up contact has mobile/phone")
```

### Configuración

- `iap.account` para SMS configurado (Odoo IAP credits o gateway custom).
- `sms.template.body` rendereable con `{{ object.name }}` y demás expresiones Jinja del partner.

### Gotchas

**AFG87S.** Iteración `for to_send_partner in followup_contacts`: SMS se envía a TODOS los followup contacts con número, NO solo al primero. Si el partner tiene 3 child contacts type='followup' con número → 3 SMS = 3 IAP credits consumidos por followup. Coste invisible. Ref `account_followup/models/account_followup_report.py:373-381`, `AccountFollowupReport._send_sms`.

**AFG88S.** `_message_sms(partner_ids=partner.ids, sms_pid_to_number={partner.id: sms_number})`: el `partner_ids` siempre es el partner principal, no `to_send_partner`. El log del SMS en chatter aparece como enviado al "main partner" aunque el número físico sea del child contact. Auditoría incorrecta. Ref `account_followup/models/account_followup_report.py:377-381`, `AccountFollowupReport._send_sms`.

**AFG89S.** `UserError` si ningún contact tiene número Y line.send_sms=True. En cron, esto se captura y logea warning, pero el envío del EMAIL del mismo followup ya ocurrió antes (si send_email=True). Inconsistencia: email enviado, SMS falla, `next_action_date` actualizado igual → cliente recibió aviso parcial. Ref `account_followup/models/account_followup_report.py:383-384`, `AccountFollowupReport._send_sms`.

**AFG90S.** `_get_sms_body` fallback default body: `"Dear client, we kindly remind you that you still have unpaid invoices. Please check them and take appropriate action. %s" % signature`. Es **texto plano** con HTML-stripped signature vía `html2plaintext`. Signatures con tablas/listas se aplanan irreconociblemente. Ref `account_followup/models/account_followup_report.py:240-247`, `AccountFollowupReport._get_sms_body`.

**AFG91S.** SMS template lang resolution: `template.with_context(lang=partner.lang or env.user.lang).body`. Si la traducción del SMS template no existe → cae a master string (English) silencioso. Diferente del email donde se renderiza el report HTML completo en master lang. Ref `account_followup/models/account_followup_report.py:240`, `AccountFollowupReport._get_sms_body`.

**AFG92S.** `sms_pid_to_number` mapea SOLO `partner.id` → number. Si quieres enviar a un child contact distinto, el dict no soporta mapping multi-partner. Workaround: `partner._message_sms` no soporta lista de partners-distintos-con-número. Limitación de diseño. Ref `account_followup/models/account_followup_report.py:380`, `AccountFollowupReport._send_sms`.

**AFG93S.** En v19: `sms_number = to_send_partner.phone` (mobile field removido del modelo `res.partner` unificado en phone). Custom code v18 con lookups `partner.mobile` rompe en upgrade. Migration script obligatorio para copiar mobile → phone si phone vacío. Ref v19 `_send_sms`.

### v19 changes

- `mobile` field consolidado en `phone` a nivel `res.partner`.
- Validación de número con `phone_validation` library más estricta.

---

## Cross-refs activos

- `enterprise/account_followup/core.md` — modelos base, ciclo de vida completo, lifecycle SQL.
- `enterprise/account_followup/gotchas.md` — AFG1-34 gotchas del core.
- `enterprise/account_followup/v19_changes.md` — refactor v19 detallado.
- `functional/account/integrations.md` — bridge account.move ↔ account.payment ↔ AML reconciliation (origen de los AMLs evaluados).
- `functional/mail/integrations.md` — mail.template renderer + mail.composer.mixin (usado por wizard manual).
- `functional/account/gotchas.md` — overdue invoices, payment_state, reconciliation impacts.
- `functional/portal/integrations.md` — `/my/invoices/overdue` route + payment.transaction portal.
- `functional/crm/integrations.md` — escalation a sales team vía mail.activity.
- `enterprise/account_reports/integrations.md` — `account_reports.followup_report` action usado en `_get_followup_report`.

---

## Configuración mínima recomendada por caso de uso

### B2B clásico — 3 niveles auto + escalation manual

```yaml
account_followup.followup.line:
  - name: "Friendly Reminder"
    delay: 15
    send_email: True
    mail_template_id: payment_reminder_soft
    auto_execute: True
    join_invoices: True
  - name: "Second Notice"
    delay: 30
    send_email: True
    send_sms: True
    mail_template_id: payment_reminder_firm
    auto_execute: True
  - name: "Final Notice + Escalation"
    delay: 60
    send_email: True
    create_activity: True
    activity_default_responsible_type: salesperson
    activity_summary: "Escalate — call customer"
    auto_execute: False  # MANUAL desde aquí

res.partner (default):
  followup_reminder_type: automatic
```

### B2C con SMS pre-vencimiento

```yaml
account_followup.followup.line:
  - name: "Pre-due reminder"
    delay: -3  # 3 días ANTES del vencimiento
    send_sms: True
    sms_template_id: pre_due_sms
    auto_execute: True
  - name: "Overdue notice"
    delay: 7
    send_email: True
    send_sms: True
    auto_execute: True

res.partner:
  mobile: <required>
  email: <required>
```

### Multi-company branch — herencia desde matriz

```yaml
# Definir en company padre
account_followup.followup.line:
  - company_id: parent_company  # niveles globales
    name: "Global L1"
    delay: 15
  - company_id: parent_company
    name: "Global L2"
    delay: 30

# Branches hijos ven líneas vía parent_of search
# NO duplicar líneas en hijos salvo override específico
```
