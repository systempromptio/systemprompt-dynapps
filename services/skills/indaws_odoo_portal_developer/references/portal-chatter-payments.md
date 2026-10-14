# Portal Chatter y Pagos

## Tabla de Contenidos

- [Portal Chatter](#portal-chatter)
- [Integración de Pagos](#integración-de-pagos)
- [Firma Digital en Portal](#firma-digital-en-portal)
- [Modales Bootstrap en Portal](#modales-bootstrap-en-portal)

---

## Portal Chatter

El chatter portal permite a los clientes leer y escribir mensajes en registros compartidos.

### Activar chatter en una detail page

```xml
<!-- En el template de la detail page -->
<div id="my_thing_communication" class="mt-4">
    <h4><strong>History</strong></h4>
    <t t-call="portal.message_thread">
        <t t-set="object" t-value="thing"/>
        <!-- Opcionales: -->
        <t t-set="token" t-value="token"/>
        <t t-set="pid" t-value="pid"/>
        <t t-set="hash" t-value="hash"/>
        <t t-set="allow_composer" t-value="True"/>
        <t t-set="two_columns" t-value="False"/>
    </t>
</div>
```

Si usas `_get_page_view_values()` en el controller, `token`, `pid` y `hash` ya están en los valores.

### Variables del message_thread

| Variable | Tipo | Default | Propósito |
|----------|------|---------|-----------|
| `object` | recordset | requerido | Registro que tiene los mensajes |
| `token` | string | None | Access token para auth pública |
| `pid` | int | None | Partner ID para chatter auth |
| `hash` | string | None | Firma HMAC del token+pid |
| `allow_composer` | bool | True | Mostrar área de escritura |
| `two_columns` | bool | False | Layout 2 columnas para chatter |
| `message_per_page` | int | 10 | Mensajes por página |
| `disable_composer` | bool | False | Deshabilitar escritura |

### Arquitectura del chatter portal

```
Template QWeb (portal.message_thread)
    → <div id="discussion" data-res_model="..." data-res_id="..." .../>
        → PortalChatterService (JS service)
            → Shadow DOM (aislamiento CSS)
                → PortalChatter (OWL component)
                    → Chatter (reutiliza componente mail)
```

El chatter portal se monta en un **Shadow DOM** para aislar sus estilos del resto del portal.

### Controller endpoints del chatter

| Endpoint | Tipo | Auth | Dónde |
|----------|------|------|-------|
| `/portal/chatter_init` | json | public | `odoo18/addons/portal/controllers/mail.py:42`, `PortalChatter.portal_chatter_init` |
| `/mail/chatter_fetch` | json | public | `odoo18/addons/portal/controllers/mail.py:77`, `PortalChatter.portal_message_fetch` |
| `/mail/update_is_internal` | json | **user** | `odoo18/addons/portal/controllers/mail.py:116`, `PortalChatter.portal_message_update_is_internal` |
| `/mail/avatar/mail.message/<id>/author_avatar/<w>x<h>` | http | public | `odoo18/addons/portal/controllers/mail.py:25`, `PortalChatter.portal_avatar` |

Los tres `type='json'` son de v18. En v19 `json` sigue valiendo como alias deprecado de
`jsonrpc` (`odoo19/odoo/http.py:815`, `route.decorator`), así que un módulo v18 arranca en v19 con warning.

### Mensajes visibles en portal

Solo aparecen mensajes del campo `website_message_ids`, y el filtro está en su propio dominio
(`odoo18/addons/portal/models/mail_thread.py:15-18`, `MailThread.website_message_ids`):

```python
domain=lambda self: [('model', '=', self._name),
                     ('message_type', 'in', ('comment', 'email', 'email_outgoing', 'auto_comment'))]
```

O sea que `notification`, `user_notification` y `sms` quedan fuera por construcción, no por una
regla de acceso.

Los mensajes internos (`is_internal=True`) no se muestran a portal users (salvo empleados).

### Posting desde portal

El controller `thread.py` maneja los posts:

```python
# Portal user publica un mensaje
# 1. Se extrae el partner del hash/pid/token
portal_partner = get_portal_partner(thread, hash, pid, token)
# 2. Se setea como author_id
post_data['author_id'] = portal_partner.id
# 3. El mensaje se crea con ese autor
```

El partner puede editar sus propios mensajes si `_is_editable_in_portal()` devuelve True
(`odoo18/addons/portal/models/mail_message.py:161`, `MailMessage._is_editable_in_portal`; lo consultan `portal/controllers/thread.py:20`, `ThreadController._is_message_editable` y
`portal/controllers/attachment.py:8`, `PortalAttachmentController._is_allowed_to_delete`).

---

## Integración de Pagos

Para añadir pagos en portal, tu controller debe heredar de `PaymentPortal` en vez de `CustomerPortal`.

### Cadena de herencia

```python
from odoo.addons.payment.controllers.portal import PaymentPortal

class MyPortal(PaymentPortal):
    # Heredas todo de CustomerPortal + métodos de pago
    pass
```

### Patrón de pago en detail page

```python
@http.route(['/my/things/<int:thing_id>'], auth='public', website=True)
def portal_thing(self, thing_id, access_token=None, **kwargs):
    thing_sudo = self._document_check_access('my.model', thing_id, access_token)

    values = {
        'thing': thing_sudo,
        'page_name': 'my_thing',
    }

    # Si tiene que pagar → añadir datos de pago
    if thing_sudo.amount_residual > 0:
        values.update(self._get_payment_values(thing_sudo))

    values = self._get_page_view_values(thing_sudo, access_token, values,
                                         'my_things_history', False, **kwargs)
    return request.render('my_module.portal_thing_page', values)

def _get_payment_values(self, thing_sudo):
    partner_sudo = (request.env.user.partner_id
                    if not request.env.user._is_public()
                    else thing_sudo.partner_id)

    providers_sudo = request.env['payment.provider'].sudo()._get_compatible_providers(
        thing_sudo.company_id.id,
        partner_sudo.id,
        thing_sudo.amount_residual,
        currency_id=thing_sudo.currency_id.id,
    )

    payment_methods_sudo = request.env['payment.method'].sudo()._get_compatible_payment_methods(
        providers_sudo.ids, partner_sudo.id,
        currency_id=thing_sudo.currency_id.id,
    )

    tokens_sudo = request.env['payment.token'].sudo()._get_available_tokens(
        providers_sudo.ids, partner_sudo.id,
    )

    return {
        'amount': thing_sudo.amount_residual,
        'currency': thing_sudo.currency_id,
        'partner_id': partner_sudo.id,
        'providers_sudo': providers_sudo,
        'payment_methods_sudo': payment_methods_sudo,
        'tokens_sudo': tokens_sudo,
        'transaction_route': thing_sudo.get_portal_url(suffix='/transaction'),
        'landing_route': thing_sudo.get_portal_url(),
        'access_token': thing_sudo._portal_ensure_token(),
    }
```

### Ruta de transacción

```python
@http.route('/my/things/<int:thing_id>/transaction', type='json', auth='public')   # v18; en v19 usa type='jsonrpc' (json = alias deprecado)
def portal_thing_transaction(self, thing_id, access_token, **kwargs):
    thing_sudo = self._document_check_access('my.model', thing_id, access_token)
    partner_sudo = (request.env.user.partner_id
                    if not request.env.user._is_public()
                    else thing_sudo.partner_id)

    kwargs.update({
        'partner_id': partner_sudo.id,
        'currency_id': thing_sudo.currency_id.id,
    })

    tx_sudo = self._create_transaction(
        custom_create_values={'my_model_ids': [Command.set([thing_id])]},
        **kwargs,
    )
    return tx_sudo._get_processing_values()
```

### Template con formulario de pago

```xml
<!-- En el template de la detail page -->
<div t-if="thing.amount_residual > 0" id="payment_section" class="mt-4">
    <h4>Pay Now</h4>

    <t t-if="company_mismatch">
        <t t-call="payment.company_mismatch_warning"/>
    </t>
    <div t-else="" id="payment_method">
        <t t-call="payment.form">
            <t t-set="show_tokenize_input" t-value="True"/>
        </t>
    </div>
</div>

<!-- O dentro de un modal -->
<div class="modal fade" id="modalPay" tabindex="-1">
    <div class="modal-dialog modal-lg">
        <div class="modal-content">
            <div class="modal-header">
                <h5 class="modal-title">Payment</h5>
                <button class="btn-close" data-bs-dismiss="modal"/>
            </div>
            <div class="modal-body">
                <t t-call="payment.form"/>
            </div>
        </div>
    </div>
</div>
```

### Flujo completo de pago

```
1. Portal page muestra botón "Pay Now" o formulario payment.form
2. Cliente selecciona método de pago (Stripe, PayPal, etc.)
3. JS frontend llama a /my/things/<id>/transaction (JSON RPC)
4. Server crea payment.transaction draft vinculada al registro
5. JS recibe processing_values → redirige a pasarela
6. Pasarela procesa → webhook actualiza transaction
7. Transaction confirma → callback actualiza registro
8. Redirect a landing_route (portal page) con mensaje de éxito
```

---

## Firma Digital en Portal

Enterprise incluye el módulo `sign` para firmas en portal.

### Patrón de firma (simplificado)

```xml
<!-- Botón de firma en sidebar -->
<a class="btn btn-primary" data-bs-toggle="modal" data-bs-target="#modalSign">
    <i class="fa fa-check me-1"/> Sign
</a>

<!-- Modal de firma -->
<div class="modal fade" id="modalSign">
    <div class="modal-dialog">
        <div class="modal-content">
            <div class="modal-body">
                <t t-call="portal.portal_signature">
                    <t t-set="object" t-value="thing"/>
                    <t t-set="name" t-value="thing.partner_id.name"/>
                    <t t-set="callUrl" t-value="thing.get_portal_url(suffix='/accept')"/>
                    <t t-set="accessToken" t-value="thing.access_token"/>
                </t>
            </div>
        </div>
    </div>
</div>
```

El componente `portal.signature_form` (OWL) captura la firma del cliente.

### Controller para aceptar firma

```python
@http.route('/my/things/<int:thing_id>/accept', type='json', auth='public')   # v18; en v19 usa type='jsonrpc' (json = alias deprecado)
def portal_thing_accept(self, thing_id, access_token=None, name=None, signature=None):
    thing_sudo = self._document_check_access('my.model', thing_id, access_token)

    if not signature:
        return {'error': _('Signature is missing.')}

    thing_sudo.write({
        'signed_by': name,
        'signed_on': fields.Datetime.now(),
        'signature': signature,  # base64 PNG
    })

    # Post message con la firma
    thing_sudo.message_post(
        body=_('Document signed by %s', name),
        message_type='comment',
        subtype_xmlid='mail.mt_note',
    )

    # Generar PDF firmado si necesario
    report = request.env['ir.actions.report']._render_qweb_pdf(
        'my_module.action_report_thing', thing_sudo.ids
    )

    return {
        'force_refresh': True,
        'redirect_url': thing_sudo.get_portal_url(),
    }
```

---

## Modales Bootstrap en Portal

Los módulos Enterprise usan modales para acciones inline en detail pages.

### Patrón modal completo

```xml
<!-- Trigger button -->
<a class="btn btn-outline-primary"
   data-bs-toggle="modal"
   data-bs-target="#modalAction"
   role="button">
    <i class="fa fa-edit me-1"/> Take Action
</a>

<!-- Modal -->
<div class="modal fade" id="modalAction" tabindex="-1" aria-hidden="true">
    <div class="modal-dialog">
        <form class="modal-content" method="post"
              t-attf-action="/my/things/#{thing.id}/action?access_token=#{thing.access_token}">
            <input type="hidden" name="csrf_token" t-att-value="request.csrf_token()"/>
            <div class="modal-header">
                <h5 class="modal-title">Confirm Action</h5>
                <button type="button" class="btn-close" data-bs-dismiss="modal"
                        aria-label="Close"/>
            </div>
            <div class="modal-body">
                <!-- Form fields -->
                <div class="mb-3">
                    <label class="form-label" for="reason">Reason</label>
                    <select class="form-select" name="reason_id" id="reason" required="required">
                        <option value="">Select a reason...</option>
                        <t t-foreach="reasons" t-as="reason">
                            <option t-att-value="reason.id" t-out="reason.name"/>
                        </t>
                    </select>
                </div>
                <div class="mb-3">
                    <label class="form-label" for="comment">Comment</label>
                    <textarea class="form-control" name="comment" id="comment" rows="3"/>
                </div>
            </div>
            <div class="modal-footer">
                <button type="button" class="btn btn-secondary" data-bs-dismiss="modal">
                    Cancel
                </button>
                <button type="submit" class="btn btn-primary">
                    Confirm
                </button>
            </div>
        </form>
    </div>
</div>
```

### Controller para el modal

```python
@http.route('/my/things/<int:thing_id>/action', type='http', auth='public',
            methods=['POST'], website=True)
def portal_thing_action(self, thing_id, access_token=None, reason_id=None,
                        comment=None, **kwargs):
    thing_sudo = self._document_check_access('my.model', thing_id, access_token)

    thing_sudo.write({
        'state': 'actioned',
        'reason_id': int(reason_id) if reason_id else False,
    })

    if comment:
        thing_sudo.message_post(
            body=comment,
            message_type='comment',
        )

    return request.redirect(thing_sudo.get_portal_url(query_string='&message=action_ok'))
```

### Modales responsive

Los modales de Bootstrap 5 ya son responsive por defecto:
- En desktop: modal centrado con backdrop
- En mobile: modal ocupa casi todo el ancho (con padding)

Para modales con mucho contenido, usar `modal-dialog-scrollable`:

```xml
<div class="modal-dialog modal-dialog-scrollable modal-lg">
```

Para modales que necesitan ser fullscreen en mobile:

```xml
<div class="modal-dialog modal-fullscreen-md-down">
```
