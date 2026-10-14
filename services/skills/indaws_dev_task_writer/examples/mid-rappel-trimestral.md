# Ejemplo Mid — Sistema de rappel trimestral

**Titulo**: Calculo de rappel trimestral por volumen de compra | **Odoo**: 18.0
**Asignado**: Laura Garcia (Mid) | **Estimacion**: 28h | **Limite**: 2026-04-15

## Contexto de Negocio

Los clientes con acuerdos de rappel reciben descuentos retroactivos por volumen trimestral. Actualmente el equipo contable lo calcula manualmente cada 3 meses consultando facturas. Genera retrasos y errores. Con esto, un wizard calcula y genera la nota de credito automaticamente.

## Objetivo

Un wizard trimestral calcula el rappel por cliente segun sus reglas (% escalonado por categoria de producto). Genera nota de credito vinculada a las facturas originales. Multi-company.

## Alcance

**Incluido**:
- Wizard de calculo trimestral
- Reglas por categoria (escalonado)
- Generacion de nota de credito
- Vinculo a facturas origen

**Excluido**:
- NO rappel acumulado anual
- NO rappel por producto individual
- NO integracion con pricelist

## Guia de Implementacion (adaptada Mid — enfoque + constraints)

El modulo `indaws_rappel` ya existe con el modelo base de acuerdos. Extiendelo.

**Puntos clave a considerar:**
- Volumen = suma de `account.move.line` confirmadas del trimestre por categoria
- Escalonado: si volumen > umbral 1 → %, > umbral 2 → %, etc. (mayor umbral gana)
- Nota de credito debe referenciar las facturas originales (campo `reversed_entry_id` o similar)
- Multi-company: cada empresa tiene sus propios acuerdos
- Edge case: que pasa si cancelan una factura despues de calcular el rappel?

**Propon tu enfoque tecnico en una sync de 15 min antes de empezar.**

## Criterios de Aceptacion

- [ ] Wizard calcula correctamente con umbrales escalonados (5% > 10k, 8% > 25k)
- [ ] Nota de credito generada vincula facturas origen
- [ ] Multi-company funciona (empresa A no ve acuerdos de empresa B)
- [ ] Edge case: factura cancelada post-rappel → documentar decision
- [ ] Tests unitarios para calculo escalonado + multi-company
- [ ] Performance: 500+ facturas por calculo sin timeout

## Referencias

| Recurso | Ruta | Para que |
|---------|------|----------|
| `indaws_rappel` | `$PROJECTS_ROOT/indawsodoo/18/indaws_rappel/` | Base existente a extender |
| `account.move.reversal` | `$ODOO_ENTERPRISE_SRC/18/account_accountant/wizard/` | Patron de nota de credito |

## Restricciones

- Usar `indaws_rappel` existente como base — no crear modulo nuevo
- No modificar `account.move` core — solo extender
- Performance: debe soportar 500+ facturas por calculo sin timeout

## Preguntas Abiertas

- (resolver en sync con Maria de contabilidad)

## Ayuda

- Timer: pide ayuda si llevas **4h** atascado en un punto
- Contacto: Toni por Slack #dev-tokyo
- Sync: Maria (contabilidad) para formato nota de credito

## Entregables

1. [ ] Rama `feature/indaws_rappel_quarterly`
2. [ ] Tests escalonado + multi-company + performance
3. [ ] PR con descripcion del enfoque
4. [ ] Staging validado por contabilidad
