---
name: indaws-odoo-security-designer
version: 0.3.2
effort: high
model: opus
description: >-
  Diseña y depura el control de acceso de Odoo (v17-v19): ir.model.access.csv,
  record rules (ir.rule global/grupo), res.groups, groups= en campos/vistas,
  acceso a menús y portal/público. Traza grupo→ACL→record rule→field groups para
  resolver "¿por qué X no puede ver/editar Y?".
  Triggers: "diseñar permisos", "crear grupo", "record rule", "AccessError por record rule",
  "no puede ver/editar", "permisos multicompañía", "acceso portal",
  "design access rights", "security groups", "why can't user see/edit",
  "field-level security".
allowed-tools: [Bash, Read, Write, Edit, Glob, Grep, AskUserQuestion]
---
<!-- verified: v18.0 @ 2026-06 -->

# Odoo Security Designer

Diseña y depura el control de acceso de Odoo de extremo a extremo: ACL (`ir.model.access`),
record rules (`ir.rule`), grupos (`res.groups`), categorías, seguridad a nivel de campo y vista,
y acceso a menús/portal. Responde "quién puede hacer qué sobre qué modelo" y resuelve cualquier
`AccessError` trazando la cadena completa de evaluación.

Refs compartidas: `references/shared/odoo-connection.md`, `references/shared/batch-limits.md`.

API y diffs de versión: NO se documentan inline. Consulta `odoo-version-api-reference` (v17-v19).
Conexión a Odoo en vivo: NO se reimplementa, usa `odoo-pilot`.

## Workflow

1. **Modelar el requisito.** Identifica actores (roles), modelos objetivo y operaciones (R/W/C/D)
   sobre qué subconjunto de registros. Usa `AskUserQuestion` si falta el rol, la compañía o el
   alcance (todos / propios / por estado / por compañía). Salida: matriz rol × modelo × CRUD × dominio.
2. **Diseñar capas coherentes.** Decide qué resuelve cada capa (lee `references/access-model.md`):
   ACL = puerta CRUD por grupo; record rules = qué filas; field groups = qué columnas; menús/vistas =
   qué se ve. Elige patrón en `references/patterns.md` (multicompañía, portal, manager/user, owner-only,
   readonly por estado). Verifica diffs `groups_id`/`category_id`→`privilege_id` [delega `odoo-version-api-reference`].
3. **Escribir los ficheros.** `ir.model.access.csv` + `security/*.xml` (grupos, categorías, record rules).
   Para scaffolding de la estructura de seguridad de un módulo nuevo [delega `odoo-module-creator`].
4. **Depurar AccessError.** Traza la ruta exacta grupo→ACL→record rule→field groups siguiendo
   `references/debugging-access.md`. Identifica QUÉ capa bloquea antes de tocar nada.
5. **Verificar por persona/rol.** Comprueba el resultado simulando el usuario real (`with_user` /
   reglas evaluadas, nunca solo `sudo`) [delega `odoo-pilot` para inspeccionar grupos/rules en vivo y
   probar como usuario]. Para una auditoría de seguridad del módulo [delega `odoo-code-auditor`].

## GOTCHAS
1. **Record rules globales se INTERSECAN (AND); las de grupo se UNEN (OR).** Una rule global
   restrictiva no se "abre" añadiendo el usuario a otro grupo. Causa nº1 de "no veo nada".
1b. **`<field name="global">` de una `ir.rule` NO decide nada — MEDIDO (v18 y v19).** Es un campo
   **calculado**: `rule['global'] = not rule.groups` (`odoo/addons/base/models/ir_rule.py:53-55`, `IrRule._compute_global` v18, `:54-56` v19). Escribirlo
   se acepta y el compute lo pisa. Consecuencia real: añadirle un grupo a la regla de multicompañía
   **apaga el aislamiento** para todo el que no esté en ese grupo, el XML sigue diciendo
   `global True` y no salta ningún aviso. Comprueba el **campo**, no el XML. → `references/patterns.md` §1.
2. **ACL sin record rule = acceso a TODAS las filas.** Olvidar la record rule expone todos los
   registros aunque el CRUD parezca correcto.
3. **`perm_*` en el CSV es por grupo, no acumulativo de forma intuitiva:** el acceso efectivo es
   la UNIÓN de todas las líneas ACL de los grupos del usuario. Una línea con `1,0,0,0` no resta.
