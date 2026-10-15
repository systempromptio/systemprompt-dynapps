## Tracking de Registros

Durante la ejecucion, registrar TODOS los registros creados, incluyendo cascade:

```json
{
  "records_tracked": [
    {
      "model": "sale.order",
      "id": 42,
      "name": "S00042",
      "prefix": "[TEST-20260309-1030]",
      "operation": "create",
      "step": 1,
      "timestamp": "2026-03-09T10:31:00Z",
      "cascade_children": [78, 79]
    },
    {
      "model": "stock.picking",
      "id": 78,
      "name": "WH/OUT/00078",
      "prefix": "[TEST-20260309-1030]",
      "operation": "cascade",
      "step": 2,
      "detected_via": "origin = S00042",
      "parent_model": "sale.order",
      "parent_id": 42,
      "timestamp": "2026-03-09T10:31:05Z"
    },
    {
      "model": "stock.move",
      "id": 150,
      "name": "WH/OUT/00078: Mesa Roble",
      "operation": "cascade",
      "step": 2,
      "detected_via": "picking_id = 78",
      "parent_model": "stock.picking",
      "parent_id": 78,
      "timestamp": "2026-03-09T10:31:05Z"
    }
  ]
}
```

---

