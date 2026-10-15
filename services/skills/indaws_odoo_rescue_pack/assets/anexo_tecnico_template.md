# Anexo técnico — Rescue Pack {CLIENTE}

> **Documento B** del Rescue Pack. Público: equipo técnico / desarrollo del cliente.
> Complementa al informe ejecutivo (Documento A) con el detalle a nivel de código.
> Render con `dynapps-brand` + `pdf-renderer`; pie "Anexo técnico".
> **Sí incluye `file:line`, nombres de método, de error y fragmentos de código** (a diferencia del ejecutivo).

---

## B1. Detalle módulo por módulo

Un bloque por módulo custom, agrupados por área funcional. Formato por módulo:

> **`{modulo}`** · salud {0-10}
> - **Qué hace:** descripción funcional (1-2 líneas).
> - **Errores:** hallazgos con severidad; identificadores y rutas en `<code>` (`archivo.py:línea`).

_(Maquetar como bloques verticales, NO tabla de 4 columnas — desborda en A4. `word-break` en los
`<code>`.)_

## B2. Análisis de flujos de negocio

Trazado de los flujos clave del sistema (p. ej. venta → cálculo de coste → lista de materiales →
fabricación → stock). Para cada flujo:
- Pasos numerados del camino feliz (con `file:line` de los puntos de entrada).
- **Dónde se rompe o degrada**, ordenado por probabilidad de incidencia.

## B3. Bugs de runtime latentes (confirmados en código)

Bloques verticales (uno por bug). **Formato que aguanta preguntas del técnico** — por bug:
- **Código actual** (`archivo.py:línea`): el **fragmento literal** (3-8 líneas) en bloque
  `<pre><code>` (monoespaciado, `white-space:pre-wrap` + `word-break` para que NUNCA desborde).
- **Reproducción:** pasos concretos para verlo saltar.
- **Fix:** el patrón correcto (breve, con snippet si aplica).

> Si el sistema aún no está en producción, encuadrar: "se manifestará al operar con datos reales".

## B4. Riesgos de rendimiento / corrupción / inconsistencia

Mismo formato **Código actual + Problema + Impacto + Fix** que B3. Cubre:
- **Rendimiento:** N+1, búsquedas sin `limit`, computes que escriben, informes pesados en onchange…
- **Corrupción silenciosa:** cálculos erróneos sin excepción (IDs hardcodeados, resolución ambigua…).
- **Inconsistencias de estado:** flujos que dejan datos incoherentes (cancelaciones que tragan
  errores, reescritura de campos contables ya posteados —p. ej. `date_maturity` fuera del plazo de
  pago—, registros huérfanos…).

## B5. Matriz de seguridad

- **Tabla resumen por módulo:** modelos con ACL · ¿acceso a `base.group_user` (todo empleado)? ·
  record rules · nº `sudo()` · controllers y su `auth` · riesgo (alto/medio/bajo).
- **Hallazgos por gravedad** (CRÍTICO/ALTO/MEDIO/BAJO) con `file:line`: ACL demasiado abierto sobre
  modelos maestro/sensibles, `sudo()` que salta permisos, rutas sin comprobación de propiedad (IDOR),
  SQL interpolado, `exec`/`eval`.
- **Comprobar explícitamente:** ¿hay `security/*.xml`? (grupos y **record rules**). Su ausencia en un
  cliente multi-compañía = sin aislamiento por registro (dato que el técnico preguntará).

## B6. Inventario técnico por módulo

Tabla: por módulo, modelos NUEVOS (`_name`) vs modelos CORE extendidos (`_inherit`), nº de campos,
vistas, wizards, crons, ficheros JS/OWL. Notas: modelos que **invaden un namespace core** sin
prefijo, ausencia total de crons, etc.

## B7. Overrides de core y acoplamiento

- **Overrides de `create`/`write`/`unlink`/`default_get`/`_prepare_*`:** ¿llaman a `super()`?
  Señalar los que **NO** (reemplazan lógica core → frágiles ante actualización).
- **Computes core reemplazados sin `super()`** (p. ej. `_compute_price_unit`, `_compute_name`).
- **Modelos core "punto caliente"**: cuántos módulos custom extienden el mismo modelo (p. ej.
  `sale.order.line`) y con qué métodos → riesgo de orden de herencia (MRO) / onchange sin super que
  corta la cadena.
- **Monkey-patches globales JS/OWL** (`patch(SearchModel/FormRenderer/ListRenderer.prototype…)`) que
  afectan a **toda la instancia**, no solo al cliente.
- **Grafo de dependencias** entre módulos custom (hub(s)) + dependencias Enterprise/OCA ocultas o no
  declaradas en el manifest; qué módulos no se pueden mantener aislados.

## B8. Dependencias Python

Tabla: librería · versión pedida · módulo que la usa · ¿declarada en `external_dependencies`? ·
estado (mantenida/descontinuada/de pago/no portable). Señales típicas a cazar: librerías
**descontinuadas** (p. ej. `PyPDF2`→`pypdf`), **de pago** (aspose), **no portables** (`win32com` =
Windows-only → falla en Linux/Odoo.sh), o **sin declarar** en ningún manifest.

## B9. Estado de tests

Qué módulos tienen `tests/` y qué cubren; cuáles no. Destacar lo crítico sin cobertura (pricing,
flujos de fabricación, herramientas peligrosas) y proponer el **plan mínimo de tests** antes de
tocar esos módulos.

---

_Cada afirmación respaldada por `file:line`/fragmento. Distinguir **hecho** (código) de **hipótesis**
(pendiente de confirmar con consultoría/cliente). Sin mención a proveedores anteriores; sin horas
ni tarifas. Maquetación: bloques verticales, `<pre><code>` que envuelve, tablas ≤3 columnas o
bloques; verificación visual de overflow horizontal a PNG antes de entregar._
