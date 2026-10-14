# Patrones de Seguridad (v17-v19)

Recetas probadas. Combinan ACL + record rules + grupos. Verifica diffs de `res.groups`
(`category_id`→`privilege_id` en v19) con `odoo-version-api-reference` antes de copiar XML.

## 1. Multicompañía

Aísla registros por compañía. Record rule **GLOBAL** (se interseca con todo).

```xml
<record id="rule_xx_multicompany" model="ir.rule">
    <field name="name">XX: multi-company</field>
    <field name="model_id" ref="model_xx_object"/>
    <field name="global" eval="True"/>
    <field name="domain_force">['|',('company_id','=',False),('company_id','in',company_ids)]</field>
</record>
```
- `company_ids` = compañías permitidas del usuario (las del selector multicompañía).
- El `'|',('company_id','=',False)` deja ver registros sin compañía (compartidos).
- El modelo debe tener `company_id = fields.Many2one('res.company', ...)`.
- Es GLOBAL a propósito. **Pero lo que la hace global NO es ese campo: es no tener grupos.**

> ### El `<field name="global">` no manda — MEDIDO (v18 y v19)
>
> `global` es un campo **calculado y almacenado**, sin `inverse`: `_compute_global` hace
> literalmente `rule['global'] = not rule.groups` (v18 `addons/base/models/ir_rule.py:53-55`, `IrRule._compute_global`,
> v19 `:54-56`; se declara por `setattr` al final del fichero porque `global` es palabra reservada
> de Python). Escribirlo en el XML **se acepta y el compute lo pisa**.
>
> Medido (`tools/bench-orm::TestReglaGlobal`):
>
> | Regla | `global` acaba en | Qué ve un usuario ajeno al grupo |
> |---|---|---|
> | sin `groups` | `True` | solo su compañía ✔ |
> | con `groups` **y** `global eval="True"` | **`False`** | **las dos compañías** ✘ |
>
> O sea: el día que alguien le añada un grupo a esta regla —para «afinarla», o heredando el
> registro— **el aislamiento multicompañía se apaga para todos los que no estén en ese grupo**, el
> XML sigue diciendo `global True`, y no hay error ni aviso en ninguna parte. La comprobación de
> treinta segundos es leer el campo, no el XML:
> `env['ir.rule'].search([('model_id.model','=','mi.modelo')]).mapped(lambda r: (r.name, r['global']))`.
>
> El `eval="True"` del ejemplo es inofensivo mientras no haya grupos, pero **induce a creer que el
> campo es la palanca**. Si quieres que se lea la intención, deja el campo y no le pongas grupos
> nunca; si quieres que no se pueda romper, comprueba `global` en un test.

## 2. Portal / Público

Usuarios `base.group_portal` y `base.group_public` no ven NADA por defecto. Necesitan ACL +
record rule explícitas. **Confirmado midiendo** (v18 y v19): un `search([])` sobre un modelo con ACL
solo para `base.group_user` lanza `AccessError` para los dos — no devuelve lista vacía, que sería el
fallo silencioso. Aquí el framework avisa bien.

```csv
access_xx_portal,xx.portal,model_xx_object,base.group_portal,1,0,0,0
```
```xml
<record id="rule_xx_portal" model="ir.rule">
    <field name="name">XX: portal sees own</field>
    <field name="model_id" ref="model_xx_object"/>
    <field name="groups" eval="[(4, ref('base.group_portal'))]"/>
    <field name="domain_force">[('partner_id','=',user.partner_id.id)]</field>
</record>
```
- Portal normalmente solo lectura (`1,0,0,0`); el alta/edición se hace por controladores con `sudo()`
  controlado, no abriendo el ACL.
- Para el público (`base.group_public`): dominio aún más restrictivo o registros marcados como públicos.
- El acceso real del portal a documentos suele ir además por `access_token` en controladores (ver `odoo-portal-developer`).

## 3. Jerarquía Manager / User

`user` ve/gestiona lo suyo, `manager` todo. `implied_ids` hace que manager herede user.

