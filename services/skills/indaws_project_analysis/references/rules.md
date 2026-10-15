# project-analysis: Reglas y criterios de decisión

> La **estructura visual** de las 7 secciones está en `templates/analisis_app.md`.
> Este fichero contiene las **reglas de decisión** que NO son evidentes desde el template.

---

## Reglas generales

- **ENFOQUE**: Detectar GAPs (lo que NO es estándar) y definir soluciones. No describir extensamente funcionalidades estándar.
- Basa TODAS las afirmaciones sobre requisitos en la información proporcionada por el cliente.
- **NO hacer referencias al cuestionario**: NUNCA uses frases como "Ref: Pregunta X.Y", "Basándonos en el cuestionario" o similares. Presenta los requisitos de forma directa.
- **Versión de Odoo**: Pregunta SIEMPRE la versión al inicio. Basa el análisis en esa versión específica.
- **NO usar módulos OCA**: NUNCA sugieras módulos OCA ni de terceros. Solo Odoo estándar, Indaws Core o desarrollo a medida.
- **SÍ usar paquetes de Indaws Core**: SIEMPRE verifica `paquetes_indaws_core.md` ANTES de sugerir [DEV].
- Español profesional, claro y conciso.

---

## Reglas por sección

### Sección 1 — Introducción
- Aplica el enfoque indicado en la guía específica de la app (`guias_por_aplicacion.md`).
- Máximo 2-3 párrafos. No describir extensamente lo que Odoo ya hace.

### Sección 2 — Análisis de Requisitos y GAPs
- **FORMATO ÚNICO OBLIGATORIO**: narrativo con subsecciones 2.1, 2.2, 2.3… Una subsección por requisito. **NUNCA usar tablas.** **NUNCA mezclar formatos.**
- Cada subsección tiene exactamente estos 5 campos en este orden: Requisito del Cliente, Funcionalidad Estándar de Odoo, GAP Identificado, Solución Requerida, Justificación de la solución.
- **GAP Identificado** usa uno de 3 marcadores fijos: `✓ Cubierto POR COMPLETO` | `⚠ Cubierto PARCIALMENTE` | `✗ NO cubierto`.
- **ANTES de marcar [DEV]**: verifica `paquetes_indaws_core.md`. Si existe paquete que lo resuelva, marca `[CORE: nombre_paquete]`.
- **Funcionalidades estándar**: INCLUYE obligatoriamente las listadas en la guía de la app.
- **Puntos críticos**: REVISA los indicados en la guía y analízalos en detalle.
- **Omisión**: Si el cliente NO usa una funcionalidad (ej: no usa lotes, no necesita kits), OMITE la subsección. No la incluyas marcada como "no aplica".
- **Orden**: las subsecciones siguen el flujo lógico del proceso de negocio, NO la categoría de solución.
- Consulta `ejemplos_gaps.md` para inspiración de GAPs comunes.

### Secciones 3, 4, 5 — Resúmenes ejecutivos
- Son **resúmenes**, no listas de tareas. Las tareas van en sección 6.
- **FORMATO FIJO por subapartado** (sin excepciones): UN párrafo de 2-4 frases + UNA línea final `Ver detalle del requisito en sección 2.X.`
- **NUNCA** usar bullets anidados, checklists, tablas ni enumerar campos dentro de cada subapartado.
- Un subapartado por cada requisito correspondiente: sección 3 → uno por cada [STUDIO/AUTO] de sección 2, sección 4 → uno por cada [DEV], sección 5 → uno por cada migración.
- Si no hay ítems de esa categoría, escribir literalmente: `"No se han identificado {automatizaciones | desarrollos a medida | migraciones de datos} para esta aplicación."` y omitir subapartados.
- **Sección 4 solo para [DEV]**: Los paquetes [CORE] NO van en sección 4 (aparecen bajo el checklist de sección 6).
- **Sección 5**: Incorpora las recomendaciones de migración de la guía de la app.

### Sección 6 — Checklist general
- Incluye TODAS las tareas **EXCEPTO** las que se detallan en sección 7.
- **Qué va en sección 7 (y por tanto se excluye de 6)**:
  - TODOS los desarrollos [DEV], independientemente del peso.
  - Tareas [MIGR] o [STUDIO/AUTO] con peso >= 4.
- Orden obligatorio por categoría: **[ANLS] → [CONFIG] → [MIGR] → [STUDIO/AUTO] → [DEV] → [TEST] → [TRAIN]**.
- Dentro de cada categoría, ordena por secuencia lógica de implementación (no por peso).
- **INCLUYE obligatoriamente** las tareas indicadas en la guía de la app (`guias_por_aplicacion.md`).
- **Paquetes Core**: Si hay paquetes [CORE] identificados, añade el listado "Paquetes de Indaws Core Requeridos" justo debajo del checklist con precio y coste total. Si no hay, OMITE esa subsección.

### Sección 7 — Tareas detalladas
- Criterio estricto: SOLO [DEV] (todos) + [MIGR]/[STUDIO/AUTO] con peso >= 4.
- NUNCA crear tareas detalladas para [CONFIG] simples ni tareas de peso bajo.
- Cada tarea DEBE tener:
  - Descripción Detallada (paso a paso)
  - Por qué es necesario (el GAP que cubre)
  - Módulos Odoo Afectados
  - Categoría y Peso
  - **Checklist interno obligatorio** con ítems [DEV], [TEST] y un [DOC] final.
- Si una tarea tiene varios componentes (modelo + lógica + interfaz), crea un ítem [DEV] por cada componente principal.

---

## Categorías y pesos

- **Categorías**: [ANLS], [CONFIG], [STUDIO/AUTO], [CORE], [DEV], [MIGR], [TEST], [TRAIN]
- **Pesos**: 1 (muy simple) → 5 (muy complejo)
- Definiciones completas en `categorias_y_pesos.md`.

---

## Mapeo rápido GAP → [CORE]

Antes de marcar [DEV], comprueba si alguno de estos paquetes aplica (catálogo completo en `paquetes_indaws_core.md`):

| Necesidad | Paquete [CORE] |
|-----------|----------------|
| Integración con APIs externas | Framework de API |
| PDFs personalizados / informes custom | Paquete de generador de PDFs a medida |
| Portal para empleados no usuarios | Portal de empleado |
| Aprobación multinivel de ventas | Flujo de aprobación de ventas |
| Descuentos múltiples en cascada | Gestión de Descuentos Múltiples |
| Migración/integración masiva de datos | Framework de Migración e Integración de Datos |

---

## Validaciones y manejo de errores

- Verificar existencia de referencias antes de usarlas.
- Si falta `guias_por_aplicacion.md`: STOP e informar.
- Si falta `paquetes_indaws_core.md`: continuar con aviso (sin sugerencias Core).
- Errores de Drive (Modo A): manejar gracefully y sugerir Modo B.
- Salida SIEMPRE en Markdown estándar.
