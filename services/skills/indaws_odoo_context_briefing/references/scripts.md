# Scripts (`scripts/`)

Cada script usable individualmente con `--help`.

| Script | Responsabilidad |
|--------|-----------------|
| `self_test.sh` | **Paso 0** — valida entorno (auth + wrappers + PyYAML) antes de pipeline |
| `inventory_modules.sh` | Lista `ir.module.module` via odoo-pilot (positional args) |
| `inventory_disk_vs_installed.py` | Cruza addons_path filesystem vs BD → disk_only / installed_without_source |
| `compute_module_hash.py` | sha256 de manifest + .py + .xml + .csv + .js + .scss de un modulo |
| `incremental_scan.py` | Lista módulos cuyo hash ha cambiado vs último scan. `--apply` re-deep-dive selectivo y mergea en `modules.yaml` |
| `studio_scan.sh` + `_studio_render.py` | Query Studio completa o ligera (`--quick`); incluye **automations manuales** |
| `extract_studio_code.py` | **Paso 4b** — exporta code de SAs/automations/computed fields a `studio_code/` |
| `resolve_addons_paths.py` | Resuelve addons_paths automáticamente: detecta versión Odoo (`auth.sh`) + parsea `docker-compose.yml` del cwd + aplica convención Indaws (`/opt/odoo/odoo{N}`, `/opt/odoo_enterprise/{N}`, `/opt/OCA/v{N}`, `/opt/PROYECTS/indawsodoo/{N}`) |
| `deep_dive_modules.py` | **Paso 3** — clasifica + resuelve `source_path` + extrae `_inherit/_name` + LOC + hash + risk; genera deep/<name>.md y modules.yaml completo. Si no se le pasa `--addons-paths`, llama a `resolve_addons_paths.py` |
| `compute_collisions.py` | Cruza `extends_models` → `collisions.yaml` |
| `update_memory.py` | Atomic write de los 4 ficheros + meta.json sidecar + warnings + delta |
| `build_knowledge_bundle.py` | **Modo knowledge K1** — pre-arma bundle compacto por módulo custom (README + deep + LOG + mapa de símbolos + manifest) sin LLM. `--modules` para tanda selectiva |
| `gen_knowledge_index.py` | **Modo knowledge K3** — regenera `knowledge/_index/symptoms.md` + `INDEX.md` de forma determinista (extrae títulos de gotchas). `--symptoms-only` para solo el índice |
| `read_inventory.py` | CLI/libreria de consulta. Inventory: `--paths`, `--path-of`. **KB**: `--symptom <q>`, `--gotchas <m>`, `--core <m>`, `--knowledge-modules` |
| `render_briefing.py` | Genera Markdown final (con clusters + anomalias + auto-recs). Flags: `--target-only` (solo modulo objetivo), `--since <fecha\|hash>` (diff mode) |
| `detect_target_impact.py` | BFS inverso sobre `depends` para dependencias inversas |
