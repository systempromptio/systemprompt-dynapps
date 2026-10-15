# Questionnaire Generator -- Workflow Detail

## Aplicaciones Disponibles

| ID | Aplicacion | Plantilla | Notas |
|----|-----------|-----------|-------|
| 1 | Configuraciones generales | `configuraciones-generales.md` | |
| 2 | CRM | `crm.md` | |
| 3 | Ventas | `ventas.md` | Seccion construccion condicional |
| 4 | Facturacion | `facturacion.md` | Seccion construccion condicional |
| 5 | Contabilidad | `contabilidad.md` | |
| 6 | Helpdesk | `helpdesk.md` | |
| 7 | Integraciones | `integraciones.md` | |
| 8 | Compras | `compras.md` | Seccion construccion condicional |
| 9 | Inventario | `inventario.md` | |
| 10 | TPV | `tpv.md` | |
| 11 | Recursos humanos | `recursos-humanos.md` | |
| 12 | Servicio externo | `servicio-externo.md` | |
| 13 | Planificacion | `planificacion.md` | NO usar si construccion |
| 14 | Proyectos y Obras | `proyectos-obra.md` | SOLO construccion (sustituye a Planificacion) |

## Paso 0: Recopilacion de Informacion

Pregunta al usuario:
1. **ID del proyecto** en Odoo (`project.project`)
2. **ID del pedido de venta** (`sale.order`)

**DETENTE** y espera la respuesta.

## Paso 1: Lectura de Datos del Pedido

Usa `odoo-pilot` para leer el pedido de venta:

```
sale.order (ID proporcionado):
- name, partner_id, x_studio_resumen, x_studio_services_ids
- x_studio_painpoint_ids, x_studio_toma_de_datos
- x_studio_conceptos_proyecto, opportunity_id
```

### 1a. Analisis de x_studio_toma_de_datos

Este campo es **critico** -- contiene la informacion recopilada de la oportunidad comercial.

- **Si tiene contenido**: Analiza toda la informacion. Extrae datos relevantes para cada aplicacion.
- **Si esta vacio**: **DETENTE INMEDIATAMENTE**. Informa al usuario y ofrece analizar la oportunidad vinculada (`crm.lead`).

### 1b. Analisis de campos complementarios

- **x_studio_resumen**: Vision general del proyecto para contextualizar.
- **x_studio_services_ids**: Aplicaciones contratadas. Determinan QUE cuestionarios generar.
- **x_studio_painpoint_ids**: Puntos de dolor. Cruzar con preguntas de cada cuestionario.
- **x_studio_conceptos_proyecto**: Lo presupuestado. Define el alcance real.

### 1c. Deteccion de Vertical de Construccion (inBuilt)

Buscar en los datos leidos indicios de construccion:

1. **`x_studio_conceptos_proyecto`**: "inBuilt", "construccion", "obra", "certificacion", "BC3", "FIEBDC", "partidas", "mediciones".
2. **`x_studio_toma_de_datos`**: constructora, subcontrata, obra civil, presupuestos de obra, certificaciones.
3. **`x_studio_resumen`**: sector de construccion.

Resultado:
- **Si se detecta**: `es_construccion = true`. Informar al usuario.
- **Si hay duda**: Preguntar al usuario.
- **Si NO se detecta**: `es_construccion = false`.

Efecto del flag `es_construccion = true`:
1. **Ventas**: Incluir seccion "5. Presupuestos de Obra" (marcada con `<!-- VERTICAL: CONSTRUCCION -->`).
2. **Compras**: Incluir seccion "9. Compras de Obra".
3. **Facturacion**: Incluir seccion "7. Certificaciones de Obra".
4. **Proyectos y Obras**: Generar ESTE cuestionario en lugar de Planificacion. Si no hay Planificacion en services_ids pero es construccion, generar igualmente.
5. **Planificacion**: **NO generar** si es construccion.

## Paso 2: Lectura del Proyecto y Knowledge

Usa `odoo-pilot` para leer el proyecto: `project.project` -> `name`, `x_studio_knowledge`.

### 2a. Verificar/crear articulo Knowledge del proyecto

- Si `x_studio_knowledge` tiene valor: usar como padre.
- Si esta vacio: crear `knowledge.article` con nombre del proyecto, `internal_permission: write`. Actualizar proyecto.

### 2b. Verificar/crear articulo "Cuestionarios de analisis"

Buscar hijos del articulo Knowledge: `knowledge.article WHERE parent_id = {knowledge_id} AND name ilike "Cuestionarios"`.
- Si existe: usar su ID como padre.
- Si no: crear con `parent_id` del Knowledge del proyecto.

## Paso 3: Generacion de Cuestionarios Pre-rellenados

Para cada aplicacion de `x_studio_services_ids`:

### 3a. Verificar que no existe ya
Buscar: `knowledge.article WHERE parent_id = {cuestionarios_id} AND name ilike "{nombre_app}"`. Si existe, saltar.

### 3b. Leer plantilla
Lee `./templates/{archivo}.md`. Si no tiene plantilla, informar y saltar.

### 3c. Pre-rellenar y personalizar

Analiza cada pregunta cruzandola con la informacion del pedido:

1. **YA CONTESTADA**: Incluye pregunta + respuesta. Marca con: `[Pre-rellenado desde toma de datos]`
2. **PARCIALMENTE CONTESTADA**: Incluye lo que se sabe. Marca con: `[Parcial - completar en sesion]`. Resalta en negrita lo que falta.
3. **PENDIENTE**: Mantiene la pregunta. Marca con: `[Pendiente]`
4. **IRRELEVANTE**: Eliminarla del cuestionario.

### 3d. Adaptar al contexto del cliente
- Usar nombre real del cliente y datos conocidos en preguntas.
- Si hay painpoints relacionados, anadir nota: "Punto de dolor identificado: {descripcion}".

### 3e. Generar contenido HTML

Convertir a HTML compatible con Odoo Knowledge:
- `<h2>` titulo, `<h3>` secciones, `<h4>` subsecciones
- Pendientes: `background-color: #fff3cd`
- Pre-rellenadas: `background-color: #d4edda`
- Parciales: `background-color: #ffe0b2`
- Painpoints: `background-color: #e3f2fd`

### 3f. Crear articulo en Knowledge
Crear `knowledge.article`: `name: "Cuestionario - {Nombre App}"`, `parent_id`, `body: HTML`.

## Paso 4: Crear articulo de Preguntas Pendientes

Recopilar TODAS las preguntas pendientes de todos los cuestionarios. Crear articulo resumen:
- `name`: "Preguntas pendientes"
- `parent_id`: ID del articulo "Cuestionarios de analisis"
- `body`: HTML agrupado por aplicacion, total pendientes al inicio

## Paso 5: Resumen y Proximos Pasos

Presentar:
1. **Resumen**: cuestionarios creados, preguntas totales vs pre-rellenadas vs pendientes, eliminadas, painpoints.
2. **Proximos pasos**: revisar respuestas pre-rellenadas, centrarse en pendientes, ejecutar `project-analysis` al completar.
