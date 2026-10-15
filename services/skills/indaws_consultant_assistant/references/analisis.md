# Análisis Funcional

## Qué es

El proceso de recopilar las necesidades del cliente mediante cuestionarios y sesiones presenciales/remotas, para producir un documento de análisis funcional que define qué se va a implementar, cómo y con qué esfuerzo.

## Cuándo aplica

Fase 2 (Análisis): Después del kick-off y antes de la planificación. Es el paso más crítico del proyecto porque define todo lo que viene después.

## Qué debe saber el consultor

### Flujo completo del análisis

```
questionnaire-generator → Sesiones de análisis (manual) → project-analysis → project-plan
```

1. **Cuestionarios** (`questionnaire-generator`): Se generan Google Docs por cada app contratada.
2. **Sesiones de análisis** (manual): El consultor rellena los cuestionarios con el cliente. Pueden ser presenciales o remotas.
3. **Análisis funcional** (`project-analysis`): Genera el documento de análisis completo en Markdown por app.
4. **Plan de proyecto** (`project-plan`): Calcula horas, distribuye en el tiempo y genera Gantt.

### Filosofía del análisis: Detección de GAPs

El objetivo principal del análisis es detectar lo que NO es estándar de Odoo:

- **GAP**: Necesidad del cliente que Odoo no cubre de serie
- **Solución**: Para cada GAP, aplicar la escalera de decisión (estándar → Core → custom)
- **Categorías de checklist**: ANLS, CONFIG, STUDIO/AUTO, CORE, DEV, MIGR, TEST, TRAIN

### Categorías de items del checklist

| Categoría | Significado | Quién ejecuta |
|-----------|-------------|---------------|
| ANLS | Pendiente de análisis adicional | Consultor |
| CONFIG | Configuración estándar de Odoo | odoo-configurator |
| STUDIO/AUTO | Campo o acción via Odoo Studio | odoo-configurator |
| CORE | Paquete Indaws Core necesario | Consultor + equipo técnico |
| DEV | Desarrollo a medida | dev-assessment → equipo técnico |
| MIGR | Migración/importación de datos | importation-data-assistant |
| TEST | Testing/validación necesaria | odoo-flow-tester |
| TRAIN | Formación al usuario | training-planner |

### Estructura del documento de análisis

Por cada aplicación:
1. Resumen ejecutivo (narrativa)
2. Funcionalidades estándar confirmadas
3. GAPs detectados con solución propuesta
4. Checklist de implementación (categorizado)
5. Notas y decisiones pendientes
6. Tareas detalladas (si hay [DEV] o complejas)

### Reglas del análisis

- **Versión de Odoo**: Preguntar al inicio, condiciona funcionalidades disponibles
- **No sugerir módulos OCA**: Solo Odoo estándar o Indaws Core
- **Verificar Indaws Core**: Antes de proponer desarrollo, consultar el catálogo de paquetes
- **Secciones 3-5**: Organizadas en subapartados por funcionalidad

## Qué debe hacer el consultor

1. Ejecutar `questionnaire-generator` para preparar los cuestionarios
2. Planificar las sesiones de análisis con el cliente
3. Rellenar los cuestionarios durante las sesiones
4. Ejecutar `project-analysis` con los cuestionarios rellenados
5. Revisar y validar el análisis antes de presentarlo al cliente
6. Ejecutar `project-plan` para generar la planificación

### Normas de entrega y presupuesto

- **El análisis incluye la planificación del proyecto**: No son fases separadas; el análisis debe terminar con la planificación.
- **Confirmar el presupuesto**: Norma general: mantener las mismas horas que el presupuesto inicial, dejando para fases posteriores cualquier trabajo adicional identificado.
- **Entrega por aplicación**: El análisis se entrega a medida que se completa cada aplicación, no al final de todo el proceso.
- **Forma de entrega**: Se entrega compartiendo el artículo de Conocimiento (Knowledge) de Odoo correspondiente.

## Skills relacionados

- `questionnaire-generator`: Genera los cuestionarios por app en Google Drive
- `project-analysis`: Genera el documento de análisis funcional
- `project-plan`: Genera la planificación a partir del análisis
- `kick-off-presentation`: Paso previo (ya ejecutado)

## Errores comunes

1. **Sugerir módulos OCA**: Indaws no trabaja con OCA, solo con Odoo estándar e Indaws Core
2. **No verificar Indaws Core**: Hay 40+ paquetes que pueden cubrir GAPs sin necesidad de desarrollo
3. **No categorizar correctamente los items**: Un item CONFIG marcado como DEV genera trabajo innecesario en el equipo técnico
4. **Análisis demasiado granular**: Agrupar por módulo/app, no crear un item por cada campo individual
