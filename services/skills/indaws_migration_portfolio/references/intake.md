# Intake — Alta de una migración en la cartera

Cada migración entra a Indaws por un **correo de Dynapps** con un **link de Drive**
que contiene un **ZIP de módulos custom** + el **tamaño de la BD** (texto, ej.
"15 GB"; NO hay dump). El contacto es siempre **Janik (Dynapps)**.

> Conexión a Odoo: `references/shared/odoo-connection.md` (siempre `odoo-pilot`).

## Campos de la ficha de intake

| Campo | Origen | Notas |
|---|---|---|
| `client` | correo Dynapps | Cliente final de Dynapps (no es el contacto) |
| `contact` | fijo | **Janik** (Dynapps) — cliente directo |
| `drive_link` | correo | URL de la carpeta Drive con el ZIP |
| `zip_name` | Drive | Nombre del ZIP de módulos custom |
| `db_size` | correo | Texto, ej. "15 GB". Solo tamaño, **no dump** |
| `version_src` | manifiestos del ZIP | Auto-deducir si no se indica (rango 14-19) |
| `version_dst` | fijo | **19.0** casi siempre |
| `est_hours` | `migration-estimator` | Horas estimadas (NO re-estimar aquí) |
| `est_eur` | derivado | `est_hours × 45` |
| `status` | gestión | `intake` → `assigned` → `in_progress` → `delivered` → `prod` |
| `assignee` | reparto | Ver `assignment.md` (vacío al alta) |
| `sale_order` | `migration-estimator` | Ref del presupuesto en borrador, si existe |
| `project_id` | `migration-project-creator` | Proyecto **interno** mientras Dynapps no acepta |
| `intake_date` | hoy | Fecha de alta |

## Reglas de intake
- **No descargar ni restaurar BD**: solo se registra el tamaño como dato de cálculo.
- **No re-estimar**: las horas vienen de `migration-estimator`. Si no hay estimación,
  marca `est_hours` como pendiente y avisa; no inventes.
- **Proyecto en interno**: al hacer intake, Dynapps aún no ha aceptado (ver
  DESIGN §3.5). No marques como aprobado.
- **Versión origen**: deducir de los `__manifest__.py` del ZIP si el correo no la da.
- **NUNCA enviar correo desde Odoo** confirmando el alta.

## Procedencia técnica
- La estimación previa (horas, presupuesto) la produce `migration-estimator`.
- La creación de proyecto/tareas se delega a `migration-project-creator` cuando se
  arranca la ejecución (no necesariamente en el momento del intake).
- El estado por gate vive en `migration-assistant` (ver `gate-tracking.md`).
