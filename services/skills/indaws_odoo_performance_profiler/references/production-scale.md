# Production-Scale Perf & Debugging (Odoo v17-v19)

Playbook para BD GRANDES de cliente: fallos que **solo emergen a escala** (cientos
de miles / millones de filas, writes masivos, crons batch) y como diagnosticarlos
**con y sin acceso a produccion**. No duplica `orm-antipatterns.md` (catalogo
generico): aqui el angulo es volumen + entorno real.

> Refs cruzadas: antipatron base en `orm-antipatterns.md`, fix en
> `optimization-patterns.md`, tecnicas de medida en `measurement-tools.md`.
> **Las lineas de los flags de TIEMPO estaban desplazadas** (corregido 2026-08-07): la tabla daba
> `--limit-time-real` en `odoo/tools/config.py:358`, `configmanager.__init__` y esa linea es del limite de **memoria** por worker gevent.
> Las reales en v18: `--limit-time-cpu` `:361`, `--limit-time-real` `:364`, `--limit-time-real-cron`
> `:367`. Los flags de **log** (`:211/:212/:213`) si estaban exactos. En v19 el bloque entero se
> desplaza ~120 lineas (ver la tabla). El **default de 120 s** era y es correcto.
>
> Notas de version: en **v19** el ORM vive en `odoo/orm/models.py` y `PREFETCH_MAX`
> esta en `odoo/tools/constants.py`; en **v18** todo en `odoo/models.py`.

---

## A. Modos de fallo que SOLO aparecen a escala

### A1. Recompute storms (`store=True` + `@api.depends` amplio/profundo)

| | |
|---|---|
| **Sintoma a escala** | Un `write` masivo (import, accion en lote, `write` sobre 50k records) tarda minutos; CPU del worker al 100%; el log muestra recomputos en cascada de campos que no tocaste. |
| **Causa** | `write` llama a `modified(fnames)` (`v18 odoo/models.py:7163`, `BaseModel.modified`; `v19 odoo19/odoo/orm/models.py:6756`, `BaseModel.modified`). El ORM recorre el **arbol de triggers** de cada campo modificado y, por cada dependiente `compute && store`, marca ids en `env.transaction.tocompute` (`odoo/models.py:7235-7243`, `BaseModel.modified`). Un `@api.depends` que cruza relaciones (`a.b.c.field`) hace `inverse()` por cada salto -> el conjunto a recomputar puede explotar muy por encima de los records escritos. Campos `recursive` reentran (`odoo/models.py:7222`, `BaseModel.modified`). |
| **Fix** | Acotar el `depends` al minimo real; partir el campo en uno store (filtrable) y otro no-store. Para procesos batch, escribir agrupado y dejar que el recompute se batchee UNA vez al final (no `write` por record). Si la cascada es inevitable, recalcular por lotes con cron en vez de inline. Suspender efectos secundarios con contexto (ver A6). |

```python
# MAL a escala: depends profundo dispara inverse() en cadena en cada write
@api.depends('order_id.partner_id.commercial_partner_id.country_id.code')
def _compute_zone(self): ...

# MEJOR: store solo lo que se filtra; precalcula el resto bajo demanda
```

### A2. N+1 en `compute` / `onchange` / `_compute_display_name` sobre recordsets grandes

| | |
|---|---|
| **Sintoma a escala** | Listas/exports de 5k+ lineas tardan; `name_get`/display_name de un m2o en una vista lista dispara 1 query por fila; query count ~= nº filas. |
| **Causa** | El prefetch agrupa por recordset hasta `PREFETCH_MAX = 1000` (`v18 odoo/models.py:331`, `PREFETCH_MAX`; en **v19** se muda a `odoo/tools/constants.py:11`, `PREFETCH_MAX`, importado en `odoo19/odoo/orm/models.py:57` lo importa). `_compute_display_name` (`odoo/models.py:1793`, `BaseModel._compute_display_name`) itera `for record in self` y lee `record[self._rec_name]`: si el recordset llega desagrupado (records sueltos de `browse(id)` uno a uno, o tras `filtered`/list-comp que rompe el grupo) cada acceso es su propia query. Por encima de 1000 ids el prefetch ya parte en bloques (`odoo/models.py:6494`, `BaseModel.mapped`), pero eso es 1 query/1000, no 1/fila. |
| **Fix** | Mantener el recordset agrupado; `mapped()` en vez de comprehension; en `_compute_display_name` override, pre-`mapped` los campos relacionales antes del bucle. En `onchange`, no recorrer `o2m` enormes. Verificar con `assertQueryCount` que el count NO escala con N. |

