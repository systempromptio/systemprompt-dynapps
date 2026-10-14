# Odoo Configurator - Workflow Detail

## Paso 1: Lectura de Contexto

### 1a. Leer tarea y checklists

Ejecutar script `scripts/fetch_task_checklist.sh <task_id>`:
- Extrae `project.task`: id, name, description, project_id, stage_id, tag_ids
- Extrae `xx.checklist.item`: todos los items con name, type, weight, state
- Extrae `project.project`: name, partner_id, x_studio_knowledge

### 1b. Filtrar items procesables

- Excluir items ya completados (`state = 'done'`)
- Excluir items de tipo DEV, MIGR u otros no configurables (se omiten silenciosamente)
- Agrupar items restantes por tipo (CONFIG, STUDIO/AUTO)

### 1c. Buscar analisis funcional

Si un item del checklist no es autoexplicativo:
1. Buscar articulo de analisis en Knowledge bajo hijo "Analisis" del `x_studio_knowledge`
2. Leer el body del articulo correspondiente a la app de la tarea
3. Cruzar nombre del item con secciones del analisis

Si aun no queda claro -> preguntar al usuario.

## Paso 2: Plan de Ejecucion (VALIDACION OBLIGATORIA)

Presentar tabla con todos los items pendientes clasificados:
- Item, Tipo (CONFIG/STUDIO/AUTO), Accion propuesta, Destino (Nota interna/Knowledge)

Leyenda de destino:
- **Nota interna**: Config simple -> nota en chatter de la tarea
- **Knowledge > Adaptaciones**: Cambio complejo/Studio -> articulo en Knowledge

DETENTE y espera aprobacion/ajustes del usuario.

## Paso 3: Ejecucion de Configuraciones

### 3a. Items CONFIG (configuracion estandar)

1. Consultar `references/configuraciones_estandar.md` para ruta exacta en Odoo
2. Ejecutar via odoo-pilot (update_record, execute_method, install_module)
3. Verificar que el cambio se aplico (lectura post-cambio)
4. Registrar resultado

PRECAUCION PRODUCCION: Advertir antes de cada cambio irreversible.

### 3b. Items STUDIO/AUTO (campos, automatizaciones)

**Fase 1: Propuesta de cambios Studio (VALIDACION OBLIGATORIA)**

Presentar listado detallado: Tipo, Modelo, Detalle, Vista. DETENTE y espera aprobacion.

**Fase 2: Ejecucion con odoo-studio**

- Campos: `create_field.py` + `add_to_view.py`
- Server Actions: Crear `ir.actions.server` via odoo-pilot
- Automated Actions: Crear `base.automation` via odoo-pilot
- Paginas/Tabs: `create_page.py`
- Campos en batch: `create_fields_batch.py`

**Fase 3: Verificacion con odoo-flow-tester**

- Campos nuevos: Modo B (Verificacion) -- confirmar existencia, tipo, vista
- Server/Automated Actions: Modo C (Ejecucion) -- crear registro test, verificar, revertir
- Cambios de vista: Verificar xpath via odoo-pilot

Si falla -> informar con error detallado y proponer correccion.

**Fase 4: Registro de resultado**

## Paso 4: Documentacion

Leer `references/patrones_documentacion.md` para determinar destino de cada item.

### 4a. Items CONFIG simples -> Nota interna en la tarea

IMPORTANTE (Bug JSON2): Usar proceso de 2 pasos:
1. Crear mensaje con `message_post` (body vacio, subtype_xmlid: mail.mt_note)
2. Escribir body HTML real via `mail.message/write` usando template `templates/nota_interna_config.md`

REGLA: Agrupar TODAS las configs simples en UNA SOLA nota por ejecucion.

### 4b. Items STUDIO/AUTO y complejos -> Knowledge "Adaptaciones"

1. Buscar articulo "Adaptaciones" bajo `x_studio_knowledge` del proyecto
2. Crear articulo hijo `knowledge.article`
3. Usar template `templates/adaptacion_knowledge.md`

## Paso 5: Actualizacion de Checklists

- NUNCA cambiar `state` ni `date_done` (el usuario los marca manualmente)
- Cambiar `type` a valor correcto si no lo tenia

## Paso 6: Resumen Final

Tabla: items ejecutados con tipo, resultado y documentacion.
Estadisticas: procesados, exitosos, con errores, documentados.
