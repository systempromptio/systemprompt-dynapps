# Workflow Detail - Full Step-by-Step

Reference document with the complete implementation details for each workflow step.

---

## Paso 0: Inicialización

```bash
# 1. Cargar credenciales
# Credenciales del entorno — ver references/shared/skill-path-resolution.md
export ODOO_URL ODOO_DB ODOO_USER ODOO_KEY

# 2. Cargar credenciales de impersonación si existen
if [[ -n "${MENTOR_IMPERSONATE_UID:-}" ]]; then
    export MENTOR_IMPERSONATE_UID MENTOR_IMPERSONATE_ROLE
fi
```

**Verificar que las credenciales existen**. Si no estan configuradas:
- Informar al usuario: "No tienes Odoo Mentor configurado. Ejecuta `/mentor-setup` para empezar."
- **DETENTE** — no continuar sin credenciales.

## Paso 1: Descubrir contexto del usuario

```bash
# Resolucion agnostica del agente — ver references/shared/skill-path-resolution.md
resolve_skill_scripts() {
    local skill="$1" c
    for c in ${DYNAPPS_SKILLS_ROOT:+"$DYNAPPS_SKILLS_ROOT"/*/skills/"$skill"/scripts} \
             "$HOME/.claude/plugins/cache"/*/*/*/skills/"$skill"/scripts \
             "$HOME/.config/opencode/skills/$skill/scripts" \
             "$HOME/.dyn-skills"/*/suites/*/skills/"$skill"/scripts; do
        [ -d "$c" ] && { printf '%s\n' "$c"; return 0; }
    done
    return 1
}
SCRIPT_DIR="$(resolve_skill_scripts odoo-mentor)"
PILOT_DIR="${ODOO_PILOT_SCRIPTS:-$(resolve_skill_scripts odoo-pilot)}"

# Detectar rol, proyectos y oportunidades CRM
CONTEXT=$("$SCRIPT_DIR/discover_context.sh" --pilot-dir "$PILOT_DIR")
```

El script `discover_context.sh` devuelve JSON con:
- `role`: pm | developer | tl | sales | sales_manager
- `projects`: array de proyectos del usuario
- `crm_opportunities`: array de oportunidades CRM
- `developer.tasks`: tareas asignadas (solo para developers)
- `developer.planning_slots`: slots de planificación (solo para developers)
- `impersonation.active`: si estamos en modo admin

**Si impersonation.active == true**: Prefija TODA la salida con:
```
[MODO ADMIN - Impersonando: {{nombre}} ({{rol}})]
```

## Paso 2: Leer estado anterior

```bash
# Leer estado de la última ejecución
STATE=$(python3 "$SCRIPT_DIR/update_state.py" read)
PREFERENCES=$(python3 "$SCRIPT_DIR/update_state.py" preferences)
```

## Paso 3: Determinar hora del día

Determina `time_of_day` según la hora actual:
- 07:00-11:00 → `morning`
- 11:00-14:00 → `midday`
- 14:00-17:00 → `afternoon`
- 17:00+ → `eod`

## Paso 4: Ejecutar análisis de salud

**IMPORTANTE**: Solo ejecutar los análisis correspondientes al rol del usuario.

### Para roles con dominio proyecto (PM, Developer, TL):

```bash
# Extraer IDs de proyectos del contexto
PROJECT_IDS=$(echo "$CONTEXT" | python3 -c "
import json, sys
data = json.load(sys.stdin)
ids = [p['id'] for p in data.get('projects', [])]
print(json.dumps(ids))
")

# Si es midday/afternoon, primero comprobar si hay cambios
if [[ "$TIME_OF_DAY" != "morning" && "$TIME_OF_DAY" != "eod" ]]; then
    DELTA=$("$SCRIPT_DIR/check_activity_delta.sh" "$LAST_CHECK" \
        --project-ids "$PROJECT_IDS" --pilot-dir "$PILOT_DIR")

    HAS_CHANGES=$(echo "$DELTA" | python3 -c "
import json, sys
print(json.load(sys.stdin).get('has_changes', False))
")

    # Si no hay cambios, salida rápida
    if [[ "$HAS_CHANGES" == "False" ]]; then
        echo "Sin novedades desde las $(echo $STATE | python3 -c 'import json,sys; print(json.load(sys.stdin).get(\"last_check\",\"?\")[:5])')."
        # Actualizar timestamp y continuar
    fi
fi

# Ejecutar health monitor para cada proyecto
PROJECT_HEALTH=$("$SCRIPT_DIR/run_project_health.sh" "$PROJECT_IDS" --pilot-dir "$PILOT_DIR")
```

