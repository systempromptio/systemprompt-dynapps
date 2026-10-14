# Fasing (obra) + git + scaffold multi-módulo

## Fasing estilo obra

Nada se implementa de golpe. Fases como una construcción, cada una cerrable y en verde antes de abrir la siguiente:

- **Cimientos** — el módulo base shippable solo (lo mínimo usable). Si el producto es "simple→complejo",
  el base es la versión mínima ("caja tonta") y cada capacidad es una ampliación (módulo por check).
- **Estructura** — ciclo de vida y relaciones que el resto carga encima.
- **Cerramientos** — los dominios pesados por capacidad (cada uno su módulo).
- **Acabados** — UI completa, settings, report, i18n, rendimiento.
- **Entrega** — QA, matriz de instalación (base solo → +cada módulo → todos), review, merge, tags.

Cada capacidad es su propio mini-ciclo (modelo→lógica→vistas→tests) y **debe instalar/desinstalar limpio**
sobre el base sin romperlo.

## Git

- **Una rama única para el v1** (`feature/<ver>-<modulo>`), no una por módulo.
- **Commit atómico por bloque** (`[ADD|IMP|FIX] <módulo>: <qué>`), solo con tests del bloque en verde.
- **Push al cerrar cada fase.** Merge al final vía PR/MR + quality gate. Sin merges parciales.
- Bridges de dominio (post-v1) → ramas propias tras el merge del v1.

## Scaffold multi-módulo

Genera el árbol de TODOS los módulos del mapa (Step 1/2) de una vez, con un **script generador** (más limpio
que decenas de writes). Cada módulo:
- `__manifest__.py` con las deps decididas (¡las del scan de reuso!), versión `<major>.0.1.0.0`, license, author de la suite.
- `models/` con las clases y **campos declarados** fielmente al diseño; la **lógica va como `TODO`** con
  referencia a la decisión (`D##`). Un scaffold informa el arranque; no es el módulo terminado.
- `security/` (privilege + grupos user/manager que implican los del base en v19: `privilege_id`), `ir.model.access.csv`.
- `res.config.settings` del base con los `module_*` de cada capacidad (default off) si el mecanismo es por check.
- `data/`, `views/`, `wizard/`, `tests/`, `hooks.py` (uninstall), `i18n/`.

### Validación obligatoria antes de entregar
Valida **sintaxis** (no confirma instalación — dilo honestamente):
- `py_compile` de todos los `.py`.
- XML bien formado (`xml.dom.minidom`).
- CSV parseables.
- Manifests evaluables como dict con `name`/`depends`.

Limpia `__pycache__` que deje el syntax-check. **No afirmes que instala** si no lo probaste en un Odoo real;
di "validado a nivel de sintaxis, instalación pendiente de docker".

## v19 recordatorios (verifica a fuente si dudas)
- `<list>` (no `<tree>`), `<chatter/>`, `invisible/readonly` (no `attrs`), `privilege_id` (no `category_id`),
  `models.Constraint` (no lista `_sql_constraints`), `Command.*` en x2many, `env._()` disponible.

## Ubicación del scaffold
Si el repo destino no está confirmado, genera en una **carpeta staging movible** del proyecto y dilo — no lo
metas en un repo de cliente equivocado ni asumas el repo de la suite.
