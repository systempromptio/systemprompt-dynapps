# Plan: preparar deep-knowledge para Odoo v20 (bump rápido)

> Estado: **DISEÑADO, sin ejecutar** (2026-06-30). Disparador de ejecución: anuncio/RC de Odoo 20.
> Pre-requisito conceptual: `references/_index/VERSION-BUMP.md` + `HOW-TO-EXTEND.md`.
> Objetivo: que añadir v20 al KB cueste **proporcional al diff**, no al tamaño del KB.

## Principio (ya asentado)

Las citas están ancladas a símbolo (`fichero.py::método`). Cambiar de versión = preguntar
*"¿este símbolo sigue existiendo en v20?"*. No se relee el KB; se procesa el diff. El motor
de la mitad A (`version_audit.py`) ya implementa esto.

## El bump tiene DOS mitades asimétricas

| Mitad | Qué | ¿Automatizado hoy? |
|---|---|---|
| **A — Rotura** | símbolos citados que desaparecieron/renombraron/movieron | **Sí** → `version_audit.py --to v20` |
| **B — Novedad** | modelos/métodos/features que v20 **añade** y el KB no menciona | **No** → punto ciego estructural |

El motor de diff por símbolo solo ve lo que se cayó: no puede ver una feature nueva porque no
hay cita previa contra la que diffear. La mitad B es hoy manual = la parte lenta. "Rápido" =
hacer baratas **las dos** mitades.

---

## Piezas a construir

### Pieza 1 — `scripts/version_inventory_diff.py` (motor de la mitad B) · LA GRANDE

- **Propósito:** qué añade/renombra v20, no solo qué se cayó.
- **Uso:** `version_inventory_diff.py --from v19 --to v20 [--models-only|--breaking-only]`
- **Entrada:** dos árboles de fuente (mismo mapeo vN→roots que `version_audit`), alcance =
  `odoo_modules` de `domains.json` (lo que el KB cubre, no todo Odoo).
- **Extrae por módulo:** modelos (`_name=`), campos (`x = fields.X(`), métodos (`def _?name(`).
- **Salida (markdown, por dominio, vía `registry.by_module()`):** modelos nuevos/eliminados;
  métodos/campos públicos nuevos en modelos ya documentados; candidatos a rename (heurística:
  símbolo caído + símbolo nuevo de nombre similar en mismo modelo). Cada línea atribuida al
  `vN_changes.md` que toca.
- **Contrato:** read-only, NO escribe, NO juzga (emite candidatos, no gotchas).
- **Riesgo de diseño = ruido.** Prioriza señal: (a) modelos nuevos + métodos públicos nuevos en
  modelos ya cubiertos, (b) breaking por encima de adiciones puras, (c) vistas
  `--models-only`/`--breaking-only`. La decisión "documentable vs trivial" sigue siendo humana.
- **Validación previa a v20:** correr con el par **v18↔v19 (ya en disco)**. Si reproduce los
  cambios v19 que el KB documenta a mano → validada para v20 a ciegas.

### Pieza 2 — `scripts/citation_lift.py` (la "pieza 4" pendiente) · ENDURECIMIENTO ÚNICO

- **Propósito:** convertir citas line-only (`fichero.py`, dos puntos, número) a `fichero.py::símbolo` (las
  line-only son invisibles al motor de diff; `--methods` solo ve símbolos).
- **Uso:** `citation_lift.py [DOM] [--apply]` (dry-run por defecto).
- **Lógica:** por cada cita line-only, localizar en fuente el `def`/campo que envuelve la línea
  NN; reusar el resolver de `cite_audit.py`. Ambiguas/no resueltas → reporte manual.
- **Cuándo:** AHORA (con v18/v19). Es one-time y abarata todos los bumps futuros.
- **Cierre de lazo:** añadir a `kb_release_check.py` un contador "citas line-only restantes"
  (reportado, tendiendo a 0).

### Pieza 3 — preflight de fuente · BARATO

- No es script nuevo: **flag `--check-roots`** en `version_audit.py` (y/o `version_inventory_diff`).
- Verifica que los roots v20 (CE/EE/OCA/indaws) existen y cuenta módulos vs esperados por root;
  aborta con lista de lo que falta. Evita work-lists falsos (un root ausente parece "todo eliminado").

### Pieza 4 — onda expansiva · CASI SIN CÓDIGO NUEVO

- El check cross-skill ya existe: `skill-designer/scripts/staleness.py --latest 20` lista qué
  skills declaran cobertura vN.
- Gap = **checklist**, no código: asegurar que `odoo-version-api-reference` (engine, delegado) y
  la chuleta v17→18→19 del `CLAUDE.md` global entran en "tocar en cada bump". → paso explícito
  en `VERSION-BUMP.md`.

---

## Pipeline objetivo (ediciones a `VERSION-BUMP.md` cuando se ejecute)

```
Pre-req (una vez, ahora):  citation_lift --apply              → cero line-only
Paso 0:   version_audit --to v20 --check-roots                → fuente presente
Paso 1a (mitad A, existe): version_audit --to v20             → símbolos caídos
Paso 1b (mitad B, NUEVO):  version_inventory_diff --from v19 --to v20  → novedades
Paso 2:   clasificar contra fuente → v20_changes.md  (rename/eliminado/movido | modelo/método nuevo)
Paso 3:   subir marcador verified:
Paso 4:   version_staleness --target v20  +  (skill-designer) staleness --latest 20
Paso 5:   re-indexar symptoms.md / cheatsheet.md
Paso 6:   kb_release_check  (+ contador line-only)
```

## Reparto y secuencia

| Cuándo | Qué | Por qué |
|---|---|---|
| **Ahora** (v18/v19 en disco) | Pieza 2 (citation_lift) + Pieza 3 (preflight) | endurecimiento; no dependen de v20; abaratan el bump real |
| **Ahora** (diseñar + validar) | Pieza 1 (inventory_diff), probada con v18↔v19 | pieza grande; validable hoy |
| **Al salir v20** | correr el pipeline completo | coste ∝ diff |
| **Coordinado** | `odoo-version-api-reference` (API/firmas) + deep-knowledge (comportamiento) | el bump es de DOS skills, no una |

## Invariante que NO se toca

Las tres tools son read-only o dry-run-by-default y **nunca afirman comportamiento** — solo
recortan el universo a verificar contra fuente. Toda afirmación v20 entra anclada a símbolo.
Release notes oficiales solo *siembran* la mitad B; jamás son autoridad (anti-alucinación).

## Checklist de ejecución

- [ ] Pieza 2 `citation_lift.py` + contador line-only en `kb_release_check` (ahora)
- [ ] Pieza 3 `--check-roots` en `version_audit.py` (ahora)
- [ ] Pieza 1 `version_inventory_diff.py` diseñada y **validada contra v18↔v19** (ahora)
- [ ] Fuente v20 provisionada en disco (al salir v20)
- [ ] Pipeline corrido + `v20_changes.md` por dominio (al salir v20)
- [ ] `odoo-version-api-reference` y chuleta `CLAUDE.md` actualizadas a v20 (coordinado)
- [ ] `kb_release_check` verde + muestra semántica v20 verificada
