<!-- verified: v19.0 @ 2026-07 (OCA queue 19.0.2.0.2 — diffs verificados vs $OCA_SRC/v18 y $OCA_SRC/v19) -->
# OCA queue_job — Cambios v19

Manifest v19: `version 19.0.2.0.2`, mismos `depends: ['mail', 'base_sparse_field', 'web']`. La
arquitectura (runner monkeypatch, NOTIFY + HTTP runjob, estados, retry, identity_key) **no cambia**.
`utils.py` y `exception.py` idénticos a v18. Diffs relevantes:

## 1. `_patch_method` ELIMINADO — el auto-delay parchea a nivel de tipo (`models/base.py`)

En v18, el auto-delay usaba el helper de clase `Base._patch_method(name, method)` (`queue_job/models/base.py:263-270`, `class Base`). En
**v19 ese classmethod desaparece**: `_patch_job_auto_delay` parchea directamente con
`setattr(type(self), name, functools.update_wrapper(patched, getattr(type(self), "foo")))` y toma el
origin con `origin_func = getattr(type(self), method_name)`. **Impacto Indaws:** cualquier código que
llame `self._patch_method(...)` (patrón usado por `indaws_data_export`, ecosystem/indaws IG31) **rompe en
v19** — hay que migrar al patrón `setattr(type(self), ...)`. Revísalo antes de portar el motor de export.

## 2. Índices vía `odoo.tools.sql.create_index` (`models/queue_job.py`)

v18 creaba los índices en `init()` con `index_exists()` + `CREATE INDEX` en SQL crudo. **v19** usa
`from odoo.tools.sql import create_index` y los declara con `create_index(name, ..., comment=...)`. Mismos
2 índices (identity_key parcial + channel/date_done para autovacuum), API distinta. Si tienes overrides de
`init()` que replicaban el patrón viejo, adáptalos.

## 3. `read_group` → `_read_group` (`models/queue_job.py`)

Los `read_group` de `_compute_dependency_graph` y `_compute_graph_jobs_count` pasan a `_read_group`
(cambio general Odoo v17+). Firma/retorno distintos (lista de tuplas, no dicts). Cross-ref
`common-skills:odoo-version-api-reference` (read_group → _read_group).

## 4. Autovacuum borra en chunks con orden estable (`models/queue_job.py:autovacuum`)

v19 añade "Delete in chunks using a stable order (matches composite index)" — el borrado se apoya en el
índice compuesto `channel,date_done,date_created`. Comportamiento equivalente (QJG20), más eficiente.

## 5. Runner: la BD del `runjob` viaja en HEADER, no en query-string (`jobrunner/runner.py`)

**Cambio de despliegue importante.** v18: `GET /queue_job/runjob?db={db}&job_uuid={uuid}`. **v19:**
`GET /queue_job/runjob?job_uuid={uuid}` con la BD en el **header `X-Odoo-Database`**
(`requests.get(url, ..., headers={"X-Odoo-Database": db_name})`). Impacto: **un reverse-proxy/balanceador
delante de Odoo debe reenviar el header `X-Odoo-Database`** o el runjob no resolverá la BD en multi-db →
jobs `enqueued` que no arrancan (variante v19 de QJG4). Además el manejo de `db_name` en el listado de BDs
se simplifica (`config["db_name"]` directo). Revisa la config de proxy al migrar a v19.
