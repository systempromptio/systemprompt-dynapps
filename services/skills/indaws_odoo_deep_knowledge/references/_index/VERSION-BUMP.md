# Playbook: documentar una versión nueva de Odoo

> Qué hacer cuando sale Odoo vN+1 y hay que poner el KB al día. **No se relee todo**: se procesa el *diff* que las tools te entregan. Pre-requisito conceptual en [`HOW-TO-EXTEND.md`](HOW-TO-EXTEND.md).
>
> **Idea central:** las citas del KB están ancladas a símbolo (`fichero.py::método`). Cambiar de versión = preguntar "¿este símbolo sigue existiendo en vN+1?". Los que NO → son renombres o eliminaciones = exactamente lo que hay que documentar. El motor que responde eso es `cite_audit --methods` apuntado a la fuente nueva.

## Procedimiento

### Paso 0 — Tener la fuente de la versión nueva
Roots típicos: `$ODOO_SRC/odoo<N>/addons` (CE) + `$ODOO_ENTERPRISE_SRC/<N>` (EE). (Resolución completa en `references/shared/odoo-source.md`.)

### Paso 1 — Sacar el work-list (un comando)
```bash
python3 scripts/version_audit.py --to v<N>
```
Emite, por dominio, los símbolos citados que **ya no existen** en la fuente vN, ordenado por impacto. (Por dentro: `ODOO_SRC_ROOTS=<vN> cite_audit.py --methods`; ver pieza 2.) Ese listado ES el trabajo.

### Paso 1b — Repartir el work-list antes de abrir la fuente (un comando)
```bash
python3 scripts/version_triage.py --to v<N>            # o --worklist <el del paso 1>
```
Clasificar N símbolos a mano son N lecturas de fuente, y casi todas las contesta un grep. El triaje reparte en cuatro cubos **sin juicio**: `YA-DOC` (el `v<N>_changes.md` ya lo nombra), `RENOMBRADO` (hay un nombre muy parecido en la fuente nueva, y lo propone), `ELIMINADO` (está en la vieja, no en la nueva) y `FANTASMA` (en ninguna). Medido sobre v19: **45 / 53 / 45 / 5** de 148 símbolos, en 13 segundos y cero tokens.

**`FANTASMA` no quiere decir "cita inventada".** Los 5 de v19 eran de `enterprise/account_consolidation`, existen en Enterprise 15, 16 y 17, y ese dominio es legacy y se audita contra su propia versión. Búscalo en las versiones anteriores antes de tocar nada.

**Y desconfía de un work-list gordo**: si un dominio entero sale marcado, casi siempre falta su árbol de fuente, no ha cambiado Odoo. Con dos raíces en vez de las seis, el informe de v19 daba **282 símbolos en vez de 148** (2026-09-01).

### Paso 2 — Clasificar cada símbolo caído (contra fuente, NO de memoria)
El triaje del paso 1b dice por dónde empezar; la decisión sigue siendo tuya. Para cada símbolo, abre la fuente vN y decide:
- **Renombrado** (`_foo`→`_bar`): documenta el mapeo en el `v<N>_changes.md` del dominio. NO borres el gotcha v18 — añade la nota de versión.
- **Eliminado**: documenta "X eliminado en vN" en `v<N>_changes.md` (breaking).
- **Movido de fichero** (p.ej. v19 movió ORM a `odoo/orm/`): re-ancla la cita al fichero nuevo.
- **Falso positivo** (el símbolo existe pero el regex no lo ve): mejora el regex de `cite_audit.py`, no el contenido.

Regla dura: igual que en [`HOW-TO-EXTEND.md`](HOW-TO-EXTEND.md), **no inventes el comportamiento nuevo** — si no localizas el reemplazo en fuente, anótalo como NO-ENCONTRADO y déjalo para verificación manual.

### Paso 3 — Actualizar el marcador de versión
Sube el `<!-- verified: ... -->` del/los fichero(s) tocados para reflejar que se verificó contra vN (p.ej. `v18.0 (base) + v19 diffs @ <fecha>`). El marcador es la fuente de verdad de "contra qué versión está esto verificado" (lo lee la pieza 3, staleness-por-versión).

### Paso 4 — Re-indexar y gatear
- Añade entradas v_N a `_index/symptoms.md` (§5 "v19 Breaking" o equivalente) y `_index/cheatsheet.md`.
- `python3 scripts/kb_release_check.py` → gates verdes + muestra semántica.

## Por qué esto es sólido a un año

- El coste de una versión nueva es **proporcional al diff** (símbolos que cambiaron), no al tamaño del KB. Un bump típico toca decenas de símbolos, no miles.
- Cada afirmación nueva entra anclada a símbolo → la **siguiente** versión vuelve a ser barata. El sistema se auto-mantiene.
- Los marcadores `verified: vX` hacen auditable, en cualquier momento, qué está al día y qué no (pieza 3).
- Las citas line-only son el único punto ciego (invisibles al motor de diff) → migrarlas a símbolo es la pieza 4.

## Anti-patrón

NO hagas un "re-scan completo" de un dominio en cada versión: es caro, propenso a re-introducir errores y borra el trabajo de verificación previo. Procesa el diff. El re-scan completo solo se justifica si el módulo fue **reescrito** entre versiones (raro; p.ej. web_editor→html_builder).
