# Secure Coding — Gates BLOQUEANTES (1-7)

Apply these patterns WHILE generating Odoo code. Code MUST pass all 7 blocking gates before delivery.

Consumer skills: `odoo-module-creator`, `odoo-server-action-creator`, `odoo-carrier-creator`, `odoo-code-auditor`, `odoo-test-generator`, `quality-gate`.

| Nivel | Efecto | Significado |
|-------|--------|-------------|
| **BLOQUEANTE** | No entregar codigo | Vulnerabilidad activa o incumplimiento directo ISO |

---

## GATE 1: Inyeccion y Sanitizacion [BLOQUEANTE]
**ISO A.8.28 — Codificacion segura**

### SQL Injection

```python
# FALLA — string formatting en SQL
cr.execute("SELECT id FROM res_partner WHERE name = '%s'" % name)
cr.execute(f"SELECT id FROM res_partner WHERE vat = '{vat}'")
cr.execute("DELETE FROM stock_move WHERE id IN (%s)" % ','.join(ids))

# PASA — parametros seguros
cr.execute("SELECT id FROM res_partner WHERE name = %s", [name])
cr.execute("SELECT id FROM res_partner WHERE vat = %s", (vat,))
cr.execute("DELETE FROM stock_move WHERE id IN %s", [tuple(ids)])
```

**Regla**: CERO sentencias SQL con `%`, `f""`, `.format()` para valores. Solo `%s` con lista de parametros.

### eval / exec

```python
# FALLA
result = eval(user_expression)
exec(code_from_field)

# PASA
from odoo.tools.safe_eval import safe_eval
result = safe_eval(user_expression, eval_context)
```

**Regla**: `eval()` y `exec()` PROHIBIDOS. Solo `safe_eval()` con contexto controlado.

### XSS (Cross-Site Scripting)

```python
# FALLA
body = '<p>%s</p>' % user_input

# PASA
from odoo.tools import html_sanitize
body = html_sanitize('<p>%s</p>' % user_input)

# En QWeb: t-out/t-field, nunca t-raw con datos de usuario
```

### Markup seguro

```python
# FALLA — concatenacion sin escape
body = Markup('<p>') + user_input + Markup('</p>')

# PASA — %s formatting (escape automatico) o escape() explicito
from markupsafe import Markup, escape
body = Markup('<p>%s</p>') % user_input
body = Markup('<p>') + escape(user_input) + Markup('</p>')
```

---

## GATE 2: Control de Acceso [BLOQUEANTE]
**ISO A.8.3 — Restriccion de acceso a la informacion**

### ACLs obligatorias

```csv
id,name,model_id:id,group_id:id,perm_read,perm_write,perm_create,perm_unlink
access_mi_modelo_user,mi.modelo.user,model_mi_modelo,base.group_user,1,0,0,0
access_mi_modelo_manager,mi.modelo.manager,model_mi_modelo,mi_modulo.group_manager,1,1,1,0
access_mi_modelo_admin,mi.modelo.admin,model_mi_modelo,base.group_system,1,1,1,1
```

**Regla**: Todo modelo nuevo DEBE tener `ir.model.access.csv`. NUNCA `perm_unlink=1` a usuarios basicos.

### Record Rules multi-company

```python
partner_id = fields.Many2one('res.partner', check_company=True)
company_id = fields.Many2one('res.company', required=True, default=lambda s: s.env.company)
```

```xml
<record id="rule_mi_modelo_company" model="ir.rule">
    <field name="name">Mi Modelo: multi-company</field>
    <field name="model_id" ref="model_mi_modelo"/>
    <field name="domain_force">['|',('company_id','=',False),('company_id','in',company_ids)]</field>
</record>
```

### Campos sensibles con groups

```python
salary = fields.Float(groups='hr.group_hr_manager')
api_key = fields.Char(groups='base.group_system')
```

### groups= en vistas heredadas

```xml
<!-- FALLA — todos ven el boton → AccessError si no estan en el grupo -->
<button name="action_view_rappels" type="object" class="oe_stat_button"/>

<!-- PASA — solo usuarios del grupo -->
<button name="action_view_rappels" type="object" class="oe_stat_button"
        groups="my_module.group_rappel_user"/>
```

### sudo() controlado

```python
# Solo donde sea estrictamente necesario, scope minimo
param = self.env['ir.config_parameter'].sudo().get_param('key')
# Preferir compute_sudo=True sobre sudo() en compute methods
total = fields.Float(compute='_compute_total', compute_sudo=True)
```

---

## GATE 3: Exposicion RPC [BLOQUEANTE]
**ISO A.8.26 — Requisitos de seguridad de aplicaciones**

```python
# FALLA — metodo sensible accesible via RPC
def process_payment(self, amount): ...

# PASA — prefijo _ o @api.private (v18.2+)
def _process_payment(self, amount): ...

@api.private  # Odoo 18.2+
def approve_document(self): ...
```

### Controladores seguros

- `auth='none'` PROHIBIDO excepto webhooks con HMAC/token
- `csrf=False` solo para APIs JSON
- `methods=['POST']` para operaciones que modifican
- NUNCA `sudo()` en controladores sin verificar permisos

---

## GATE 4: Datos Sensibles [BLOQUEANTE]
**ISO A.5.12 / A.5.34 (RGPD)**

- NUNCA loguear passwords, API keys, tokens, datos bancarios
- Secrets en `ir.config_parameter` o campos con `groups='base.group_system'`
- NUNCA hardcodear secrets en Python/XML
- Datos personales con `groups=` restrictivo y `mail.thread` para auditoria

---

## GATE 5: Dependencias y Configuracion [BLOQUEANTE]
**ISO A.8.8 / A.8.9**

- Dependencias externas con version minima, sin CVE activas
- Config variable en `ir.config_parameter` o `res.config.settings`
- Zero `print()` en produccion (usar `_logger`)
- Zero endpoints debug sin `auth='user'` + `groups='base.group_system'`

---

## GATE 6: Auditabilidad [BLOQUEANTE]
**ISO A.8.15**

- Modelos con estados → `_inherit = ['mail.thread']` + `tracking=True`
- Acciones criticas con `message_post()`
- NUNCA borrar `mail.message`, `ir.logging` desde codigo

---

## GATE 7: Server Actions [BLOQUEANTE]
**ISO A.8.28**

- No `cr.commit()`, no `cr.rollback()`
- No SQL destructivo (`DROP TABLE`, borrado masivo)
- `sudo()` solo justificado
- Archivar (`active=False`) en vez de borrar
