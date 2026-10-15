# Impacto de la IA en el peso Studio ↔ Custom

> Actualiza el **principio anti-código** de la escalera `Estándar > Core > Studio > Custom`.
> NO cambia la filosofía Core (estándar primero, multi-cliente, configurable-no-a-medida, no
> hardcodear) — cambia el **peso relativo de Studio frente a Custom** para lógica no trivial.
> Este es el único punto donde vive esta doctrina; el resto la referencia.

## La premisa histórica (y por qué se mueve)

El orden `Studio > Custom` se apoyaba en una premisa: **el código es deuda técnica** — hay que
escribirlo, mantenerlo y migrarlo en cada cliente y cada versión, así que "el mejor código es
el que no se escribe" → preferir Studio (sin código) sobre un módulo a medida. Con IA esa
premisa se debilita por **dos pilares**:

### Pilar 1 — el coste del código se desploma
La IA **escribe, mantiene y migra** código de forma barata (junto con las skills de migración
y los diffs de versión). La ventaja clásica de Studio ("sin código → más fácil de mantener y
migrar") se reduce, porque la IA también migra el código de un módulo.

### Pilar 2 — gobernanza y calidad (el pilar decisivo)
Lo que era la VENTAJA de Studio ("cualquiera lo customiza, sin programar") es ahora su mayor
RIESGO:
- **Studio = muchas manos, a menudo no técnicas** → customizaciones de baja calidad, lentas,
  con bugs y, sobre todo, **inconsistencia de datos**. Y **sin gate de revisión**.
- **Módulo = 1 técnico + IA + las skills de calidad** → revisado, consistente, testeable,
  con nomenclatura/Core compliance y robusto a versión.

**Asimetría real en este ecosistema** (la evidencia del pilar 2): existe un pipeline de
calidad para *módulos* — `indaws-core-auditor`, `quality-gate`, `odoo-code-auditor`,
`dev-assessment` + gotchas de `odoo-deep-knowledge` — pero **NO existe ningún auditor de
calidad para customizaciones Studio**. Lo que entra por Studio queda ingobernado.

## El nuevo criterio

El criterio deja de ser **"¿hay código? evítalo"** y pasa a ser **"¿es código GOBERNADO?"**
— es decir: en un módulo (preferible `indaws_*` Core si es reutilizable), revisado por las
skills, testeable, con datos consistentes y migrable. Código gobernado ya no es la "deuda"
que era; Studio ingobernado sí es un pasivo.

## Cómo queda la escalera (matizada)

- **Estándar** y **Core**: sin cambios — siguen primero (un módulo Core reutilizable gana a
  reinventar nada).
- **Studio**: correcto para lo **trivial y de bajo riesgo** — una etiqueta, un campo simple,
  una vista o automatización sencilla — donde montar un módulo no compensa y el riesgo de
  gobernanza es bajo. Sigue siendo no-código y rápido para ese tramo.
- **Custom (módulo)**: para **lógica no trivial, validaciones, flujos, integridad de datos**,
  ahora **sube de peso frente a Studio**: con IA + skills es barato y, sobre todo, **gobernado**.
  Antes se evitaba por coste/deuda; hoy se prefiere a una maraña de Studio ingobernable.

## Qué NO cambia
- La filosofía Core íntegra: estándar antes que Core antes que custom; el cliente se adapta al
  sistema; multi-cliente por diseño; cero hardcoding; configuración, no a medida.
- Las Golden Rules (Import/Export → Core; Dashboards → Standard) y la migración 1:1 (custom tal
  cual, nunca Studio).
- Que el peor fallo sigue siendo **reimplementar lo que Odoo ya hace**.
