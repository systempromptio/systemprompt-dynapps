# Modelo de Acceso de Odoo (v17-v19)

Cinco capas, evaluadas en este orden. Una petición debe pasar TODAS las aplicables.

```
Usuario
  └─ grupos (res.groups)  ──►  ¿pertenece al grupo?
        └─ ACL (ir.model.access)  ──►  ¿CRUD permitido sobre el MODELO?
              └─ record rules (ir.rule)  ──►  ¿permitido sobre ESTAS FILAS?
                    └─ field groups (groups= en el campo)  ──►  ¿puede ver/escribir ESTA COLUMNA?
                          └─ vista/menú (groups=)  ──►  ¿lo VE en la UI?
```

Las 3 primeras capas son seguridad real (ORM/RPC). Las 2 últimas son visibilidad: ocultan en la
UI pero NO protegen si el ACL+rules permiten la operación vía RPC.

## 1. Grupos — `res.groups`

Unidad de asignación de permisos. Un usuario tiene N grupos; el acceso efectivo es la UNIÓN de lo
que aportan sus grupos (más herencia vía `implied_ids`).

- `implied_ids`: heredar permisos de otro grupo (manager implica user).
- `users`: M2O inverso a `res.users.groups_id`.
- `category_id` (`module_category`): agrupa grupos en la UI de ajustes.
  - **v17/v18**: campo `category_id`. Selección de grupo por categoría se muestra como radio/checkbox.
  - **v19**: el concepto pasa a `privilege_id` (privilegios). Verifica la firma exacta y el reparto
    de grupos por privilegio con `odoo-version-api-reference` antes de escribir XML.
- `comment`: descripción del grupo.

XML típico:
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

Categoría:
```xml
<record id="module_category_xx" model="ir.module.category">
    <field name="name">XX</field>
    <field name="sequence">20</field>
</record>
```

## 2. ACL — `ir.model.access` (`ir.model.access.csv`)

Puerta CRUD a nivel de MODELO, por grupo. Columnas:

```csv
id,name,model_id:id,group_id:id,perm_read,perm_write,perm_create,perm_unlink
access_xx_user,xx.user,model_xx_object,group_xx_user,1,1,1,0
access_xx_manager,xx.manager,model_xx_object,group_xx_manager,1,1,1,1
```

Reglas clave:
- El acceso efectivo es la **UNIÓN** de todas las líneas ACL de los grupos del usuario. No hay resta:
  una línea `1,0,0,0` no quita un write que otra línea concede.
- **`group_id` vacío ⇒ aplica a TODOS los usuarios** (incluido portal/público). Casi siempre un error.
- Si no existe NINGUNA línea ACL que conceda la operación al usuario ⇒ `AccessError`.
- ACL controla el modelo, no las filas. Sin record rule, conceder `perm_read=1` expone TODAS las filas.

## 3. Record Rules — `ir.rule`

Filtran QUÉ FILAS sobre las que ya hay permiso CRUD. Campos: `model_id`, `domain_force`,
`groups`, `perm_read/write/create/unlink`, `global` (calculado: `True` si `groups` vacío).

### Combinación — la regla de oro
- **Reglas GLOBALES** (sin grupos) se **INTERSECAN (AND)**. Todas deben cumplirse. Una rule global
  restrictiva NO se puede "abrir" añadiendo grupos al usuario.
- **Reglas de GRUPO** (con `groups`) se **UNEN (OR)** entre sí.
- Resultado: `(AND de todas las globales) AND (OR de las de grupo aplicables al usuario)`.

**Medido el 2026-08-15** (`tools/bench-orm`, `TestRecordRules`). Tres registros —A con `flag=1`,
B con `flag=2`, C con `flag=3`—, una regla **global** `[('flag','!=',3)]` y dos reglas de grupo:
X ve `flag=1`, Y ve `flag=2`.

| Usuario | Ve |
|---|---|
| solo en X | `['A']` |
| en X y en Y | `['A','B']` |
| **en ninguno de los dos** | `['A','B']` |
| en `base.group_system` (sin ser superusuario) | `['A','B']` |
| cualquiera con `.sudo()` | `['A','B','C']` |

> ⚠️ **El matiz que cuesta caro: una regla de grupo NO restringe a quien no está en ese grupo.**
> El usuario que no pertenece ni a X ni a Y ve **A y B** — todo lo que las globales permiten. Es
> el error de diseño más natural: se añade una `ir.rule` con `groups` creyendo que limita, y solo
> limita **a los miembros de ese grupo**. Para restringir a todo el mundo, la regla tiene que ser
> **global** (sin `groups`).

Y las dos últimas filas confirman lo que dice más abajo esta misma página: `base.group_system`
**no** salta las reglas —las cumple igual que cualquiera— y quien las salta es `sudo()`/`env.su`.
- Si un usuario no tiene NINGUNA rule de grupo para una operación pero hay rules de grupo definidas
  por otros grupos, podría quedar sin acceso a filas aunque el ACL lo permita.

#### El corolario que muerde: "no le pongo regla al aprobador" ≠ "el aprobador lo ve todo"

Los grupos altos **implican** a los bajos (`group_system` → … → `group_user`). Así que una rule
puesta sobre `base.group_user` **también aplica al administrador**, y si es la ÚNICA rule de grupo
del modelo, el admin ve exactamente lo mismo que un usuario raso.

