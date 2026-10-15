# Árbol de Decisión - Clasificación de Alertas

Documento de referencia para la lógica de clasificación de alertas y nudges del Mentor.

---

## Entrada

```
- role: pm | developer | tl | sales | sales_manager
- project_scores: [{project_id, score, prev_score, scenario}]
- crm_scores: {score, stale_rate, win_rate, governance_pct}
- time_of_day: morning | midday | afternoon | eod
- state: alertas previas emitidas hoy, número de iteración del /loop
```

---

## Paso 1: Filtro de cambios

Antes de generar cualquier salida, comprobar si algo ha cambiado desde la última iteración.

| Condición | time_of_day | Resultado |
|-----------|-------------|-----------|
| NO hay cambios | `morning` | Briefing mínimo con scores actuales (sin alertas) |
| NO hay cambios | `midday` / `afternoon` | **SILENCIO** — "Sin novedades desde las {{hora}}. ¡Sigue así!" |
| NO hay cambios | `eod` | Resumen del día con evolución (aunque sea plano) |
| SÍ hay cambios | cualquiera | Continuar al Paso 2 |

> **Definición de "cambio"**: delta de score >= umbral (default 5 puntos), cambio de etapa en opp, nueva tarea vencida, cambio de estado de tarea, nuevo timesheet, nueva cita.

---

## Paso 2: Clasificación por dominio

### Dominio Proyecto (roles: PM, Developer, TL)

| Nivel | Condición | Color | Descripción |
|-------|-----------|-------|-------------|
| CRITICAL | `score < 40` O `caída > 15 puntos` en una iteración | 🔴 | Proyecto en peligro. Intervención urgente. |
| WARNING | `score 40-69` O `caída 10-15 puntos` | 🟠 | Proyecto en riesgo. Requiere atención. |
| MONITOR | `score 70-79` O `caída 5-9 puntos` | 🟡 | Proyecto en vigilancia. Tendencia negativa. |
| HEALTHY | `score >= 80` Y `estable o subiendo` | 🟢 | Proyecto sano. Sin acción necesaria. |

**Reglas adicionales para proyectos:**
- Si `go-live en <= 14 días` Y `score < 70` → subir un nivel de urgencia
- Si `checklist_items_vencidos > 3` → mínimo WARNING
- Si `bottleneck_count > 0` con `risk_score > 7` → mínimo WARNING

### Dominio CRM (roles: Sales, Sales Manager, PM)

| Nivel | Condición | Color | Descripción |
|-------|-----------|-------|-------------|
| CRITICAL | `CRM score < 40` O `win_rate < 20%` | 🔴 | Pipeline en peligro. Acción inmediata. |
| WARNING | `CRM score 40-69` O `stale_rate > 30%` | 🟠 | Pipeline débil. Requiere atención. |
| MONITOR | `CRM score 70-79` O `governance_pct > 15%` | 🟡 | Pipeline en vigilancia. Datos incompletos. |
| HEALTHY | `CRM score >= 80` | 🟢 | Pipeline sano. |

**Reglas adicionales para CRM:**
- Si `opp estancada > 15 días` en etapa post-cualificación → mínimo WARNING
- Si `demo en <= 2 días` Y `sin guión preparado` → mínimo WARNING
- Si `presupuesto sin respuesta > 21 días` → mínimo WARNING

### Dominio Equipo (roles: TL, Sales Manager)

| Nivel | Condición | Color | Descripción |
|-------|-----------|-------|-------------|
| CRITICAL | `empleado sin horas > 5 días laborables` | 🔴 | Anomalía grave en imputación. |
| WARNING | `empleado sin horas 3-5 días laborables` | 🟠 | Posible olvido de imputación. |
| MONITOR | `empleado sin horas 2 días laborables` | 🟡 | Seguimiento leve. |

---

## Paso 3: Agregación cross-domain

Cuando un usuario tiene visibilidad en múltiples dominios (ej: PM ve proyectos y CRM), se aplica agregación:

- **El peor nivel gana** para el resumen general
- Si `proyecto = WARNING` + `CRM = CRITICAL` → nivel general = **CRITICAL**
- Cada dominio mantiene su clasificación individual en las tablas detalladas
- El estado general del dashboard refleja el nivel agregado

**Tabla de agregación:**

| Proyecto | CRM | Equipo | Resultado |
|----------|-----|--------|-----------|
| HEALTHY | HEALTHY | HEALTHY | HEALTHY |
| Cualquiera | CRITICAL | Cualquiera | CRITICAL |
| CRITICAL | Cualquiera | Cualquiera | CRITICAL |
| WARNING | WARNING | Cualquiera | WARNING |
| MONITOR | HEALTHY | MONITOR | MONITOR |

---

## Paso 4: Filtro por rol

Cada rol solo recibe datos de los dominios que le corresponden.

| Rol | Proyectos | CRM | Equipo | Horas propias |
|-----|-----------|-----|--------|---------------|
| PM | ✅ (propios) | ✅ (demos en sus proyectos) | ❌ | ❌ |
| Developer | ✅ (sus tareas) | ❌ | ❌ | ✅ |
| TL | ✅ (depto.) | ✅ (si aplica) | ✅ | ❌ |
| Sales | ❌ | ✅ (propio) | ❌ | ❌ |
| Sales Manager | ❌ | ✅ (todo el equipo) | ✅ (vendedores) | ❌ |

---

## Paso 5: Reglas anti-molestia

Aplicar las reglas definidas en `smart_frequency.md`:

1. **Deduplicación diaria** — ¿Ya se emitió esta misma alerta hoy?
2. **Umbral de delta** — ¿El cambio supera el umbral mínimo (default 5 puntos)?
3. **Cooldown post-CRITICAL** — ¿Han pasado >= 2h desde la última alerta CRITICAL del mismo ítem?
4. **Límite de elogios** — ¿Se ha emitido ya un elogio hoy?
5. **Modo minimal** — ¿El usuario tiene `minimal_mode = true`? Si es así, suprimir MONITOR y HEALTHY.
6. **Silencio inteligente** — ¿Es mediodía/tarde sin cambios? → Silencio.

> Véase `references/smart_frequency.md` para la documentación completa de cada regla.

---

## Paso 6: Adjuntar recomendaciones de skills

Para cada alerta que sobreviva los filtros, comprobar si hay un skill recomendado aplicable.

- Consultar la matriz de recomendaciones en `references/skill_recommendations.md`
- Máximo **3 recomendaciones** por revisión
- No repetir la misma recomendación para el mismo ítem en el mismo día
- Priorizar recomendaciones vinculadas a alertas CRITICAL > WARNING > MONITOR

> Véase `references/skill_recommendations.md` para la matriz completa.

---

## Paso 7: Seleccionar plantilla

Según el `time_of_day`, seleccionar la plantilla de salida:

| time_of_day | Plantilla | Ruta |
|-------------|-----------|------|
| `morning` | Briefing matutino completo | `templates/morning-briefing.md` |
| `midday` | Revisión mediodía | `templates/midday-check.md` |
| `afternoon` | Pulso vespertino | `templates/afternoon-pulse.md` |
| `eod` | Resumen del día | `templates/eod-summary.md` |

**Reglas especiales de plantilla:**
- Si `eod` + `es viernes` → incluir sección de revisión semanal en `eod-summary.md`
- Si `morning` + `primera ejecución` → usar `templates/onboarding-welcome.md` en su lugar
- Si la salida resultante está vacía tras filtros → aplicar silencio inteligente (Paso 5.6)
