# Assignment — Reparto de migraciones entre devs

Sustituye el **Excel que montaba Luis** para repartir las 16+ migraciones (+2 Suiza)
entre los devs (Toni / Rafa / Aylen, con Luis de vacaciones). El reparto **balancea
por horas estimadas**, no por número de migraciones.

## Algoritmo de reparto
1. Tomar todas las migraciones con `status` ∈ {`intake`, `assigned`} y sus
   `est_hours` (de `migration-estimator`; ver `intake.md`).
2. Ordenar migraciones por `est_hours` **descendente** (las grandes primero).
3. Asignación greedy: cada migración va al dev con **menor carga acumulada**.
4. Respetar restricciones manuales (cliente preasignado, skill específica, idioma
   para las 2 de Suiza si aplica).
5. Fijar `assignee`, `target_date` y `priority` (1 alta / 2 media / 3 baja).
6. Recalcular y mostrar la carga por dev (suma de horas y nº de migraciones).

## Balanceo
- Una migración de 150h pesa como varias pequeñas: nunca contar "1 migración = 1 cupo".
- Objetivo: minimizar la diferencia de **horas acumuladas** entre devs.
- Avisar si un dev queda muy por encima del rango sano (ver DESIGN: 100-150h/migración).

## Formato de salida (Excel/CSV)
Columnas del listado de reparto (compatibles con el Excel previo):

```
client, contact(=Janik), version_src, version_dst, db_size,
est_hours, est_eur, assignee, target_date, priority, status, project_id, sale_order
```

- Generar como **CSV** (Write) para abrir en Excel; opcionalmente una tabla Markdown
  para revisión rápida en el chat.
- Una fila por migración; una tabla resumen de carga por dev al final.

## Reglas
- **No re-estimar**: usa `est_hours` tal cual; si falta, marca pendiente.
- **No ejecutar lo técnico** al asignar: solo planifica. La ejecución la lanza el
  Workflow delegando en `dynapps-migrator` / `migration-project-creator`.
- **NUNCA enviar correo desde Odoo** para notificar el reparto.
- Persistir el reparto en el estado de la cartera (ver `gate-tracking.md`).
