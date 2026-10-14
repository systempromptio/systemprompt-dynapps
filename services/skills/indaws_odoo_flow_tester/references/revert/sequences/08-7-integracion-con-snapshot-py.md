## 7. Integracion con `snapshot.py`

### 7.1 Funciones existentes

El script `snapshot.py` ya integra captura y comparacion de secuencias:

| Funcion | Proposito |
|---------|-----------|
| `capture_sequences(pilot_dir)` | Captura `ir.sequence` (estandar) + `ir.sequence.date_range` |
| `_compare_sequences(pre, post)` | Compara snapshots y detecta secuencias avanzadas |
| `verify_restoration(...)` | Verifica que las secuencias volvieron a su estado original |

### 7.2 Flujo integrado con snapshot.py

```bash
# 1. Pre-test
python3 snapshot.py capture --pilot-dir $PILOT_DIR --output /tmp/pre.json

# 2. Ejecutar tests...

# 3. Post-test (antes de revertir)
python3 snapshot.py capture --pilot-dir $PILOT_DIR --output /tmp/post.json

# 4. Ver que secuencias cambiaron
python3 snapshot.py compare /tmp/pre.json /tmp/post.json

# 5. Revertir registros (flows.py --revert <modelo>) + restaurar secuencias (este documento)

# 6. Post-revert
python3 snapshot.py capture --pilot-dir $PILOT_DIR --output /tmp/post_revert.json

# 7. Verificar restauracion completa
python3 snapshot.py verify /tmp/pre.json /tmp/post_revert.json \
    --tolerance-sequences 0
```

### 7.3 Tolerancia de secuencias en verificacion

El parametro `--tolerance-sequences` controla cuantos numeros de diferencia se aceptan:

| Valor | Significado |
|-------|-------------|
| `0` (default) | Coincidencia exacta — la secuencia debe volver al valor pre-test |
| `1-2` | Tolera 1-2 numeros de avance — acepta que registros reales fueron creados |
| `>2` | Poco recomendado — indica que algo fallo en la restauracion |

```bash
# Verificacion estricta (recomendada si no hubo actividad concurrente)
python3 snapshot.py verify /tmp/pre.json /tmp/post_revert.json \
    --tolerance-sequences 0

# Verificacion tolerante (si hubo actividad real durante el test)
python3 snapshot.py verify /tmp/pre.json /tmp/post_revert.json \
    --tolerance-sequences 2
```

### 7.4 Interpretacion del output de verificacion

```
Sequence Changes:
    Sale Order: +3 (42 -> 45)        ← Secuencia avanzada, necesita restaurar
    INV Sequence (2026): +2 (156 -> 158)  ← Rango de fechas avanzado

Verification: FAILED
  Mismatches: 2
    [sequence] Sequence 'Sale Order': advanced by 3 (pre=42, post=45)
    [sequence] Sequence 'INV Sequence': advanced by 2 (pre=156, post=158)
```

Si la verificacion falla por secuencias tras la restauracion, revisar:
1. Se ejecuto la restauracion para TODAS las secuencias afectadas?
2. Hubo registros reales creados despues de la restauracion?
3. El write sobre `number_next_actual` fue exitoso? (verificar retorno de update_record.sh)

---

