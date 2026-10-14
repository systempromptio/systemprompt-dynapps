# Executive Projects Tracker — Workflow Detail

## Carpeta de datos

Resuelta en este orden:
1. `EPT_DATA_DIR` (variable de entorno)
2. Marcador `.executive-projects-tracker/projects/` en cwd o cualquier ancestro (estilo git)
3. Fallback global `~/.indaws/executive-projects-tracker/projects/`

Para aislar una cartera por proyecto:
```bash
cd /ruta/a/mi/proyecto
python3 .../scripts/manage_local.py init
```

## Modelo de datos local (`<id>.json`)

```jsonc
{
  "project_id": 1083,
  "name": "...", "partner": "...",
  "user_id": 1133, "user_name": "María Erades", "user_email": "maria.erades@indaws.es",
  "status": "active",          // active | completed
  "cadence": "weekly",         // weekly | biweekly
  "registered_at": "2026-05-04",
  "baseline": {
    "stage": "...", "date_start": "...", "date_end": "...",
    "production_date": "2026-06-29", "hours_total": 523,
    "captured_at": "2026-05-04"
  },
  "current": { /* snapshot completo */ },
  "history": [ /* snapshots resumidos */ ],
  "reviews": [
    {
      "date": "2026-05-04",
      "alerts": [...],
      "any_alert": true,
      "escalated": false,
      "action_committed": null,
      "production_date_confirmed": null,
      "production_date_at_review": "2026-06-29"
    }
  ],
  "comments": [...]
}
```

## Cadencia (calculada automáticamente al alta)

```
etapa == "Análisis"                       → semanal
duración (date_end - date_start) > 90 d   → quincenal
resto                                     → semanal
```

Override: `set-cadence <id> weekly|biweekly`

`list-due` filtra proyectos cuya cadencia toca hoy:
- weekly: última review hace ≥ 6 días o sin review
- biweekly: última review hace ≥ 13 días o sin review

## Las 6 alertas

| # | Tipo | Cuándo se enciende |
|---|---|---|
| 1 | `overdue_tasks` | ≥1 tarea con `date_deadline < hoy` y `remaining_hours > 0` (excluyendo etapas terminadas) |
| 2 | `tasks_without_date` | ≥1 tarea sin `date_deadline` (excluyendo etapas terminadas) |
| 3 | `no_movement` | ≥7 días sin que cambien horas consumidas ni checklists completados |
| 4 | `advance_gap` | `T_pct − C_pct > 10` puntos |
| 5 | `tasks_without_checklists` | ≥1 tarea con `allocated_hours > 4` sin checklist definido |
| 6 | `client_update_missing` | No hay `project.update` enviado al cliente en últimos 7 días |

**Etapas terminadas** (config.json): `Realizado`, `Hecho`, `Realizada`, `Done`, `Cancelado`

**Excepción `overdue_tasks`**: no se dispara si checklist al 100% con >1 ítem, o si la etapa está en `skip_overdue_if_stage_and_checklist_100` y checklist al 100%.

**Escalación**: si la review previa tenía `action_committed` y la misma alerta sigue activa, se inserta bloque amarillo "Acción comprometida no realizada" en el detalle del proyecto.

## Modos de uso

### Modo 0 — Inicializar carpeta local
```bash
python3 "$SKILL_DIR/scripts/manage_local.py" init
```

### Modo 1 — Descubrir candidatos a alta
```bash
"$SKILL_DIR/scripts/discover_projects.sh" > /tmp/ept_candidates.json
python3 "$SKILL_DIR/scripts/manage_local.py" list > /tmp/ept_local.json
```

### Modo 2 — Dar de alta un proyecto
```bash
"$SKILL_DIR/scripts/fetch_project_state.sh" 1083 \
  | python3 "$SKILL_DIR/scripts/manage_local.py" register
```

### Modo 3 — Alta masiva
1. `discover_projects.sh` + `manage_local.py list`
2. Calcular diferencia
3. **PARAR** y mostrar lista para confirmación
4. Iterar Modo 2 por cada id

