# Training Planner -- Mode PLANIFICAR Detail

## Paso P0: Recopilacion de Datos

Recopilar SIEMPRE antes de continuar:

1. **Credenciales Odoo**: URL, base de datos, API Key, usuario
2. **ID del proyecto** en Odoo
3. **ID del articulo de Knowledge con el analisis funcional** (knowledge.article padre con hijos por app)
4. **Parametros de formacion**:
   - Formato: presencial / online / mixto (default: online)
   - Horas maximas por semana (default: 2h)
   - Dia habitual de reuniones (default: lunes)
5. **Tarea de Reuniones/Formaciones**: ID de la tarea donde se crearan los checklists.
   - Si no lo sabe, buscar con: `search_records.sh project.task '[["name","ilike","reunion"],["project_id","=",<project_id>]]'`
   - Tambien: `search_records.sh project.task '[["name","ilike","formacion"],["project_id","=",<project_id>]]'`

**DETENTE** y espera la respuesta.

## Paso P1: Extraccion de Contexto del Proyecto

Ejecutar consultas via `odoo-pilot`:

1. **Tareas del proyecto** con fechas:
   ```bash
   $PILOT_DIR/search_records.sh "project.task" '[["project_id","=",<project_id>]]' '["name","planned_date_begin","date_deadline","stage_id"]' 100
   ```

2. **Checklists de cada tarea** (detectar items con `type = 'training'`):
   ```bash
   $PILOT_DIR/search_records.sh "xx.checklist.item" '[["project_id","=",<project_id>]]' '["name","task_id","weight","type","date_deadline","date_done"]' 500
   ```

3. **Analisis funcional** desde Knowledge:
   ```bash
   $PILOT_DIR/search_records.sh "knowledge.article" '[["id","=",<knowledge_id>]]' '["name","body"]' 1
   $PILOT_DIR/search_records.sh "knowledge.article" '[["parent_id","=",<knowledge_id>]]' '["name","body"]' 50
   ```

4. **Checklists existentes en tarea de formaciones** (para no duplicar):
   ```bash
   $PILOT_DIR/search_records.sh "xx.checklist.item" '[["task_id","=",<task_formaciones_id>]]' '["name","date_deadline","weight","type"]' 100
   ```

5. **Articulo Knowledge de Formaciones existente** (para no duplicar):
   ```bash
   $PILOT_DIR/search_records.sh "knowledge.article" '[["name","ilike","formacion"],["parent_id.name","ilike","<nombre_proyecto>"]]' '["name","id"]' 5
   ```

## Paso P2: Analisis y Diseno del Plan

Consultar `./references/reglas_planificacion.md` para reglas.

### a) Identificar formaciones necesarias
Para cada app del proyecto (extraida del analisis):
1. Determinar si requiere formacion (`./references/contenido_por_app.md`)
2. Estimar duracion segun complejidad y GAPs
3. Identificar la tarea de configuracion correspondiente y su `planned_date_end`

### b) Agrupar por afinidad funcional
- Agrupar apps que compartan flujo de negocio
- Respetar maximo de horas por sesion
- No agrupar si perfiles de asistentes difieren

### c) Programar en el calendario
1. Ubicar cada formacion la semana posterior a finalizacion de configuracion
2. Si varias formaciones caen en la misma semana, distribuir
3. Respetar maximo de horas semanales

### d) Rellenar semanas vacias con reuniones de seguimiento
**OBLIGATORIO**: Toda semana del proyecto debe tener una entrada:
1. Calcular rango total (primera semana -> ultima semana)
2. Identificar semanas sin formacion programada
3. Anadir "Reunion de seguimiento" en cada semana vacia
4. Usar dia habitual indicado

## Paso P2.5: Validacion del Plan (OBLIGATORIO)

**DETENTE** y presenta el plan con formato:
- Datos del proyecto (formato, max/semana, dia, rango)
- Calendario completo: tabla con Semana, Fecha, Tipo, Sesion, Duracion
- Detalle de cada formacion

Espera: Si / Ajustes. **NUNCA** crear checklists sin esta validacion.

## Paso P3: Creacion en Odoo

### a) Crear checklists en tarea de Reuniones/Formaciones

Por cada entrada del calendario, en orden cronologico:
```bash
$PILOT_DIR/create_record.sh "xx.checklist.item" '{
  "name": "<nombre>",
  "task_id": <task_formaciones_id>,
  "project_id": <project_id>,
  "weight": 1,
  "state": "pending",
  "type": "training",
  "date_deadline": "<YYYY-MM-DD>"
}'
```

**CRITICO — la fecha**: `x_studio_fecha_fin` se desdoblo en dos campos. La fecha de
la sesion va en `date_deadline` (la PREVISTA). `date_done` es cuando se impartio de
verdad y se rellena despues; usarlo aqui haria nacer el calendario entero marcado
como ya impartido.

**El tipo**: `training` para las formaciones. Para las **reuniones de seguimiento**
el tipo esta SIN DECIDIR — ninguno de los ocho valores encaja de forma obvia. Hasta
que se decida, crear las reuniones **sin `type`** y avisar al consultor.

Naming convention:
- Formaciones: `Formacion: {Nombre sesion}`
- Reuniones: `Reunion de seguimiento`

### b) Crear articulo Knowledge con el plan

**IMPORTANTE**: Usar `create` para crear el articulo. NUNCA hacer multiples `write` al body.

Reglas del HTML:
- Todo con estilos inline (font-family, color, padding, border)
- Tipografia: `Montserrat`, color: `#1C265D`
- Tablas con bordes inline, filas alternadas (#ffffff / #f8f9fa)
- Reuniones de seguimiento en gris cursiva
- NO usar clases CSS ni Bootstrap

### c) Presentar resultados
Lista de checklists creados con IDs + articulo Knowledge con URL.
