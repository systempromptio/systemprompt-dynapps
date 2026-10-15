# project-odoo-creator: Workflow Details

## Paso 0: Recopilacion de Datos (STOP)

**Datos obligatorios:**
1. **ID del project.project**: Proyecto ya creado en Odoo
2. **Plan de proyecto**: Documento de `project-plan` (pegar, adjuntar o ruta)
3. **Aplicaciones del proyecto**: Lista de apps incluidas (extraer del plan si las incluye)

**DETENTE** y espera respuesta del usuario.

---

## Paso 1: Lectura y Validacion del Proyecto

a) **Proyecto** (`project.project`): id, name, partner_id, date_start, date, user_id, tag_ids, description, task_count

b) **Cliente** (`res.partner`): id, name, commercial_partner_id

c) **Verificacion de tareas existentes**: Si `task_count > 0`, preguntar:
- Opcion A: Eliminar existentes y crear nuevas
- Opcion B: Anadir sin tocar existentes
- Opcion C: Cancelar

**DETENTE** si hay tareas existentes.

---

## Paso 2: Procesamiento del Plan

Consulta `./references/estructura_proyecto_odoo.md` para mapeo de campos.

a) Extraer tareas del plan:

| Campo plan | Campo Odoo | Notas |
|------------|------------|-------|
| Nombre | `name` | Prefijo: [CONFIG], [DEV], [MIGR], etc. |
| Horas | `allocated_hours` | Horas planificadas |
| Fecha Inicio | `planned_date_begin` | YYYY-MM-DD 08:00:00 |
| Fecha Fin | `date_deadline` | YYYY-MM-DD 18:00:00 |
| Modulo/Categoria | `tag_ids` | Crear etiqueta si no existe |
| Checklist interno | `xx.checklist.item` | Items del detalle |

b) Extraer checklists: Items con prefijo de tipo ([CONFIG], [DEV], [MIGR], [TEST], [TRAIN], [DOC], [STUDIO/AUTO])

---

## Paso 3: Validacion con el Usuario (STOP)

Presentar resumen: tareas a crear (tabla), total horas, total items checklist, estructura Knowledge.

**DETENTE** y espera aprobacion.

---

## Paso 4: Creacion en Odoo

### 4a: Campo x_studio_knowledge
Verificar si existe. Si no, crear via `ir.model.fields` (many2one a knowledge.article).

### 4b: Knowledge articles
Consulta `./references/estructura_knowledge.md` para definicion completa.

Patron search-before-create (idempotente):

1. **Articulo raiz**: workspace, nombre = proyecto. Si ya existe, preguntar.
2. **7 subarticulos**:

| Orden | Nombre | Icono | Compartible |
|-------|--------|-------|-------------|
| 1 | Kick-Off | (cohete) | SI |
| 2 | Analisis | (lupa) | SI |
| 3 | Cuestionarios de Analisis | (lista) | SI |
| 4 | Formaciones | (gorro) | SI |
| 5 | Actas de Reunion | (nota) | SI |
| 6 | Adaptaciones | (engranaje) | SI |
| 7 | Interno | (candado) | NO |

Compartibles: `internal_permission: "write"`. Interno: `is_desynchronized: true`.
Compartir con cliente: `knowledge.article.member` con `permission: "read"` en articulo raiz.

3. **Vincular**: `project.project.x_studio_knowledge = root_article_id`

### 4c: Etapas del proyecto
Vincular etapas estandar (IDs: 10, 574, 737, 1008, 11, 23, 26, 25, 35, 12, 37) via `[4, project_id]` en Many2many.

### 4d: Crear tareas
Para cada tarea:
- `project.task`: name, project_id, allocated_hours, planned_date_begin, date_deadline, description, tag_ids, user_ids=[[5]], stage_id=10, sequence
- `xx.checklist.item` por cada item: name, task_id, project_id, weight, sequence, state="pending", type

Mapeo tipo: [CONFIG]->"CONFIG", [DEV]->"DEV", [MIGR]->"MIGR", [TEST]->"TEST", [TRAIN]->"TRAIN", [DOC]->"CONFIG", [ANLS]->"ANLS", [STUDIO/AUTO]->"STUDIO/AUTO"

---

## Paso 5: Resumen de Creacion

Template: `./templates/resumen_creacion.md`

Incluye: tareas creadas con IDs, Knowledge articles con IDs/URLs, links al proyecto, estadisticas totales.
