# Snapshot: output y presentación al TL

## Output de `snapshot.py` (5 bloques pipe-separated)

```
=== CONFIG ===
Pasar siguiente semana|1780
Planificar (1ª Semana)|1748
stage_a_planificar|1008
emp:26|uid:112|Brenda
emp:14|uid:45|Daniel Malaver
...

=== PENDIENTES SEMANA ANTERIOR (3) ===
slot_id|task_id|tarea|dev|horas|proyecto|url
22100|16640|Database migration v18|Akash|40:00|Eurolotes|https://...

=== TAREAS A PLANIFICAR (8) ===
ALERTA_SIN_HORAS:2
task_id|prio|tarea|proyecto|horas|actual|sugerido|score|motivo|url
16150|Urgente|Montar servidores|Eurolotes|8:00|Toni|Toni|92|plan_user + historial 17:30|https://...
17092|Alta|Lotes automaticos|Gastraval|8:00||Daniel|78|historial 10:18|https://...
17115|Alta|Integracion Pasarelas|Aroma|0:00|||0|SIN_HORAS|https://...

=== CAPACIDAD EQUIPO ===
dev|disponible_ini|ya_asignado|ausencia|disponible_post|alerta|tickets_abiertos
Brenda|30:00|10:00|0:00|22:00||3
Daniel|32:00|8:00|0:00|24:00||1
Akash|0:00|40:00|0:00|0:00|SIN_DISPONIBILIDAD|5
TOTAL_DISPONIBLE_INI:62:00
TOTAL_DISPONIBLE_POST:46:00

=== RESUMEN ===
pendientes:3
tareas:8
asignadas:6
sin_candidato:0
sin_horas:2
```

## Presentación al TL: 3 tablas markdown

**Tabla 1: Pendientes** (si hay)

| Slot | Tarea | Dev | Horas | Proyecto | Enlace |
|------|-------|-----|-------|----------|--------|

Preguntar: *"Cuales estan hechas? Cuales replanificar? (por defecto replanifica todas)"*

**Tabla 2: Propuesta de asignacion**

| # | ID | Prio | Tarea | Proyecto | Horas | Sugerido | Score | Motivo | Enlace |
|---|---|---|---|---|---|---|---|---|---|

Si hay tareas con `SIN_HORAS`: alertar y pedir horas al TL antes de continuar.

**Tabla 3: Capacidad**

| Dev | Disponible | Tras asignar | Alerta | Tickets |
|-----|------------|-------------|--------|---------|
