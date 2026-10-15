# Reglas de alertas y escalación

## Filosofía

El skill detecta **6 alertas binarias** (sí / no — sin niveles de severidad) y un caso especial de **ALERTA MÁXIMA** cuando una alerta sigue activa tras una acción ya comprometida por el responsable la semana anterior. No se calcula health score ni se comparan métricas contra baseline.

El **baseline** se guarda al dar de alta el proyecto y sólo se modifica explícitamente con `update-baseline` o `update-prod-date`. Sirve como referencia histórica, no para detectar drift.

---

## Las 6 alertas

| # | Tipo | Cuándo se enciende | Detalle |
|---|---|---|---|
| 1 | `overdue_tasks` | ≥1 tarea con `date_deadline < hoy` y `remaining_hours > 0` | Excluye tareas en `completed_task_stages` |
| 2 | `tasks_without_date` | ≥1 tarea sin `date_deadline` | Excluye tareas en `completed_task_stages` |
| 3 | `no_movement` | ≥7 días sin cambios en horas consumidas ni checklists completados | Sólo si hay snapshot previo en `history` |
| 4 | `advance_gap` | `T_pct − C_pct > 10` puntos | Configurable en `alert_thresholds.advance_gap_pp` |
| 5 | `tasks_without_checklists` | ≥1 tarea con `allocated_hours > 4` sin ningún checklist definido | Excluye tareas en `completed_task_stages` |
| 6 | `client_update_missing` | No hay `project.update` enviado al cliente en últimos 7 días | Detectado por `mail.message` con subtype Discussions |

### Criterio "enviado al cliente"

Para considerar que un `project.update` se envió al cliente:
- Existe un `mail.message` en su chatter (`model='project.update'`, `res_id=<update_id>`).
- `subtype_id == 1` (Discussions, equivalente a publicación pública).
- `partner_ids` no vacío (hay al menos un destinatario).

Las **notas internas** (subtype Note) y las **notificaciones automáticas** (message_type=notification) NO cuentan.

---

## Etapas terminadas

Configuradas en `config.json: completed_task_stages`. Por defecto:

```
["Realizado", "Hecho", "Realizada", "Done", "Cancelado"]
```

Las tareas en estas etapas se **excluyen** de los conteos de las alertas 1, 2 y 5. La razón: son trabajo ya hecho, no necesitan tener fecha ni checklist activos.

---

## Métricas T%, C%, H%

- **T%** (Timeline): `(hoy − date_start) / (date_end − date_start) × 100`. % de tiempo del calendario consumido.
- **C%** (Checklists, ponderado): `Σ(peso completado) / Σ(peso total) × 100`. Avance real del trabajo.
- **H%** (Horas): `(total_hours − total_hours_available) / total_hours × 100`. Consumo de horas presupuestadas.

T puede superar el 100% si el proyecto se ha retrasado (date_end ya pasó). Es información, no alerta.

La alerta `advance_gap` salta cuando `T − C > 10pp`: estamos consumiendo más calendario del que estamos entregando trabajo.

---

## Escalación → ALERTA MÁXIMA

Una alerta se marca como **ALERTA MÁXIMA** cuando:

1. La review **anterior** (penúltima en `reviews[]`) tenía un `action_committed` no nulo.
2. La misma alerta del review anterior **sigue activa** en la review actual.

Es decir: el responsable se comprometió a hacer algo, y a la siguiente revisión la alerta no se ha resuelto.

En el email se renderiza un bloque destacado:

> **ALERTA MÁXIMA — acción comprometida no realizada**
>
> En el seguimiento del 2026-04-27 me comentaste:
> > "Replanificará las 8 tareas vencidas antes del viernes"
>
> La misma alerta sigue activa. Necesito explicación de qué ha pasado y la nueva fecha en que estará resuelto.

Se calcula sobre cualquier alerta repetida con `action_committed` previo, no sólo sobre la primera.

---

## Salida a producción y fecha fin de proyecto

**Tratamiento asimétrico**:

- **`production_date`** (campo Studio `x_studio_fecha_prevista_salida_produccin`): es la fecha **viva**. Si se mueve, hay que actualizarla con `update-prod-date`. El email pide siempre confirmación de esta fecha.
- **`date_end`** (campo Odoo `date`): es la fecha **histórica del baseline** del proyecto. Si el proyecto se retrasa, NO se actualiza — sólo se mueve la `production_date`. Por eso una `date_end` vencida (en el pasado) **no es alerta** y no se incluye en el recordatorio del email.

El recordatorio final del email lista únicamente `production_date` por proyecto.

---

## Configuración

Todos los umbrales en `config.json`:

```json
{
  "alert_thresholds": {
    "no_movement_days": 7,
    "advance_gap_pp": 10
  },
  "completed_task_stages": ["Realizado", "Hecho", "Realizada", "Done", "Cancelado"],
  "cadence_rules": {
    "stage_always_weekly": ["Análisis"],
    "biweekly_min_duration_days": 90
  }
}
```