4. **Línea ACL sin grupo (`group_id` vacío) aplica a TODO usuario**, incluido el portal/público. Casi siempre un error.
5. **`groups=` en un campo/vista oculta pero NO protege a nivel ORM** si el ACL/record rule lo permite vía RPC. La seguridad real está en ACL+rules, no en la vista.
6. **`sudo()` salta ACL y record rules**: úsalo solo en código de sistema; nunca como "fix" de un AccessError de usuario.
7. **v19 parte los grupos del usuario en DOS campos, y confundirlos DENIEGA acceso a todo el mundo.** No es solo el rename `groups_id`→`group_ids`: v19 añade **`all_group_ids`** = directos **+ implicados** (`odoo/addons/base/models/res_users.py:258-259`, `Groups._search`, computed `compute_sudo=True` con `search=`; su compute es `user.all_group_ids = user.group_ids.all_implied_ids`, `:447-449`). No existe en v18 (0 ocurrencias). Regla: **para AUTORIZAR usa `all_group_ids`**; `group_ids` solo para *editar* la pertenencia directa. Un check contra `group_ids` sobre un grupo que casi todos tienen de forma **implícita** (p.ej. `base.group_user`) deniega a todo el mundo — y es un 403 real, ya visto. Además `category_id` en `res.groups` pasa a `privilege_id`. Diffs completos en `odoo-version-api-reference`.
8. **Portal/público necesitan record rules explícitas** (`base.group_portal`, `base.group_public`); sin ellas no ven nada aunque exista la ACL.
9. **`active=False` (registros archivados) requiere contexto/regla**: un usuario sin la rule adecuada "pierde" registros archivados sin error.
10. **Quien ignora record rules es el superusuario (`sudo()` / `env.su`), no el grupo `group_system` per se** — el ORM consulta `self.env.su`. El admin las salta porque suele operar en superusuario; depurar como admin enmascara el bug. Prueba siempre con el rol real.
11. **"No le pongo rule al aprobador" NO significa "el aprobador lo ve todo"** — los grupos altos IMPLICAN a los bajos, así que una rule sobre `base.group_user` aplica también al admin; si es la única rule de grupo, el admin ve solo lo suyo. Síntoma medido: una petición creada por un usuario de servicio era invisible para TODOS los humanos → flujo headless muerto (nadie podía aprobarla). El aprobador necesita su propia rule permisiva, que se UNE (OR). Detalle + test en los dos sentidos: `references/access-model.md`.
12. **La whitelist AST estática NO garantiza seguridad en runtime.** Si validas una expresión de usuario (un `domain_force` dinámico, una fórmula, cualquier `safe_eval`/`eval`) comprobando estáticamente los opcodes/nodos permitidos, eso bloquea el ACCESO peligroso (imports, dunder, atributos), pero una expresión perfectamente "válida" igual **peta en ejecución** (`1/0`, método sobre recordset vacío, clave inexistente). Envuelve SIEMPRE la evaluación en `try/except` con un fallback explícito; nunca dejes que un error de lógica en una expr dinámica tumbe el flujo. Mismo principio en `odoo-server-action-creator` (G32) y `odoo-qweb-report-builder` (§11).