### A3. Falta de indice en campos de `search` / `_order` / `group_by` / dominio

| | |
|---|---|
| **Sintoma a escala** | `search`/`read_group` que iba bien en demo tarda segundos en prod; `EXPLAIN ANALYZE` muestra `Seq Scan` sobre tabla con millones de filas; el `_order` por un campo sin indice mete un `Sort` caro. |
| **Causa** | Sin indice b-tree, Postgres escanea toda la tabla. Campos en `_order`, en `groupby` de `read_group`/`_read_group` (`odoo/models.py:1965`, `BaseModel._read_group`), o en dominios frecuentes, escalan O(filas). |
| **Fix** | `index=True` en el campo confirmado por `EXPLAIN`. Para `ilike '%x%'` -> `index='trigram'` (requiere `pg_trgm`). Para filtros muy selectivos sobre un subconjunto (p.ej. `state='draft'` en tabla 99% `done`) -> **indice parcial** manual en `init()`. Re-`EXPLAIN` tras añadirlo: debe pasar a `Index Scan`. |

```python
def init(self):
    tools.create_index(self.env.cr, 'mymodel_state_draft_idx',
                        self._table, ['state'])  # o CREATE INDEX ... WHERE state='draft'
```

| Cuando | Indice |
|--------|--------|
| `=`, `in`, `<`, `>`, `order by`, FK join | `index=True` (b-tree) |
| `ilike '%x%'` (comodin inicial) | `index='trigram'` |
| Filtro selectivo sobre minoria de filas | indice parcial (`WHERE`) en `init()` |
| Campo de baja cardinalidad muy filtrado + selectivo | parcial; si no, no indexar (penaliza write) |

### A4. `read_group` / `search` / `search_count` sin acotar

| | |
|---|---|
| **Sintoma a escala** | `search([])` o `search_count` global trae/cuenta millones; OOM o request matada por `--limit-time-real`. |
| **Causa** | `search` sin `limit` materializa todos los ids. `search_count(domain, limit=None)` (`odoo/models.py:1718`, `BaseModel.search_count`) cuenta TODO salvo que pases `limit` (upperbound). `_read_group` sobre tabla enorme sin dominio acotado agrega millones de filas. |
| **Fix** | `search` SIEMPRE con `limit` en flujos UI/batch. Para "¿hay alguno?" usar `search_count(domain, limit=1)` o `bool(search(domain, limit=1))`, NUNCA `len(search([...]))`. Para "¿existe este recordset?" usar `exists()` (solo valida ids ya cargados, no escanea). Acotar el dominio de `read_group` (fecha, compañia) antes de agregar. |

```python
# MAL: cuenta toda la tabla
has_any = bool(self.env['account.move.line'].search_count([('x','=',v)]))
# BIEN: para en el primer match
has_any = bool(self.env['account.move.line'].search([('x','=',v)], limit=1))
```

`search_count` vs `exists()`: `search_count` lanza SQL (`COUNT`); `exists()` filtra
ids ya en el recordset contra los que existen en BD (barato, no escanea por dominio).

### A5. Coste de cache ORM / `invalidate` / `flush` en bucles

| | |
|---|---|
| **Sintoma a escala** | Proceso batch lento sin queries N+1 obvias; mucho tiempo en `flush`/`invalidate`; SQL crudo que lee valores viejos. |
| **Causa** | Cada `flush_model`/`flush_recordset` (`odoo/models.py:6765`, `BaseModel.flush_model`, y `odoo/models.py:6777`, `BaseModel.flush_recordset`) materializa los recomputos pendientes; llamarlo por iteracion serializa el batch. `invalidate_model(fnames, flush=True)` (`odoo/models.py:7120`, `BaseModel.invalidate_model`) por defecto flushea antes de invalidar. SQL crudo lee el estado de BD, no el cache: si hay writes ORM pendientes, lee sucio. |
| **Fix** | `mapped`/`filtered` cuando los datos ya estan en cache (0 queries); SQL crudo/`read_group` cuando hay que agregar grandes volumenes (evita materializar). En batch: NO flushear por iteracion; dejar que el ORM agrupe y flushee al cierre de transaccion. Antes de SQL crudo de lectura: `flush_model(['campo'])`; tras UPDATE crudo: `invalidate_model(['campo'])`. |

