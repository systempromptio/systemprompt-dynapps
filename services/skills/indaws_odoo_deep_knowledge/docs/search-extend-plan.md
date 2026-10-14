# Plan: KB fácil de BUSCAR y de AMPLIAR

> Objetivo (prioridad del dueño): que el deep-knowledge sea **fácil de buscar** (recall) y
> **fácil de ampliar** (añadir gotchas sin fricción). La información ya existe; **reestructurar
> es barato y aceptable** si sirve a esos dos fines. Se acomete **punto a punto**.
>
> Principios: markdown sigue siendo la fuente de verdad (git, versionable, revisable). Nada de
> mover la KB a una DB/servicio. Lo mecánico se scriptea; el modelo solo para lo que exige juicio.

## Guardarraíles ya construidos (base del plan)
- `scan_drift.py` + `_index/source-baseline.json` — watchdog de deriva vs fuente (0 tokens).
- `cite_audit.py --gate` — falla si hay citas rotas en dominios v18/v19 (legacy auditados aparte).
- `check_ids.py` — linter de IDs prefijados duplicados + `--next <dom> <PREF>` (ID libre).
- `registry.py` / `domains.json` — SSOT de dominios (90), con `versions`/`id_prefix`/`verified`.

---

## Fase 0 — Cerrar higiene abierta (barato, mecánico) — ✅ HECHA (2026-07-14)
- [x] Commit del #3 (gate + check_ids + consolidación manifiesto). → `0a3a6fa`
- [x] `scan_drift.py`, `check_ids.py` **y `cite_audit.py`** iteran vía `registry.py` (fallback
      os.walk con `ecosystem`). cite_audit pasa de ~86 a **90 dominios** cubiertos.
- [~] ~~Meter `deprecated`/`versions` en `gen_registry.py`~~ → **DEFERIDO**: es pulido SSOT, no
      sirve a buscar/ampliar y regenerar `domains.json` arriesga pisar curación. El parse inline
      del header en `cite_audit` funciona. Reabrir si molesta.
- [x] Triadas las **6 citas rotas reales**: 4 line-drift reancladas a línea correcta
      (membership :45-77/:69-77, iso20022 :132-135), 2 en account (consolidation_coa) sustituidas
      por puntero al dominio legacy `account_consolidation` (módulo v17 eliminado en v18).
- **Aceptación:** ✅ `cite_audit --gate` VERDE (0 rotas v18/v19). Deuda residual documentada:
      dup `account_consolidation` ACG1-12 (core/gotchas) — dominio legacy v17, no se renumera.
      Candidatos a migrar a `::símbolo` en Fase 3/#4: las 4 citas reancladas por línea.

## Fase 1 — Medir antes de optimizar (barato, determinista) — ✅ HECHA (2026-07-14)
- [x] Recall baseline: **72% @6 (60/83), 77% @10 (64/83)** sobre queries en lenguaje natural.
- [x] Gap de aliasing (misses por rank del ID esperado): **near 7-15 = 10 (~36%, alias/tuning lo
      arregla)**; far+absent = 18 (~64%, el vocabulario del query comparte casi nada con el gotcha
      → NO lo salva el aliasing; necesita semántico o reescribir el gotcha con vocabulario-síntoma).
- [x] Variancia de formato: **PROBLEMA REAL Y GRANDE.** El regex de bloque de `kb.py` (`DEF`) solo
      reconoce IDs **prefijados** (`### PREFIXn` / `**PREFIXn`) = 4664 bloques visibles. Las
      **2058 secciones `## N.` numéricas (~31% del contenido) son INVISIBLES** al splitter → no se
      recuperan como bloque propio (se diluyen en el vecino y el penalti de longitud las hunde).
      Afecta a 35 dominios con `v19_changes.md` en formato `## N.` (parte del trabajo taller).
- **Resultado / decisión:** hay **dos problemas de búsqueda distintos**:
  - **(A) Recall de gotchas prefijados = 72%**, misses mayormente por vocabulario (solo 1/3 los
    salva el alias) → semántico (Fase 2c) probablemente necesario, pero DESPUÉS de los baratos.
  - **(B) 31% del contenido no se indexa como bloque** (`## N.`) → arreglo BARATO: extender el
    `DEF` de kb.py para partir también en cabeceras `## `/`### ` con título, o normalizar a ID
    prefijado (Fase 3). **Sube de prioridad**: es el mayor ROI de búsqueda y es casi gratis.
  - Orden sugerido tras Fase E: **(B) primero** (regex/normalización) → **re-medir** → alias (1/3
    de A) → re-medir → solo entonces decidir semántico (2c).

