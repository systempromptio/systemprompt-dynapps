# Reglas de Alertas - Workload Billing Monitor

## Precio Objetivo

- **Precio hora objetivo**: 75 €/hora
- Este valor es fijo y se usa como referencia para calcular la facturación esperada

## Fórmulas

```
facturacion_objetivo = total_horas_planificadas × 75
cobertura_pct = (total_facturacion / facturacion_objetivo) × 100
gap_euros = facturacion_objetivo - total_facturacion
```

## Clasificación de Alertas

| Alerta | Icono | Condición | Significado |
|--------|-------|-----------|-------------|
| Verde | 🟢 | `cobertura_pct >= 100` | Facturación cubre o supera las horas planificadas |
| Amarilla | 🟡 | `0 < cobertura_pct < 100` | Hay facturación pero insuficiente para las horas |
| Roja | 🔴 | `total_facturacion == 0` | Sin hito de facturación para el mes |
| Excluido | ⚪ | Nombre contiene "Interno" o "Indaws" | Proyecto interno, no requiere facturación |

## Departamentos Monitorizados

Solo se analizan empleados de estos departamentos (nombres exactos en Odoo):
- **Consultoría**
- **Desarrollo**
- **Marketing externo**

## Modelos Odoo Involucrados

| Modelo | Uso | Campos clave |
|--------|-----|-------------|
| `hr.employee` | Empleados por departamento | `name`, `department_id` |
| `project.project.plan.hour.line` | Planificación mensual | `project_id`, `employee_id`, `hours`, `date_start`, `date_end` |
| `project.milestone` | Hitos de facturación | `project_id`, `deadline`, `x_studio_price` |
| `project.project` | Datos del proyecto | `name`, `id` |

## Exclusiones

- Proyectos cuyo nombre contenga (case-insensitive):
  - "Interno"
  - "Indaws"
- Estos proyectos se listan en el informe como "excluidos" pero no generan alertas
