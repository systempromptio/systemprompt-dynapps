# Handoff de sesión — odoo-deep-knowledge (2026-07-14)

Estado para retomar en sesión fresca. Todo lo hecho está en `main` (rama de trabajo original: `kb/heldout-recall-eval`, ya mergeada). Versión actual: **SKILL 2.6.4 · plugin 2.2.4**. Release-check **8/8 verde**.

## Qué se hizo esta sesión (mergeado a main)

| PR | Contenido |
|----|-----------|
| #62 | **Batch ES-síntoma a escala**: 2322 líneas `_ES:` (una por gotcha inglés) → recall keyword sube. Fixes: apply regex a 3 formatos de cabecera, gen_index quita `_ES` del título, recall_eval `--file=`. |
| #63 | **QA muestral** de las líneas ES (100 → 92 ok/8 minor/0 bad). Pulidas las 8 minor. Tooling: `sample_es_lines.py`, `qa_es_workflow.js`. |
| #64 | **PoC embeddings fase 1**: MiniLM/mpnet/e5-small/e5-large vs grep+`_ES`. Doc `docs/embeddings-poc-findings.md`. |
| #65 | **Gate cobertura `_ES`** (100%, anti-pudrición, `scan_es_coverage.py`, gate #8) + **PoC fase 2** (traduce-query, híbrido RRF, recall honesto arms-length). |
| #66 | **Verificación de correctitud** a escala (120 gotchas, 93%) + fix 2 REFUTED (BUSG20, PRG12). Tooling: `sample_gotchas.py --json`, `verify_gotchas_workflow.js`. |

## Números honestos (verificados esta sesión)

- **Correctitud del contenido: ~93%** (111/119 verificables, muestra 120 vs fuente v18). Sólido, confirma el ~92% previo. → un gotcha equivocado desinforma; es el eje de más valor.
- **Recall REAL ≈ 38%** sobre queries arms-length (usuario NO comparte vocabulario con el gotcha). **El 84-95% anterior era FUGA LÉXICA** (queries escritas mirando los gotchas). Ningún método rompe ~38%: grep+`_ES`, embeddings, traduce+e5, híbrido — todos ahí.
- **Cobertura `_ES`: 100%** (2125 gotchas EN, 0 gaps; gate lo mantiene).

## Aprendizajes clave (no repetir el camino)

1. **Embeddings multilingües NO baten grep+`_ES`** en CPU. e5-small: 95% en EN, **10% en ES** (cross-lingual débil). MiniLM/mpnet (paraphrase) mediocres en retrieval. e5-large descartado (compute-bound, ~40min/indexado en CPU).
2. **Traduce-query (ES→EN) arregla el colapso cross-lingual** (e5 0→38% arms-length) pero **no sube el techo** de recall.
3. **Híbrido RRF ciego es condicional**: ayuda si los componentes son parejos, PERJUDICA si uno domina (grep 95→70). El correcto es fallback-con-confianza.
4. Para **grupo internacional**: KB en inglés + traduce-query→EN + retrieval híbrido; matar la idea de símtomas por-idioma (`_ES` fue correcto para KB español-first, no escala a N idiomas).

## Backlog vivo (retomar en fresco, por prioridad)

### A. Correctitud — 6 PARTIAL a corregir (detalle en memoria [[kb-correctness-verification]])
- **MMG16**: "orphaned campaigns accumulate" = inferencia no probada → suavizar.
- **MTG28**: cita `_compute_maintenance_team_id`; el real es `_compute_maintenance_request` (maintenance.py:106) → fix nombre.
- **SLDG3**: línea :135 → debe ser :175 (slide_channel.py) → fix línea.
- **MCG3**: "email queda sin leer si falla" inexacto para errores capturados (:242 marca Seen tras except) → matizar.
- **PG8**: "el blanket gana aunque más alto" no respaldado (`_select_seller` ordena por price_discounted) → suavizar.
- **ICG22**: núcleo OK; afirmaciones alrededor (eliminación consolidación, cuentas IC, aging) son funcionales no presentes en fuente → marcar/suavizar.
- Al acabar: re-correr `verify_gotchas_workflow.js` para número más ajustado.

### B. Búsqueda — fase 3 (única palanca para >38%, no medida)
- **LLM-in-the-loop**: query-expansion (LLM reescribe la consulta del usuario en varias formulaciones técnicas antes de recuperar) o rerank de un pool amplio. Mete un LLM en cada búsqueda → decisión de coste/latencia, no solo técnica.
- Alternativa: embedder API (Voyage / OpenAI-3-large) — no probado (no había API keys en el entorno).

### C. Otros ejes no-búsqueda (sin empezar)
- **Eval end-to-end** (la diana real): ¿cargar la skill hace que Claude diagnostique mejor un bug real? Coger N bugs, resolver con/sin skill, comparar. Caro (oráculo por caso) pero es el ROI de verdad.
- **Mapa de cobertura**: cruzar dominios documentados vs lista real de módulos Odoo → puntos ciegos (gap conocido: OCA-no-contable).
- **Frescura/drift**: KB anclado v18+diffs v19; v19 ya está, v20 viene. ¿Se corre `scan_drift.py`? (ver [[kb-drift-watchdog]]).
- **Contradicciones internas**: no hay chequeo de gotchas que se contradigan entre dominios.
- **Bloat de tokens**: bloques-gotcha largos cuestan al servirse; hay `KB_LENCAP` para ranking pero no para tamaño servido.

## Tooling añadido (todo en `scripts/`, repetible)

- `sample_es_lines.py [N]` — muestra determinista de líneas `_ES` (QA).
- `qa_es_workflow.js` — QA de líneas ES (agentes juzgan vs cuerpo del gotcha).
- `scan_es_coverage.py [--list|--check]` — cobertura `_ES` (gate #8; baseline en `_index/es_coverage_baseline`).
- `armslength_queries_workflow.js` — genera queries ES arms-length (recall con menos sesgo léxico).
- `sample_gotchas.py [N] [SEED] [--json]` — muestra para verificación de correctitud.
- `verify_gotchas_workflow.js` — verificación escéptica de correctitud contra fuente.
- `recall_eval.py --file=PATH` — recall contra holdout arbitrario.
- Holdouts: `recall_heldout.json` (ES difícil), `recall_heldout2.json` (ES), `recall_heldout2_en.json` (EN espejo).

## Reproducción del PoC de embeddings (efímero)

El venv (`fastembed`/`torch`/`sentence-transformers`) y los vectores se generaron en scratchpad y **se borraron** (no son dep del skill). Para rehacerlo: `python3 -m venv`, `pip install torch --index-url https://download.pytorch.org/whl/cpu && pip install sentence-transformers`, y el script `emb_poc.py` (versión en `scripts/`). El eval híbrido/arms-length (`hybrid_poc.py`, `heldout_armslength*.json`) vivía en scratchpad — regenerable con `armslength_queries_workflow.js`.

## Nota de coste

Esta sesión terminó con contexto ~320K/turno (caro). Retomar en **sesión fresca**: arranca con git + memoria, coste normal. Memorias relevantes: [[kb-correctness-verification]], [[embeddings-vs-grep-poc]], [[skills-release-roadmap]] (A3.5 corregido), [[kb-drift-watchdog]].
