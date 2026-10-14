# Dashboard TL — Semana {semana}

**Equipo**: {equipo} | **TL**: {nombre_tl} | **Fecha**: {fecha}

---

## Metricas Clave

| Metrica | Valor | Tendencia | Objetivo |
|---------|-------|-----------|----------|
| Horas facturables equipo | {h_fact} | {trend_fact} | >60% |
| Tickets abiertos | {n_tickets} | {trend_tickets} | <{n_devs*3} |
| Tickets urgentes | {n_urgentes} | {trend_urg} | 0 |
| Tareas bloqueadas | {n_bloq} | {trend_bloq} | 0 |
| Ratio planificado/real | {ratio_est} | {trend_est} | 0.9-1.1 |
| Capacidad utilizada | {util}% | {trend_util} | 70-85% |
| Deuda tecnica (custom/total) | {debt}% | {trend_debt} | <30% |

## Capacidad del Equipo

| Dev | Planif. | Real | % Facturable | Tickets | Alerta |
|-----|---------|------|-------------|---------|--------|
| {dev} | {h_plan} | {h_real} | {pct_fact}% | {tickets} | {alerta} |
| **TOTAL** | {total_plan} | {total_real} | {total_pct}% | {total_tick} | |

## Estado de Proyectos

| Proyecto | Tareas activas | Horas semana | Release previsto | Estado |
|----------|---------------|-------------|-----------------|--------|
| {proyecto} | {n_tareas} | {h_semana} | {fecha_release} | {estado} |

## Tickets por Antiguedad

| Rango | Cantidad | Accion sugerida |
|-------|----------|----------------|
| <3 dias | {n1} | Normal |
| 3-7 dias | {n2} | Revisar prioridad |
| >7 dias | {n3} | Escalar |
| >14 dias | {n4} | Reunion urgente |

## Estimaciones: Calibracion

| Tamano | Ratio plan/real | Tendencia | Correccion sugerida |
|--------|----------------|-----------|---------------------|
| XS (0-2h) | {r_xs} | {t_xs} | {c_xs} |
| S (2-8h) | {r_s} | {t_s} | {c_s} |
| M (8-20h) | {r_m} | {t_m} | {c_m} |
| L (20-40h) | {r_l} | {t_l} | {c_l} |
| XL (>40h) | {r_xl} | {t_xl} | {c_xl} |

## Alertas Activas

- {emoji} **{tipo_alerta}**: {descripcion} — Accion: {accion}

## Forecast 4 Semanas

| Semana | Cap. Neta | Carga | Delta | Alertas |
|--------|-----------|-------|-------|---------|
| {semana_1} | {cap_1} | {carga_1} | {delta_1} | {alertas_1} |
| {semana_2} | {cap_2} | {carga_2} | {delta_2} | {alertas_2} |
| {semana_3} | {cap_3} | {carga_3} | {delta_3} | {alertas_3} |
| {semana_4} | {cap_4} | {carga_4} | {delta_4} | {alertas_4} |

## Skills del Equipo (Resumen)

| Skill | Cobertura | Nivel medio | Referentes | Bus Factor |
|-------|-----------|-------------|------------|------------|
| {skill} | {cobert}% | {media} | {refs} | {bus} |

## Action Items del TL

| Prioridad | Accion | Origen | Fecha limite |
|-----------|--------|--------|-------------|
| {prio} | {accion} | {origen} | {deadline} |

---

> Generado con datos de: `/standup-facilitator`, `/team-task-assigner`,
> `/timesheet-report`, `/ticket-report`, `estimation_history.py`, `skill_matrix.py`
