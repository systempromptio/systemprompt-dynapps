# ANÁLISIS FUNCIONAL - {NOMBRE_APLICACION}

**Cliente**: {NOMBRE_CLIENTE}
**Fecha**: {FECHA}
**Versión de Odoo**: {VERSION_ODOO}

---

## 1. Introducción

{Descripción breve del propósito estándar de la aplicación en Odoo (2-3 párrafos máximo)}

{Exposición de los objetivos del cliente para esta aplicación basándote en el cuestionario}

{APLICAR el enfoque indicado en la guía específica de la app desde references/guias_por_aplicacion.md}

---

## 2. Análisis de Requisitos y Detección de GAPs

{ENFOQUE PRINCIPAL: Identifica qué NO es estándar en Odoo y por qué}

{IMPORTANTE: NO hacer referencias explícitas al cuestionario. Presenta los requisitos del cliente de forma directa}

{CONSULTAR: references/ejemplos_gaps.md para inspiración de GAPs comunes}

{CONSULTAR: references/shared/paquetes_indaws_core.md ANTES de sugerir [DEV] - verificar si existe paquete que resuelva el GAP}

{FORMATO OBLIGATORIO: narrativo con subsecciones 2.1, 2.2, 2.3… Una subsección por cada requisito analizado. NO usar tablas, NO mezclar formatos}

### 2.1 {Nombre del Requisito}

- **Requisito del Cliente**: {¿Qué necesita hacer el cliente? Descripción directa, 1-3 frases}
- **Funcionalidad Estándar de Odoo**: {¿Qué ofrece Odoo en la versión indicada? Describir con precisión lo que cubre el estándar}
- **GAP Identificado**: {✓ Cubierto POR COMPLETO | ⚠ Cubierto PARCIALMENTE: explicar qué falta | ✗ NO cubierto: explicar QUÉ falta y POR QUÉ}
- **Solución Requerida**: [CONFIG] / [STUDIO/AUTO] / [CORE: nombre del paquete] / [DEV] / [MIGR]
- **Justificación de la solución**: {1-2 frases explicando por qué esa categoría y no otra — p.ej. por qué [DEV] en lugar de [CORE]}

### 2.2 {Siguiente Requisito}

{Misma estructura. Repetir 2.X para cada requisito del cliente}

{REGLAS OBLIGATORIAS de esta sección:}
- {INCLUIR las funcionalidades estándar listadas en la guía de la app (guias_por_aplicacion.md)}
- {REVISAR los puntos críticos indicados en la guía y analizarlos en detalle}
- {OMITIR completamente las funcionalidades que el cliente NO utiliza (no escribir "no aplica")}
- {Ordenar subsecciones por flujo lógico del proceso de negocio, no por categoría de solución}

---

## 3. Resumen de Automatizaciones y Configuraciones Avanzadas ([STUDIO/AUTO])

{REGLA OBLIGATORIA: Crear UN subapartado por cada requisito marcado como [STUDIO/AUTO] en sección 2. Si no hay ninguno, escribir "No se han identificado automatizaciones ni configuraciones avanzadas para esta aplicación." y omitir subapartados}

{FORMATO FIJO por subapartado: 1 párrafo de 2-4 frases + 1 línea de referencia a la sección 2.X correspondiente. NO listar tareas, NO incluir checklist, NO usar bullets anidados}

### 3.1 {Nombre de la Automatización/Configuración}

{Párrafo único de 2-4 frases describiendo: qué proceso se automatiza, qué campos/reglas Studio se necesitan, complejidad estimada (baja/media/alta).}

Ver detalle del requisito en sección 2.X.

### 3.2 {Siguiente Automatización}

{Misma estructura: párrafo único + línea de referencia}

---

## 4. Resumen de Desarrollos a Medida Requeridos ([DEV])

{REGLA OBLIGATORIA: Crear UN subapartado por cada requisito marcado como [DEV] en sección 2. NO incluir paquetes [CORE] aquí (esos aparecen bajo el checklist de sección 6). Si no hay desarrollos [DEV], escribir "No se han identificado desarrollos a medida para esta aplicación." y omitir subapartados}

{FORMATO FIJO por subapartado: 1 párrafo de 2-4 frases + 1 línea de referencia a la sección 2.X correspondiente. NO listar tareas}

### 4.1 {Nombre del Desarrollo}

{Párrafo único de 2-4 frases describiendo: qué funcionalidad requiere código Python, por qué no se puede hacer con Studio ni con paquetes Core, integraciones o vistas complejas si aplica, complejidad técnica (media/alta/muy alta).}

