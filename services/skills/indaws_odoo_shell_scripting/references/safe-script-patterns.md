# Patrones Seguros para Scripts de Odoo Shell

## Plantilla base obligatoria

```python
# ============================================================
# Script: <descripción breve>
# Modelo:  <res.partner / sale.order / ...>
# Propósito: <qué corrige y por qué>
# Autor:   <nombre>
# Fecha:   <YYYY-MM-DD>
# Odoo:    v18
# ============================================================

DRY_RUN = True   # Cambiar a False solo tras verificar el dry-run

# --- Dominio: parametrizado aquí, no embebido en el search ---
DOMAIN = [('state', '=', 'draft'), ('date_order', '<', '2024-01-01')]
BATCH_SIZE = 500

Model = env['sale.order']

records = Model.search(DOMAIN)
total = len(records)
print(f"[INFO] Registros encontrados: {total}")
if total == 0:
    print("[WARN] Sin registros. Revisar dominio.")
    raise SystemExit(0)

processed = 0
errors = 0

for i in range(0, total, BATCH_SIZE):
    batch = records[i:i + BATCH_SIZE]
    try:
        with env.cr.savepoint():
            if not DRY_RUN:
                batch.write({'state': 'cancel'})
            processed += len(batch)
            print(f"[OK] Batch {i//BATCH_SIZE + 1}: {len(batch)} registros procesados")
        if not DRY_RUN:
            env.cr.commit()
    except Exception as e:
        errors += 1
        print(f"[ERROR] Batch {i//BATCH_SIZE + 1}: {e}")

print(f"\n[SUMMARY] Total: {total} | Procesados: {processed} | Errores: {errors}")
if DRY_RUN:
    print("[DRY-RUN] Sin cambios aplicados. Revisar counts y ejecutar con DRY_RUN=False.")
```

## Patrón dry-run

- `DRY_RUN = True` al inicio, comentado con instrucción de cambio.
- Dentro del bloque `if not DRY_RUN:` va el `write()` o `unlink()`.
- El log de conteos se ejecuta SIEMPRE (tanto en dry como en real).
- NUNCA llamar `env.cr.commit()` cuando `DRY_RUN = True`.

## Savepoints

```python
# Savepoint atómico por batch: rollback automático si hay excepción
with env.cr.savepoint():
    records.write({'field': value})
# Si raises: rollback del savepoint, la transacción sigue abierta
# Si no raises: cambios quedan pendientes hasta el siguiente commit
```

Usar savepoint POR batch, no un único savepoint global para todo el script.
Un savepoint global no protege contra OOM en datasets grandes.

## Batching y commit periódico

```python
# Commit cada N registros para no acumular locks en producción
BATCH_SIZE = 500   # Ajustar según peso del modelo (500–2000 típico)

for i in range(0, total, BATCH_SIZE):
    batch = records[i:i + BATCH_SIZE]
    with env.cr.savepoint():
        batch.write({...})
    env.cr.commit()   # Libera locks; punto de no-retorno para este batch
```

Después de `commit()`, un error en el siguiente batch NO revierte los anteriores.
Documentar esto en el script para que el usuario lo sepa.

## Idempotencia

El script debe poder ejecutarse dos veces sin doble-efecto. Técnicas:

```python
# 1. Filtrar solo registros que aún NO tienen el valor destino
DOMAIN = [('state', '=', 'draft'), ('target_field', '!=', 'expected_value')]

# 2. Check explícito antes del write
for rec in batch:
    if rec.field == new_value:
        continue   # Ya corregido, skip
    rec.write({'field': new_value})

# 3. Usar write con valores condicionales (más eficiente en batch)
to_fix = batch.filtered(lambda r: r.field != new_value)
if to_fix:
    to_fix.write({'field': new_value})
```

## Logging de conteos

Siempre imprimir:
- Count ANTES del write.
- Count después con query de verificación (no confiar solo en `processed`).
- Errores capturados.

```python
# Verificación post-ejecución (ejecutar tras el script)
after = Model.search_count(DOMAIN)
print(f"[VERIFY] Registros que aún cumplen dominio (deben ser 0): {after}")
```

## Evitar problemas de caché ORM

**Tras un `write()` del ORM no hace falta invalidar**: el propio `write()` actualiza la caché
(`odoo/orm/fields.py:1501-1503`, `Field.write`, v19 @d1d161033`). Invalidar ahí es ruido, no seguridad.

Donde sí es obligatorio es **después de escribir por SQL crudo**, que pasa por debajo del ORM y
deja la caché con el valor viejo:

```python
# Tras cr.execute() que MODIFICA datos, la caché sigue mintiendo
env.cr.execute("UPDATE sale_order SET note = %s WHERE id IN %s", (nota, tuple(ids)))
records.invalidate_recordset(['note'])   # obligatorio: el ORM no se ha enterado

# O forzar recarga de un campo concreto
records.read(['campo_modificado'])
```

## Cuándo usar SQL directo en lugar de ORM

Solo si el ORM es prohibitivamente lento (>100k registros simples, sin lógica):

```python
# SQL directo: más rápido pero SIN triggers, SIN recompute, SIN mail tracking
env.cr.execute(
    "UPDATE res_partner SET name = %s WHERE id = ANY(%s)",
    ('Test', list(records.ids))
)
# OBLIGATORIO tras SQL directo si hay campos computados dependientes:
env['res.partner'].invalidate_model(['name'])
Model.browse(records.ids)._recompute_recordset(['computed_field'])
```

Documentar en el script que se usa SQL directo y las consecuencias.

## Checklist pre-entrega

- [ ] `DRY_RUN = True` por defecto
- [ ] Dominio parametrizado al inicio
- [ ] Assertion de count antes del write (warning si 0)
- [ ] Savepoint por batch
- [ ] `commit()` solo fuera del bloque dry-run
- [ ] Log `Before: N / After: M`
- [ ] Query de verificación incluida al final
- [ ] Cabecera con modelo, propósito, fecha, versión Odoo
- [ ] Usuario confirmó backup (prod) o entorno (staging)
