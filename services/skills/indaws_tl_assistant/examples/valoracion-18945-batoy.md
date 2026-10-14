# Valoración Tarea #18945 — Distribuciones Batoy

**Tarea**: Facturación individual por albarán en compras | **Proyecto**: Distribuciones Batoy
**Odoo**: 18.0 | **TL**: Toni Hermosilla | **Fecha**: 2026-03-20

---

## Decisión Técnica

### Necesidad

El módulo Core `indaws_purchase_stock_picking_invoice` permite facturar compras desde albaranes, pero **siempre agrupa** las facturas por proveedor. El cliente necesita poder elegir entre:

- **Agrupado** (comportamiento actual): varios albaranes → 1 factura por proveedor
- **Individual** (nuevo): 1 albarán → 1 factura

### Pirámide Anti-Código

| Nivel | ¿Aplica? | Motivo |
|-------|----------|--------|
| Odoo Estándar | No | Odoo estándar no factura por albarán en compras |
| Studio | No | Requiere lógica Python en el flujo de creación de facturas |
| **Indaws Core** | **Sí** | Mejora del módulo existente `indaws_purchase_stock_picking_invoice` |
| Custom | No | Es un caso horizontal, útil para cualquier cliente |

### Veredicto: Mejora de Core

Facturar compras 1 albarán = 1 factura es un caso de negocio horizontal. La mejora se incorpora al módulo Core `indaws_purchase_stock_picking_invoice`, evitando deuda técnica y mantenimiento custom.

---

## Estimación de Esfuerzo

### Desglose

| Componente | Descripción | H base | Factor | H ajustadas |
|------------|-------------|--------|--------|-------------|
| Análisis | Lógica actual de agrupación en `action_invoice_create` | 1h | x1.0 | 1h |
| Modelos Python | Añadir opción individual/agrupado, condicionar `purchases_by_partner` | 2h | x1.0 | 2h |
| Vistas XML | Opción en wizard o acción para elegir modo de facturación | 1h | x1.0 | 1h |
| Tests | Flujo agrupado + flujo individual, compras simples y múltiples | 1.5h | x1.0 | 1.5h |
| Review + merge | Code review + merge a Core | 1h | x1.0 | 1h |
| **TOTAL** | | **6.5h** | | **6.5h** |

### Factores de Riesgo

| Factor | Aplica | Justificación |
|--------|--------|---------------|
| Módulo nuevo | No | Mejora de módulo Core existente |
| Sin spec clara | No | Necesidad bien definida: agrupar o 1:1 |
| Impacto datos | No | No afecta datos existentes, solo flujo de creación |
| Dev junior | Por asignar | Si se asigna junior, aplicar x1.3 |

No se aplican multiplicadores de riesgo — el alcance es claro y acotado.

### Estimación Final (esfuerzo real, con factor IA)

| Escenario | Horas |
|-----------|-------|
| Optimista | 4h |
| **Esperado** | **5h** |
| Pesimista | 8h |

> Reducción IA ~25% aplicada en modelos Python, vistas XML y tests.

---

## Propuesta Económica al Cliente

| Concepto | Horas |
|----------|-------|
| Análisis + desarrollo + tests | 5-7h |
| Consultoría: validación del flujo | 1-2h |
| **Total propuesta** | **6-9h** |

**Comunicar al cliente**: rango 6-9h, presentando 9h como escenario esperado.

---

## Notas

- El esfuerzo de desarrollo es mejora de Core pero lo asume el cliente que lo solicita
- El beneficio para Indaws es que se incorpora al producto y no genera deuda técnica
- Nivel de esfuerzo: **Baja-Media** — PM puede gestionar con TL de soporte
- Las horas son el esfuerzo real con asistencia IA: es la única cifra, y es la que se comunica
