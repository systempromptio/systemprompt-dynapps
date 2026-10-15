# Pipeline (diagrama completo)

Contratos de cada subagent en `references/pipeline-detail.md`.

```
[OPUS principal] Paso 0: self_test.sh — verificar entorno (auth + wrappers + PyYAML)
   |
   |-- [SONNET subagent A] Paso 1: inventory_modules.sh    (paralelo con B,C)
   |   delega a odoo-pilot -> JSON instalados
   |
   |-- [SONNET subagent B] Paso 4: studio_scan.sh          (paralelo con A,C)
   |   query Studio fields/actions/automations -> studio.yaml
   |
   |-- [SONNET subagent C] Paso 1b: inventory_disk_vs_installed.py (paralelo con A,B)
   |   cruzar addons_path con BD -> disk_only / installed_without_source
   v
[OPUS principal] Paso 2: clasificar (classification-rules.md)
   + decidir que modulos requieren deep-dive (hash cambiado | en target | nunca scaneados)
   |
   |   Si modo == 'incremental': lanzar incremental_scan.py para listar candidatos
   v
[N SONNET subagents en paralelo] Paso 3: deep-dive
   delegar a skill `odoo-module-scanner` (o usar full_scan inline)
   -> deep/<name>.md
   |
   v
[SONNET subagent] Paso 4b: extract_studio_code.py
   exporta SAs (.py), automations (.py con SA inline), computed fields (.py)
   actualiza studio.yaml con code_file refs
   |
   v
[OPUS principal] Paso 5: compute_collisions.py -> collisions.yaml
   |
   v
[SONNET subagent opcional] Paso 6: dev-assessment
   solo si modo=target y usuario describio funcionalidad nueva
   |
   v
[OPUS principal] Paso 7: update_memory.py (OBLIGATORIO, atomic write)
   modules.yaml + studio.yaml + collisions.yaml + SNAPSHOT.md + meta.json
   + studio_code/ + warnings + delta + disk_only
   |
   v
[SONNET subagent] memory-check valida `.ai-memory/`
   |
   v
[OPUS principal] render_briefing.py -> Markdown final al usuario
```

## Pipeline del modo `knowledge` (explicito)

K0 resolver custom (modules.yaml) → K1 `build_knowledge_bundle.py` (bundle/módulo, sin LLM) →
K2 [N SONNET, 1 por tanda] leer bundle + VERIFICAR anclas vs fuente → `core.md`+`gotchas.md` →
K3 `gen_knowledge_index.py` → `symptoms.md`+`INDEX.md` (determinista).

Pipeline detallado, contrato de formato de gotchas y lecciones de rendimiento en `references/knowledge-mode.md`.