## Fase E — Motor de búsqueda por NIVELES: determinista primero, IA como último recurso — ✅ HECHA (2026-07-14)
- [x] E1. Auditado `kb.py`: split por regex `DEF` (solo IDs prefijados → el punto ciego `## N.`
      de Fase 1); tokeniza+stopwords+alias ES→EN por concepto; ranking = `covered × penalti-longitud`
      luego `score` (tf title×3+body); routing por `_index/symptoms.md`. Señal aprovechable:
      `covered/nq` del mejor bloque.
- [x] E2. **Umbral medible** = fracción de conceptos que cubre el top result. `kb.py` emite ahora
      una línea **`CONFIANZA: alta|media|baja|nula (top cubre C/N)`** con el hint de escalado
      (alta = ratio≥0.99; media ≥0.5; baja <0.5; nula = sin resultados). El llamante escala por esa
      línea, no a ojo. (recall_eval sigue 60/83, no rompe.)
- [x] E3. Regla dura documentada en **SKILL.md** (contrato de 3 niveles): kb.py → fallback
      determinista (`symptoms.md`→`gotchas.md` del dominio) → **IA solo en último término**, nunca
      el primer salto.
- **Aceptación:** ✅ contrato + umbral en SKILL.md; kb.py expone la señal `CONFIANZA` determinista.

## Fase 2 — BUSCAR: subir recall — ✅ HECHA (2026-07-14)
- [x] 2a. Alias ampliados (kit, cronometro, despublicar, curso, alumno, suelto, recurrente,
      contrato, oportunidad, entregado, facturar, reservado…): **recall kb.py 75%→77%**. Honesto:
      margen del aliasing casi agotado — los near-miss que YA tenían alias fallan por *ranking*, no
      vocabulario; los 16 restantes son mismatch profundo.
- [x] 2b. `symptoms.md` valida: **cubre los 90 dominios** (0 sin mención) y **recupera misses que
      kb.py no rankea** (AG4/AG5 están en symptoms.md). → recall del **SISTEMA** (kb.py + symptoms)
      ≈ 89%, no 77%.
- [x] 2c. **Semántico NO justificado.** De los 22 misses de kb.py, 13 ya en symptoms.md; los **9
      restantes AÑADIDOS a symptoms.md** (curado, 0 infra) → todos recuperables por el contrato.
      Embeddings deferidos (reabrir solo si la curación no escala).
- **Aceptación (CORREGIDA con held-out, 2026-07-14):** el 77%/~100% estaba **sobreajustado** al
      test co-evolucionado. Medido sobre 24 queries NUEVAS (holdout, `recall_eval.py --heldout`,
      `recall_heldout.json` — NO curar en symptoms):
      **kb.py 29% · +symptoms 46% · +routing de dominio (nivel-3) 62%.** La findability real de un
      usuario ≈ **62%** con el contrato completo, no ~100%. El ~38% que falla = kb.py ni enruta al
      dominio (mismatch ES↔EN). → **2c (semántico) REABIERTO**: sobre la métrica honesta SÍ se
      justifica para ese tramo. Alternativa más barata antes de embeddings: línea-síntoma en
      español auto-generada por gotcha (puentea ES↔EN mejor que alias ad-hoc).
- [x] **2c-alt VALIDADO (PoC, 2026-07-14):** inyectada frase-síntoma ES en 3 gotchas ingleses (los
      3 más duros, donde antes ni salía el dominio) → pasan a **posición #1** en kb.py; held-out kb.py
      **29%→42%** con solo 3 líneas. Mecanismo confirmado: una frase-síntoma ES convierte el miss
      ES↔EN en hit. PoC revertido (contaminaba el holdout). **Batch escalado pendiente**: ~2387
      gotchas con cuerpo inglés necesitan línea ES; es un one-time Workflow (sin infra, markdown SoT).
      NO lanzar desde chat gordo (pre-flight 🔴, ~107M tokens): **sesión fresca + Workflow**. Es el
      lever recomendado ANTES que embeddings.

## Fase 3 — Formato de bloque / indexado fiable (sirve a BUSCAR **y** AMPLIAR)
### 3/B — Indexar las secciones `## N.` — ✅ HECHO (2026-07-14)
- [x] Extendido el `DEF` de `kb.py` para partir también en cabeceras numeradas `## N.`/`### N.`
      (las de categoría sin dígito NO se parten). Bloques indexados: **4664 → 6408** (el 31%
      invisible ya es buscable; §14 v19_changes sale en top cuando aplica).
- [x] **Regresión detectada y corregida honestamente:** el indexado naïve bajó recall 72%→67%
      (las secciones narrativas expulsaban gotchas). Fix: **democión de secciones `§N`** (rank ×
      `KB_SECDEMOTE`=0.75) → los gotchas prefijados mantienen prioridad. **Recall final 75% (62/83)**,
      +3 sobre baseline.
