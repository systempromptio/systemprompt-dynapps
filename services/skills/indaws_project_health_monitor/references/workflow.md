# project-health-monitor: Workflow Details

## Paso 0: Recopilacion de Datos (STOP)

**SIEMPRE** recopila antes de empezar:

1. **Credenciales Odoo**: URL, DB, API Key, Usuario
2. **Identificador del proyecto**: ID numerico o nombre
3. **Fecha de referencia** (opcional): Default = hoy

**DETENTE** y espera respuesta del usuario.

---

## Paso 1: Conexion y Extraccion de Datos

```bash
export ODOO_URL="<url>" ODOO_DB="<db>" ODOO_KEY="<api_key>" ODOO_USER="<usuario>"
./scripts/fetch_project_data.sh <project_id> --pilot-dir /path/to/odoo-pilot/scripts
```

**El script obtiene** (6 conjuntos):
- Proyecto (`project.project`): nombre, fechas, horas
- Tareas (`project.task`): etapa, horas, fechas, estado. Solo `active=True`
- Checklists (`xx.checklist.item`): items, `weight`, `state`, `date_deadline`, `date_done`, `type`. Excluye tareas archivadas
- Timesheets (`account.analytic.line`): horas por tarea/empleado/fecha
- Asignacion horas (`project.project.hour`): total, pedido de venta
- Desglose departamento (`project.project.hour.line`): por departamento

**Validaciones post-extraccion**:
- Proyecto existe con fechas inicio/fin -> OBLIGATORIO (sin fechas = ERROR)
- `total_hours` > 0
- Sin checklists -> penalizacion gobernanza (-15 pts)
- Sin timesheets -> continuar con metricas parciales
- Desajuste horas tareas vs proyecto -> alertar
- Tareas sin fecha limite -> listar
- Tareas >=5h sin checklists -> alertar

**Busqueda por nombre**: Si se proporciona nombre en vez de ID:
`search_records.sh project.project '[["name","ilike","<nombre>"]]'`
Si multiples resultados, pedir confirmacion.

---

## Paso 2: Calculo de Metricas

**OBLIGATORIO: USAR SIEMPRE `calculate_health.py`. NUNCA calcular manualmente.**

```bash
./scripts/fetch_project_data.sh <project_id> > /tmp/phm_data.json
python3 ./scripts/calculate_health.py < /tmp/phm_data.json > /tmp/phm_metrics.json
# Con fecha especifica:
python3 ./scripts/calculate_health.py --date 2026-02-10 < /tmp/phm_data.json
```

**Campos del JSON de salida**: metrics (T%/C%/H%), penalties, health_score, scenario, timeline, hours, checklists, departments, stages, tasks, bottlenecks, alerts, validation.

**NUNCA hacer calculos fuera del script.**

---

## Paso 2.5: Validacion del Dashboard (STOP)

**DETENTE** y presenta dashboard ANTES de generar informe:
- Health Score con barra visual e icono
- Tabla T%/C%/H% con deltas
- Escenario detectado
- Indicadores clave: burn rate, factor aceleracion, EAC
- Bottlenecks detectados
- Avisos especiales

Espera respuesta: Si/Correcto -> Paso 3, Observaciones -> ajustar, Preguntas -> explicar.

**NO generar informe sin validacion del usuario.**

---

## Paso 3: Evaluacion y Plan de Accion

Consulta `./references/plan_accion_patrones.md` para patrones de accion.

a) Clasificar escenario: ON_TRACK, LOW_PRODUCTIVITY, INEFFICIENCY, BLOCKED, OVERCOST, OVERDUE
b) Identificar bottlenecks por Risk Score
c) Generar plan priorizado: acciones inmediatas, corto plazo, estrategicas. Personalizar con datos reales (tareas, responsables, contexto).

---

## Paso 4: Generacion del Informe

**Solo tras validacion del dashboard (Paso 2.5).**

Template: `./templates/informe_salud.md`
Fichero: `informe_salud_{nombre_proyecto}_{YYYYMMDD}.md`

8 secciones: Dashboard Progreso, Resumen Proyecto, Progreso Checklists por Tarea, Detalle Items Criticos, Consumo Horas, Evaluacion Riesgos, Plan Accion Priorizado, Proximos Pasos.

---

## Paso 4.5: Nota interna en el chatter del proyecto (opcional)

Si el usuario lo solicita (si el health check se lanza desde project-weekly-update, la nota la publica
su `create_update.py`: no repetir este paso). El resultado se publica como **nota interna** en el chatter de
`project.project`. Ya NO se escribe en `project.update`: sus 16 campos `x_studio_*` de salud
(`x_studio_health_score`, `x_studio_internal_report`, `x_studio_last_health_check`...) se
eliminaron en portal.dynapps.es y cualquier escritura falla con "Invalid field". No leer ni
escribir esos campos.

```bash
export PHM_SNAPSHOT_DIR="<carpeta del proyecto>/.phm"   # opcional; default ~/.claude/phm-snapshots
# 1. Evaluacion anterior: la mas reciente entre snapshot local y ultima nota del chatter
python3 ./scripts/health_note.py previous --project-id <id> > /tmp/phm_prev.json
# 2. HTML de la nota (con variacion respecto a la anterior si existe)
python3 ./scripts/health_note.py build --metrics /tmp/phm_metrics.json \
  --project-id <id> --project-name "<nombre>" --previous /tmp/phm_prev.json > /tmp/phm_nota.html
# 3. DRY-RUN: ensenar el HTML al usuario y la llamada que se haria (no publica)
python3 ./scripts/health_note.py post --project-id <id> --html /tmp/phm_nota.html
# 4. (STOP) Solo tras OK explicito del usuario:
python3 ./scripts/health_note.py post --project-id <id> --html /tmp/phm_nota.html --confirm
python3 ./scripts/health_note.py snapshot --metrics /tmp/phm_metrics.json --project-id <id>
```

**Garantias de "cero correos"** (las aplica el script, no hace falta repetirlas a mano):
- `message_post` con `subtype_xmlid='mail.mt_note'`, `message_type='comment'`, `body_is_html=True`,
  `partner_ids=[]`, sin plantilla, contexto `mail_create_nosubscribe` + `mail_post_autofollow=False`
  (no anade seguidores).
- Antes de publicar comprueba que ningun seguidor esta suscrito al subtipo Nota; si lo hay, se
  niega a publicar (exit 3). El subtipo es interno: los contactos del cliente nunca lo ven.
- Tras publicar verifica el `mail.message` creado (sin `partner_ids`) y que no hay ningun
  `mail.mail` en cola para el; si algo no cuadra sale con exit 2.

**Contenido de la nota**: encabezado `🩺 Health check · DD/MM/YYYY` (marca para localizar el
historico), Health Score y estado, escenario, C% (items y peso), T% (dias), H% (horas consumidas
de presupuesto), delta C-T, burn rate actual y esperado, EAC, factor de aceleracion, fecha de
evaluacion, top 3 bottlenecks y avisos. Pie con linea `PHM-DATA v1 | clave=valor | ...`: es el
contrato para comparar evaluaciones; no cambiar su formato sin versionarlo (`v2`).

**Comparacion con la evaluacion anterior**: sale SIEMPRE del snapshot local o de la ultima nota
`PHM-DATA` del chatter (`health_note.py previous`), nunca de campos de Odoo.