```xml
<!-- Intención: "cada uno ve lo suyo; el aprobador ve todo" -->
<record id="own_rule" model="ir.rule">
    <field name="groups" eval="[Command.link(ref('base.group_user'))]"/>
    <field name="domain_force">[('requester_id', '=', user.id)]</field>
</record>
<!-- ...y NADA para el grupo aprobador -->
```
Resultado real: el admin **solo ve las suyas** (medido: veía las peticiones 1 y 2, no la 3, creada
por un usuario de servicio). El flujo headless queda muerto — hay una petición pendiente que ningún
humano puede resolver. No es que las reglas fallen: es que "sin regla" no significa "sin filtro"
cuando el grupo implicado sí la tiene.

**Arreglo:** dar al aprobador su propia rule permisiva, que se UNE (OR) con la restrictiva:

```xml
<record id="approver_rule" model="ir.rule">
    <field name="groups" eval="[Command.link(ref('base.group_system'))]"/>
    <field name="domain_force">[('company_id', 'in', company_ids)]</field>
</record>
```

**Cómo comprobarlo** (no lo asumas): `Model.with_user(approver).search([])` y verifica que aparece un
registro creado por OTRO. Y pon el test en los dos sentidos — el raso sigue viendo solo lo suyo, y el
aprobador no cruza de compañía.

Ejemplo global (multicompañía):
```xml
<record id="rule_xx_company" model="ir.rule">
    <field name="name">XX: multi-company</field>
    <field name="model_id" ref="model_xx_object"/>
    <field name="global" eval="True"/>
    <field name="domain_force">['|',('company_id','=',False),('company_id','in',company_ids)]</field>
</record>
```

Ejemplo por grupo (owner-only para user, todo para manager):
```xml
<record id="rule_xx_own" model="ir.rule">
    <field name="name">XX: own records</field>
    <field name="model_id" ref="model_xx_object"/>
    <field name="groups" eval="[(4, ref('group_xx_user'))]"/>
    <field name="domain_force">[('user_id','=',user.id)]</field>
</record>
<record id="rule_xx_all" model="ir.rule">
    <field name="name">XX: all records</field>
    <field name="model_id" ref="model_xx_object"/>
    <field name="groups" eval="[(4, ref('group_xx_manager'))]"/>
    <field name="domain_force">[(1,'=',1)]</field>
</record>
```

Variables disponibles en `domain_force`: `user`, `company_id`, `company_ids`, `time`.

### Admin
Quien ignora las record rules es el superusuario (`sudo()` / `env.su`), no el grupo
`base.group_system` per se: el ORM comprueba `self.env.su`, no el grupo. El admin las salta
porque suele operar en superusuario. Depurar como admin enmascara fallos de filas: prueba
SIEMPRE con el rol real.

## 4. Seguridad a nivel de campo — `groups=` en el campo

```python
secret = fields.Char(groups="base.group_system")
```
Oculta el campo a quien no tenga el grupo y lo excluye de `read`/`write` por ORM normal. PERO un
campo `groups=` no sustituye a ACL/record rules para la fila; es una capa adicional de columna.
Útil para campos sensibles; no es la barrera principal.

**Medido el 2026-08-15 en v18 y v19**, con un usuario SIN el grupo. Y aquí las dos versiones **no
hacen lo mismo**:

| Cómo se accede | v18 | v19 |
|---|---|---|
| `rec.secret` — **acceso por atributo** | **devuelve el valor** | `AccessError` |
| `rec.read(['secret'])` | `AccessError` | `AccessError` |
| `rec.write({'secret': …})` | `AccessError` | `AccessError` |

> ⚠️ **En v18 el acceso por atributo NO comprueba el `groups=` del campo.** La barrera está en
> `check_field_access_rights` (`odoo18/odoo/models.py:3777`, `BaseModel.check_field_access_rights`), y quien la llama es `read()`
> (`:3861`) y `write()` (`:3898`) — no el `Field.__get__`, que en v18 no comprueba nada
> (`odoo18/odoo/fields.py:1225-1244`, `Field.__get__`).
>
> Para la superficie de usuario eso basta, porque el cliente y el RPC pasan por `read`. Pero si tu
> propio código server-side hace `rec.with_user(u).secret` y devuelve ese valor —un endpoint, un
> informe, un export— **el `groups=` no te protege en v18**: lo has leído tú, no el usuario. La
> protección de columna es del borde RPC, no del intérprete.
>
> **v19 cerró el agujero**: `Field.__get__` empieza por
> `if not (env.su or record._has_field_access(self, 'read')): record._check_field_access(...)`
> (`odoo19/odoo/orm/fields_relational.py` hereda de `odoo/orm/fields.py:1647-1650`, `Field.__get__`). O sea que en
> v19 el mismo endpoint revienta con `AccessError` donde en v18 devolvía el dato. **Al migrar
> 18 → 19, código server-side que leía por atributo campos con `groups=` deja de funcionar** — y el
> fallo es en tiempo de ejecución, no al instalar.

## 5. Vistas y menús — `groups=`

`<field name="x" groups="..."/>`, `<page groups="...">`, `<menuitem groups="...">`. Solo visibilidad
en la UI. Un usuario sin el grupo no ve el elemento, pero si llega por RPC y ACL+rules lo permiten,
la operación pasa. Nunca dependas de esto para seguridad real.

## Orden de fallo (para depurar)
1. ¿El usuario está en algún grupo con línea ACL para la operación? No ⇒ AccessError de modelo. **Medido**: sin ninguna fila en `ir.model.access`, un `search([])` con `with_user` lanza `AccessError: You are not allowed to access '<modelo>'` — el modelo entero, no una fila.
2. ¿Pasa las record rules (AND global / OR grupo) para esas filas? No ⇒ AccessError de registro / registro "desaparece".
3. ¿El campo concreto tiene `groups=` que lo bloquea? ⇒ campo no presente / error al escribir.
4. ¿La vista/menú lo ocultan? ⇒ no es seguridad, solo UI.
