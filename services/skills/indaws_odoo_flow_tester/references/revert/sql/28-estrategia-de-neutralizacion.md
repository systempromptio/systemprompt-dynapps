## Estrategia de Neutralizacion

Cuando un registro NO puede borrarse (hash fiscal, estado irreversible, constraint FK):

### 1. Desactivar

```bash
update_record.sh <model> '[<id>]' '{"active": false}'
```

El registro desaparece de busquedas normales pero persiste en la BD.

### 2. Renombrar con marca de neutralizacion

```bash
# Patron: [NEUTRALIZED-TEST-YYYYMMDD] nombre_original
update_record.sh <model> '[<id>]' '{"name": "[NEUTRALIZED-TEST-20260309] nombre_original"}'
# O en el campo de referencia segun el modelo:
update_record.sh account.move '[<id>]' '{"ref": "[NEUTRALIZED-TEST-20260309] ref_original"}'
update_record.sh sale.order '[<id>]' '{"client_order_ref": "[NEUTRALIZED-TEST-20260309]"}'
```

### 3. Mover a journal/warehouse de test

```bash
# Para facturas: mover a un diario de test (crear uno si no existe)
TEST_JOURNAL=$(search_records.sh account.journal '[["code","=","TTEST"]]' '["id"]' 1)
# Si no existe:
TEST_JOURNAL=$(create_record.sh account.journal '{"name": "Test Journal", "code": "TTEST", "type": "general"}')
# Mover factura (solo en draft)
update_record.sh account.move '[<id>]' '{"journal_id": '$TEST_JOURNAL'}'
```

### 4. Impacto minimo garantizado

Un registro neutralizado correctamente:
- No aparece en busquedas normales (`active=False`)
- Es identificable por prefijo `[NEUTRALIZED-TEST-*]`
- No afecta saldos contables si esta anulado (NC) o en draft
- No afecta stock si tiene devolucion inversa
- No afecta secuencias (el hueco queda, pero es aceptable)
- Puede limpiarse posteriormente con SQL si se obtiene acceso

---

