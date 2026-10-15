# Document Types Reference

Mapping between recognized document types and their Odoo models, move types, and required fields.

## Type Table

| Document Type | OCR Label | Odoo Model | `move_type` | Required Fields |
|---|---|---|---|---|
| Vendor Invoice | `invoice` | `account.move` | `in_invoice` | partner_id, ref, invoice_date, invoice_line_ids |
| Vendor Credit Note | `credit_note` | `account.move` | `in_refund` | partner_id, ref, invoice_date, invoice_line_ids |
| Customer Invoice | `invoice` (outgoing) | `account.move` | `out_invoice` | partner_id, invoice_date, invoice_line_ids |
| Customer Credit Note | `credit_note` (outgoing) | `account.move` | `out_refund` | partner_id, invoice_date, invoice_line_ids |
| Delivery Note | `delivery_note` | `stock.picking` | — | partner_id, picking_type_id, move_ids_without_package |
| Purchase Order | `purchase_order` | `purchase.order` | — | partner_id, order_line, date_order |
| Expense Receipt | `expense_receipt` | `hr.expense` | — | name, total_amount, date |

## Classification Keywords (Spanish / English)

| Type | Keywords |
|---|---|
| Credit Note | nota de credito, credit note, abono, rectificativa |
| Delivery Note | albaran, delivery note, packing slip, nota de entrega |
| Purchase Order | pedido de compra, purchase order, orden de compra, P.O. |
| Expense Receipt | ticket, recibo, receipt, expense, gasto, justificante |
| Customer Invoice | factura emitida, sales invoice, factura de venta |
| Vendor Invoice | factura, invoice, fra., n factura |

## JSON Payload Examples

### account.move (in_invoice)

```json
{
  "move_type": "in_invoice",
  "partner_id": 42,
  "ref": "FRA-2024-001",
  "invoice_date": "2024-06-15",
  "invoice_date_due": "2024-07-15",
  "currency_id": 1,
  "state": "draft",
  "invoice_line_ids": [
    [0, 0, {
      "name": "Consulting services",
      "quantity": 10,
      "price_unit": 150.00,
      "tax_ids": [[6, 0, [1]]]
    }],
    [0, 0, {
      "name": "Travel expenses",
      "quantity": 1,
      "price_unit": 250.00,
      "tax_ids": [[6, 0, [1]]]
    }]
  ]
}
```

### account.move (in_refund)

```json
{
  "move_type": "in_refund",
  "partner_id": 42,
  "ref": "RECT-2024-003",
  "invoice_date": "2024-06-20",
  "state": "draft",
  "invoice_line_ids": [
    [0, 0, {
      "name": "Returned goods",
      "quantity": 5,
      "price_unit": 50.00,
      "tax_ids": [[6, 0, [1]]]
    }]
  ]
}
```

### stock.picking

```json
{
  "partner_id": 42,
  "picking_type_id": 1,
  "origin": "ALB-2024-150",
  "move_ids_without_package": [
    [0, 0, {
      "name": "Widget A",
      "product_uom_qty": 100
    }],
    [0, 0, {
      "name": "Widget B",
      "product_uom_qty": 50
    }]
  ]
}
```

### purchase.order

```json
{
  "partner_id": 42,
  "date_order": "2024-06-10",
  "order_line": [
    [0, 0, {
      "name": "Raw material X",
      "product_qty": 500,
      "price_unit": 2.50
    }]
  ]
}
```

### hr.expense

```json
{
  "name": "Restaurant Taxi Madrid",
  "total_amount": 45.80,
  "date": "2024-06-12"
}
```

## Odoo One2many Command Reference

| Command | Format | Meaning |
|---|---|---|
| Create | `[0, 0, {vals}]` | Create a new child record |
| Update | `[1, id, {vals}]` | Update existing child record |
| Delete | `[2, id, 0]` | Remove and delete child record |
| Link | `[4, id, 0]` | Link existing record |
| Replace | `[6, 0, [ids]]` | Replace all with given IDs |