13. **Pisar una ACL o una rule AJENA por su XML ID no se deshace al desinstalar — y es invisible leyendo código.** Si tu `ir.model.access.csv` declara una fila con el XML ID de otro módulo (`product.access_product_category_manager`) o tu `ir_rule.xml` reescribe un `<record id="project.task_type_manager_rule">`, Odoo **actualiza la fila existente** y `ir.model.data` sigue con `module='product'` / `'project'`: la fila NO es tuya. Desinstalar tu módulo no la borra ni la restaura, así que **el permiso neutralizado se queda neutralizado para siempre**, sin módulo que lo explique. Y si el fichero de origen es `<data noupdate="1">` (`odoo19/addons/project/security/project_security.xml:45`, `project_comp_rule`, `odoo19/addons/hr/security/hr_security.xml:27`, `odoo_enterprise/19/knowledge/security/ir_rule.xml:2`), **ni siquiera un `-u` del módulo dueño lo repara**. Obligatorio: `uninstall_hook` que restaure cada fila. Medido en 13 PR de gobierno: **5 módulos lo tenían mal y 7 bien**, y ninguno de los 5 se veía leyendo el diff.
14. **Restaurar a `(1,1,1,1)` puede conceder MÁS que el estándar.** El `uninstall_hook` de la G13 tiene que llevar el valor REAL de cada fila leído de la fuente, no el patrón. Contraejemplos medidos en 19.0: `account.access_account_lock_exception_manager` es **`1,0,1,0`**, `account_accountant.access_account_change_lock_date` es **`1,1,1,0`**, `hr_skills.access_hr_skill_employee` es **`1,0,1,0`**. Un bucle que asume `1,1,1,1` le da al contable escritura y borrado sobre excepciones de bloqueo que Odoo nunca le dio — **un arreglo más laxo que el estándar que dice restaurar es peor que el defecto**, y ningún test lo caza salvo que asierte la tupla exacta. Una `ir.rule` que no declara `perm_*` los tiene los cuatro a `True` por defecto: ahí `(True,True,True,True)` sí es el estándar.
15. **Un módulo que solo QUITA permisos no puede romper la suite estándar de Odoo** — porque Odoo la ejecuta como administrador, y estos módulos meten `base.group_system` en `implied_by_ids` de su propio grupo para que el admin no quede fuera. Medido cuatro veces con delta vacío (219→219, 1052→1052, 276→276, 286→286) y el `AccessError` del módulo apareciendo **cero** veces en el log estándar. La excepción que sí rompe: cuando la restricción cae sobre un **usuario normal** en un modelo que el estándar ejercita como usuario normal (`project.task.type` y las etapas personales de la app To-do), o cuando el módulo además **escribe datos** o **toca una vista**. Sirve para decidir cuándo la medición vale la pena y cuándo es ceremonia cara.

## NEVER
- Resolver un AccessError con `sudo()` o añadiendo el usuario a `group_system`.
- Dejar líneas ACL sin `group_id` salvo intención explícita y justificada.
- Confiar en `groups=` de vista/campo como única barrera de seguridad.
- Copiar XML de seguridad entre versiones sin verificar diffs de `res.groups` (v17-v19).
- Documentar API de Odoo aquí: delega en `odoo-version-api-reference`.

## ALWAYS
- Emparejar cada ACL con la record rule que limite las filas (o justificar acceso total).
- Distinguir rule global (AND) de rule por-grupo (OR) al diseñar y al depurar.
- Verificar el resultado con el rol real (`with_user`), no como admin/`sudo`.
- Razonar capa por capa: grupo → ACL → record rule → field groups → vista/menú.
- Código fuente y external IDs en inglés (`references/shared/coding-standards.md`).
- **Provenance marker (ALWAYS):** every generated file (except `.csv`/`.json` — no comment syntax) ends with `Made with odoo-security-designer · v<version del frontmatter de esta skill>`, wrapped in the file-type comment, idempotent. Full rule: `references/shared/skill-provenance-marker.md`.

## Delegates to
| Skill | Para qué |
|-------|----------|
| odoo-version-api-reference | Cambios `res.groups` `category_id`→`privilege_id`, `groups_id`, diffs v17-v19 |
| odoo-pilot | Inspeccionar grupos/ACL/rules en vivo y probar como usuario (`with_user`) |
| odoo-module-creator | Scaffolding de los ficheros de seguridad de un módulo nuevo |
| odoo-code-auditor | Auditoría de seguridad del módulo |
| odoo-deep-knowledge | Quirks de evaluación de ir.rule/ACL/multicompañía |

## Consumed by
| Skill | Para qué |
|-------|----------|
| guion | Diseño y depuración del control de acceso como paso del flujo |
| odoo-wizard-builder (developer) | Generar el `ir.model.access` del wizard |

## REFERENCES
| Fichero | Contenido | Leer cuando |
|---------|-----------|-------------|
| [references/access-model.md](references/access-model.md) | Capas (ACL/record rules/grupos/field-level/menú), orden de evaluación, AND global vs OR grupo | Diseñar o entender el modelo |
| [references/patterns.md](references/patterns.md) | Multicompañía, portal/público, jerarquía manager/user, owner-only, readonly por estado | Elegir e implementar un patrón |
| [references/debugging-access.md](references/debugging-access.md) | Trazar un AccessError a su causa, herramientas, queries a `ir.model.access`/`ir.rule` | Depurar un AccessError concreto |
