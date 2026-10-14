# Pipeline Detail

Contratos y ejecucion de cada subagent del pipeline.

## ⚠️ READ-ONLY contract

Toda operacion contra Odoo en este pipeline es **estrictamente de lectura**. Cualquier subagent que use `odoo-pilot` debe limitarse a:

- `auth.sh` (autenticacion)
- `search_records.sh`
- `execute_method.sh` con uno de estos metodos: `read`, `search`, `search_read`, `search_count`, `fields_get`, `name_search`, `name_get`, `default_get`, `exists`, `get_metadata`, `browse`.

**Prohibido**: `create_record.sh`, `update_record.sh`, `delete_record.sh`, `execute_method.sh` con `create`/`write`/`unlink`/`copy`, o cualquier metodo custom que mute estado (`button_*`, `action_*`, etc.).

`_studio_render.py` enforza esto con un guard runtime (lista blanca `READ_ONLY_METHODS`). Cualquier intento de invocar un metodo mutante levanta excepcion.

## Step 1 — Inventario [SONNET subagent]

**Pattern**: A (single subagent, fire-and-forget hasta retorno).

**Prompt al subagent**:
```
Lista todos los modulos instalados en Odoo via la skill odoo-pilot.

⚠️ READ-ONLY: prohibido cualquier metodo mutante (create/write/unlink/copy
ni button_*/action_*). Solo auth.sh + search_records.sh.

Pasos:
1. Ejecutar `auth.sh` para autenticar.
2. Ejecutar `search_records.sh` sobre `ir.module.module` con domain
   `[('state', 'in', ['installed', 'to upgrade', 'to install'])]`
   y campos: name, display_name, latest_version, installed_version,
   state, author, summary, depends, category_id, application,
   auto_install, license, website.
3. Para cada modulo, intentar localizar su path en filesystem (ver
   addons_path config). Si no accesible, marcar `path: null` y
   `scan_source: remote_only`.
4. Output: JSON array a stdout con todos los campos + path.

NO hacer deep-dive de modulos. NO leer codigo. Solo metadata.
NO mutar la BD bajo ningun concepto.

Coste maximo: 1 query de search + 1 read en bloque (limit=1000).
```

**Output esperado**: JSON array con N entradas. Persistido temporalmente en `.tmp/inventory_raw.json`.

**Failure modes**:
- Conexion Odoo falla -> abort pipeline, mostrar error al usuario
- Queries timeout -> retry 1 vez con limit reducido
- Sin acceso a filesystem -> degrade a `scan_source: remote_only` para todos

## Step 2 — Clasificar [OPUS principal]

NO subagent. Lo hace el principal directamente porque:
- Es un mapping deterministico
- Necesita los datos completos del Step 1 en contexto
- Decide que modulos van al Step 3

**Algoritmo**:
1. Para cada modulo del JSON: aplicar `classification-rules.md`.
2. Asignar `type` y `classification_confidence`.
3. Construir lista de candidatos a deep-dive:
   - Si modo=full: todos los `type in [custom, indaws_core]`
   - Si modo=target: solo `target_module` + dependencias inversas (BFS) + cualquier no-estandar con hash cambiado
   - Si modo=quick: ninguno (skip Step 3)
   - Si modo=studio-only: ninguno (skip Step 3)

**Output**: lista `to_deep_dive: [name, ...]` para Step 3.

## Step 3 — Deep-dive [SONNET subagents en paralelo]

**Pattern**: B (N subagents en paralelo, output agregado).

**Trigger**: si `to_deep_dive` no esta vacio.

**Prompt por subagent (uno por modulo)**:
```
Analiza el modulo Odoo `{module_name}` y produce un deep-dive completo.

Path: `{module_path}` (puede ser null -> usar `odoo-pilot` para query
remoto a ir.model.fields, ir.ui.view, ir.cron filtrados por module=name).

Pasos:
1. Invocar la skill `odoo-module-scanner` con el modulo.
2. Recoger:
   - Modelos extendidos (_inherit en codigo)
   - Modelos nuevos (_name)
   - Campos añadidos (count + lista de los 10 mas relevantes)
   - Vistas heredadas vs nuevas
   - Hooks: ir.cron, ir.actions.server, base.automation declarados
   - Dependencias inversas (otros modulos que lo declaran en depends)
   - Test coverage (existe tests/? cuantos test_*.py?)
   - LOC Python (sin comentarios/blank)
3. Escribir markdown a `.ai-memory/inventory/deep/{module_name}.md`
   con secciones: Modelos / Vistas / Hooks / Dependencias / Tests / Notas.
4. Output a stdout: resumen JSON con los campos para `modules.yaml`
   (extends_models, new_models, fields_added, views_inherited,
    has_cron, has_server_actions, has_automated_actions, loc, hash).

NO leer las dependencias del modulo (otro subagent las hara si tocan).
Coste limitado: solo este modulo.
```

**Concurrency**: lanzar todos en paralelo (1 message con N Agent calls).

