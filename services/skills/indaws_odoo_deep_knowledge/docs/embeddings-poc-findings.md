# PoC: retrieval semántico multilingüe (embeddings) vs grep+`_ES`

**Fecha:** 2026-07-14 · **Contexto:** ¿para un grupo internacional conviene KB en inglés + embeddings
multilingües en vez de líneas-síntoma por-idioma (`_ES`)? Este PoC lo mide con datos.

## Montaje

- **Corpus:** un doc por bloque-gotcha, **EN-only** (se elimina la línea `_ES`) → 4517 gotchas.
  El objetivo es medir si un modelo multilingüe casa una query ES/EN contra contenido inglés
  **sin ninguna ayuda por-idioma**.
- **Métrica:** recall@6 + MRR sobre 3 sets held-out: `recall_heldout2.json` (20 ES),
  `recall_heldout2_en.json` (20 EN, mismos gotchas traducidos a mano), `recall_heldout.json` (24 ES, difícil).
- **Baseline:** `kb.py` (grep + diccionario ES→EN + líneas `_ES`), que SÍ usa contenido español.
- **Motor:** fastembed (ONNX, CPU) para MiniLM/mpnet/e5-large; sentence-transformers (torch CPU) para e5-small.
  Script: `scripts/emb_poc.py` (requiere venv externo con fastembed + torch + sentence-transformers; NO es
  dependencia del skill). Reproducible con semilla; embeddings cacheados por (modelo, corpus).

## Resultados — recall@6 (embeddings EN-only, SIN `_ES`) vs grep+`_ES`

| Set | MiniLM-L12 | mpnet-base | **e5-small** | **grep+`_ES`** |
|---|---|---|---|---|
| holdout2 **ES** (20) | 60% | 40% | **10%** | **95%** |
| holdout2 **EN** (20) | 70% | 75% | **95%** (MRR 0.77) | 75% |
| holdout ORIG **ES** (24, difícil) | 38% | 29% | 33% | 38% |

Tiempos de indexado (4517 docs, CPU): MiniLM 114s · mpnet 1178s · e5-small 172s · e5-large ~40min (compute-bound,
no terminó; threads/batch no ayudan → descartado en CPU sin GPU/int8).

## Hallazgo clave

**Ningún embedder viable en CPU es a la vez bueno en (a) retrieval y (b) alineación cross-lingual ES→EN:**

- **e5-small** (modelo de *retrieval* correcto, prefijos `query:`/`passage:`): **el mejor en inglés (95%, bate a
  grep) y el peor en español (10%)** — targets ES en rank 1000-2000. Su capacidad cross-lingual es débil; brilla
  EN↔EN.
- **MiniLM/mpnet** (modelos *paraphrase/STS*, simétricos): cross-lingual decente pero retrieval mediocre → 40-75%.
- **grep+`_ES`**: 95% ES, pero a base de contenido español escrito a mano (no escala a FR/NL/DE).

Nota: mpnet (más grande, mejor en MTEB) rindió **por debajo** de MiniLM en ES → el ranking de benchmark ≠
rendimiento en esta tarea asimétrica de dominio.

## Conclusión para un grupo internacional

El "que la IA traduzca on the fly" es correcto, **pero va en la QUERY, no en el KB**:

1. **Query en inglés → e5-small = 95%, cero contenido por-idioma.** Retrieval semántico sobre KB inglés funciona
   de sobra *si la query entra en inglés*.
2. **Query en otro idioma directo → embeddings cross-lingual = 10-60%** (flojo en CPU). Aquí `_ES` tapa el agujero
   (95%) pero no escala a N idiomas.
3. **Solución agnóstica al idioma:** `query(cualquier idioma) → traducir a EN (MT/LLM barato) → retrieval semántico
   (e5-small) o híbrido (grep ∪ e5-small) sobre KB inglés`. Un paso de traducción, un KB inglés, **cero símtomas
   por-idioma**. Evidencia: el holdout EN (= la query ES "traducida") da 95%.

En ese mundo, las líneas `_ES` **sobran**. Fueron la decisión correcta para el KB actual (español-first, grep-only);
dejan de serlo con traduce-query + embeddings.

