# Depurar un AccessError (v17-v19)

Objetivo: localizar QUÉ capa bloquea antes de cambiar nada. Nunca "arreglar" con `sudo()` o metiendo
al usuario en `group_system`. Para inspección en vivo y pruebas como usuario usa `odoo-pilot`.

## 0. Leer el mensaje de error
Odoo distingue dos AccessError:
- **"You are not allowed to {operation} on '{Model}' ({model.tech})"** ⇒ falla la ACL (capa modelo).
  El usuario no está en ningún grupo con esa `perm_*`.
- **"... due to security restrictions ... (Document type: ..., Operation: ...)" / "Records: ..."** ⇒
  falla una RECORD RULE (capa fila). El CRUD del modelo está, pero el dominio excluye esas filas.
- Registro que "desaparece" sin error ⇒ casi siempre una record rule de READ que lo filtra.

## 1. Determinar grupos del usuario
```
res.users.read(uid, ['groups_id'])  →  lista de grupos
```
Nota version: el campo es `groups_id` en v17/v18, pero pasa a `group_ids` en v19
(`read(uid, ['groups_id'])` FALLA en v19; usa `group_ids`).

**Y al depurar un `AccessError` en v19, `group_ids` NO es lo que quieres leer.** v19 parte el
concepto en dos campos (`odoo/addons/base/models/res_users.py`, verificado 2026-08-07):

| Campo | Qué contiene | Para qué |
|---|---|---|
| `group_ids` | grupos asignados **directamente** | editar la pertenencia |
| `all_group_ids` | directos **+ implicados** (`:258-259`; compute `user.group_ids.all_implied_ids`, `:447-449`) | **decidir si tiene acceso** |

No existe en v18 (0 ocurrencias). Al trazar por qué un usuario NO puede hacer algo, lista
`all_group_ids`: si miras `group_ids` verás vacío o corto para grupos que el usuario tiene por
implicación y concluirás que le falta un grupo que en realidad tiene. Es el mismo error que en código
produce un 403 para todo el mundo.
¿Tiene el grupo que esperabas? Si no, problema de asignación, no de diseño.

## 2. Capa ACL — `ir.model.access`
Busca líneas para el modelo y los grupos del usuario:
```
domain: [('model_id.model','=','xx.object'),
         ('group_id','in', <grupos_del_usuario>)]
fields: ['group_id','perm_read','perm_write','perm_create','perm_unlink']
```
- ¿Existe alguna línea con la `perm_*` de la operación a 1? El acceso es la UNIÓN; basta una línea.
- ¿Hay una línea con `group_id` vacío? Aplica a todos (posible error de diseño).
- Sin ninguna línea que conceda ⇒ ESTE es el bloqueo. Añade/ajusta la ACL del grupo correcto.

## 3. Capa Record Rules — `ir.rule`
```
domain: [('model_id.model','=','xx.object')]
fields: ['name','global','groups','domain_force',
         'perm_read','perm_write','perm_create','perm_unlink']
```
Clasifica:
- **Globales** (`global=True` / sin `groups`): se aplican SIEMPRE y se INTERSECAN (AND). Una sola
  global cuyo dominio no incluya la fila ⇒ bloqueo, y no se abre con grupos.
- **De grupo**: solo cuentan las de grupos del usuario; entre ellas se UNEN (OR). Si el usuario no
  tiene ninguna rule de grupo aplicable pero otras existen, puede quedarse sin filas.

Evalúa el `domain_force` mentalmente con los datos del registro problemático:
- ¿`company_id` del registro está en `company_ids` del usuario? (multicompañía)
- ¿`user_id`/`partner_id` coincide con `user`? (owner-only)
- ¿`state` cumple el dominio? (readonly por estado)

Prueba el dominio en vivo como el usuario real (vía `odoo-pilot`, `with_user`):
```
xx.object.with_user(uid).search([('id','=', <id_problema>)])
```
Si devuelve vacío ⇒ una record rule de read lo filtra. Si devuelve el registro pero el write falla
⇒ una rule con `perm_write` lo excluye.

## 4. Capa campo — `groups=`
Si el error es al leer/escribir un CAMPO concreto (no el registro entero), revisa el campo en el modelo:
```
ir.model.fields  domain: [('model','=','xx.object'),('name','=','<campo>')]
field: ['groups']  (o inspecciona la definición Python)
```
Campo con `groups=` que el usuario no tiene ⇒ excluido del read/write.

## 5. Capa vista/menú
Si "no lo ve" pero por RPC SÍ puede ⇒ es solo `groups=` en vista/menú (visibilidad), no seguridad.
Revisa `<field groups=...>`, `<menuitem groups=...>`.

## Confirmar la hipótesis sin parchear a ciegas
- Reproduce con `with_user(uid)` (no `sudo`): aísla la capa exacta.
- El admin (`group_system`) ignora record rules ⇒ si falla como admin, es ACL o campo, no rule.
- Tras el fix, re-verifica con el rol real, no como admin.

## Tabla síntoma → causa
> **Los literales, verificados contra v18 y medidos en el banco el 2026-08-15.** Esta tabla solo
> sirve si el texto que trae coincide con el que sale por pantalla, así que van los de verdad.
> «security restrictions» —que ponía esta tabla— **no aparece en v17, v18 ni v19**: nadie lo va a
> encontrar buscándolo.

| Síntoma (literal) | Capa | Dónde mirar |
|---|---|---|
| `You are not allowed to access '<Modelo>' (<modelo>) records.` | ACL | `ir.model.access` del grupo (`odoo/addons/base/models/ir_model.py:28`, `ACCESS_ERROR_HEADER`) |
| `Uh-oh! Looks like you have stumbled upon some top-secret records.` | Record rule | `ir.rule` (global AND / grupo OR) (`odoo/addons/base/models/ir_rule.py:218`, `IrRule._make_access_error`, v19 `:221`) |
| Registro no aparece, sin error | Record rule de read | `ir.rule` perm_read + dominio |
| Añadir grupo no soluciona | Record rule GLOBAL | `ir.rule` global=True |
| Falla solo un campo | Field groups | `groups=` del campo |
| Funciona como admin, falla como user | Record rule | probar con `with_user`, no admin |
| Falla vía RPC pero oculto en UI | Vista/menú (no seguridad) | `groups=` en vista/menuitem |
| Portal no ve nada | Falta ACL+rule portal | `base.group_portal` |
