# Optimization Patterns (Odoo v17-v19)

Patrones de correccion, emparejados con los antipatrones de `orm-antipatterns.md`.
Aplica el fix, luego re-mide con la tecnica de `measurement-tools.md`. Verifica
firmas por version con `odoo-version-api-reference`.

> Cada patron tiene un trade-off. No apliques "por defecto": mide el antes/despues.

---

## 1. Prefetch y `mapped` (vs N+1) — antipatrones 1, 4

Reemplaza acceso a campos en bucle por operaciones sobre el recordset completo, que
disparan el prefetch por lotes.

```python
# MAL: N+1
names = []
for order in orders:
    names.append(order.partner_id.country_id.name)

# BIEN: 1-2 queries (prefetch + mapped)
names = orders.mapped('partner_id.country_id.name')
```

- Mantén el recordset agrupado: no extraigas records sueltos antes de leer campos.
- Si necesitas filtrar y luego leer, `filtered(...)` conserva el prefetch del padre.
- `read(['f1','f2'])` trae varios campos en una query cuando vas a leer en masa.

---

## 2. Mover el `search`/agregacion fuera del bucle — antipatrones 2, 6

```python
# MAL: search por iteracion
for partner in partners:
    total = self.env['sale.order'].search_count([('partner_id','=',partner.id)])

# BIEN: read_group en una query
groups = self.env['sale.order'].read_group(
    [('partner_id','in', partners.ids)],
    fields=['partner_id'], groupby=['partner_id'], lazy=False)
counts = {g['partner_id'][0]: g['__count'] for g in groups}
```

- `read_group` agrega en PostgreSQL (`GROUP BY`/`SUM`/`COUNT`) y devuelve pocas filas.
- **`_read_group` con tuplas de aggregates NO es de v19** (corregido 2026-08-07): la firma
  `_read_group(domain, groupby=(), aggregates=(), having=(), …)` ya está en
  `odoo18/odoo/models.py:1965`, `BaseModel._read_group` (v19: `odoo/orm/models.py:1861`), y el cambio viene de **v16.3**.
  Además `read_group` **sigue existiendo** en las dos (v18 `:2806`, v19 `:2749`), así que no hay
  sustitución forzada: son dos APIs coexistiendo. Confirma cuál usar con
  `odoo-version-api-reference` según la versión objetivo.
- Para conteos simples, `search_count` una sola vez con `in` sobre los ids.

---

## 3. `store=True` + `@api.depends` (vs computed-no-store en filtros) — antipatron 3

```python
total = fields.Monetary(compute='_compute_total', store=True)

@api.depends('line_ids.subtotal')
def _compute_total(self):
    for rec in self:
        rec.total = sum(rec.line_ids.mapped('subtotal'))
```

- Solo si el campo se filtra/ordena/agrega o se lee en masa. `store=True` crea
  columna indexable y evita recomputar en lectura.
- **Trade-off:** recalcula en cada `write` de los `depends`. Mide el coste de
  escritura; si el modelo se escribe mucho y se lee poco, quizas no compense.
- `@api.depends` debe listar TODOS los campos origen (incluidos los de relaciones
  `line_ids.subtotal`), o el valor queda obsoleto.

---

## 4. Indices (`index=True`) — antipatron 5

```python
ref = fields.Char(index=True)          # btree estandar
name = fields.Char(index='trigram')    # para ilike '%x%' (Postgres pg_trgm)
```

- Añade `index=True` solo a campos confirmados con `EXPLAIN` como Seq Scan en filtro
  frecuente. Para `ilike` con comodin inicial, indice `trigram`.
- **Trade-off:** cada indice ralentiza inserts/updates y ocupa disco. No indexar
  campos de baja cardinalidad ni poco filtrados.
- Tras añadirlo, actualiza el modulo y re-`EXPLAIN` para confirmar Index Scan.

---

## 5. Batch read/write — antipatron 7

```python
# MAL: write/create en bucle
for vals in data:
    self.env['model'].create(vals)

# BIEN: create con lista (una operacion, un recompute batched)
self.env['model'].create(data)   # data = [vals, vals, ...]

# write sobre el recordset entero cuando los valores coinciden
records.write({'state': 'done'})
```

- `create([...])` y `write` sobre recordset agrupan INSERT/UPDATE y recomputos.
- Si cada record necesita valores distintos, agrupa por valor comun y haz un `write`
  por grupo, o usa `create` con la lista completa de dicts.

---

## 6. `read_group` para agregaciones — antipatron 6, 9

Ya cubierto en patron 2. Para informes, precalcula los agregados en
`_get_report_values` con `read_group` y pasa diccionarios a la plantilla en vez de
agregar dentro de QWeb.

---

