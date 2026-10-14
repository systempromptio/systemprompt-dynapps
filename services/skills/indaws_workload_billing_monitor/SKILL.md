---
name: indaws-workload-billing-monitor
version: 1.0.3
effort: medium
model: sonnet
description: >-
  Analiza la carga de trabajo planificada vs facturación por proyecto y empleado en Odoo. Detecta
  proyectos sin hitos de facturación (alerta roja) o con facturación insuficiente (amarilla)
  respecto a horas planificadas × 75€. Presenta dashboard y genera informe mensual de 7 secciones.
  Solo lectura. Usar cuando pidan analizar carga planificada vs facturación por proyecto.
---

# Monitor de Carga de Trabajo vs Facturación

## Workflow

1. **Paso 0 — Recopilación**: Pedir mes/año a analizar (default: mes actual). DETENTE y espera.
2. **Paso 1 — Extracción**: Ejecutar `scripts/fetch_workload_data.sh <YYYY> <MM>`. Obtiene 4 conjuntos: empleados (Consultoría/Desarrollo/Marketing externo), planificación mensual, hitos de facturación, proyectos. Excluye automáticamente proyectos con "Interno" o "Indaws" en el nombre.
3. **Paso 2 — Análisis**: Ejecutar `python3 scripts/analyze_workload.py < /tmp/wbm_data.json > /tmp/wbm_analysis.json`. Calcula cobertura = facturación_real / (horas × 75€). Ver `references/alert_rules.md`.
4. **Paso 2.5 — Dashboard**: DETENTE — presentar KPIs globales + distribución de alertas + top alertas rojas. Esperar confirmación.
5. **Paso 3 — Informe**: Solo tras validación. Generar `informe_carga_facturacion_{YYYY}_{MM}.md` con 7 secciones usando `templates/informe_carga.md`.

Ver `references/workflow-detail.md` para el detalle completo.

> Note: `common:references/shared/` — auth via `auth.sh`, context `es_ES`.

## Field contract (preflight OBLIGATORIO)
Antes de leer Odoo, verifica que existen (ver `references/shared/field-contract-preflight.md`):
- modelo `project.milestone`: campo `x_studio_price`
Si falta algo → STOP con mensaje accionable (qué falta, dónde se configura en Studio). NO continúes con datos parciales.

## GOTCHAS

1. **`project.project.plan.hour.line` puede no existir** en instancias sin módulo de planificación → terminar limpiamente con "Sin datos de planificación".
2. **`x_studio_price` en `project.milestone`** es campo Studio — puede no existir si el módulo Studio no está instalado o el campo fue renombrado.
3. **Proyectos con hitos pero sin planificación** (huérfanos) → listados en sección 5 del informe, no como alerta; son facturación sin coste planificado.
4. **Proyectos con "Indaws" en el nombre** → excluidos de alertas pero listados en el informe como "excluidos" para trazabilidad.
5. **Precio objetivo (75€/hora) es constante** — no configurable por el usuario en el workflow. Si necesita cambiarse, hacerlo en el script.
6. **Sin planificación en el período** → no hay datos que analizar; informar al usuario y terminar.
7. **Departamentos por nombre exacto**: "Consultoría", "Desarrollo", "Marketing externo" — un renombrado en Odoo rompe el filtro silenciosamente.

## NEVER
- Modificar datos en Odoo. Este skill es solo lectura.
- Inventar datos. Si falta información, indicarlo claramente.
- Generar el informe sin haber validado el dashboard (Paso 2.5).

## ALWAYS
- Presentar el dashboard (Paso 2.5) y esperar aprobación antes de generar el informe.
- Usar odoo-pilot con contexto `{"lang": "es_ES"}`.
- Redondear importes a 2 decimales y porcentajes a 1 decimal.

## Delegates to

| Skill | Suite | Para qué |
|-------|-------|----------|
| `odoo-pilot` | common | Autenticación y consultas a Odoo |

## Consumed by

| Skill | Suite | Para qué |
|-------|-------|----------|
| migration-billing-tracker | migration-lab | Cruce carga del equipo vs facturación de migraciones |

## REFERENCES

- `references/alert_rules.md` — Reglas de clasificación de alertas (roja/amarilla/verde)
- `references/workflow-detail.md` — Detalle completo de cada paso y estructura del informe
- `templates/informe_carga.md` — Plantilla del informe de 7 secciones
- `scripts/fetch_workload_data.sh` — Extracción de los 4 conjuntos de datos
- `scripts/analyze_workload.py` — Cálculo de cobertura y clasificación de alertas
