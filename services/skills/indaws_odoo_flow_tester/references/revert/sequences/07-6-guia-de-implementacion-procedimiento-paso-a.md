## 6. Guia de Implementacion — Procedimiento Paso a Paso

### 6.1 Flujo completo

```
┌─────────────────────────────────────────────────────────────────┐
│  PASO 1: Capturar snapshot pre-test                             │
│  - snapshot.py capture → snapshot_pre.json                      │
│  - Incluye secuencias, balances, stocks, conteos               │
├─────────────────────────────────────────────────────────────────┤
│  PASO 2: Registrar timestamp de inicio                          │
│  - T_START = datetime.now()                                     │
│  - Guardar IDs de ultimos registros por modelo critico          │
├─────────────────────────────────────────────────────────────────┤
│  PASO 3: Ejecutar tests                                         │
│  - Todos los registros con prefijo [TEST-YYYYMMDD-HHMM]        │
│  - Tracking de IDs creados en JSON                              │
├─────────────────────────────────────────────────────────────────┤
│  PASO 4: Capturar snapshot post-test                            │
│  - snapshot.py capture → snapshot_post.json                     │
├─────────────────────────────────────────────────────────────────┤
│  PASO 5: Verificar registros reales intercalados                │
│  - Buscar registros creados entre T_START y ahora               │
│  - Excluir los que tienen prefijo [TEST-...]                    │
│  - Si hay registros reales → ABORTAR restauracion de secuencias │
├─────────────────────────────────────────────────────────────────┤
│  PASO 6: Eliminar registros de test                             │
│  - Seguir revert/protocol/ (orden inverso)                      │
│  - Verificar que todos los registros de test fueron eliminados  │
├─────────────────────────────────────────────────────────────────┤
│  PASO 7: Restaurar secuencias                                   │
│  - Comparar snapshot_pre vs estado actual                       │
│  - Para cada secuencia avanzada: restaurar number_next_actual   │
│  - Verificar con snapshot.py verify                             │
├─────────────────────────────────────────────────────────────────┤
│  PASO 8: Validacion final                                       │
│  - snapshot.py verify snapshot_pre.json snapshot_final.json     │
│  - Confirmar 0 mismatches en secuencias                         │
└─────────────────────────────────────────────────────────────────┘
```

### 6.2 Paso 5 en detalle — Deteccion de registros reales intercalados

Este es el paso mas critico. Si hay registros reales creados durante el test,
restaurar secuencias podria causar duplicados.

```bash
# Buscar facturas creadas durante el periodo de test que NO son de test
search_records.sh account.move \
  '[["create_date",">=","2026-03-10 14:00:00"],["create_date","<=","2026-03-10 15:30:00"],["ref","not ilike","[TEST-"]]' \
  '["id","name","ref","create_date","state"]' \
  100

# Buscar pedidos de venta creados durante el test que NO son de test
search_records.sh sale.order \
  '[["create_date",">=","2026-03-10 14:00:00"],["create_date","<=","2026-03-10 15:30:00"],["client_order_ref","not ilike","[TEST-"]]' \
  '["id","name","client_order_ref","create_date","state"]' \
  100
```

**Decision tras la verificacion:**

| Resultado | Accion |
|-----------|--------|
| 0 registros reales intercalados | Proceder con restauracion de secuencias |
| Registros reales encontrados pero con numeros ANTERIORES a los de test | Proceder — los registros reales no usan numeros de test |
| Registros reales con numeros POSTERIORES o intercalados con test | **NO restaurar secuencias** — aceptar el hueco |

### 6.3 Paso 7 en detalle — Restauracion automatizada

```bash
# 1. Comparar snapshots para identificar secuencias afectadas
python3 snapshot.py compare snapshot_pre.json snapshot_post.json \
    --output /tmp/diff.json

# 2. Leer el diff y restaurar cada secuencia afectada
# (El LLM procesa el JSON y genera los comandos)

# Ejemplo de restauracion basado en el diff:
# diff.changes.sequences = [
#   {"type": "standard", "id": 15, "name": "Sale Order",
#    "pre_next": 42, "post_next": 45, "advanced_by": 3}
# ]

# Restaurar:
update_record.sh ir.sequence 15 '{"number_next_actual": 42}'

# Para date_range:
# {"type": "date_range", "id": 8, "sequence_name": "INV",
#  "pre_next": 156, "post_next": 159, "advanced_by": 3}
update_record.sh ir.sequence.date_range 8 '{"number_next_actual": 156}'
```

### 6.4 Manejo de condiciones de carrera (race conditions)

**Problema:** Entre el Paso 5 (verificar registros reales) y el Paso 7 (restaurar),
un usuario podria crear un registro que consuma un numero de secuencia.

**Mitigaciones:**

1. **Ventana temporal minima:** Ejecutar Pasos 5-7 lo mas rapido posible, idealmente
   en segundos.

2. **Verificacion post-restauracion:** Despues de restaurar, verificar que el
   `number_next_actual` es correcto:
   ```bash
   search_records.sh ir.sequence 15 '["number_next_actual"]'
   # Debe coincidir con el valor pre-test
   ```

3. **Horario de baja actividad:** Ejecutar tests en produccion fuera de horario laboral
   cuando la probabilidad de registros concurrentes es minima.

4. **Bloqueo temporal (solo si es posible):** En entornos controlados, desactivar
   temporalmente la creacion de registros por otros usuarios (ej: desactivar grupos
   de acceso momentaneamente). **No recomendado en produccion real.**

5. **Aceptar el riesgo:** Si el entorno tiene actividad constante, documentar que la
   restauracion de secuencias es "best effort" y puede dejar huecos menores.

---

