# project-meeting-minutes: Workflow Details

## Paso 0: Recopilacion de Datos (STOP)

**Datos obligatorios:**
1. **ID del proyecto** (project.project): numerico o nombre para buscar
2. **Tipo de reunion**: seguimiento | kick-off | validacion | formacion | analisis
3. **Notas de la reunion**: Texto libre pegado por el usuario

**Datos opcionales (con defaults):**
4. **Fecha**: Default = hoy
5. **Asistentes**: Extraer de notas si no se proporcionan
6. **App/modulo** (solo tipo "analisis"): Que app se analizo

**DETENTE** y espera respuesta del usuario.

---

## Paso 1: Lectura de Datos de Odoo

a) **Proyecto** (`project.project`): id, name, partner_id, date_start, date, x_studio_knowledge, user_id

**Validacion critica**: Si `x_studio_knowledge` esta vacio, DETENTE. Necesita `project-odoo-creator` primero.

b) **Knowledge**: Buscar hijos del articulo raiz:
- "Actas de Reunion" (OBLIGATORIO)
- "Cuestionarios de Analisis" (solo tipo analisis)
- "Analisis" (para deteccion de alcance)
- Leer body actual de "Actas de Reunion" (para prepend)

c) **Tareas** (`project.task`): id, name, description, stage_id, allocated_hours, date_deadline

d) **Checklists** (`xx.checklist.item`): name, task_id, project_id, weight, state, type, user_id, sequence, date_deadline, date_done

e) **Cliente** (`res.partner`): id, name, commercial_partner_id

f) **Articulos de analisis** (para deteccion de alcance): buscar hijos de "Analisis", leer name y body

g) **Cuestionarios existentes** (solo tipo analisis): buscar hijos de "Cuestionarios de Analisis"

---

## Paso 2: Procesamiento de Notas

Consulta `./references/tipos_reunion.md` para secciones especificas por tipo.

**Secciones comunes** (todos los tipos):
1. Asistentes
2. Resumen ejecutivo (3-5 puntos)
3. Temas tratados
4. Decisiones tomadas
5. Acciones pendientes
6. Preguntas abiertas
7. Riesgos identificados
8. Proxima reunion

**Secciones por tipo**:
- seguimiento: estado tareas, blockers, progreso
- kick-off: version ligera (el completo lo gestiona kick-off-presentation)
- validacion: entregables revisados, aceptados/rechazados
- formacion: temas cubiertos, ejercicios, dudas
- analisis: decisiones funcionales, flujos, requisitos -> ver Paso 2b

---

## Paso 2b: Rellenado de Cuestionario (SOLO tipo "analisis")

a) Cargar cuestionario de Knowledge o template de `questionnaire-generator`
b) Mapear respuestas: Respondida (verde), Parcial (ambar), Pendiente (rojo)
c) NUNCA sobreescribir respuestas existentes salvo cambio de criterio explicito
d) Presentar cuestionario rellenado al usuario con estadisticas

**DETENTE** y espera aprobacion.

---

## Paso 3: Deteccion de Cambios de Alcance (CRITICO)

Consulta `./references/deteccion_alcance.md` para algoritmo completo.

a) Extraer temas de las notas
b) Cotejar contra 3 fuentes: analisis, tareas, checklists
c) Clasificar:

| Categoria | Criterio | Accion |
|-----------|----------|--------|
| Dentro del alcance | Match en analisis/tareas/checklists | Documentar, sin accion |
| Adicion menor | Encaja en tarea existente pero sin checklist | Proponer nuevo item (con aprobacion) |
| Cambio de alcance | No aparece en nada | BANDERA ROJA. Requiere presupuesto. NUNCA crear nada |
| Ambiguo | Match parcial | Preguntar al usuario |

d) Presentar deteccion al usuario

**DETENTE** y espera decision. OBLIGATORIO.

---

## Paso 4: Generacion del Acta (HTML)

Template: `./templates/acta_reunion.md`

Reglas:
- Estilos INLINE (Odoo Knowledge no soporta CSS externo)
- Colores Indaws: Blue Lilac (#6B68FA) headers, Blue Space (#1C265D) texto
- Cada tipo tiene color de badge (ver `tipos_reunion.md`)
- Cambios de alcance con banner ambar (#E5B92B fondo, #1C265D texto)
- Acciones en tabla: Accion, Responsable, Fecha limite

---

## Paso 5: Validacion del Acta (STOP)

Presentar acta completa con resumen de acciones, destinos de escritura.

**DETENTE** y espera aprobacion.

---

## Paso 6: Escritura en Odoo

Consulta `./references/reglas_acta_acumulativa.md` para patron detallado.

**6a**: Acumular acta en "Actas de Reunion" (prepend, nueva arriba)
Cada acta envuelta en `<div data-acta-fecha="{YYYY-MM-DD}" data-acta-tipo="{tipo}">...</div>`

**6b**: Escribir cuestionario (solo tipo analisis) - search-before-create en Knowledge

**6c**: Crear items de checklist (si aprobados) en `xx.checklist.item`

**6d**: Lo que NUNCA se hace:
- NO crear tareas (project.task)
- NO crear presupuestos (sale.order)
- NO actualizar client requests directamente (solo sugerir client-request-manager)

---

## Paso 7: Resumen y Sugerencias

Presentar: acta publicada, acciones ejecutadas, cambios de alcance detectados, sugerencias (client-request-manager para peticiones, quotation-builder para ampliaciones).