### Para roles con dominio CRM (Sales, Sales Manager, PM):

```bash
# Sales Manager: analizar todo el equipo
if [[ "$ROLE" == "sales_manager" ]]; then
    CRM_TEAM_ID=$(echo "$CONTEXT" | python3 -c "import json,sys; print(json.load(sys.stdin).get('crm_team_id', 0))")
    CRM_HEALTH=$("$SCRIPT_DIR/run_crm_health.sh" --team-id "$CRM_TEAM_ID" --pilot-dir "$PILOT_DIR")
# Sales/PM: analizar pipeline personal
else
    CRM_HEALTH=$("$SCRIPT_DIR/run_crm_health.sh" --user-id "$TARGET_UID" --pilot-dir "$PILOT_DIR")
fi
```

## Paso 4b: Obtener trabajo pendiente y plan de acción

**Para roles con dominio proyecto (PM, Developer, TL)** — ejecutar en morning y midday:

```bash
# Obtener health + pending work juntos (flag --fetch-pending)
HEALTH_DATA=$("$SCRIPT_DIR/run_project_health.sh" "$PROJECT_IDS" \
    --pilot-dir "$PILOT_DIR" --fetch-pending)

# Extraer health_results y pending_work del output combinado
PROJECT_HEALTH=$(echo "$HEALTH_DATA" | python3 -c "
import json, sys
data = json.load(sys.stdin)
print(json.dumps(data.get('health_results', data if isinstance(data, list) else [])))
")
PENDING_WORK=$(echo "$HEALTH_DATA" | python3 -c "
import json, sys
data = json.load(sys.stdin)
print(json.dumps(data.get('pending_work', [])))
")

# Generar plan de acción con gamificación
ACTION_PLAN=$(python3 -c "
import json
ctx = {
    'health_results': $PROJECT_HEALTH,
    'pending_work': $PENDING_WORK,
    'state': $STATE,
    'preferences': $PREFERENCES,
}
print(json.dumps(ctx))
" | python3 "$SCRIPT_DIR/build_action_plan.py")
```

## Paso 4c: CRM Actions (para roles Sales/Sales Manager/PM)

```bash
# Obtener historial de chatter para oportunidades
OPP_IDS=$(echo "$CONTEXT" | python3 -c "
import json, sys
data = json.load(sys.stdin)
ids = [o['id'] for o in data.get('crm_opportunities', [])]
print(json.dumps(ids))
")

CHATTER_HISTORY=$("$SCRIPT_DIR/fetch_crm_history.sh" "$OPP_IDS" \
    --limit 5 --pilot-dir "$PILOT_DIR")

# Generar acciones CRM
CRM_ACTIONS=$(python3 -c "
import json
ctx = {
    'crm_health': $CRM_HEALTH,
    'opportunities': $(echo $CONTEXT | python3 -c 'import json,sys; print(json.dumps(json.load(sys.stdin).get(\"crm_opportunities\", [])))'),
    'chatter_history': $CHATTER_HISTORY,
}
print(json.dumps(ctx))
" | python3 "$SCRIPT_DIR/build_crm_actions.py")
```

## Paso 5: Calcular nudges

