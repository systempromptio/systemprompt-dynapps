# Capa 2a (comportamiento), Capa 2b (navegador) y Capa 3 (visual)

La Capa 1 valida qué **existe**. Estas capas validan qué **funciona**, qué **funciona al
usarlo** y qué **se ve**, y se aplican **selectivamente** (son caras) sobre lo que la metadata
no cierra.

## Capa 2a — Comportamiento por API (delegar a `common-skills:odoo-flow-tester`)

La metadata no captura lógica compute/onchange ni el resultado de un flujo. Para
eso se ejecuta el **mismo set de flujos** contra **origen** y **migrado** y se
comparan resultados.

**Alcance (no ejecutar todo — sería carísimo):**
- Los flujos de los **módulos custom-críticos** (los que tocan el negocio).
- Los **UNKNOWN** de la Capa 1 que sean de comportamiento, no de metadata.
- Los flujos afectados por diffs marcados INTER_VERSION que cambien comportamiento
  (para confirmar que el cambio de Odoo no rompe el proceso del cliente).

**Cómo:**
1. Delegar en `common-skills:odoo-flow-tester` (sub-modo **Migration Integrity**), que
   ya sabe ejecutar flujos vía API con reversión automática y comparar snapshots
   `v_prev ↔ v_new`.
2. Pasar como flujos custom los de los módulos custom-críticos; como documento de
   análisis, el `parity_report.md` de la Capa 1 (así el tester prioriza lo que la
   Capa 1 marcó dudoso).
3. Un flujo que da distinto resultado en origen vs migrado y **no** se explica por
   un INTER_VERSION documentado ⇒ **regresión de comportamiento** → al informe.

> No dupliques la validación de datos transaccionales (saldos, asientos): eso ya
> lo cubre el propio Migration Integrity de `odoo-flow-tester`. Aquí solo se usa
> para **paridad de comportamiento** de funcionalidad.

## Capa 2b — Navegador (Playwright MCP)

La Capa 2a valida el **resultado** de un flujo por API. No ve lo que vive en el cliente web:
plantillas OWL con `t-inherit`, patches de componentes, widgets, vistas Studio, wizards y
botones que disparan JS. Un `xpath` heredado que no casa deja el pipeline verde y la
funcionalidad fuera de la pantalla.

**Alcance — dirigido por hallazgos, no catálogo fijo** (5–15 flujos):
1. **DEACTIVATED reactivados** — una vista reapagada/reencendida puede duplicar columnas o
   tumbar el registry; es la fuente número uno de daño silencioso.
2. **Vistas Studio** — sin código en disco, invisibles a cualquier análisis estático.
3. **Assets portados**: `t-inherit`, patches, widgets, SCSS de tema.
4. **UNKNOWN de la Capa 1** que sean de interacción, no de metadata.
5. **TODOs del port** resueltos "a ojo" + wizards/botones de los custom-críticos.

Si la lista supera 15, se recorta por prioridad y el resto va a la Capa 3. No se amplía el
presupuesto.

**Reglas no negociables** (detalle completo en `browser-layer.md`):
- **Nunca es un gate.** Produce hallazgos; nunca un pass/fail que bloquee una entrega.
- **Nunca va a CI.** Un hallazgo confirmado se cubre, si acaso, con un test Python barato.
- **≤12 llamadas por flujo.** Excedido → *no concluyente* → a la Capa 3. Un no concluyente
  **no es un OK**.
- Un flujo que se comporta distinto y no se explica por un INTER_VERSION documentado ⇒
  **regresión**, por el default de paridad 1:1.

**Si Playwright MCP no está disponible**, la capa se omite con una línea en el informe.

## Capa 3 — Visual / maquetación (checklist humano)

Lo visual —**píxeles**: render de informes, PDFs, posición de columnas, fidelidad de
layout entre versiones— **no se automatiza**: es token-pesado y flaky. Se deriva a un
**checklist humano**, siguiendo el patrón de `common-skills:odoo-flow-coverage-planner`.

**Qué entra en el checklist (por cada elemento visual relevante):**
- Ruta exacta en la UI (menú → vista/acción/informe).
- Cómo probarlo (pasos).
- Criterio de aceptación (qué debe verse, comparado con la versión antigua).

**Semillas del checklist:**
- Los flujos que la Capa 2b marcó **NO CONCLUYENTE** (presupuesto agotado).
- Los **informes/QWeb** (categoría `reports` del inventario) que existan en ambos
  → verificar maquetación a ojo (los cambios de layout entre versiones son la
  fuente típica de "se ve distinto").
- Las vistas marcadas **DEACTIVATED** y reactivadas de forma curada → confirmar
  que renderizan sin romper el frontend. (Que *carguen* sin error lo comprueba ya la Capa 2b;
  aquí se juzga solo el **aspecto**.)
- Cualquier diff **INTER_VERSION** en vistas de cara al usuario → confirmar que el
  cambio estándar de Odoo es aceptable para el cliente (o escalar a negocio).

El resultado del checklist se adjunta al informe de decisiones como la parte que
requiere validación humana; no bloquea el gate automático, pero sí la entrega.