### A6. Write amplification: `mail.thread` tracking, attachments, automations, crons

| | |
|---|---|
| **Sintoma a escala** | Import/batch de 100k records genera 100k+ `mail.message` y `mail.tracking.value`; tabla `mail_message`/`ir_attachment` infladas; el batch tarda x10. |
| **Causa** | Cada `write` sobre un campo `tracking=True` en un modelo `mail.thread` prepara tracking (`mail_thread.py` `_track_prepare`, `_track_finalize` via `cr.precommit`); ademas computes store que tocan campos tracked re-disparan tracking (`mail/models/mail_thread.py:438-442`, `MailThread`). `base_automation` envuelve `create`/`write` y ejecuta `_process` por regla aplicable (`base_automation/models/base_automation.py:779-820`, `make_create.create`): server actions en bucle de records. Crons batch sin `limit` procesan toda la cola en una pasada y mueren por timeout. |
| **Fix** | En imports/migraciones/batch: `with_context(tracking_disable=True)` (desactiva TODO mail.thread) o `mail_notrack=True` (solo no trackea valores) — las dos claves estan documentadas en `mail/models/mail_thread.py:93-95`, `MailThread`, y se leen en `mail/models/mail_thread.py:289`, `MailThread.create`, y `mail/models/mail_thread.py:350-353`, `MailThread.write`. Desactivar reglas `base_automation` durante carga masiva o filtrarlas. Crons: procesar por lotes con `limit` y re-encolar; respetar `--limit-time-real-cron`. Purgar `ir.attachment` huerfanos (versiones de informes, logos duplicados). |

```python
# carga masiva sin generar mensajes/tracking ni disparar automations de mail
records = Model.with_context(tracking_disable=True).create(big_vals_list)
```

### A7. Informes QWeb / wkhtmltopdf y vistas SQL `_auto=False` a volumen

| | |
|---|---|
| **Sintoma a escala** | Informe de 1 doc va bien; el de 500 lineas o el batch de 200 facturas tarda minutos o lo mata `--limit-time-real`; wkhtmltopdf consume RAM/CPU. |
| **Causa** | N+1 en plantilla (`t-foreach` accediendo a relacionales sin prefetch) escala con nº lineas; agregaciones dentro de QWeb se reevaluan por nodo. wkhtmltopdf es CPU-bound y monoproceso por render. Modelos report `_auto=False` (vista SQL) sin indices en las columnas filtradas hacen Seq Scan en cada apertura del informe/pivot. |
| **Fix** | Precalcular relacionales y agregados en `_get_report_values` con `mapped`/`read_group`; pasar dicts a la plantilla. Prefetch de lineas antes del render. Para batches grandes, paginar/encolar el render. En vistas SQL, indexar las columnas de filtro de la query origen. Subir `--limit-time-real` solo para el worker de informes si es legitimo. |

---

## B. Como DIAGNOSTICAR

### B1. SIN acceso a produccion (leer codigo + reproducir)

| Tecnica | Como |
|---------|------|
| **Query-count mental** | Lee el metodo y cuenta: ¿hay `for` que accede a relacional? ¿`search`/`write` dentro? ¿`depends` que cruza relaciones? Marca el patron contra `orm-antipatterns.md`. |
| **Predecir cascada de recompute** | Sigue el `@api.depends`: cada salto de relacion = un `inverse()` en `modified()`. Profundo/ancho = storm potencial. |
| **`assertQueryCount` + `@warmup`** | Reproduce con datos sinteticos y fija que el count NO escala con N (ver `measurement-tools.md` §3). Delega la generacion a `odoo-test-generator`. |
| **Datos sinteticos a volumen** | Crea 10k+ records en test/shell y mide; muchos N+1 son invisibles con 3 records. Si no hay staging, levanta local con `docker-deployment` y restaura un dump del cliente. |

