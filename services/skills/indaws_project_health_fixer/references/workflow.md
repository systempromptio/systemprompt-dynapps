# project-health-fixer: Workflow Details

## Paso 0: Recopilacion de Datos (STOP)

**SIEMPRE** recopila antes de empezar:

1. **Credenciales Odoo** (si no configuradas): URL, DB, API Key, Usuario
2. **ID del proyecto** en Odoo
3. **Ruta al JSON de metricas** de `calculate_health.py` (ej. `/tmp/phm_metrics.json`)
4. **Ruta al JSON de datos raw** de `fetch_project_data.sh` (ej. `/tmp/phm_data.json`)
5. **Documento de analisis** (opcional): Markdown de `project-analysis` para comparar planificado vs real
6. **Alcance deseado**:
   - "Solo gobernanza" (rapido, solo Pasos 2-3)
   - "Revision completa" (gobernanza + progreso + desviacion si aplica)

**Si no hay JSONs disponibles**: Buscar diagnostico en `project.update` mas reciente (`x_studio_internal_report`). Para ejecutar correcciones, re-extraer:
```bash
./scripts/fetch_project_data.sh <project_id> --pilot-dir /path/to/odoo-pilot/scripts > /tmp/phm_data.json
python3 ../project-health-monitor/scripts/calculate_health.py --input /tmp/phm_data.json > /tmp/phm_metrics.json
```

**DETENTE** y espera respuesta del usuario.

---

## Paso 1: Diagnostico Automatico

```bash
python3 ./scripts/diagnose_issues.py \
  --metrics /tmp/phm_metrics.json \
  --project-data /tmp/phm_data.json > /tmp/phf_diagnosis.json
```

El script clasifica TODOS los problemas en 3 categorias:
- **Gobernanza**: Corregibles directamente en Odoo (fechas, horas, checklists)
- **Progreso**: Requieren decision del consultor (vencidos, bottlenecks)
- **Estrategico**: Requieren replanteo (desviacion)

---

## Paso 1.5: Validacion del Diagnostico (STOP)

Presenta resumen al consultor con:
- Score actual vs score potencial
- Escenario detectado
- Tabla de problemas de Gobernanza (con penalizacion, afectados, fix)
- Tabla de problemas de Progreso (afectados)
- Evaluacion Estrategica (si requiere desviacion)
- Comparacion con analisis (si se proporciono)

**DETENTE**. El consultor puede aprobar, ajustar prioridades o saltar categorias.

---

## Paso 2: Proponer Correcciones de Gobernanza

Consulta `./references/fix_patterns.md` y propone valores concretos:

- **Tareas sin fechas**: Analizar cronograma y proponer deadlines logicos
- **Desajuste de horas**: Identificar tareas con horas consumidas > 0 pero allocated_hours = 0
- **Tareas sin checklists**: Extraer items de Seccion 6 del analisis o proponer items genericos

---

## Paso 2.5: Validacion del Batch (STOP)

Presenta tabla detallada con cada operacion propuesta:
- Tabla de fechas limite propuestas (Tarea, ID, Deadline, Razon)
- Tabla de redistribucion de horas (Tarea, ID, Actual, Propuesto, Razon)
- Tabla de checklists a crear (Tarea, Item, Peso, Fuente)
- Total: X updates, Y creates. Penalizacion recuperada estimada

**DETENTE**. El consultor puede aprobar, modificar valores o rechazar operaciones individuales.

---

## Paso 3: Ejecutar Correcciones de Gobernanza

Ejecutar operaciones aprobadas via odoo-pilot:
```bash
$PILOT_DIR/update_record.sh "project.task" '[TASK_ID]' '{"date_deadline":"2026-03-15"}'
$PILOT_DIR/update_record.sh "project.task" '[TASK_ID]' '{"allocated_hours":40}'
$PILOT_DIR/create_record.sh "xx.checklist.item" '{"name":"...","project_id":ID,"task_id":TID,"weight":P,"state":"pending",...}'
```

**Reglas**: Una a una, si falla continuar con siguiente, fechas en YYYY-MM-DD.

---

## Paso 3.5: Resultado de Ejecucion (STOP)

Tabla con resultado individual por operacion (Operacion, Modelo, ID, Resultado).
Si alcance es "Solo gobernanza", saltar a Paso 7.

---

## Paso 4: Plan de Accion para Progreso

Para cada **tarea vencida**, presentar opciones con recomendacion (Cerrar, Reprogramar, etc.).
Para cada **item de checklist vencido**: Completar o Reprogramar.
Para **bottlenecks** (top 5 por risk_score): Priorizar, reasignar o dividir.

Consulta `./references/fix_patterns.md` seccion 2 para arboles de decision.

---

## Paso 4.5: Validacion del Plan de Accion (STOP)

Tabla consolidada de decisiones por cada tarea/item/bottleneck.

**DETENTE**.

---

## Paso 5: Ejecutar Cambios de Progreso

Decisiones aprobadas se ejecutan via odoo-pilot, igual que Paso 3.

---

## Paso 6: Evaluacion de Desviacion (CONDICIONAL)

**Se activa si**: Score < 40 Y factor de aceleracion > 2.0, O T% > 100 Y C% < 80, O consultor lo solicita.

Presenta 3 opciones: Extension, Reduccion scope, Mixto.
Para cada opcion calcula: semanas adicionales, tareas aplazadas, score proyectado.

Consulta `./references/fix_patterns.md` seccion 3 para formulas.

---

## Paso 6.5: Validacion de Desviacion (STOP, condicional)

El consultor elige opcion, ajusta. Ejecutar cambios aprobados.

---

## Paso 7: Generar Informe de Correccion

Template: `./templates/fix_report.md`. Fichero: `informe_correccion_{proyecto}_{YYYYMMDD}.md`

6 secciones: Diagnostico inicial, Correcciones gobernanza, Decisiones progreso, Plan desviacion, Comparacion analisis, Recomendacion + proxima evaluacion.

---

## Comparacion con Documento de Analisis

**Trabajo del LLM** (sin script adicional):
1. Lee documento de analisis Markdown
2. Extrae tareas de Seccion 6 (Checklist General) y Seccion 7 (Tareas Detalladas)
3. Compara semanticamente con tareas en Odoo (del JSON de datos raw):
   - Tareas del analisis sin equivalente en Odoo -> "Falta crear en Odoo"
   - Tareas en Odoo sin equivalente en analisis -> "Posible scope creep"
   - Items de checklist del analisis vs items en `xx.checklist.item` -> cobertura
4. Presenta diferencias como parte del diagnostico (Paso 1.5)

**Razon**: El matching es semantico y los documentos no tienen formato uniforme.
