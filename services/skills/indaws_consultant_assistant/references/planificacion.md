# Planificación

## Qué es

El proceso de convertir el análisis funcional en un plan de proyecto con horas estimadas, fases temporales y un diagrama Gantt. Define cuánto tardará cada tarea y en qué orden se ejecutará.

## Cuándo aplica

Fase 2 (Análisis): Después de generar el análisis funcional y antes de crear el proyecto en Odoo.

## Qué debe saber el consultor

### Algoritmo peso → horas

El skill `project-plan` convierte los pesos de los items del checklist (1-5) en horas estimadas usando un algoritmo determinista:

- Peso null/0 se trata como peso 1
- Factores de ajuste por categoría (CONFIG vs DEV vs MIGR)
- Las tareas se agrupan por módulo (UNA tarea por módulo, no una por item)

### Fases del proyecto

Metodología Waterfall/Híbrido con solapamientos permitidos:

1. **Análisis** (ya completado al llegar aquí)
2. **Configuración**: Ejecución del día a día del consultor
3. **Formación**: Puede solaparse con configuración
4. **Desarrollo**: En paralelo, ejecutado por equipo técnico
5. **Migración de datos**: Puede empezar cuando se tenga la estructura
6. **Validación**: Después de completar configuración + formación
7. **Pre-producción**: Fase final antes del go-live
8. **Go-Live**: Evento puntual

### 3 Tareas obligatorias

Todo plan debe incluir estas tareas con horas calculadas automáticamente:

1. **Configuraciones generales**: Setup base del sistema (usuarios, empresa, idiomas)
2. **Soporte**: Horas reservadas para incidencias durante la implantación
3. **Reuniones**: Horas para kick-off, seguimientos semanales, presentaciones

### 2 Validaciones obligatorias

El `project-plan` tiene 2 checkpoints donde el usuario debe aprobar:

1. **Paso 2.5 — Validación de tareas y horas**: El usuario revisa la lista de tareas y las estimaciones antes de que se genere el cronograma
2. **Paso 3.5 — Validación de cronograma**: El usuario revisa el Gantt propuesto y las fases antes de que se generen los documentos finales

Estos checkpoints son OBLIGATORIOS. No se pueden saltar.

### Entregables

- **Tabla de planificación**: Tareas con horas, categorías, dependencias
- **Gantt**: Diagrama temporal con fases y dependencias
- **Detalle de tareas**: Checklist interno de cada tarea agrupada

## Qué debe hacer el consultor

1. Tener el análisis funcional completo y validado
2. Ejecutar `project-plan` con el análisis como input
3. Revisar las estimaciones de horas (Validación 1)
4. Revisar el cronograma propuesto (Validación 2)
5. Presentar el plan al cliente para aprobación

## Skills relacionados

- `project-analysis`: Genera el input necesario (análisis funcional)
- `project-plan`: Ejecuta la planificación completa
- `project-odoo-creator`: Siguiente paso (crear el proyecto en Odoo)

## Errores comunes

1. **59 tareas granulares**: El error más grave — las tareas deben agruparse por módulo, no una tarea por cada item del checklist
2. **No validar con el usuario**: Saltarse las validaciones obligatorias genera planes que no reflejan la realidad
3. **Olvidar las tareas obligatorias**: Config generales, Soporte y Reuniones son siempre necesarias
4. **No considerar solapamientos**: Desarrollo puede ir en paralelo con configuración
5. **Estimar sin contexto**: Las horas dependen del tamaño del cliente, complejidad y experiencia del equipo
