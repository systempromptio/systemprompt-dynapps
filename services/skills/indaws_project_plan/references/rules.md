# project-plan: Rules and Guidelines

## General Rules

- **SIEMPRE** pide datos obligatorios antes de empezar (Paso 0)
- **OBLIGATORIO**: Validacion 1 (tareas/horas) y Validacion 2 (cronograma) antes de generar documentos
- **NUNCA** inventes tareas que no esten en el analisis (excepto 3 genericas obligatorias)
- **NUNCA** generes documentos sin las dos validaciones
- **VALIDA** coherencia horas totales vs bolsa (+-15%)
- Si exceden bolsa: informar y sugerir priorizacion
- Si menores que bolsa: sugerir donde usar restantes (testing, formacion, contingencia)

## Format Rules

- Espanol profesional, claro y conciso
- Planificar en semanas naturales (lunes a viernes)
- Fechas: DD/MM/YYYY
- Tareas de desarrollo: prefijo `[DEV]`
- Tareas de migracion: prefijo `[MIGR]`

## Validations

**Interactivas (obligatorias)**:
1. Validacion 1 (Paso 2.5): tareas y horas
2. Validacion 2 (Paso 3.5): cronograma y fechas

**Automaticas**:
3. Suma horas = Bolsa +-15%
4. Tareas en orden cronologico
5. Dependencias respetadas (config antes que migracion)
6. Go-live 1 mes antes del fin
7. Tareas genericas incluidas
8. Gantt coherente con tabla
9. Tareas agrupadas por modulo (NO granulares)
10. Solo individuales para seccion 7 del analisis
11. Resumen con checklist de cada tarea

## Success Criteria

- Tareas extraidas correctamente del analisis
- Tareas agrupadas por modulo (una por modulo)
- Solo seccion 7 como individuales
- Usuario ha validado horas (V1) y cronograma (V2)
- Horas coherentes con bolsa
- Cronograma respeta dependencias
- Tabla + Gantt + Resumen con checklists generados
- Plan validado y ejecutable
