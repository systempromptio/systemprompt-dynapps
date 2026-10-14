# Plantilla: Candidatos a migración

Entornos por detrás de una versión objetivo, con el footprint de personalización como proxy de complejidad. Lógica
**agnóstica de versión** en `metrics-and-thresholds.md` (sección "Candidatos a migración"). **Datos:** receta
"Versión / edición" para las series (1 consulta) + footprint desde `env_data` por candidato (una lectura por entorno).
Reglas en `report-rules.md`.

## Versión objetivo (`target`)
- Si el usuario la indica ("¿qué entornos son viables de migrar a la **20**?"), usa esa.
- Si no, `target = max(serie mayor del parque)`. **Indica en la cabecera cuál usaste y de dónde sale.**

Candidato = `serie < target`; `lag = target − serie`.

## Formato de salida

```
# 🚀 Candidatos a migración → v[TARGET] — [DD/MM/AAAA]
*Objetivo v[TARGET] ([indicado por el usuario | máx. del parque]) · [N] candidatos de [TOTAL] conectados*

| Entorno | Versión | lag | Studio (campos/vistas) | Automations | Crons | LOC custom | Complejidad | Urgencia |
|---|---|---|---|---|---|---|---|---|
| cliente-x.indaws.cloud | 18.0+e | 2 | 78 / 38 | 17 | 1 | 12.400 | 🔴 Alta | 🟠 2 por detrás |
| cliente-y.indaws.cloud | 19.0+e | 1 | 5 / 2  | 1  | 0 | 800 | 🟡 Media | 🟡 1 por detrás |
| cliente-z.indaws.cloud | 17.0   | 3 | 0 / 0  | 0  | 0 | sin dato | ✅ Baja | 🔴 EOL |
```
Omite la columna "LOC custom" si ningún candidato del informe tiene aún `loc_total` calculado (flota en proceso de
rellenarse) — no la rellenes con ceros ni la fuerces si no aporta.

## Reglas de contenido
- **Serie** = entero antes del primer punto de `odoo_version` (`"18.0+e"→18`, `"saas~20.1"→20`).
- **Urgencia** por `lag` (ventana de soporte = 3): ≥3 🔴 EOL · 2 🟠 · 1 🟡 · 0 = no candidato.
- **Complejidad (orientativa, NO horas):** `studio_fields + studio_views + automated_actions + custom_crons` →
  ✅ Baja ≤10 · 🟡 Media 11–50 · 🔴 Alta >50. Si hay `loc_total` para los módulos custom del entorno, muéstralo como
  columna adicional (líneas de código real, no cuenta de elementos) — refuerza o matiza la lectura del conteo, no la
  sustituye. La estimación real la hace `odoo-effort-estimator`.
- Orden: por urgencia (lag) desc y, dentro, por complejidad desc. Enterprise con licencia caducando sube prioridad
  (señálalo).
- **Hosting Odoo.sh:** si el entorno está en Odoo.sh, añade la fecha en que su versión sale de Odoo.sh (deadline duro
  del SSOT `references/shared/odoo-lifecycle-licensing.md`) como urgencia extra y palanca para traerlo a hosting
  Indaws. Enterprise en una versión fuera de las 3 más recientes (hoy ≤ v16): señala "posible +25% al renovar" (confirmar con comercial).
- Si nadie está por detrás del `target`, dilo ("Todos al día respecto a v[TARGET]").

**Cierre:** ofrece priorizar/cruzar (p. ej. "los EOL con licencia caducando", "solo hosting Indaws", "y si el objetivo
fuera la 19 en vez de la 20").