```xml
<record id="group_xx_user" model="res.groups">
    <field name="name">XX / User</field>
    <field name="category_id" ref="module_category_xx"/>
</record>
<record id="group_xx_manager" model="res.groups">
    <field name="name">XX / Manager</field>
    <field name="category_id" ref="module_category_xx"/>
    <field name="implied_ids" eval="[(4, ref('group_xx_user'))]"/>
</record>
```
```csv
access_xx_user,xx.user,model_xx_object,group_xx_user,1,1,1,0
access_xx_manager,xx.manager,model_xx_object,group_xx_manager,1,1,1,1
```
Las record rules de filas: ver patrón 4 (owner-only para user) + rule `[(1,'=',1)]` para manager.
Recuerda: rules de grupo se UNEN (OR), así manager con su rule "todo" no se ve limitado por la de user.

**Medido** (v18 y v19): `implied_ids` funciona para las **ACL** — un manager que solo tiene el grupo
manager lee un modelo cuya única ACL está en el grupo user, y sigue sin poder crear porque esa ACL
es `1,0,0,0`. El permiso se hereda entero, con sus ceros incluidos: implicar un grupo **no amplía**
lo que ese grupo podía hacer.

## 4. Owner-only (cada usuario ve solo lo suyo)

```xml
<record id="rule_xx_own" model="ir.rule">
    <field name="name">XX: own records</field>
    <field name="model_id" ref="model_xx_object"/>
    <field name="groups" eval="[(4, ref('group_xx_user'))]"/>
    <field name="domain_force">[('user_id','=',user.id)]</field>
</record>
```
- El modelo necesita un campo de propietario (`user_id`, `create_uid`, `partner_id`...).
- Por grupo (OR), de modo que un manager con otra rule más amplia no quede limitado.
- Si fuera global, NADIE (ni manager) vería lo de otros: úsalo global solo si esa es la intención.

## 5. Readonly por estado (bloquear edición tras confirmar)

La seguridad por estado es difícil de expresar solo con record rules de write (puedes, con
`domain_force` sobre `perm_write`, pero limita filas, no transiciones). Dos enfoques:

- **UI**: en la vista, `<field name="x" readonly="state != 'draft'"/>` o `attrs`/atributos según versión.
  Solo visual, no seguridad.
- **ORM (real)**: sobreescribir `write()`/`unlink()` y lanzar `UserError`/`AccessError` si
  `state not in ('draft',)`, salvo grupo elevado. Esto es lógica de negocio, no ACL.
- Record rule de write limitada a borradores (filtra qué filas son escribibles):
```xml
<record id="rule_xx_write_draft" model="ir.rule">
    <field name="name">XX: write only draft</field>
    <field name="model_id" ref="model_xx_object"/>
    <field name="groups" eval="[(4, ref('group_xx_user'))]"/>
    <field name="perm_read" eval="False"/>
    <field name="perm_create" eval="False"/>
    <field name="perm_unlink" eval="False"/>
    <field name="domain_force">[('state','=','draft')]</field>
</record>
```
  Con `perm_write` activo y resto a `False`, restringe solo el write a filas en borrador. Combínalo
  con una rule de read amplia para que sí puedan VER los confirmados.

  **Medido el 2026-08-15** con exactamente esa regla (`perm_write=True`, el resto `False`,
  `domain_force=[('state','=','draft')]`):

  | Operación del usuario del grupo | Resultado |
  |---|---|
  | `search([])` | ve **los dos**: borrador y confirmado — la regla de write no toca la lectura |
  | `write` sobre el borrador | OK |
  | `write` sobre el confirmado | `AccessError` |

  O sea que el patrón funciona tal como está escrito, y la advertencia de combinarlo con una rule
  de read amplia es innecesaria **si no hay otra rule de read**: al no declarar `perm_read`, esta
  simplemente no participa en la lectura.

## Notas de versión
- v17/v18: grupos por `category_id` (`ir.module.category`).
- v19: privilegios — `privilege_id` sustituye al rol de `category_id` en el reparto de grupos.
  Confirma estructura y `eval` con `odoo-version-api-reference` antes de generar XML.