Ver detalle del requisito en sección 2.X.

### 4.2 {Siguiente Desarrollo}

{Misma estructura: párrafo único + línea de referencia}

---

## 5. Resumen de Migración de Datos ([MIGR])

{REGLA OBLIGATORIA: Crear UN subapartado por cada tipo de dato a migrar identificado en sección 2 o en la guía de migración de la app. Si no hay migraciones, escribir "No se han identificado migraciones de datos para esta aplicación." y omitir subapartados}

{FORMATO FIJO por subapartado: 1 párrafo de 2-4 frases + 1 línea de referencia a la sección 2.X correspondiente}

### 5.1 {Tipo de Datos a Migrar}

{Párrafo único de 2-4 frases describiendo: qué datos específicos (maestros/transacciones/históricos), sistema origen, transformaciones necesarias, volumen estimado, riesgos principales.}

Ver detalle del requisito en sección 2.X.

### 5.2 {Siguiente Migración}

{Misma estructura: párrafo único + línea de referencia}

---

## 6. Checklist General de Tareas

{CONSULTAR: references/categorias_y_pesos.md para definiciones y guía de pesos}

{IMPORTANTE: NO incluir en este checklist las tareas [DEV] detalladas en sección 7}

{INCLUIR OBLIGATORIAMENTE las tareas estándar indicadas en la guía de la app}

### Orden de categorías:
[ANLS] → [CONFIG] → [MIGR] → [STUDIO/AUTO] → [DEV] → [TEST] → [TRAIN]

| Categoría | Tarea | Peso |
|-----------|-------|------|
| [ANLS] | {Descripción del análisis requerido} | {1-5} |
| [CONFIG] | {Verbo + Objeto + Contexto específico} | {1-5} |
| [CONFIG] | {Siguiente configuración} | {1-5} |
| [MIGR] | {Migración de datos específica} | {1-5} |
| [STUDIO/AUTO] | {Automatización o campos custom} | {1-5} |
| [DEV] | {Solo desarrollos simples (peso < 4) que NO están en sección 7} | {1-3} |
| [TEST] | {Validación de flujos o funcionalidades} | {1-5} |
| [TRAIN] | {Formación a grupo de usuarios específico} | {1-5} |

{Dentro de cada categoría, ordenar por secuencia lógica de implementación}

### Paquetes de Indaws Core Requeridos

{INCLUIR ESTA SECCIÓN solo si hay paquetes [CORE] identificados}

Los siguientes paquetes de Indaws Core se instalarán para cubrir funcionalidades no estándar de Odoo:

1. **[CORE] {Nombre del Paquete}** (Precio: {XXXX}€)
   - Cubre: {Funcionalidad que resuelve}
   - Requisitos relacionados: Ver sección 2.X

2. **[CORE] {Siguiente Paquete}** (Precio: {XXXX}€)
   - Cubre: {Funcionalidad que resuelve}
   - Requisitos relacionados: Ver sección 2.Y

**Coste total de paquetes Indaws Core**: {TOTAL}€

{Si NO hay paquetes [CORE], OMITIR esta subsección completamente}

---

## 7. Tareas Detalladas

{CREAR tareas detalladas SOLO para:}
- {TODOS los desarrollos [DEV] (independientemente del peso)}
- {Tareas complejas: Peso >= 4 de categorías [MIGR] o [STUDIO/AUTO]}

{NO crear tareas detalladas para [CONFIG] ni tareas de peso bajo}

### 7.1 {Título de la Tarea}

**Descripción Detallada**:
{Explicación completa de qué hay que hacer exactamente, paso a paso}

**Por qué es necesario**:
{Explicación del GAP que cubre esta tarea}

**Módulos Odoo Afectados**:
{Lista de módulos/modelos de Odoo que se verán afectados}

**Categoría y Peso**: [{CATEGORÍA}] - Peso {X}

**Checklist de Implementación**:

| # | Tipo | Descripción | Estado |
|---|------|-------------|--------|
| 1 | [DEV] | {Descripción del desarrollo 1} | ⬜ Pendiente |
| 2 | [DEV] | {Descripción del desarrollo 2} | ⬜ Pendiente |
| 3 | [TEST] | {Tests funcionales a realizar} | ⬜ Pendiente |
| 4 | [DOC] | Generación de documentación funcional | ⬜ Pendiente |

### 7.2 {Siguiente Tarea Detallada}

{Repetir estructura para cada tarea compleja identificada}

---

{FIN DEL ANÁLISIS FUNCIONAL}
