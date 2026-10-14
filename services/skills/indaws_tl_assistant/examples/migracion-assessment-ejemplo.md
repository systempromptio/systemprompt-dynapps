# Evaluación de Migración — Cliente Agrícola del Sureste

**Versión origen**: 16.0 | **Versión destino**: 18.0 | **TL**: Toni Hermosilla
**Fecha**: 2026-03-10

---

## Resumen Ejecutivo

Cliente con 12 módulos custom sobre Odoo 16.0. Migración a 18.0 motivada por fin de soporte y necesidad de nuevas funcionalidades de inventario.

## Inventario de Módulos

| Módulo | Tipo | Decisión | Justificación | Esfuerzo |
|--------|------|----------|---------------|----------|
| `agrisu_stock_lot_tracking` | Custom | **Migrar** | Trazabilidad por lotes es core del negocio, no cubierta por estándar | M (16-24h) |
| `agrisu_purchase_approval` | Custom | **Eliminar** | Odoo 18 tiene `purchase_approval` nativo que cubre el caso | 2h (validar + migrar datos) |
| `agrisu_sale_discount` | Custom | **Reemplazar por Core** | `indaws_sale_discount` cubre descuentos por cliente/producto | 4h (mapear config) |
| `agrisu_hr_attendance_report` | Custom | **Migrar** | Informe específico del convenio agrario, sin equivalente | S (8-12h) |
| `agrisu_product_labels` | Custom | **Eliminar** | Studio puede cubrir etiquetas con QWeb report | 3h (recrear en Studio) |
| `agrisu_stock_wave` | Custom | **Eliminar** | Odoo 18 `stock_picking_batch` cubre wave picking | 2h (validar flujo) |

### Módulos Studio

8 campos `x_studio_*` detectados en `product.template` y `sale.order`. Migración automática por Upgrade Service.

### Integraciones

| Integración | Estado | Riesgo |
|-------------|--------|--------|
| API balanza (pesaje en recepción) | Activa, módulo `agrisu_stock_lot_tracking` | **Alto** — depende de hardware, necesita testing in-situ |
| Export CSV a gestoría | Cron semanal | **Bajo** — adaptar si cambian campos |

---

## Estimación Global

| Concepto | Horas |
|----------|-------|
| Migrar 2 módulos custom (lot_tracking + hr_attendance) | 24-36h |
| Eliminar 3 módulos + validar equivalentes | 7h |
| Reemplazar 1 módulo por Core | 4h |
| Testing integración balanza | 8h |
| Testing funcional general | 8h |
| **Total** | **38-48h** |

---

## Riesgos Principales

1. **Integración balanza** — hardware legacy, sin entorno de test remoto. Requiere visita presencial
2. **Trazabilidad lotes** — módulo más complejo, cambios en `stock.move.line` entre v16 y v18. Consultar `/odoo-version-api-reference`
3. **Datos históricos** — 3 años de trazabilidad. Verificar que Upgrade Service migra `stock.lot` correctamente

## Recomendación

Migración viable en 3-4 sprints. Empezar por la base de datos con Upgrade Service, luego migrar `agrisu_stock_lot_tracking` (ruta crítica). Dejar testing de balanza para sprint final con visita presencial.

Delegar análisis detallado a `/odoo-database-analyzer` para confirmar estimaciones por módulo.