**Failure modes per subagent**:
- Path null + modulo es indaws_core/custom: degrade — usar info de odoo-pilot solo, marcar `hash: unknown`, `risk_factors` añade "scan_source: remote_only (+3pt)"
- odoo-module-scanner no disponible: fallback a inspection manual (read manifest + glob *.py + grep `_inherit`/`_name`/`fields\\.`)

## Step 4 — Studio scan [SONNET subagent]

**Pattern**: A (single subagent), corre EN PARALELO con Step 3.

**Prompt**:
```
Escanea las customizaciones Studio en la BD Odoo activa.

⚠️ READ-ONLY: solo lectura. execute_method.sh permitido SOLO con metodo `read`.
Prohibido cualquier metodo mutante.

Pasos:
1. Ejecutar `auth.sh`.
2. Comprobar si `web_studio` esta installed:
   `search_records.sh ir.module.module domain="[('name','=','web_studio'),('state','=','installed')]"`
   Si no esta, output minimo y exit (`studio_available: false`).
3. Si Studio activo, ejecutar las queries descritas en
   `references/studio-detection.md`:
   - models con state=manual y model like x_%
   - fields con state=manual y name like x_%
   - server_actions cruzadas con ir.model.data module='studio_customization'
   - automated_actions cruzadas igual
   - menus cruzados igual
   - views modificadas por Studio (arch_fs=False, cruzar con ir.model.data)
4. Output: YAML completo a `.tmp/studio.yaml` con la estructura de
   `references/inventory-schema.md`.

NO mutar la BD bajo ningun concepto.

Coste: ~6 queries Odoo. Si total_fields > 200, paginar.
```

**Output**: `.tmp/studio.yaml` listo para mover en Step 7.

## Step 5 — Compute collisions [OPUS principal]

NO subagent. Es un cruce determinista de YAMLs.

**Algoritmo**:
1. Leer `.tmp/modules_partial.yaml` (resultado de Step 1+2+3 agregados).
2. Leer `.tmp/studio.yaml`.
3. Por cada modulo no-estandar (custom + indaws_core), recoger `extends_models`.
4. Por cada field Studio, su `model` cuenta como extension Studio.
5. Agrupar por modelo. Si count >= 2 -> entrada en `collisions.yaml`.
6. Calcular `severity` segun `risk-scoring.md`.
7. Escribir `.tmp/collisions.yaml`.

## Step 6 — Dev-assessment [SONNET subagent OPCIONAL]

**Trigger**: solo si modo=target Y el usuario incluyo descripcion de funcionalidad nueva en su mensaje.

**Prompt**:
```
Evalua si la siguiente funcionalidad ya existe o se puede resolver con
Odoo standard / Indaws Core / Studio antes de desarrollar custom.

Funcionalidad: {USER_DESCRIPTION}
Modulo objetivo: {TARGET_MODULE}
Contexto del proyecto: {SUMMARY_FROM_INVENTORY}

Pasos:
1. Invocar la skill `dev-assessment` con la descripcion.
2. Output al stdout: bloque markdown con
   - Alternativas standard encontradas
   - Modulos Indaws Core que cubren el caso
   - Posibilidad Studio (no-code)
   - Recomendacion final + esfuerzo estimado
```

**Output**: snippet markdown que se pega en seccion 4 del briefing.

## Step 7 — Memory sync [OPUS principal]

NO subagent. Lo hace el principal con `update_memory.py`.

**Algoritmo**:
1. Acquire lock `.ai-memory/inventory/.lock`.
2. Calcular delta vs `modules.yaml` previo.
3. Escribir 4 ficheros + deep/ a `.tmp/`.
4. Calcular hash global.
5. Escribir SNAPSHOT.md a `.tmp/`.
6. Move atomico de los 4 ficheros + SNAPSHOT.md.
7. Limpiar deep-dives huerfanos (modulos desinstalados).
8. Release lock.
9. Invocar `memory-check sync` (subagent sonnet) para validar.

**Failure handling**:
- Lock taken -> abort con mensaje claro
- Write fail -> rollback (.tmp se descarta, originales intactos)
- memory-check fail -> warn pero no rollback (memoria ya escrita)

## Step 8 — Render briefing [OPUS principal]

NO subagent. Render directo con `render_briefing.py`.

**Algoritmo**:
1. Leer los 4 ficheros recien escritos.
2. Aplicar template de `references/briefing-template.md`.
3. Si dev-assessment se ejecuto, embedar su output.
4. Output a stdout (visible al usuario).

## Resumen de coste por modo

| Modo | Subagents sonnet | Calls a opus principal | Tokens estimados |
|------|------------------|-----------------------|------------------|
| quick | 2 (inventory + memory-check) | 1 sintesis | ~10K |
| target sin cambios en deps | 3 (inventory + studio + memory-check) | 1 sintesis | ~15K |
| target con 5 deep-dives | 8 (inventory + studio + 5 deep + memory-check) | 1 sintesis | ~40K |
| target con dev-assessment | 9 (igual + dev-assess) | 1 sintesis | ~50K |
| full (15 no-estandar) | 18 (inventory + studio + 15 deep + memory-check) | 1 sintesis | ~80K |
| studio-only | 2 (studio + memory-check) | 1 sintesis | ~8K |

(Cifras orientativas para BD con 200 modulos.)
