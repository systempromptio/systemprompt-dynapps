# ORM Antipatterns Catalog (Odoo v17-v19)

Catalogo de antipatrones de rendimiento. Para cada uno: como detectarlo, el
sintoma observable y por que es lento. El fix esta en `optimization-patterns.md`.

> Regla previa: confirma cada antipatron con codigo + medicion. No marques por
> simple inspeccion visual; un `browse` en bucle sobre 2 records no es un problema.

---

## 1. N+1 queries (acceso a campos en bucle)

**Deteccion.** Bucle `for rec in records:` que dentro accede a un campo
relacional/computed-no-store de otro modelo, especialmente tras `filtered`,
`sorted` con key relacional, o list comprehensions que desagrupan el recordset.

**Sintoma.** El query count escala linealmente con el numero de records (1 + N).
Logs SQL con la misma SELECT repetida cambiando solo el `id`.

**Por que es lento.** Cada acceso a un record fuera del recordset prefetcheado
dispara su propia query; se pierde el prefetch por lotes (`PREFETCH_MAX`, ver
`references/shared/batch-limits.md`). N round-trips a BD en vez de 1-2.

---

## 2. `search` / `browse` / `read` dentro de un bucle

**Deteccion.** `self.env['x'].search(...)`, `.browse(...)` o `.read(...)` cuyo
dominio depende de la variable de iteracion.

**Sintoma.** Una query (o varias) por iteracion; tiempo total lineal con N.

**Por que es lento.** Cada `search` es un round-trip + planificacion SQL. Mover el
filtro a un unico `search`/`read_group` fuera del bucle colapsa N queries en 1.

---

## 3. Campo computed sin `store` usado en filtros/orden/informes

**Deteccion.** `fields.X(compute='_c')` sin `store=True` que aparece en un dominio
de `search`, en `order`, en agregaciones o se lee masivamente en un informe.

**Sintoma.** Odoo carga TODOS los registros candidatos y computa en Python para
poder filtrar/ordenar; explosion de queries y memoria. `search` sobre el campo
puede incluso fallar o ser O(n) sobre toda la tabla.

**Por que es lento.** Sin `store`, no hay columna que la BD pueda filtrar/indexar;
el ORM materializa y recomputa en memoria. Con cron/informe sobre miles de filas
es letal.

---

## 4. Falta de prefetch / iteracion sobre records sueltos

**Deteccion.** Codigo que extrae records individuales de un recordset
(`records[0]`, iterar una lista de browse de un id cada uno, `mapped` parcial) y
luego accede a campos.

**Sintoma.** Prefetch desactivado; N+1 encubierto.

**Por que es lento.** El mecanismo de prefetch agrupa accesos por recordset; al
romper la agrupacion cada record consulta solo lo suyo.

---

## 5. Indice faltante (`index=True`) en campo filtrado

**Deteccion.** Campo usado con frecuencia en dominios de `search`, en `order`, o en
joins de relaciones, sin `index=True` (ni indice DB equivalente). Confirmar con
`EXPLAIN` que hay Seq Scan.

**Sintoma.** Queries lentas que crecen con el tamaño de la tabla; `EXPLAIN ANALYZE`
muestra `Seq Scan` sobre tabla grande.

**Por que es lento.** Sin indice, PostgreSQL escanea la tabla entera para filtrar.

> Trade-off: indexar campos de baja cardinalidad o poco filtrados penaliza writes
> sin ganancia. Mide con `EXPLAIN` antes y despues.

---

## 6. Oportunidad de `read_group` desaprovechada (agregar en Python)

**Deteccion.** `search` que trae muchos records solo para sumar/contar/agrupar en
Python (`sum(r.amount for r in recs)`, `len(recs.filtered(...))`, dict de conteos).

**Sintoma.** Se materializan miles de records y campos solo para una agregacion;
memoria y tiempo altos.

**Por que es lento.** La agregacion la puede hacer PostgreSQL (`GROUP BY`/`SUM`)
devolviendo un puñado de filas, en vez de transferir y construir N objetos ORM.

---

## 7. Escrituras no batched (`write` en bucle)

**Deteccion.** `rec.write({...})` o asignacion de campo dentro de un bucle por
record; `create` de uno en uno en bucle.

**Sintoma.** Un UPDATE/INSERT y un disparo de recompute por iteracion.

**Por que es lento.** Cada `write` recalcula campos dependientes, dispara
constraints y hace round-trip. Un `write` sobre el recordset completo (cuando los
valores son iguales) o `create` con lista de dicts agrupa la operacion.

---

## 8. Dominios costosos

**Deteccion.** Dominios con `child_of`/`parent_of` sobre jerarquias grandes,
`like`/`ilike` con comodin inicial (`'%x'`), `OR` amplios, o subdominios sobre
campos no almacenados/no indexados.

**Sintoma.** `search` lento; `EXPLAIN` con Seq Scan, sorts grandes o subplans caros.

**Por que es lento.** `ilike '%x'` no usa indice b-tree; `child_of` expande el
arbol; subdominios sobre computed-no-store fuerzan materializacion.

---

## 9. Informes lentos (QWeb / carga de datos)

**Deteccion.** Plantilla QWeb que accede a campos relacionales dentro de bucles
`t-foreach` sin prefetch previo; informe que hace `search`/`browse` por linea;
agregaciones en plantilla.

**Sintoma.** Render del PDF/HTML tarda segundos; query count explota con el numero
de lineas del documento.

**Por que es lento.** Mismo N+1 del punto 1 pero en capa de presentacion; ademas
QWeb evalua expresiones por nodo. Conviene precalcular en el modelo/`_get_report_values`.

---

## 10. SQL crudo mejorable o peligroso

**Deteccion.** `self.env.cr.execute(...)` con queries que el ORM ya cubre, sin
`flush` previo, con concatenacion de strings (riesgo) o sin indices de apoyo.

**Sintoma.** Resultados inconsistentes (datos sin flushear), o SQL que no aprovecha
indices, o reimplementa lo que `read_group` haria seguro.

**Por que es lento/arriesgado.** SQL crudo salta el cache ORM: lee datos sucios si
hay writes pendientes; sin `EXPLAIN` puede ignorar indices. Solo justificado cuando
el ORM es demostrablemente insuficiente y siempre con `flush` previo.

---

## Mapa sintoma -> antipatron

| Sintoma observado | Antipatron probable |
|-------------------|---------------------|
| Query count crece lineal con N records | 1, 2, 7 |
| Misma SELECT repetida cambiando `id` | 1, 4 |
| `search`/`order` sobre campo computed peta o va O(n) | 3 |
| `EXPLAIN` muestra Seq Scan en tabla grande | 5, 8 |
| Mucha RAM al sumar/contar | 6 |
| Informe PDF tarda y escala con lineas | 9 |
| Datos inconsistentes tras SQL crudo | 10 |
