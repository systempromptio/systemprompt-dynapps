## 3. Snapshot de Secuencias Pre-Test

### 3.1 Captura via `snapshot.py`

El sistema `snapshot.py` ya incluye captura de secuencias. Al ejecutar un snapshot
pre-test, se almacenan automaticamente:

```bash
python3 snapshot.py capture \
    --pilot-dir /path/to/odoo-pilot/scripts \
    --output /tmp/snapshot_pre.json
```

Esto ejecuta `capture_sequences()` que captura:
- Todas las secuencias estandar (`ir.sequence` con `use_date_range = False`)
- Todas las secuencias por rango de fechas (`ir.sequence.date_range`)

### 3.2 Formato JSON del snapshot de secuencias

```json
{
  "sequences": {
    "standard": [
      {
        "id": 15,
        "name": "Sale Order",
        "code": "sale.order",
        "number_next_actual": 42,
        "prefix": "S"
      }
    ],
    "date_range": [
      {
        "id": 8,
        "sequence_id": 12,
        "sequence_name": "INV Sequence",
        "date_from": "2026-01-01",
        "date_to": "2026-12-31",
        "number_next_actual": 156
      }
    ]
  }
}
```

### 3.3 Query manual de secuencias (sin snapshot.py)

Si se necesita capturar secuencias manualmente via odoo-pilot:

```bash
# Secuencias estandar
search_records.sh ir.sequence \
  '[["use_date_range","=",false]]' \
  '["id","name","code","number_next_actual","prefix","suffix","padding","implementation"]' \
  500

# Secuencias por rango de fechas
search_records.sh ir.sequence.date_range \
  '[]' \
  '["id","sequence_id","date_from","date_to","number_next_actual"]' \
  500
```

### 3.4 Snapshot selectivo de secuencias criticas

Para un test que solo afecta ventas y facturacion, capturar solo las secuencias relevantes:

```bash
# Solo secuencias de venta y contabilidad
search_records.sh ir.sequence \
  '["|",["code","=","sale.order"],["code","ilike","account"]]' \
  '["id","name","code","number_next_actual","prefix"]' \
  100
```

---

