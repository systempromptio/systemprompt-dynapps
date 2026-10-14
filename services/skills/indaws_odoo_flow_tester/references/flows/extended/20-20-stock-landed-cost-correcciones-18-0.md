## 20. stock.landed.cost — Correcciones 18.0

### Correcciones validadas en tests en vivo (2026-03-10)

#### Campo `split_method` requerido en lineas de coste
- Las lineas de coste (`stock.landed.cost.lines`) requieren el campo `split_method`.
- Valores posibles: `'equal'`, `'by_quantity'`, `'by_current_cost_price'`, `'by_weight'`, `'by_volume'`
- Si no se especifica, puede fallar al validar el landed cost.

**Ejemplo de linea de coste:**
```json
{
  "name": "Transporte",
  "product_id": "<int (servicio)>",
  "price_unit": 100.0,
  "split_method": "equal",
  "account_id": "<int>"
}
```

---

