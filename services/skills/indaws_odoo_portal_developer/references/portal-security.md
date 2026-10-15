# Portal Security — Mixins, Tokens, Record Rules

## Tabla de Contenidos

- [portal.mixin](#portalmixin)
- [Access Tokens](#access-tokens)
- [Signed Tokens (Hash + PID)](#signed-tokens-hash--pid)
- [Record Rules para Portal](#record-rules-para-portal)
- [Portal Users y Grupos](#portal-users-y-grupos)
- [Portal Sharing](#portal-sharing)
- [Checklist de Seguridad](#checklist-de-seguridad)

---

## portal.mixin

**Archivo**: `odoo18/addons/portal/models/portal_mixin.py` (v18)

Modelo abstracto que da a cualquier modelo las capacidades de portal.

### Campos

| Campo | Tipo | Propósito |
|-------|------|-----------|
| `access_url` | Char (computed) | URL del portal (ej: `/my/orders/123`). `odoo18/addons/portal/models/portal_mixin.py:13`, `PortalMixin`; la base devuelve `'#'` (`:25-27`), así que **hay que sobreescribir `_compute_access_url`** |
| `access_token` | Char (copy=False) | Token UUID para acceso sin login. `odoo18/addons/portal/models/portal_mixin.py:16`, `PortalMixin.access_token` |
| `access_warning` | Text (computed) | Mensaje de warning para el usuario portal. `odoo18/addons/portal/models/portal_mixin.py:19`, `PortalMixin.access_warning`, vacío por defecto (`:23`) |

### Implementación mínima

```python
class MyModel(models.Model):
    _name = 'my.model'
    _inherit = ['portal.mixin', 'mail.thread']

    partner_id = fields.Many2one('res.partner', required=True)

    def _compute_access_url(self):
        for record in self:
            record.access_url = f'/my/things/{record.id}'
```

### Métodos clave

**`_portal_ensure_token()`** — `odoo18/addons/portal/models/portal_mixin.py:29-34`, `PortalMixin._portal_ensure_token`
- Genera UUID4 si no existe
- `self.sudo().write({'access_token': str(uuid.uuid4())})` (`:33`). El `write` no es por permisos: el comentario del propio código dice que es **para vaciar la caché**, o si no el `return self.access_token` de la línea siguiente devolvería `False`
- Devuelve el token

**`get_portal_url(suffix, report_type, download, query_string, anchor)`** — `odoo18/addons/portal/models/portal_mixin.py:115`, `PortalMixin.get_portal_url`
- Construye URL completa: `access_url + suffix + ?access_token=xxx&report_type=pdf...`
- Usado en templates: `record.get_portal_url(report_type='pdf', download=True)`

**`_get_share_url(redirect, signup_partner, pid, share_token)`** — `odoo18/addons/portal/models/portal_mixin.py:36`, `PortalMixin._get_share_url`
- URL para compartir con partners externos
- Incluye `access_token`, `pid`, `hash` (firma HMAC)
- Si `redirect=True`: devuelve `/mail/view?...` (redirect endpoint)

**`_get_access_action(access_uid, force_website)`** — `odoo18/addons/portal/models/portal_mixin.py:68`, `PortalMixin._get_access_action`
- Decide qué acción ejecutar al acceder al registro
- Si usuario portal (`share=True`): redirige a URL portal
- Si usuario interno: abre formulario backend

---

## Access Tokens

### Generación

```python
# Token: UUID4 (128 bits de entropía)
import uuid
token = str(uuid.uuid4())  # ej: "a1b2c3d4-e5f6-7890-abcd-ef1234567890"
```

- Se genera on-demand (no al crear el registro)
- `copy=False`: no se copia al duplicar un registro
- Se almacena en BD como `Char`

### Validación en Controller

<sub>Verificado 2026-08-14 contra `odoo18/addons/portal/controllers/portal.py:100-119`, `get_records_pager`: el pseudocódigo de abajo coincide línea por línea, incluido `check_access('read')` — que es la firma de v18; en v17 eran `check_access_rights` + `check_access_rule`.</sub>

```python
def _document_check_access(self, model_name, document_id, access_token=None):
    document = request.env[model_name].browse([document_id])
    document_sudo = document.with_user(SUPERUSER_ID).exists()
    if not document_sudo:
        raise MissingError("This document does not exist.")
    try:
        document.check_access('read')  # Intenta acceso normal
    except AccessError:
        if not access_token or not document_sudo.access_token or \
           not consteq(document_sudo.access_token, access_token):
            raise  # Token inválido → AccessError
    return document_sudo  # Token válido → devuelve sudoed
```

**Puntos críticos**:
- `consteq()`: comparación constant-time (previene timing attacks)
- Devuelve `document_sudo` (SUPERUSER) — todo el rendering se hace con permisos elevados
- Si el token es válido, se bypasean TODAS las reglas de acceso

### Ejecutado, no leído (v18 y v19) — `tools/bench-orm/addons/dyn_portal`

Un documento del cliente A, una record rule que limita a cada usuario de portal a **su** partner, y
cuatro maneras de pedirlo:

| Quién pide | Con token | Qué recibe |
|---|---|---|
| usuario de portal **B** (la regla lo excluye) | no | `AccessError` |
| anónimo, sin identificar | no | `AccessError` |
| usuario de portal **B** | **sí** | **el documento** |
| anónimo | **sí** | **el documento** |
| cualquiera | token equivocado | `AccessError` |

Y lo que vuelve viene con `env.su = True` (medido). Tres consecuencias que conviene tener presentes
al diseñar una página de portal:

1. **El enlace ES la credencial.** No hay que estar identificado, ni ser el cliente del documento:
   quien tenga la URL con el token entra. Reenviar un correo es compartir el acceso.
2. **A partir de `_document_check_access` no queda ninguna barrera del ORM.** Lo que el controlador
   decida pintar con ese recordset —campos, líneas, adjuntos, registros relacionados— se pinta.
   Filtrar es responsabilidad del controlador; el ORM ya no va a decir que no.
3. **El token no existe hasta que alguien lo pide.** `portal.mixin` no lo genera al crear: lo genera
   `_portal_ensure_token()` (medido: `access_token` está vacío en un registro recién creado). Un
   enlace construido a mano antes de esa llamada no vale.


### En rutas

```python
# Detail page: auth="public" permite acceso sin login
@http.route(['/my/things/<int:thing_id>'], auth='public', website=True)
def portal_thing(self, thing_id, access_token=None, **kwargs):
    try:
        thing_sudo = self._document_check_access('my.model', thing_id, access_token)
    except (AccessError, MissingError):
        return request.redirect('/my')
    # ... render con thing_sudo
```

**`auth='public'`** es obligatorio para detail pages que soporten tokens.
`auth='user'` requiere login — no funciona con tokens.

---

## Signed Tokens (Hash + PID)

Mecanismo adicional para autenticar partners en el chatter sin necesidad de login.

### Cómo funciona

El código real (`odoo18/addons/portal/models/mail_thread.py:65-85`, `MailThread._sign_token`), no un parecido:

```python
def _sign_token(self, pid):
    self.ensure_one()
    if self._mail_post_token_field not in self._fields:            # :76
        raise NotImplementedError(_(
            "Model %(model_name)s does not support token signature, "
            "as it does not have %(field_name)s field."))
    secret = self.env["ir.config_parameter"].sudo().get_param("database.secret")   # :83
    token = (self.env.cr.dbname, self[self._mail_post_token_field], pid)           # :84
    return hmac.new(secret.encode('utf-8'), repr(token).encode('utf-8'),
                    hashlib.sha256).hexdigest()                                    # :85
```

Tres cosas que no se adivinan:

- Es **`hmac.new` a pelo**, no el helper `odoo.tools.hmac(env, scope, message)`. No hay string
  de scope: lo que se firma es el `repr()` de la tupla.
- `_mail_post_token_field` vale `'access_token'` por defecto y **se sobreescribe por modelo**
  (`odoo18/addons/portal/models/mail_thread.py:13`, `MailThread._mail_post_token_field`).
- Si el modelo no tiene ese campo, **lanza `NotImplementedError`**, no devuelve `False`
  (`odoo18/addons/portal/models/mail_thread.py:76-81`, `MailThread._sign_token`). Un modelo con `portal.mixin` lo tiene; uno sin él, no.

**Uso**: Cuando se comparte un documento por email, el link incluye:
- `access_token`: token del registro
- `pid`: ID del partner destinatario
- `hash`: firma HMAC que prueba que el link es para ese partner

### Validación

```python
# odoo18/addons/portal/utils.py:6-17
def validate_thread_with_hash_pid(thread, _hash, pid):
    if not _hash or not pid:
        return False
    pid = int(pid)
    if consteq(_hash, thread._sign_token(pid)):
        return True
    parent_sign_token = thread._portal_get_parent_hash_token(pid)      # :11
    return parent_sign_token and consteq(_hash, parent_sign_token)

def validate_thread_with_token(thread, token):
    return token and consteq(token, thread[thread._mail_post_token_field])
```

**El segundo camino importa y es fácil pasarlo por alto**: si el hash no casa con el del propio
registro, se prueba con el del **padre lógico**. `_portal_get_parent_hash_token` devuelve `False`
en la base (`odoo18/addons/portal/models/mail_thread.py:87-93`, `MailThread._portal_get_parent_hash_token`) y lo sobreescriben los modelos que se pueden compartir tanto sueltos
como a través de un M2o padre. O sea: **un hash firmado para el padre autoriza en el hijo**. Si
razonas sobre quién puede comentar un documento, ese camino cuenta.

### Cuándo usar cada uno

| Método | Cuándo | Ejemplo |
|--------|--------|---------|
| `access_token` solo | Acceso directo al documento | `/my/orders/123?access_token=xxx` |
| `hash` + `pid` | Chatter: identificar quién comenta | Responder a email de notificación |
| Ambos | Compartir link completo | Email de invitación |

---

## Record Rules para Portal

Cada modelo que se muestra en portal DEBE tener record rules para `base.group_portal`.

### Patrón estándar

```xml
<!-- security/ir_rules.xml -->
<record id="my_model_rule_portal" model="ir.rule">
    <field name="name">My Model: portal users see own</field>
    <field name="model_id" ref="model_my_model"/>
    <field name="domain_force">
        [('partner_id', 'child_of', [user.commercial_partner_id.id])]
    </field>
    <field name="groups" eval="[(4, ref('base.group_portal'))]"/>
    <field name="perm_read" eval="True"/>
    <field name="perm_write" eval="False"/>
    <field name="perm_create" eval="False"/>
    <field name="perm_unlink" eval="False"/>
</record>
```

### Variantes de domain

| Patrón | Domain | Uso |
|--------|--------|-----|
| Por `partner_id` | `[('partner_id', 'child_of', [user.commercial_partner_id.id])]` | Registros del cliente |
| Por `message_partner_ids` | `[('message_partner_ids', 'child_of', [user.commercial_partner_id.id])]` | Registros donde está suscrito (sale.order) |
| Por `partner_id` OR follower | `['|', ('partner_id', ...)]` + `[('message_partner_ids', ...)]` | Combinado |

### ACLs necesarias

```csv
# security/ir.model.access.csv
id,name,model_id:id,group_id:id,perm_read,perm_write,perm_create,perm_unlink
access_my_model_portal,my.model.portal,model_my_model,base.group_portal,1,0,0,0
```

**ACLs dan el permiso base** (read). **Record rules filtran** qué registros.

### Errores comunes

| Error | Causa | Fix |
|-------|-------|-----|
| Portal user ve todos los registros | Falta record rule para `group_portal` | Añadir rule con domain por partner |
| Portal user no ve nada | Falta ACL para `group_portal` | Añadir línea en ir.model.access.csv |
| "Access Denied" con token válido | El controller no usa `auth="public"` | Cambiar a `auth="public"` |
| Token funciona pero chatter no | Falta `_mail_post_token_field` en modelo | Heredar de `portal.mixin` |

---

## Portal Users y Grupos

### Grupos de seguridad

| Grupo | XML ID | Propósito |
|-------|--------|-----------|
| Portal | `base.group_portal` | Usuarios portal (clientes, proveedores) |
| Public | `base.group_public` | Usuarios anónimos |
| Internal | `base.group_user` | Usuarios backend |

Estos grupos son **mutuamente excluyentes**. Un usuario es portal O interno, nunca ambos.

### Detección del tipo de usuario

```python
# En controller
user = request.env.user
user._is_internal()  # → True si grupo interno
user._is_portal()    # → True si grupo portal
user._is_public()    # → True si grupo público

# Campo computado
user.share  # → True si portal/público, False si interno
```

### Creación de portal users

El wizard `portal.wizard` crea usuarios portal:

1. Seleccionar partners → "Grant Portal Access"
2. Crea `res.users` con email como login
3. Añade al grupo `base.group_portal`
4. Envía email de invitación con link de signup

### Redirect de portal users

Portal users son redirigidos automáticamente:
- `/` → `/my`
- `/web` → `/my`
- Login → `/my`

---

## Portal Sharing

**Modelo**: `portal.share` (TransientModel)

### Flujo de compartir

```python
# 1. Usuario backend abre wizard de compartir
record.action_share()  # → abre portal.share wizard

# 2. Wizard genera URL con token
share_url = record._get_share_url(
    redirect=True,
    pid=partner.id,
    share_token=True,
)
# → /mail/view?model=my.model&res_id=123&access_token=xxx&pid=456&hash=hmac

# 3. Envía email con el link
# Si partner tiene usuario portal → link directo
# Si no tiene usuario → link con signup
```

### Dos tracks de sharing

| Track | Condición | URL |
|-------|-----------|-----|
| Public link | Registro tiene `access_token` | `/mail/view?...&access_token=xxx` |
| Signup link | Partner sin usuario portal | `/mail/view?...&redirect=/auth/signup?token=yyy` |

Tras compartir, el partner se suscribe al registro (`message_subscribe`).

---

## Errores de seguridad comunes (real-world)

Errores encontrados en auditorías reales de módulos portal en producción:

### ACL demasiado permisivo

```csv
# ❌ MAL: portal puede escribir y crear registros
access_picking_portal,...,base.group_portal,1,1,1,0

# ✅ BIEN: solo lectura (write solo si hay formulario de edición en portal)
access_picking_portal,...,base.group_portal,1,0,0,0
```

Mail.message necesita create (para el chatter) pero no write:
```csv
access_mail_message_portal,...,base.group_portal,1,0,1,0
```

### Record rules faltantes — el error más peligroso

El controller filtra por partner en Python, pero eso es **seguridad cosmética**. Cualquier llamada RPC directa al ORM bypasea el controller:

```
# Sin record rule, un usuario técnico puede hacer:
RPC: search_read('stock.picking', [])  → VE TODOS los pickings 😱
```

El controller y la record rule deben coincidir en el filtro, pero la **record rule es la barrera real**:

```xml
<!-- SIEMPRE crear record rule, aunque el controller ya filtre -->
<record id="my_model_rule_portal" model="ir.rule">
    <field name="name">Portal: own records only</field>
    <field name="model_id" ref="model_my_model"/>
    <field name="groups" eval="[(4, ref('base.group_portal'))]"/>
    <field name="domain_force">
        [('partner_id', 'child_of', user.partner_id.id)]
    </field>
    <field name="perm_read" eval="True"/>
    <field name="perm_write" eval="False"/>
    <field name="perm_create" eval="False"/>
    <field name="perm_unlink" eval="False"/>
</record>
```

### No olvidar modelos relacionados

Si el portal muestra `stock.picking` con sus `stock.move` lines, ambos modelos necesitan ACL + record rule:

```csv
access_picking_portal,...,stock.model_stock_picking,base.group_portal,1,0,0,0
access_move_portal,...,stock.model_stock_move,base.group_portal,1,0,0,0
```
```xml
<!-- stock.move rule via picking -->
<record id="portal_stock_move_rule" model="ir.rule">
    <field name="model_id" ref="stock.model_stock_move"/>
    <field name="groups" eval="[(4, ref('base.group_portal'))]"/>
    <field name="domain_force">
        [('picking_id.partner_id', 'child_of', user.partner_id.id)]
    </field>
</record>
```

### sudo() en controller — cuándo sí y cuándo no

```python
# ❌ MAL: sudo() porque no hay record rules
pickings = request.env['stock.picking'].sudo().search(domain)

# ✅ BIEN: record rules configuradas → ORM filtra automáticamente
pickings = request.env['stock.picking'].search(domain)

# ✅ OK: sudo() en detail page DESPUÉS de _document_check_access
picking_sudo = self._document_check_access('stock.picking', id, token)
# picking_sudo ya es sudoed — el acceso fue validado por token/permisos
```

---

## Checklist de Seguridad

Al crear una extensión de portal, verificar:

- [ ] **portal.mixin**: modelo hereda de `portal.mixin`
- [ ] **`_compute_access_url()`**: implementado, devuelve URL correcta
- [ ] **ACL**: `ir.model.access.csv` con read para `base.group_portal`
- [ ] **Record rule**: filtrada por `partner_id` o `message_partner_ids` + `commercial_partner_id`
- [ ] **Record rule permisos**: solo `perm_read=True` (write/create/unlink=False salvo necesidad)
- [ ] **Controller auth**: `auth="public"` en detail pages (para tokens), `auth="user"` en list pages
- [ ] **Token validation**: `_document_check_access()` en cada detail route
- [ ] **Domain en controller**: `_prepare_*_domain(partner)` filtra por partner
- [ ] **has_access check**: `Model.has_access('read')` antes de `search_count()` en counters
- [ ] **No sudo() innecesario**: Solo usar sudo donde el acceso ya se validó con token
- [ ] **Session history limitada**: `ids[:100]` máximo
- [ ] **consteq()** para comparaciones de tokens
- [ ] **Campos sensibles**: No exponer campos que el portal user no debería ver
- [ ] **Write desde portal**: Si permites escritura, validar cada campo individualmente
