# Reglas de Planificacion de Formaciones

## Principios Generales

1. **Una reunion por semana**: Toda semana del proyecto debe tener exactamente una entrada en el checklist — o formacion o reunion de seguimiento
2. **Formaciones al final de la configuracion**: Las formaciones de cada app se programan para la semana en que termina (o la siguiente a) la configuracion de esa app
3. **Duracion por sesion**: 1-1.5h por formacion online, 2-3h presencial
4. **Maximo semanal**: Respetar el limite de horas semanales indicado por el consultor (default: 2h/semana)
5. **Reuniones de seguimiento**: Toda semana sin formacion tiene una "Reunion de seguimiento" en el checklist

## Agrupacion por Afinidad Funcional

Agrupar apps relacionadas en una misma sesion cuando:
- Comparten flujo de negocio (ej: Ventas + Facturacion)
- La duracion combinada no excede el maximo por sesion
- Tiene sentido didactico verlas juntas

**Agrupaciones tipicas**:
- Ventas + CRM (flujo comercial)
- Compras + Inventario (flujo de aprovisionamiento)
- Facturacion + Contabilidad (flujo financiero)
- RRHH + Ausencias + Nominas (flujo de personas)
- Proyecto + Partes de horas (flujo de servicios)
- Fabricacion + PLM (flujo productivo)

**NO agrupar** si:
- Los perfiles de asistentes son muy diferentes
- Una de las apps tiene complejidad alta (muchos GAPs o custom)
- La duracion combinada excede el maximo

## Calculo de Fechas

### Logica de programacion
1. Obtener todas las tareas del proyecto con `planned_date_begin` y `planned_date_end`
2. Para cada app con formacion, identificar la tarea de configuracion correspondiente
3. Programar la formacion para la semana en que termina la configuracion (o la siguiente)
4. Si varias formaciones caen en la misma semana, distribuir respetando el maximo semanal

### Relleno de semanas vacias
1. Calcular el rango total del proyecto (primera tarea → ultima tarea)
2. Identificar todas las semanas del rango
3. Cada semana sin formacion programada → añadir "Reunion de seguimiento"
4. La fecha del checklist es el dia de la semana habitual de reuniones (preguntar al consultor, default: lunes)

### Formato de fechas en checklists
- Campo: `date_deadline` (date) — la fecha PREVISTA de la sesion
- Formato: `YYYY-MM-DD`
- Usar el lunes de cada semana como fecha por defecto (o el dia que indique el consultor)

## Checklists en Odoo

### Modelo
- Modelo: `xx.checklist.item`
- Campos principales:
  - `name`: Nombre del checklist (ej: "Formacion: Ventas y CRM")
  - `task_id`: ID de la tarea de Reuniones/Formaciones
  - `project_id`: ID del proyecto
  - `weight`: Peso (siempre 1 para formaciones y reuniones)
  - `date_deadline`: Fecha PREVISTA de la sesion
  - `date_done`: Cuando se impartio de verdad. NO se rellena al crear
  - `state`: Selection. Un item nuevo nace en `pending`
  - `type`: Selection cerrada. `training` para formaciones; SIN DECIDIR para reuniones

### Naming convention
- Formaciones: `Formacion: {Nombre de la sesion}` (ej: "Formacion: Ventas y CRM")
- Reuniones: `Reunion de seguimiento` (siempre el mismo nombre)

### Orden de creacion
1. Ordenar cronologicamente todas las entradas (formaciones + reuniones)
2. Crear en ese orden para que aparezcan ordenadas en Odoo

## Articulo Knowledge

### Reglas de creacion
- **SIEMPRE** usar `create` para crear el articulo (nunca multiples `write` al body)
- Un unico `create` con todo el HTML en el campo `body`
- Si necesitas actualizar, eliminar el articulo y crear uno nuevo (evita corrupcion de `html_field_history`)

### Estilos obligatorios (inline)
Odoo 19 no procesa CSS externo ni clases Bootstrap. Todo debe ir con estilos inline.

**Tipografia**: `font-family: 'Montserrat', sans-serif`
**Color principal**: `#1C265D` (azul Indaws)
**Color secundario**: `#875A7B` (violeta Odoo)

### Estructura del articulo
1. Titulo del plan
2. Informacion general (formato, duracion, frecuencia)
3. Tabla de formaciones con columnas: Semana, Fecha, Sesion, Duracion, Descripcion
4. Detalle de cada sesion con objetivos y contenido

### Template HTML
Ver `./templates/knowledge_plan.md` para el template completo con estilos inline.

## Validaciones

### Antes de crear en Odoo
1. El plan ha sido validado por el consultor
2. La tarea de Reuniones/Formaciones existe y tiene ID valido
3. El articulo padre de Knowledge existe (buscar "Formaciones" en el proyecto)
4. Todas las fechas son coherentes (no en el pasado, dentro del rango del proyecto)
5. Toda semana del proyecto tiene exactamente una entrada

### Despues de crear
1. Verificar que todos los checklists se han creado correctamente
2. Verificar que el articulo Knowledge es accesible y se renderiza bien
3. Presentar resumen al consultor con IDs creados
