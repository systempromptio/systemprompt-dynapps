## 2. Secuencias Criticas por Modulo

### 2.1 Tabla de referencia

| Modulo | Codigo de Secuencia | Modelo | Nivel de Riesgo | Notas |
|--------|---------------------|--------|-----------------|-------|
| `account` | `account.move.sequence` (*) | `account.move` | **CRITICO (fiscal)** | Facturas, asientos. Regulado por ley en muchos paises. Huecos pueden ser ilegales. |
| `account` | `account.payment.sequence` | `account.payment` | **ALTO** | Pagos. Secuencia propia por diario de pago. |
| `sale` | `sale.order` | `sale.order` | **ALTO** | Presupuestos/pedidos. Visible para clientes. |
| `purchase` | `purchase.order` | `purchase.order` | **ALTO** | Pedidos de compra. Visible para proveedores. |
| `stock` | Configurado por `stock.picking.type` | `stock.picking` | **ALTO** | Albaranes. Cada tipo (entrada, salida, interno) tiene su secuencia. |
| `mrp` | `mrp.production` | `mrp.production` | **MEDIO** | Ordenes de fabricacion. Uso interno. |
| `hr_expense` | `hr.expense.report` | `hr.expense.sheet` | **BAJO** | Informes de gastos. Uso interno. |
| `helpdesk` | `helpdesk.ticket` | `helpdesk.ticket` | **BAJO** | Tickets. Uso interno. |
| `repair` | `repair.order` | `repair.order` | **MEDIO** | Ordenes de reparacion. |
| `pos` | `pos.order` | `pos.order` | **ALTO** | Pedidos POS. Fiscal en muchos paises. |

(*) En Odoo 18.0, `account.move` no usa `ir.sequence` directamente para facturas.
Las facturas obtienen su `name` del campo `sequence_prefix` + `sequence_number` del
propio `account.move`, gestionado internamente por `_compute_name()` y
`_set_next_sequence()`. Sin embargo, el journal puede tener una secuencia asociada
via `ir.sequence` para otros tipos de asientos.

### 2.2 Caso especial: Secuencia de facturas en Odoo 18.0

En Odoo 18.0, las facturas (`account.move`) usan un mecanismo de secuencia **propio**
que NO pasa por `ir.sequence`:

```python
# account.move._set_next_sequence()
# Busca el ultimo move del mismo journal + tipo con sequence_number mas alto
# e incrementa: sequence_number = last.sequence_number + 1
```

Esto implica que para restaurar la secuencia de facturas:
- **NO** basta con modificar `ir.sequence`
- Se necesita verificar que al borrar los registros de test, el proximo `sequence_number`
  que calcule Odoo sea el correcto (basado en el ultimo registro real restante)
- Si se borran todos los registros de test creados despues de los reales, la secuencia
  se "auto-restaura" porque `_set_next_sequence()` busca el MAX existente

**Implicacion practica:** Para facturas, la estrategia principal es **borrar los registros
de test** (lo cual ya hace el protocolo de `references/revert/protocol/`). El numero se recupera automaticamente
si los registros de test son los ultimos de la secuencia.

---

