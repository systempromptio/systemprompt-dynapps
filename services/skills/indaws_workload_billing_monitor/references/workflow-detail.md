# Workload Billing Monitor — Workflow Detail

## Paso 0: Recopilación de Datos

1. **Credenciales Odoo** (si no están en el entorno):
   - URL, base de datos, API Key, usuario

2. **Mes y año a analizar** (opcional): Default = mes actual.

**DETENTE** y espera la respuesta del usuario antes de continuar.

## Paso 1: Extracción de Datos de Odoo

```bash
export ODOO_URL="<url>"
export ODOO_DB="<db>"
export ODOO_KEY="<api_key>"
export ODOO_USER="<usuario>"

./scripts/fetch_workload_data.sh <YYYY> <MM> --pilot-dir /path/to/odoo-pilot/scripts
```

**El script obtiene** (4 conjuntos de datos):

1. **Empleados** (`hr.employee`): departamentos Consultoría, Desarrollo, Marketing externo
2. **Planificación mensual** (`project.project.plan.hour.line`): horas planificadas por empleado/proyecto
3. **Hitos de facturación** (`project.milestone`): `deadline` en el mes y campo `x_studio_price`
4. **Proyectos** (`project.project`): datos básicos incluyendo `date_start` y `date` (fecha fin)

**Filtro de proyectos internos**: se excluyen proyectos con "Interno" o "Indaws" en el nombre.

**Validaciones post-extracción**:
- ✅ Empleados encontrados en los departamentos objetivo
- ✅ Planificación mensual existe para el período seleccionado
- ⚠️ Sin hitos para algún proyecto → se reportará como alerta roja
- ❌ Sin planificación → informar al usuario y terminar

## Paso 2: Análisis y Cruce de Datos

```bash
python3 ./scripts/analyze_workload.py < /tmp/wbm_data.json > /tmp/wbm_analysis.json
```

**Cálculos por proyecto**:
- `total_horas_plan`: suma de horas planificadas de todos los empleados
- `total_facturacion`: suma de `x_studio_price` de hitos con deadline en el mes
- `facturacion_objetivo`: `total_horas_plan × 75€`
- `cobertura_pct`: `(total_facturacion / facturacion_objetivo) × 100`
- `gap_euros`: `facturacion_objetivo - total_facturacion`

**Clasificación de alertas**:
- 🟢 **Verde**: `total_facturacion >= facturacion_objetivo` (cobertura ≥ 100%)
- 🟡 **Amarilla**: `0 < total_facturacion < facturacion_objetivo` (facturación insuficiente)
- 🔴 **Roja**: `total_facturacion == 0` (sin hito de facturación)
- ⚪ **Excluido**: proyecto interno

**Output JSON**:
- `projects`: lista con horas, facturación, alerta, empleados asignados
- `employees`: vista por empleado con proyectos y distribución de horas
- `summary`: totales globales (horas, facturación esperada, real, gap)
- `alerts`: lista de alertas ordenadas por severidad (roja > amarilla)

## Paso 2.5: Dashboard de Validación (OBLIGATORIO)

```markdown
## DASHBOARD: Carga vs Facturación - {Mes} {Año}

### Resumen Global
| Indicador | Valor |
|-----------|-------|
| Empleados analizados | XX |
| Proyectos con planificación | XX |
| Proyectos excluidos (internos) | XX |
| Horas totales planificadas | XXXh |
| Facturación objetivo (×75€) | XX.XXX € |
| Facturación real (hitos) | XX.XXX € |
| Gap de facturación | XX.XXX € |

### Distribución de Alertas
- 🔴 Alertas rojas (sin facturación): X proyectos
- 🟡 Alertas amarillas (facturación insuficiente): X proyectos
- 🟢 Proyectos OK: X proyectos

**¿Los datos son correctos? ¿Genero el informe completo?**
```

## Paso 3: Estructura del Informe (7 secciones)

Fichero: `informe_carga_facturacion_{YYYY}_{MM}.md`

1. **Resumen Ejecutivo** — KPIs globales, distribución de alertas
2. **Alertas Rojas** — proyectos sin facturación (tabla con fechas inicio/fin, horas, empleados, gap €)
3. **Alertas Amarillas** — proyectos con facturación insuficiente (tabla con % cobertura)
4. **Proyectos OK** — proyectos con cobertura completa
5. **Facturación Sin Planificación** — proyectos con hitos pero sin horas planificadas (huérfanos)
6. **Vista por Empleado** — cada empleado con proyectos, horas y estado
7. **Recomendaciones** — acciones sugeridas basadas en las alertas

Usar template `./templates/informe_carga.md`.
