# Seguridad: expresiones y helpers

El editor deja que el usuario pinte **fórmulas** y **funciones con nombre**. Ambas
ejecutan código sobre el registro del informe → superficie de inyección. Dos capas.

## Nivel 1 — chip de expresión (`data-expr` → `t-out`)

Validación **estática por `ast`** en `_expr_is_safe(expr)` antes de emitir:

- **Whitelist de nombres**: `doc`, `line` + funciones seguras
  `sum, min, max, len, abs, round, any, all, format_lang, format_date`.
- **Métodos permitidos** sobre recordsets: `mapped, filtered, sorted, get`.
- **Prohibido** (rechaza el árbol): `env`, `write`, `unlink`, `create`, dunder `__*__`,
  `import`, atributos peligrosos, llamadas fuera de la whitelist.
- Expresión inválida → se emite **vacío** (no rompe el informe).

UI: panel con textarea + botón "Probar" (`test_expr`, eval con `builtins` vacíos).

**Al extender**: si añades una función a la whitelist, comprueba que no da acceso
transitivo a `env`/escritura. Añádela tanto a la whitelist de `_expr_is_safe` como al
namespace del `safe_eval`/`t-out` de emisión.

## Nivel 2 — funciones con nombre (`indaws.report.helper`)

Modelo `indaws.report.helper`: `name` / `label` / `model_id` / `code` (python que
asigna `result`, con `doc`, `line`, `env` en el namespace). Se ejecuta con
`safe_eval(mode='exec')` — **igual que las acciones de servidor de Odoo**.

- Menú **Report Designer › Funciones**, restringido a `group_system`.
- Chip `data-helper` → `t-out="doc.env['indaws.report.designer'].browse(id).report_helper('name',doc,line)"`.
- API: `list_helpers` / `report_helper` / `test_helper`.
- Seguridad `ir.model.access` añadida (solo admin gestiona helpers).

**Modelo de amenaza**: como el helper corre `exec` con `env`, quien pueda **crear/editar**
helpers puede correr código arbitrario como el usuario. Por eso la gestión está tras
`group_system`. Un usuario normal solo **inserta** un helper ya definido en su informe,
no lo edita. No relajar el `group_system` sin entender esto.

## Saneo del HTML incrustado

El HTML rico del editor se incrusta en el arch QWeb → riesgo de inyección de plantilla
(`t-esc`/`t-out`/`<t>` metidos a mano por el usuario). `_html_to_xml` quita atributos
`t-*` y `groups` del HTML de usuario y desenvuelve `<t>` antes de incrustar. Los `t-*`
legítimos los pone el **emisor**, nunca vienen del HTML crudo.

## Reglas al tocar seguridad

- Nunca `eval`/`safe_eval` de una expresión de usuario sin pasar por `_expr_is_safe`.
- Nunca emitir `t-field`/`t-out` de un path sin `_resolve_field` (validación contra modelo).
- Nunca incrustar HTML de usuario sin `_html_to_xml` (strip de `t-*`/`groups`, unwrap `<t>`).
- Gestión de helpers = `group_system`. Insertar ≠ editar.

Made with report-designer-developer · v0.1.0
