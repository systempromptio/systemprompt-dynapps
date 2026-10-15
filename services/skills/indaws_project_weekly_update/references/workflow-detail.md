# Project Weekly Update -- Workflow Detail

## Principio fundamental

Este skill procesa **UN proyecto por invocacion**. Todos los ficheros temporales
se nombran con el ID del proyecto: `/tmp/pwu_{project_id}_*.json` para evitar
colisiones en modo batch con subagentes. Nunca reutilices ficheros de
ejecuciones anteriores.

**NUNCA se salta project-health-monitor.** Es el Paso 0 obligatorio, se ejecuta
automaticamente al inicio del skill, no es un "prerequisito manual".

## Paso 0: Recopilacion de Datos (CRITICO)

Recopila:

1. **Credenciales Odoo**: URL, base de datos, API Key, usuario
2. **Identificador del proyecto**: ID numerico del proyecto en Odoo.
3. **Semana** (opcional): Fecha del lunes de la semana a reportar. Default: semana actual.
4. **Idioma** (opcional): `es` (default) o `en`. Algunos clientes piden el
   update en ingles; preguntalo solo si no esta claro por el contexto del
   proyecto (p.ej. cliente internacional).

**DETENTE** y espera la respuesta del usuario antes de continuar.

Exporta las credenciales al entorno (necesarias para todos los pasos):
```bash
export ODOO_URL="<url>" ODOO_DB="<db>" ODOO_USER="<usuario>" ODOO_API_KEY="<api_key>"
# Compatibilidad con odoo-pilot (mismo valor):
export ODOO_KEY="$ODOO_API_KEY"
```

Define prefijo temporal unico para el proyecto:
```bash
export PID=<project_id>
export TMP=/tmp/pwu_${PID}
```

## Paso 1: Ejecutar project-health-monitor (AUTOMATICO, NO SKIP)

Este paso ejecuta el skill `project-health-monitor` para el proyecto indicado.
NO es un prerequisito opcional: forma parte del workflow y debe ejecutarse
siempre al inicio.

Resuelve el directorio de `project-health-monitor` de forma portable (vecino
de este skill en la misma suite):

```bash
# Este skill (project-weekly-update) esta en skills/project-weekly-update/.
# project-health-monitor esta como hermano: skills/project-health-monitor/.
# Resolvemos ambos desde el SKILL_DIR del skill que el LLM acaba de cargar.
SKILL_DIR="$(dirname "$(readlink -f "${BASH_SOURCE[0]:-$0}")")/.."
PHM="${SKILL_DIR}/../project-health-monitor/scripts"

# Si el LLM no tiene BASH_SOURCE (ejecucion inline), puede pasar --phm-dir
# explicitamente o usar la ruta relativa desde su cwd:
# PHM="../project-health-monitor/scripts"

$PHM/fetch_project_data.sh $PID > ${TMP}_phm_data.json
python3 $PHM/calculate_health.py < ${TMP}_phm_data.json > ${TMP}_phm_metrics.json
```

Si alguno de los scripts no se encuentra en la ruta resuelta, el LLM debe
localizar el skill `project-health-monitor` en la instalacion (via `Glob`
sobre `**/project-health-monitor/scripts/fetch_project_data.sh`) y usar esa
ruta. Nunca hardcodear rutas absolutas especificas de maquina.

Si el usuario pidio una semana concreta:
```bash
python3 $PHM/calculate_health.py --date 2026-02-10 < ${TMP}_phm_data.json > ${TMP}_phm_metrics.json
```

**Validacion obligatoria antes de continuar:**
```bash
python3 -c "
import json, sys
with open('${TMP}_phm_metrics.json') as f:
    d = json.load(f)
required = ['metrics', 'health_score', 'checklists', 'bottlenecks', 'tasks']
missing = [k for k in required if k not in d]
if missing:
    print(f'ERROR: Faltan campos en metricas: {missing}', file=sys.stderr)
    sys.exit(1)
score = d['health_score'].get('score', 0)
estado = d['health_score'].get('estado', '')
if not score and not estado:
    print('ERROR: health_score y estado vacios, metricas no utiles', file=sys.stderr)
    sys.exit(1)
print(f'OK: Score={score}, Estado={estado}')
"
```

Si falla la validacion: **DETENTE**, informa al usuario y no continues.
Nunca intentes suplir las metricas con valores inventados.

## Paso 2: Extraer Datos Semanales

```bash
./scripts/fetch_weekly_data.sh $PID --pilot-dir /path/to/odoo-pilot/scripts > ${TMP}_weekly.json
# Con semana especifica:
./scripts/fetch_weekly_data.sh $PID --week-start 2026-02-10 > ${TMP}_weekly.json
```

El script obtiene:
- Timesheets de la semana (`account.analytic.line`).
- Checklists completados esta semana (`xx.checklist.item`).
- Project.update existente (`project.update`) para la fecha del viernes.

## Paso 2.5: Detectar Update Existente (CONDICIONAL)

Si `existing_update` no es null en `${TMP}_weekly.json`, ya hay un
`project.update` para esa semana.

**DETENTE** y pregunta al usuario:
- **Modificar**: Continuar, usar `--update-id <id>` en Paso 5.
- **Cancelar**: No hacer nada, terminar el skill.

## Paso 3: Preparar Datos Estructurados