### Modo 4 — Revisión (1 proyecto)
```bash
"$SKILL_DIR/scripts/fetch_project_state.sh" $PID \
  | python3 "$SKILL_DIR/scripts/manage_local.py" set-current $PID

python3 "$SKILL_DIR/scripts/detect_alerts.py" --project-id $PID \
  | python3 "$SKILL_DIR/scripts/manage_local.py" add-review $PID

python3 "$SKILL_DIR/scripts/render_portfolio.py" \
   --project-ids $PID --filename "informe_proyecto_${PID}.md"
```

### Modo 5 — Revisión semanal de cartera (lunes)
```bash
DUE_IDS=$(python3 "$SKILL_DIR/scripts/manage_local.py" list-due \
          | python3 -c "import json,sys; print(','.join(str(p['project_id']) for p in json.load(sys.stdin)))")

for PID in ${DUE_IDS//,/ }; do
   "$SKILL_DIR/scripts/fetch_project_state.sh" $PID \
     | python3 "$SKILL_DIR/scripts/manage_local.py" set-current $PID
   python3 "$SKILL_DIR/scripts/detect_alerts.py" --project-id $PID \
     | python3 "$SKILL_DIR/scripts/manage_local.py" add-review $PID
done

python3 "$SKILL_DIR/scripts/render_portfolio.py" --project-ids "$DUE_IDS"
python3 "$SKILL_DIR/scripts/render_consultant_report.py" \
   --consultant "María Erades" --out-file /tmp/seguimiento_maria_erades.html
```

### Modo 6 — Registrar respuesta del responsable (martes)

Leer hilo en Outlook:
```
mcp__claude_ai_Microsoft_365__outlook_email_search
  query: "[PROJECT REVIEW] Seguimientos de proyecto <NOMBRE_CONSULTOR>"
```

Registrar respuesta:
```bash
python3 "$SKILL_DIR/scripts/manage_local.py" respond 1083 \
  "Replanificará las 8 tareas vencidas antes del 2026-05-09"

python3 "$SKILL_DIR/scripts/manage_local.py" confirm-prod-date 1083
# o
python3 "$SKILL_DIR/scripts/manage_local.py" update-prod-date 1083 2026-07-15
```

### Modo 7 — Gestión libre
```bash
python3 "$SKILL_DIR/scripts/manage_local.py" comment 1083 "..."
python3 "$SKILL_DIR/scripts/manage_local.py" complete 1083
python3 "$SKILL_DIR/scripts/manage_local.py" reopen 1083
python3 "$SKILL_DIR/scripts/manage_local.py" set-cadence 1083 biweekly
python3 "$SKILL_DIR/scripts/manage_local.py" show 1083
```

### Modo 8 — Informe individual por consultor (Outlook-pegable)

```bash
python3 "$SKILL_DIR/scripts/render_consultant_report.py" \
   --consultant "María Erades" --out-file /tmp/seguimiento_maria_erades.html
# o por email:
python3 "$SKILL_DIR/scripts/render_consultant_report.py" \
   --owner-email maria.erades@indaws.es --out-file /tmp/seguimiento_maria_erades.html
```

Genera HTML con: 5 KPIs en cabecera, tabla resumen de proyectos con avance + delta pp,
detalle por proyecto (pill de seguimiento, vencidas, sin fecha, comentario comprometido).

**Pill de seguimiento**: "Al día" si hay `project.update` en últimos 7 días; si no, "Seguimiento pendiente · último DD/MM/YYYY".

## Estructura del informe (`render_portfolio.py`)

`informe_seguimiento_cartera_{YYYY}_W{WW}.md`:

1. **Resumen de cartera** (totales, alertas máximas)
2. **Por responsable** (`## Nombre · email`):
   - Tabla "Cartera (vista CEO)": id, nombre, etapa, cadencia, salida producción, avance C/H/T, horas, estado
   - Email a enviar con tabla resumen + sección por proyecto + cierre conjunto + recordatorio fechas

**Fechas en el email**: DD/MM/YYYY. En JSON local: ISO.
**Tablas de tareas**: columna Etapa incluida.
**Estado del proyecto**: 0 alertas → verde · 1-3 → naranja · ≥4 → rojo.