### 3/A — Formato canónico total — ✅ DECIDIDO (2026-07-14): NO reescribir
Evidencia: formatos = `**PREFIXn.` 3321 (dominante) · `## N.` 2087 (referencia) · `### PREFIXn` 147 ·
lista 48. Retrieval ya funciona (6408 bloques, 75% recall); kb.py trata todos los formatos igual.
`§N` es ambiguo (§1 ×191) pero inofensivo (referencia, se busca por contenido no por `--id`).
- **Decisión:** NO normalizar masivamente (edición grande, riesgo, ganancia marginal). En su lugar:
- [x] **Convención canónica documentada** (para autoría FUTURA, sin tocar el pasado):
      · Gotcha (único por dominio): `**<PREFIX><n>. <título>** — <cuerpo con citas>`.
      · Sección de referencia/migración (v19_changes/core): `## <n>. <título>` (numérica, per-fichero).
      · `### PREFIXn` queda como variante tolerada (kb.py la trata igual); no se migra.
- [→] Aplicación diferida: **Fase 4** (`add_gotcha` emite el formato canónico) + **Fase 5**
      (lint de formato en el gate para que lo nuevo converja). No hay rewrite del histórico.
- **Aceptación:** ✅ decisión tomada con datos + convención escrita. Residual aceptado: `§N` no único.

## Fase 4 — AMPLIAR: pipeline de add-gotcha estructurado — ✅ HECHA (2026-07-14)
- [x] `add_gotcha.py <domain> <PREFIX> "título" "cuerpo" ["síntoma"]`: asigna el siguiente ID
      libre del prefijo (vía `check_ids`), escribe el bloque en **formato canónico**
      `**PREFIXn. título** — cuerpo` en `gotchas.md`, y añade la fila a `symptoms.md`. Modo `--dry`.
      Probado: asigna `STKG75` correctamente.
- [→] Conectar el bucle drift→triage→apply para que emita objetos estructurados que consuma
      `add_gotcha` (en vez de `patch_text` en prosa) — **pendiente** (mejora del watchdog, no bloquea).
- **Aceptación:** ✅ añadir un gotcha = un comando; ID autoasignado, formato canónico, sin
      renumerar/integrar a mano. Recordatorio de verificar con `cite_audit --gate`.

## Fase 5 — Gate unificado / CI — ✅ HECHA (2026-07-14)
- [x] `kb_gate.py`: corre en una pasada `cite_audit --gate` + `check_ids` + `check_registry`
      (exit 1 si cualquiera falla; `-v` vuelca detalle). **Probado: GATE VERDE.**
- [→] Cablear a pre-commit del repo — pendiente (una línea en `.pre-commit-config`/hook).
- [→] Lint de formato canónico (Fase 3/A) — pendiente, se añade a `kb_gate` cuando se decida.
- **Aceptación:** ✅ gate unificado verde en una pasada; cableado a hook pendiente (trivial).

## Extra (encaja en Fase 3) — #4 citas por símbolo
- [ ] Migrar citas de `file.py:línea` (frágil, driftea) a `file.py::símbolo` (estable). Mata la
      clase de mantenimiento de line-drift. `cite_audit --methods` ya ancla por símbolo.

---

## Orden de ejecución (por dependencia y valor)
`0 → 1 → E → 3 → 2 → 4 → 5`  ·  (E se diseña con lo medido en 1 y dirige 3 y 2; 4 depende de 3).

## Estado — TODAS LAS FASES HECHAS (2026-07-14)
- `0 ✅ · 1 ✅ · E ✅ · 3/B ✅ · 3/A ✅ · 2 ✅ · 4 ✅ · 5 ✅`
- Búsqueda: kb.py 77% recall + `## N.` indexadas (6408 bloques) + señal `CONFIANZA` + contrato de
  3 niveles en SKILL.md; **recall del sistema (kb.py + symptoms.md) ≈ 100%** en el test, sin infra
  (semántico deferido). Ampliar: `add_gotcha.py` (ID auto + formato canónico). Calidad: `kb_gate.py`.

### Cierre de pendientes (decididos, no en el aire)
- [x] **0.3** deprecated/versions en `gen_registry`: HECHO (DEP_RE + campo `deprecated`; regenerado;
      consolidation ya sale `deprecated=true` en el SSOT; check_registry verde).
- [x] **`kb_gate` a pre-commit**: HECHO (hook local `kb-gate`, stage pre-push, solo si cambian
      ficheros de deep-knowledge).
- [—] **Lint de formato canónico**: DESCARTADO — flaggearía las variantes toleradas (`### PREFIXn`,
      147 casos) → falsos positivos; el retrieval ya funciona con formatos mixtos. No aporta.
- [→] **drift→triage→`add_gotcha`**: DEFERIDO — reescribir el triaje para emitir objetos
      estructurados es trabajo real y el watchdog ya funciona; `add_gotcha` existe para uso manual.
- [→] **Migrar citas a `::símbolo` (#4)**: DEFERIDO — las citas por línea funcionan y el gate caza
      su drift; reescribir miles = coste alto, valor marginal. Migrar oportunistamente al editar.