```bash
python3 ./scripts/prepare_update.py \
  --weekly-data ${TMP}_weekly.json \
  --metrics ${TMP}_phm_metrics.json \
  --project-data ${TMP}_phm_data.json \
  --language es \
  --health-report <path al informe .md si existe> \
  > ${TMP}_update_data.json
```

Argumentos:
- `--language es|en` — idioma del update. Ajusta el nombre canonico del
  registro (`Seguimiento Semanal ...` o `Weekly Update ...`). Default: `es`.
- `--health-report` — opcional pero recomendado: anade el informe interno
  (Markdown → HTML) a la nota interna de salud del chatter del proyecto.

## Paso 4: Generar HTML (UNICO paso del LLM)

Este es el UNICO paso donde el LLM genera contenido.

1. Lee `${TMP}_update_data.json`
2. Lee `./templates/weekly_update_html.md`
3. Lee `./references/contenido_actualizacion.md`

Tareas del LLM:

a) **Redactar "Trabajo Realizado"**: parrafo narrativo a partir de
   `work_done.by_task` y `weekly_checklists.by_task`. Los timesheets se leen
   para entender QUE se hizo, pero NUNCA se muestran al cliente.

b) **Generar "Trabajos terminados"**: si `weekly_checklists.total_completed > 0`,
   renderizar los items completados.

c) **Redactar "Proximos Pasos"**: 3-5 items a partir de `next_steps`.

d) **Rellenar tabla de estado**: columnas Tarea, Etapa, Inicio, Fin, Progreso.

e) **Rellenar seccion de progreso**: bloques C% y T%, fechas del proyecto,
   go-live si existe.

f) **Aplicar template HTML con estilos inline**.

**Guarda el HTML final en `${TMP}_description.html`.**

### Reglas obligatorias de redaccion (NO negociables)

- **Idioma**: TODO el HTML en el idioma solicitado (`es` default, `en` si el
  cliente lo pide). No mezclar idiomas. `create_update.py` aborta si detecta
  contaminacion (marcadores del idioma opuesto: p.ej. "Weekly Update" cuando
  se pidio `es`, o "Seguimiento Semanal" cuando se pidio `en`).
- **Tono**: voz pasiva, profesional formal.
- **NO mostrar horas al cliente.** Los timesheets son datos internos.
- **NO mencionar**: "checklists ponderados", "Health Score numerico",
  "risk score", "bottleneck", "penalizacion".
- **Traduccion de Health Score**:
  - >= 80: "El proyecto avanza segun lo previsto"
  - 60-79: "El proyecto presenta algunas desviaciones que estamos gestionando"
  - 40-59: "El proyecto requiere atencion en varios frentes"
  - < 40: "El proyecto presenta desviaciones significativas que requieren accion inmediata"

## Paso 4.5: Validacion del Borrador (OBLIGATORIO)

**DETENTE** y presenta al usuario:
1. El HTML generado.
2. Resumen de lo que se va a crear/actualizar:
   - Accion: Crear nuevo / Actualizar existente (ID: X)
   - Nombre canonico (de `odoo_values.name`)
   - Nota interna de salud que se publicara en el chatter del proyecto
     (generar con `--dry-run`: deja el HTML en `${TMP}_update_data.json.health_note.html`).

Espera: Aprobar / Modificar / Cancelar.

## Paso 5: Escribir el Registro en Odoo (via create_update.py)

**Usa SIEMPRE `create_update.py`.** No compongas llamadas manuales a
`create_record.sh` / `update_record.sh`: el script garantiza que el nombre,
las fechas y la nota de salud salgan del JSON y no de valores inventados
por el LLM.

**Crear nuevo:**
```bash
python3 ./scripts/create_update.py \
  --data ${TMP}_update_data.json \
  --html ${TMP}_description.html
```

**Actualizar existente:**
```bash
python3 ./scripts/create_update.py \
  --data ${TMP}_update_data.json \
  --html ${TMP}_description.html \
  --update-id <update_id>
```

El script:
- Valida que haya datos de salud reales (score/estado + JSON de metricas de
  `health_metrics_path` o `--metrics`) y que ningun campo `x_studio_*` se cuele
  en `odoo_values` (se eliminaron de `project.update`).
- Valida que el nombre sea canonico y que el HTML no mezcle idiomas.
- Comprueba que ningun seguidor del proyecto esta suscrito al subtipo Nota
  (si lo hay, aborta ANTES de escribir nada).
- Crea/actualiza el `project.update` (solo campos estandar) via XML-RPC.
- Publica la salud como NOTA INTERNA en el chatter de `project.project` con
  `health_note.py` (subtype `mail.mt_note`, sin `partner_ids`, sin seguidores
  nuevos, sin plantilla), con enlace al update y el informe interno si existe.
  Guarda el snapshot local para la comparacion de la semana siguiente.
- Devuelve `{"status": "created"|"updated", "id": N, "health_note": {...}}`.
  Si el update se escribio pero la nota falla, sale con codigo 2 (no deshace el
  update): publicar la nota a mano con `health_note.py post --confirm`.

**Limpieza (opcional al final):**
```bash
rm -f ${TMP}_*.json ${TMP}_description.html
```

## Campos y nota de salud

Ver `references/odoo-field-mapping.md` para los campos estandar que se escriben
en `project.update` y el contenido de la nota interna. El LLM no necesita
cargar ese reference salvo que este debuggeando el Paso 5.