## Recomendación

- **Grupo internacional (multi-idioma):** traduce-query-a-EN + retrieval **híbrido** (grep ∪ e5-small) sobre KB
  inglés. Abandonar el rumbo de `_ES`-por-idioma.
- **KB español-first de hoy:** `_ES`+grep sigue siendo lo mejor (95% ES, 0 infra extra).
- **No hacer:** e5-large en CPU (40 min/indexado); traducir el *contenido* del KB (inútil, la comprensión ya es
  agnóstica — Claude lee EN y responde en cualquier idioma).

## Fase 2 (2026-07-14): traduce-query, híbrido RRF y recall honesto (arms-length)

Tabla ampliada (grep+`_ES` / e5-small EN-only / híbrido RRF), recall@6 / MRR:

| Set | grep+`_ES` | e5-small | híbrido RRF |
|---|---|---|---|
| holdout2 ES (directo) | 95% / .88 | 10% / .04 | 70% / .42 |
| holdout2 EN (mano) | 80% / .67 | 95% / .77 | 90% / .79 |
| holdout2 ES→EN (traducido) | 80% / .73 | 90% / .77 | 85% / .78 |
| ORIG ES (difícil) | 38% / .31 | 33% / .24 | 42% / .30 |
| **arms-length ES** | 38% / .26 | 0% / .00 | 22% / .10 |
| **arms-length ES→EN** | 25% / .21 | 38% / .27 | 38% / .32 |

**#1 traduce-query — CONFIRMADO parcialmente.** Traducir la query ES→EN sube e5 de 10%→90% en holdout2
y de 0%→38% en arms-length. La traducción **arregla el colapso cross-lingual** (e5 en ES directo es 0-10%).
Pero NO sube el techo de recall, solo lleva el no-inglés al mismo nivel que el inglés.

**#3 híbrido RRF — NO es gratis, es condicional.** Ayuda solo cuando los componentes son parejos (ORIG ES
38/33→42). PERJUDICA cuando uno domina (holdout2 ES: grep 95 → híbrido 70, la basura del e5 contamina la
fusión). El híbrido correcto es **fallback con confianza** (grep primario; e5 solo si grep viene flojo), NO
unión ciega.

**#4 arms-length — el hallazgo clave sobre INTEGRIDAD DE LA MÉTRICA.** Con queries donde el usuario NO
comparte vocabulario con el gotcha (lo realista), **todo topa en ~38%**: grep+`_ES` 38%, e5-traducido 38%,
híbrido 38-42%. **El 84-95% de los holdouts anteriores era fuga léxica** (las queries se escribieron mirando
los gotchas). El recall HONESTO del sistema ronda **~38%**, coincidiendo con el set "difícil" y con el techo
que el propio docstring de kb.py predijo (~28% miss del grep). **Ni el grep ni los embeddings ni la traducción
rompen ese techo** — el salto entre el lenguaje llano del usuario y el framing técnico del gotcha es el muro real.

**Qué rompería el ~38% (no medido, hipótesis para fase 3):** LLM en el bucle — expansión de query (el LLM
reescribe la consulta del usuario en varias formulaciones técnicas antes de recuperar) o rerank de un pool
amplio de candidatos; o un embedder mucho más fuerte vía API (Voyage/OpenAI-3-large). NO es cuestión de
grep-vs-embeddings: ambos comparten el mismo techo.

Tooling: `scripts/scan_es_coverage.py` (+ gate #8 en kb_release_check, cobertura _ES 100%),
`scripts/armslength_queries_workflow.js`, `scripts/recall_heldout2_en.json`. Los sets arms-length y el eval
híbrido viven en scratchpad (dependen del venv externo).

## Pendiente si se retoma

- Medir la **traducción real ES→EN** con un LLM (aquí se usó EN hecho a mano como proxy) — tiene su propia
  verificación de calidad.
- Probar e5-small/e5-base con **GPU o int8** para el número de e5-large sin el coste CPU.
- Prototipar el **híbrido** (unión grep+embeddings + rerank) y medir si sube el set difícil por encima de 38%.
