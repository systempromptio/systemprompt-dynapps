# project-health-monitor: Rules and Guidelines

## General Rules

- **SIEMPRE** pide los datos obligatorios antes de empezar (Paso 0)
- **OBLIGATORIO**: Presenta dashboard de validacion (Paso 2.5) y espera aprobacion
- **NUNCA** generes informe completo sin validacion del usuario
- **NUNCA** inventes datos. Si falta informacion, indicalo claramente
- **SIEMPRE** consulta archivos de referencia para algoritmos y patrones
- Si `fetch_project_data.sh` falla, intentar llamadas manualmente con odoo-pilot
- Adapta lenguaje al interlocutor (PM tecnico vs directivo)

## Data and Metrics

- Pesos null o 0 en checklists -> tratar como peso = 1
- Fechas en formato DD/MM/YYYY para el informe
- Redondear porcentajes a 1 decimal
- Redondear horas a 2 decimales
- Health Score es entero (0-100)
- Si hay discrepancias horas tareas vs timesheets, usar timesheets como fuente primaria

## Format

- Espanol profesional, claro y conciso
- Tablas legibles en Markdown estandar
- Informe autonomo (comprensible sin documentacion adicional)
- Nombre fichero: `informe_salud_{proyecto}_{YYYYMMDD}.md` (sin espacios, lowercase)

## Odoo Connection

- Usa SIEMPRE `fetch_project_data.sh` como metodo principal
- Si no disponible, usar scripts de `odoo-pilot` directamente
- Verificar que `xx.checklist.item` existe antes de leer
- No asumir que todos los campos custom existen; manejar ausencias gracefully

## Validations

**Interactiva (obligatoria)**:
1. Validacion Dashboard (Paso 2.5): usuario confirma datos y metricas

**Automaticas**:
2. Proyecto encontrado con fechas validas
3. Health Score calculado con formula correcta
4. Escenario clasificado segun umbrales
5. Bottlenecks ordenados por Risk Score descendente
6. Plan de accion corresponde al escenario detectado
7. Informe contiene las 8 secciones del template
8. Avisos de datos parciales si faltan checklists o timesheets

## Success Criteria

- Conexion a Odoo y extraccion correcta
- T%, C%, H% calculados correctamente
- Health Score refleja estado real
- Usuario ha validado dashboard
- Escenario identificado correctamente
- Bottlenecks con datos concretos (tareas, responsables)
- Plan de accion especifico y accionable (no generico)
- Informe Markdown completo, autonomo y bien formateado
