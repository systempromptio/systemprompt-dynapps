# GOTCHAS

1. **`odoo-pilot/search_records.sh` usa args POSICIONALES**, no `--model`/`--domain`. Firma: `<model> <domain_json> [fields_json] [limit] [offset]`. Verificar versión con `--help` antes de cambiar la skill.
2. **`odoo-pilot/execute_method.sh` antiguo extraía `result` con `sed`** — fallaba con contenido multilínea (ej. `code` de SAs). El fix de la skill (jq + python fallback) está en common-skills:odoo-pilot. Si el wrapper instalado es viejo, `extract_studio_code.py` lo evita usando solo `search_records.sh`.
3. **Studio NO tiene hash de archivo** — siempre re-escanear, vive en BD.
4. **Hash incremental requiere acceso al filesystem del addons_path**. Si no, marcar `hash: unknown` y `risk: high`.
5. **Hash AHORA incluye .xml + .csv + .js + .scss + .css** además de .py — un cambio de vista o seguridad ya invalida el hash.
6. **`.ai-memory/inventory/` puede ser destructivo si se mezcla con otros proyectos**. Escribir SOLO bajo `.ai-memory/inventory/`.
7. **Atomic writes** — escribir a `.tmp` y `mv`.
8. **Modulos `to upgrade` y `to install`** — incluirlos con su `state` real.
9. **Dependencias inversas en `target`** — BFS sobre `depends` invertido.
10. **`extends_models` en modulos heredados** — distinguir extension Python de XML. Solo Python cuenta para colisiones de modelo.
11. **Studio fields en modelos custom** — reportar donde vive el campo.
12. **Multi-company** — inventario global, no por compañía.
13. **v14-v15 sin Studio** — degradar silenciosamente.
14. **`base.automation` v17+** — el `code` ya NO está en `base.automation`; vive en `ir.actions.server` enlazada via `action_server_ids`. `extract_studio_code.py` resuelve esto: por cada automation graba su SA enlazada inline.
15. **Automations creadas via Settings** (no exportadas en `studio_customization`) son frecuentes en proyectos legacy. Studio scan AHORA las incluye con `origin: manual`. Si re-exportas Studio, esas se pierden — alertar al usuario.
16. **`studio_customization` cuenta como `custom` en classification**, NO como `studio`. La etiqueta `studio` solo se usa en `studio.yaml`.
17. **Hash cross-mode delta**: cuando el scan anterior tenía `hash: unknown` y el nuevo lo calcula, NO se cuenta como "upgraded" (sería ruido). Comparación solo entre dos hashes reales.
18. **Cross-mount Docker**: si el container monta paths bajo `/opt/odoo/...` pero el host los tiene en `/opt/PROYECTS/...`, el hash debe correrse contra los paths del HOST (donde la skill tiene acceso). Indicar mappings via `--addons-paths`.
19. **Reconciliación type vs subtype**: si el classify por author dice "oca" pero el path del modulo está en `/opt/odoo/odoo{N}/`, gana el path → corrige a `standard`. Idem para "standard" en `/opt/OCA/` → corrige a `oca`. El filesystem es la fuente de verdad.
20. **`hash_algo` versionado**: cuando se cambia el set de extensiones que entran al hash (v1=py only → v2=py+xml+csv+js+scss), se incrementa `HASH_ALGO`. `compute_delta` no marca como "upgraded" cambios que cruzan algoritmos.
21. **`monkey_patches` heurística**: detectamos `ClassName.attr = ...` top-level fuera de `_inherit`. Anti-pattern Odoo. Cuidado con falsos positivos en helpers (`Command`, `fields`).
22. **Resolver cache TTL 60s** en `/tmp/odoo-context-briefing-resolver-<hash>.json`. Para forzar recompute: borrar el cache file o llamar a `resolve()` con `use_cache=False`.