```python
@warmup
def test_scale(self):
    recs = self.env['model'].search([], limit=2000)
    with self.assertQueryCount(__system__=8):   # constante, NO ~2000
        recs.mapped('partner_id.country_id.name')
```

### B2. CON acceso (instancia viva)

> Conexion/ejecucion SIEMPRE via `odoo-pilot`. Flags verificados en
> `$ODOO_SRC/odoo18/odoo/tools/config.py`.

| Tecnica | Comando / como | Verificado |
|---------|----------------|-----------|
| Modo dev | `odoo-bin -d DB --dev=all` (recarga, sin cache assets/QWeb) | — |
| Logging SQL | `--log-sql` (atajo de `--log-handler=odoo.sql_db:DEBUG`) | v18 `odoo/tools/config.py:213`, `configmanager.__init__` · v19 `:329` |
| Logging fino ORM | `--log-handler=odoo.orm:DEBUG` (ej. del help) | v18 `odoo/tools/config.py:211`, `configmanager.__init__` · v19 `:324` |
| Log web/requests | `--log-web` (atajo `odoo.http:DEBUG`) | v18 `odoo/tools/config.py:212`, `configmanager.__init__` · v19 `:327` |
| Timeout request | `--limit-time-real` (def 120s), `--limit-time-cpu` (def 60s) | v18 `odoo/tools/config.py:364`, `configmanager.__init__`, y `odoo/tools/config.py:361` · v19 `:488,485` |
| Timeout cron | `--limit-time-real-cron` (def -1 = usa real) | v18 `odoo/tools/config.py:367`, `configmanager.__init__` · v19 `:491` |
| Memoria worker | `--limit-memory-soft` / `--limit-memory-hard` | `odoo/tools/config.py:342`, `configmanager.__init__`, y `odoo/tools/config.py:350` |
| Workers prefork | `--workers N` (0 = sin prefork) | `odoo/tools/config.py:339`, `configmanager.__init__` |
| Requests/worker | `--limit-request` (def 2**16) recicla el worker | `odoo/tools/config.py:368`, `configmanager.__init__` |

```bash
# reproducir UNA accion contando SQL (filtra ruido segun regla de entorno)
odoo-bin -d DB --dev=all --log-sql 2>&1 \
  | grep -v 'unknown parameter.*export\|selection attribute will be ignored\|Will use the Wkhtmlto'
```

**Postgres (donde esta el tiempo real a escala):**

| Herramienta | Para que |
|-------------|----------|
| `EXPLAIN (ANALYZE, BUFFERS) SELECT ...` | Seq Scan vs Index Scan, filas reales vs estimadas, lecturas de disco |
| `pg_stat_statements` | Top queries por tiempo total/llamadas en toda la BD (la vista mas util en prod) |
| slow query log (`log_min_duration_statement`) | Captura queries > umbral sin instrumentar la app |
| `pg_stat_user_tables` / `pg_stat_user_indexes` | Detectar seq scans altos y tablas sin indice usado |

```sql
-- top 10 por tiempo total acumulado
SELECT query, calls, total_exec_time, mean_exec_time
FROM pg_stat_statements ORDER BY total_exec_time DESC LIMIT 10;

EXPLAIN (ANALYZE, BUFFERS) SELECT id FROM account_move_line WHERE move_id = 123;
```

**CPU / Python (cuando no es BD):**

| Herramienta | Para que |
|-------------|----------|
| `py-spy top --pid <PID>` / `py-spy record -o fl.svg --pid <PID> -d 30` | Flame graph del worker en marcha, SIN reiniciar (ver `measurement-tools.md` §4) |
| Profiler integrado `odoo.tools.profiler.Profiler` | SQL + frames Python combinados (`measurement-tools.md` §5) |
| `perf record -p <PID>` (Linux) | Si el coste esta en C/wkhtmltopdf, no en Python |

**Workflow en prod:** `pg_stat_statements` -> aisla la query top -> `EXPLAIN
(ANALYZE, BUFFERS)` -> si Seq Scan ⇒ A3 (indice); si tiempo en app no en BD ⇒
`py-spy` ⇒ A1/A2/A6. Reproduce en local (`docker-deployment` + dump) antes de
tocar nada. Fija la ganancia con `assertQueryCount` (`odoo-test-generator`).
