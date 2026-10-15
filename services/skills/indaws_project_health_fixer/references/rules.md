# project-health-fixer: Rules and Guidelines

## General Rules

- **SIEMPRE** pide los datos obligatorios antes de empezar (Paso 0)
- **NUNCA** ejecutes cambios en Odoo sin aprobacion explicita del consultor
- **NUNCA** recalcules metricas; usa los valores del JSON de `calculate_health.py`
- **NUNCA** inventes IDs de tareas o items; usa siempre los del JSON
- **OBLIGATORIO**: Presentar cada batch de cambios antes de ejecutar (Pasos 2.5, 4.5, 6.5)

## Correction Rules

- Las fechas se presentan en DD/MM/YYYY al consultor pero se envian a Odoo en YYYY-MM-DD
- Al redistribuir horas, nunca reducir `allocated_hours` de una tarea que ya tiene `effective_hours > allocated_hours`
- Al crear checklists, usar peso 1-2 para items simples, 3-5 para items complejos
- Los updates a multiples tareas con mismo campo pueden agruparse: `update_record.sh "project.task" '[ID1,ID2]' '{"field":"value"}'`

## Tone

- Interno (consultor a consultor), no para el cliente
- Practico y directo: "Propongo asignar deadline 15/03 a esta tarea porque..."
- Si hay desviacion: transparente pero orientado a soluciones

## Validations

**Interactivas (obligatorias)**:
1. Validacion Diagnostico (Paso 1.5)
2. Validacion Batch Gobernanza (Paso 2.5)
3. Validacion Plan Progreso (Paso 4.5)
4. Validacion Desviacion (Paso 6.5)

**Automaticas**:
5. JSON de metricas leido correctamente
6. Script `diagnose_issues.py` clasifica 3 tipos de issues
7. Score potencial calculado correctamente
8. Operaciones de Odoo ejecutadas y reportadas
9. Informe generado con todas las secciones aplicables

## Success Criteria

- JSON de metricas y datos raw leidos correctamente
- Diagnostico clasifica problemas en 3 categorias
- Score potencial calculado correctamente
- El consultor ha validado cada batch antes de ejecucion
- Correcciones aprobadas ejecutadas en Odoo
- Informe de correccion refleja todas las acciones
- Si aplica desviacion: opciones presentadas con calculos
- Consultor tiene plan claro de proximos pasos
