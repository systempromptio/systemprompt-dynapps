## Secuencias: Cuando Restaurar y Cuando No

### Seguro restaurar

| Situacion | Accion |
|-----------|--------|
| Test creo N registros y TODOS fueron borrados | Restaurar `number_next_actual` al valor pre-test |
| Secuencia sin implementation='standard' (solo `no_gap`) | `update_record.sh ir.sequence '[<id>]' '{"number_next_actual": <pre_value>}'` |
| Secuencia con date_range | `update_record.sh ir.sequence.date_range '[<range_id>]' '{"number_next_actual": <pre_value>}'` |

### NO restaurar (riesgo de duplicacion)

| Situacion | Por que no | Alternativa |
|-----------|-----------|-------------|
| Otros usuarios crearon registros durante el test | Podrian existir registros con numeros intermedios | Aceptar el hueco |
| Secuencia con `implementation='standard'` (PostgreSQL sequence) | Requiere `setval()` en PostgreSQL ademas del ORM | Solo con SQL y verificacion previa |
| Factura/asiento con hash fiscal | El numero forma parte del hash encadenado | NUNCA restaurar |
| Entorno multi-empresa concurrente | Race condition entre secuencias | Aceptar el hueco |

### Verificacion antes de restaurar

```bash
# 1. Obtener valor pre-test (del snapshot)
PRE_VALUE=42

# 2. Obtener valor actual
search_records.sh ir.sequence '[["code","=","sale.order"],["company_id","=",1]]' '["id","number_next"]' 1

# 3. Verificar que no hay registros con numeros intermedios
# Si el prefijo es "S" y pre_value=42, current=45, verificar que no existen S00042, S00043, S00044
search_records.sh sale.order '[["name",">=","S00042"],["name","<=","S00045"]]' '["id","name"]' 0
# Si devuelve registros que NO son del test → NO restaurar

# 4. Si todo limpio, restaurar
update_record.sh ir.sequence '[<seq_id>]' '{"number_next_actual": 42}'
```

---

