# Presupuesto vs real por gate

Objetivo: comparar lo **estimado en Fase 1** (`migration-estimator`) con las **horas
reales** consumidas (`timesheet-report`), por gate, y calcular el desvío. Es control
de gestión, **no** base de facturación (se factura por hito, no por horas).

## Fuentes
- **Estimado**: los 4 buckets de Fase 1 (Code Migration, Database Upgrade, Unit
  Testing, Project Management). Lo fijó `migration-estimator`; aquí **no se
  recalcula**. Si no existe, marcar "estimado no disponible".
- **Real**: horas de `timesheet-report` por proyecto de la migración. Excluir
  holiday_id / global_leave_id (regla del entorno) y a los CTOs si aplica.

## Mapeo bucket → gate (supuesto, hazlo explícito)
El estimado es por bucket y el avance es por gate; el mapeo **no es 1:1**. Mapeo por
defecto (documentar siempre como supuesto):

| Gate | Buckets que consume |
|---|---|
| 1 Informe enviado | (Fase 1, ya hecho — normalmente fuera del control de desvío de ejecución) |
| 2 Entrega | Code Migration + Database Upgrade + Unit Testing |
| 3 Salida a producción | Project Management (seguimiento) + correcciones reportadas |

Las **correcciones** que pida Dynapps (alcance Indaws) caen tras la entrega; si
generan horas reales no presupuestadas, aparecen como desvío del gate 3.

## Cálculo
Por gate (y total):

```
desvio_h   = real_h - estimado_h
desvio_pct = desvio_h / estimado_h * 100      (si estimado_h > 0)
desvio_eur = desvio_h * 45
```

- `desvio_h > 0` → sobrecoste (real supera estimado) → **rojo**.
- `desvio_h <= 0` → dentro de presupuesto → verde.
- `estimado_h = 0` y `real_h > 0` → trabajo no presupuestado → **rojo**, nota
  explícita (típico de correcciones).

## Tabla de salida

| Migración | Gate | Estimado h | Real h | Desvío h | Desvío % | Desvío € | Estado |
|---|---|---|---|---|---|---|---|
| Luminex | 2 Entrega | 90 | 78 | -12 | -13% | -540 | ✅ |
| Luminex | 3 Producción | 10 | 22 | +12 | +120% | +540 | 🔴 |

Añadir fila **TOTAL** por migración y, si la vista es de cartera, total global.

## Reglas
- Valorar siempre a **45 €/h**.
- No mezclar el desvío (control) con la facturación (por hito). Son columnas/vistas
  distintas; la factura del gate no cambia porque haya desvío.
- El desvío rojo es **aviso**, no bloqueo: renegociar con Dynapps es decisión humana.
- Si el mapeo bucket→gate es dudoso, reporta a nivel de migración (total) y avisa de
  la imprecisión en vez de inventar precisión por gate.

## Delegación
- Horas reales: `timesheet-report`.
- Cruce carga equipo vs facturación: `workload-billing-monitor` (management). No
  recalcular carga aquí.