```bash
# Construir contexto para el decision tree (incluye action plan + CRM actions)
NUDGE_CONTEXT=$(python3 -c "
import json
context = {
    'role': '$ROLE',
    'time_of_day': '$TIME_OF_DAY',
    'project_health': $PROJECT_HEALTH,
    'crm_health': $CRM_HEALTH,
    'state': $STATE,
    'preferences': $PREFERENCES,
    'activity_delta': ${DELTA:-null},
    'developer_context': $(echo $CONTEXT | python3 -c 'import json,sys; print(json.dumps(json.load(sys.stdin).get(\"developer\")))' 2>/dev/null || echo 'null'),
    'action_plan': ${ACTION_PLAN:-null},
    'crm_actions': ${CRM_ACTIONS:-null},
    'gamification': $(echo ${ACTION_PLAN:-'{}'} | python3 -c 'import json,sys; print(json.dumps(json.load(sys.stdin).get(\"gamification\")))' 2>/dev/null || echo 'null'),
}
print(json.dumps(context))
")

NUDGE=$(echo "$NUDGE_CONTEXT" | python3 "$SCRIPT_DIR/calculate_nudge.py")
```

## Paso 6: Generar salida

Lee la plantilla correspondiente de `templates/` y rellena con los datos del nudge.

**Consulta las plantillas en `./templates/`** para el formato exacto de cada tipo de salida:
- `morning-briefing.md` → Briefing completo
- `midday-check.md` → Revisión breve
- `afternoon-pulse.md` → Pulso de plazos
- `eod-summary.md` → Resumen del día

**Adapta el contenido al rol**:
- **PM**: Muestra scores de proyecto + alertas + demos próximas
- **Developer**: Muestra tareas pendientes + planning slots + horas
- **TL**: Muestra vista agregada del equipo + proyectos en riesgo
- **Sales**: Muestra pipeline CRM + opps estancadas + skills sugeridos
- **Sales Manager**: Muestra CRM del equipo + scoring por vendedor

## Paso 7: Actualizar estado

```bash
# Guardar nuevo estado
echo "$NUDGE" | python3 -c "
import json, sys
nudge = json.load(sys.stdin)
new_state = nudge.get('new_state', {})
print(json.dumps(new_state))
" | python3 "$SCRIPT_DIR/update_state.py" write

# Si es EOD, guardar en historial
if [[ "$TIME_OF_DAY" == "eod" ]]; then
    echo "$NUDGE" | python3 -c "
import json, sys
nudge = json.load(sys.stdin)
entry = nudge.get('history_entry', {})
print(json.dumps(entry))
" | python3 "$SCRIPT_DIR/update_state.py" append-history

    # Actualizar MEMORY.md con resumen del día
    # (Claude actualiza el archivo directamente)
fi
```

## Paso 8: Publicar en Odoo Chatter (si aplica)

Solo para alertas CRITICAL y WARNING, si el canal "chatter" está en las preferencias:

```bash
# Para cada alerta CRITICAL/WARNING en proyectos
"$SCRIPT_DIR/draft_chatter_note.sh" project.project "$PROJECT_ID" \
    "<p><strong>🔴 Odoo Mentor - Alerta</strong></p><p>$MESSAGE</p>" \
    --pilot-dir "$PILOT_DIR"
```

**IMPORTANTE**: Si `impersonation.active == true`, NO publicar notas en chatter (modo lectura).

## Paso 9: Configurar loop (solo en morning)

Si `time_of_day == morning` y es una sesión de tarea programada:

```
/loop 3h Consulta Odoo para cambios desde la última revisión. Compara write_dates.
Si hay cambios: genera actualización breve con solo información nueva y recomendaciones de skills.
Si no hay cambios: output "Sin novedades."
Si la hora actual > 17:00: genera resumen EOD (si es viernes: revisión semanal), actualiza MEMORY.md, cancela este loop.
```

## Paso 10: Viernes - Revisión Semanal

Si es viernes y `time_of_day == eod`:

1. Incluir sección de revisión semanal en el resumen
2. Comparar scores con los del lunes (de `history.json`)
3. Para cada proyecto con score < 70, sugerir ejecutar `/project-weekly-update`
4. Actualizar MEMORY.md con resumen semanal