## 7. Vistas SQL (modelos de solo lectura para reporting)

Para dashboards/informes analiticos sobre grandes volumenes, un modelo respaldado por
vista SQL (`_auto = False` + `init()` con `CREATE VIEW`) agrega en BD y evita
materializar records transaccionales.

```python
class SaleReport(models.Model):
    _name = 'x.sale.report'
    _auto = False
    def init(self):
        tools.drop_view_if_exists(self.env.cr, self._table)
        self.env.cr.execute(f"CREATE VIEW {self._table} AS (SELECT ...)")
```

- Solo lectura. Indexa las columnas de filtro de la query origen.
- Justificado cuando el reporting agrega millones de filas; no para casos pequeños.

---

## 8. Optimizar informes QWeb — antipatron 9

- Precalcula todo lo relacional en `_get_report_values` con `mapped`/`read_group` y
  pásalo como contexto; evita `search`/`browse` dentro de `t-foreach`.
- Prefetchea los recordsets de las lineas antes del render (`docs.mapped('line_ids...')`).
- Saca agregaciones de la plantilla al Python del modelo.

---

## 9. Control de `flush` / recompute — antipatrones 7, 10

```python
self.env['model'].flush_model(['field'])      # persiste cambios pendientes
self.env.cr.execute("SELECT ... FROM model")  # SQL crudo ve datos correctos
self.env['model'].invalidate_model(['field']) # invalida cache tras SQL directo
```

- Antes de SQL crudo de lectura: `flush_model`/`flush_recordset` para no leer datos
  sucios del cache ORM.
- Tras un UPDATE crudo: `invalidate_model`/`invalidate_recordset` para que el ORM no
  sirva valores cacheados obsoletos.
- En procesos masivos, agrupa los recomputos: deja que el ORM batchee en vez de
  forzar `flush` por iteracion.
- Firmas `flush_*`/`invalidate_*` estabilizadas en v16+ (medido 2026-08-17: los cuatro
  —`flush_model`, `flush_recordset`, `invalidate_model`, `invalidate_recordset`— aparecen en v16 y
  siguen en v17, v18 y v19; **0 ocurrencias en v15**).

> **Y la mitad que faltaba: los nombres VIEJOS mueren en v17, no en v16.** `flush(fnames, records)`
> (`odoo16/odoo/models.py:5646`, `BaseModel.flush`) e `invalidate_cache(fnames, ids)` (`:6055`) **siguen existiendo en
> v16** y dan **0 ocurrencias en v17, v18 y v19**. O sea:
>
> | | v15 | v16 | v17+ |
> |---|---|---|---|
> | `flush()` / `invalidate_cache()` | únicos | **conviven** con los nuevos | **eliminados** |
> | `flush_model` / `invalidate_recordset`… | no existen | ya están | únicos |
>
> **v16 es la ventana de solape**, y por eso el código de esa época compila en las dos formas. Al
> auditar un módulo que viene de v15/v16, lo que hay que buscar **no es si usa los nuevos, sino si
> todavía llama a los viejos**: en v17 eso es `AttributeError`, no una deprecación con aviso.
>
> *(Cuidado al grepear: en v17+ **sigue habiendo un `def flush(`** en `models.py` —v17 `:1203`,
> v19 `:948`— pero es un helper anidado de `_convert_records` con firma `(*, xml_id, model)`, nada
> que ver. Contar `def flush(` a secas da un falso superviviente.)*
>
> Para detalles por versión, `odoo-version-api-reference`.

---

## 10. SQL crudo seguro y rapido — antipatron 10

- Usa SQL crudo SOLO cuando el ORM es demostrablemente insuficiente (agregaciones
  complejas, ventanas). Nunca por costumbre.
- Parametriza SIEMPRE (`cr.execute(sql, params)`), nunca concatenar (el `%s` de
  psycopg2 es seguro).
- `flush` antes de leer; `invalidate` despues de escribir (patron 9).
- `EXPLAIN ANALYZE` la query y añade indices de apoyo si hay Seq Scan.

---

## Tabla antipatron -> patron

| Antipatron | Patron de fix |
|------------|---------------|
| 1 N+1 / 4 prefetch roto | 1 (mapped/prefetch) |
| 2 search en bucle | 2 (read_group/search fuera) |
| 3 computed-no-store en filtro | 3 (store + depends) |
| 5 indice faltante | 4 (index=True / trigram) |
| 6 agregar en Python | 2, 6, 7 (read_group / vista SQL) |
| 7 write en bucle | 5 (batch create/write) |
| 8 dominio costoso | 4 (indice trigram) + revisar dominio |
| 9 informe lento | 6, 8 (precalcular + prefetch QWeb) |
| 10 SQL crudo | 9, 10 (flush/invalidate + parametrizar) |
