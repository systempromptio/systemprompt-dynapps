---
name: indaws-odoo-i18n-manager
version: 0.4.2
effort: medium
model: sonnet
description: >-
  Traduce modulos Odoo v17-v19: exportar e importar .po/.pot, strings sin traducir en Python/JS/OWL, overrides e instalar idiomas. Usala cuando se hable de traducir, traduccion o idiomas. Incluye el CLI de export por versión (flag v17/v18, `odoo i18n` v19), qué extrae un template OWL, traducciones que no aparecen y load_translations.
allowed-tools: [Read, Write, Edit, Bash, Agent]
---

# Odoo i18n Manager

Ciclo completo de internacionalización de módulos Odoo v17-v19: strings traducibles en código (Python, JS, templates OWL) + exportación, traducción e importación de `.po`.

Conexión: `references/shared/odoo-connection.md`

## Workflow

### 1. Diagnosticar necesidad

Preguntar: **versión Odoo** (v17/v18/v19 — cambia `env._()` **y** el CLI de export), **tarea** (exportar `.pot`, importar `.po`, arreglar código, JS, instalar idioma, override), **módulo(s)** y ruta, **idioma(s)** objetivo.

### 2. Escanear código + .po — MOTOR determinista [script, ~0 tokens LLM]

`python3 scripts/scan_i18n.py --module <path> --target-version v17|v18|v19 [--md]` → JSON con **issues Python** (f-string o `.format()` dentro de `_()`, `env._()` CRÍTICO si target v17, uso de `_lt`), **issues de templates OWL** (texto dentro de un `t-att-*` → NUNCA se extrae; frases partidas por un `<t t-esc/>`) y **análisis de cada `.po`/`.pot`** (nombre válido, entradas, sin traducir, fuzzy) + bloque `confidence`. NO grep a mano.

**Cobertura real** (lo que un "todas traducidas" no ve): exporta el `.pot` de verdad y pásalo con `--pot f.pot [--lang es]` → `missing` / `stale` / `reference_gap`. Ver `references/coverage-and-guardrails.md`.

El check de OWL solo mira los 9 atributos que muestran TEXTO y solo marca literales que parecen frase (`t-att-class="'fa-times'"` no es traducible; marcarlo sería ruido). Validado contra un fichero real pre-arreglo (3 CRÍTICOS + 1 aviso) y el mismo corregido (0).

**Fallback (LLM):** si el script no corre, escanear a mano con `references/translation-mechanics.md` + `references/po-workflow.md`.

Corregir los CRÍTICOS: f-strings → `_("..%s") % var`; `env._()` solo v18+; `_lt` en atributos de clase.

### 3. Exportar términos — generar `.pot` / `.po`

Cargar `references/po-workflow.md`. Extracción 100% mecánica. **CLI** (recomendado):
```bash
# v17 / v18
python odoo-bin --i18n-export=<módulo>.pot --modules=<módulo> --i18n-overwrite -d <db> --stop-after-init 2>&1 | tail -20
# v19 — la flag desapareció; es un subcomando (también import / loadlang)
odoo i18n export -c <conf> -d <db> -l pot -o <módulo>.pot <módulo>
```
**Exporta el `.pot` REAL antes de escribir un `.po` a mano**: es la única forma fiable de saber qué `msgid` produce tu código (los partidos, los unidos, y los que **no** genera — todo lo que viva en un `t-att-*`). Escribirlo "de memoria" produce entradas que no casan con nada. **UI**: Ajustes → Traducciones → Exportar traducción.

### 3b. Traducir en lote (multi-módulo) — MOTOR determinista [dedup + reuso]

Pasar `.po` enteros al LLM quema tokens ×N. `scripts/translate_batch.py`: `extract` da la unión ÚNICA deduplicada de lo pendiente + reuso desde la TM oficial (`--tm` con los `.po` de Odoo/OCA → 0 LLM); el LLM traduce SOLO lo único → `map.json`; `apply` rellena cada `.po` (idempotente, respeta `fuzzy`). Orden completo y "con qué traducir" en `references/po-workflow.md` §"Traducción en lote".

### 4. Editar / importar `.po`

`references/po-workflow.md` para `msgid`/`msgstr`, `fuzzy`, `noupdate` y Poedit.
```bash
python odoo-bin --i18n-import=<path>.po --language=es_ES \
  --modules=<módulo> -d <db> --stop-after-init 2>&1 | tail -20
```
**UI**: Ajustes → Traducciones → Importar traducción.

### 5. Traducciones web / JavaScript

Cargar `references/js-web-translations.md`. Verificar que los strings JS usan `_t(...)` y que los
`.po` los incluyen; si faltan, re-exportar tras añadir el módulo JS al manifest.

### 6. Instalar idioma / load_translations vía API [delega a odoo-pilot]

**[DELEGA]** `odoo-pilot` → `execute_method.sh` sobre `base.language.install` (`lang=es_ES`); recarga
con `ir.translation` → `load_translations` (v17) o `ir.module.module` →
`button_immediate_upgrade` con contexto de idioma (v18). Ver `references/po-workflow.md`.

### 7. Verificar

1. `.po` en `i18n/` con código válido (`es.po` o `es_ES.po`, ambos valen — GOTCHA 4)
2. Sin `fuzzy` sin resolver
3. Forzar recarga: modo desarrollador → Traducciones → Actualizar términos
4. Panel/JS: `fetch('/web/webclient/translations?mods=<módulo>')` y buscar el `msgid`. Si no está,
   el problema es la EXTRACCIÓN (típicamente un `t-att-*`), no la traducción

**Guardarraíl obligatorio** — test que prohíba castellano en la fuente, exija `.po` sano y compruebe
que cada referencia `code:` sigue viva. Los 3 asserts, con el detalle de por qué el check de tildes
solo no basta: `references/coverage-and-guardrails.md`.

## GOTCHAS

1. **f-strings dentro de `_()` rompen la extracción** — `_("Hola %s") % name` correcto; `_(f"Hola {name}")` no extrae nada: el `.pot` queda vacío para ese string.
2. **`env._()` solo existe en v18+** — en v17, `_()` de `odoo.tools.translate`. Mezclarlos → `AttributeError` en v17.
3. **`_lt` es lazy, no evalúa en import** — correcto en atributos de clase y `fields.Selection`; incorrecto donde se necesita el string ya evaluado (p.ej. `raise UserError`).
4. **`es.po` SÍ se carga para `es_ES`** (verificado contra fuente) — `get_po_paths()` (`odoo/tools/translate.py`) recorre `get_base_langs(lang)` = `["es","es_ES"]` en `i18n/` **y** `i18n_extra/`. Para LATAM (`es_MX`, `es_AR`…) además busca **`es_419.po`**. Lo que se ignora es un nombre que no sea código de idioma (`Spanish.po`).
5. **`noupdate=True` en datos XML bloquea reimportación** — esas traducciones no se actualizan al upgrade; quitar la flag o importar a mano.
6. **Strings en `default=` no se traducen en runtime** — usar `default=lambda self: _("texto")`.
7. **Traducciones JS no aparecen sin `web` en depends** — el bundle JS solo incluye `.po` de módulos cargados en el cliente.
8. **`fuzzy` bloquea la importación** — las entradas `#, fuzzy` no se importan.
9. **`load_translations` cambió en v18** — en v17 método de `ir.translation`; en v18 el modelo se refactorizó. Consultar `odoo-version-api-reference` antes de llamarlo por RPC.
10. **Override de término de módulo base** — registro `ir.translation` con `module=<tu_módulo>` o `.po` con el mismo `msgid`. Nunca editar el `.po` del módulo base (se sobreescribe).
11. **Un valor `t-att-*` es JS: sus literales NUNCA se extraen** — de un template OWL solo salen nodos de texto y **9 atributos** (`OWL_TRANSLATED_ATTRS`: `title`, `aria-label`, `placeholder`, `alt`, `label`, `data-tooltip`, `aria-placeholder`, `aria-roledescription`, `aria-valuetext`). Un ternario en `t-att-title` se queda sin traducir **en silencio**: muévelo al componente como getter con `_t`. Detalle en `references/js-web-translations.md`.
12. **Frase partida alrededor de un `<t t-esc/>` se extrae en trozos** — dos `msgid` que ningún traductor recompone (y el orden de palabras cambia por idioma). Un solo término: getter con `_t("Primera fila · %s más", n)`.
13. **Tocar el texto FUENTE sin tocar el catálogo es una REGRESIÓN** — (a) barrer la fuente a inglés sin entregar `i18n/es.po` en el mismo commit deja en inglés una UI que ya se veía en castellano; (b) **reescribir un `msgid` ya traducido borra su traducción en silencio**: el `msgid` ES la clave, así que cambiar una palabra del literal crea una entrada nueva sin `msgstr`, deja la vieja huérfana, y **ningún test lo caza**. Mejora un mensaje ya traducido por el **payload** (`%(campo)s`), no por el literal. Cicatriz y qué hacer si hay que cambiarlo: `references/coverage-and-guardrails.md`.
14. **En v19 `--i18n-export` ya no existe** — subcomando `odoo i18n export|import|loadlang`; en v17/v18 sigue la flag. Tres trampas medidas: los **módulos van ANTES de `-l`**, la conexión sale **solo de `-c <conf>`**, y **escribe dentro del `i18n/` del módulo** (si está montado read-only, no exporta nada). Detalle en `references/po-workflow.md`. **Y el nombre del `.po` no es el código de idioma: lo decide `get_base_langs`** — `es_ES` carga `es.po` *y* `es_ES.po`, así que el genérico cubre todas las variantes y llamarlo `es_ES.po` es MENOS cobertura; Odoo mismo exporta `es.po` con `-l es_ES`. En 19.0 no existe idioma con `code = es`.
15. **"Todas las entradas traducidas" NO es cobertura** — 124 entradas con `msgstr` pueden dejar 75 de 199 términos en inglés. El único check válido es el diff contra el `.pot` exportado: ver `references/coverage-and-guardrails.md`.
16. **Editar un `.po` a mano falla en silencio de dos maneras** — (a) una traducción llega al registro solo por su referencia `model:`: con solo `#: code:...xml:0`, `_load_module_terms` no la aplica, así que sincroniza `occurrences` con el `.pot` (`polib`), no solo los `msgid`; (b) a una entrada de código sin el comentario `#. odoo-python` (o `#. odoo-javascript`) **el cargador la descarta entera** — ni error, ni warning, ni test en rojo, la UI simplemente sigue en inglés. Por eso las de código no se teclean: se exportan (GOTCHA 14) y se mergea sobre lo exportado. Citas, cicatriz y la caché en proceso que obliga a reiniciar, en `references/coverage-and-guardrails.md`.
17. **El extractor JS/Python UNE literales partidos** — `_t("una frase " + "continuada")` y `_("adyacentes" "en Python")` son UN `msgid` (`message_buffer` de `odoo/tools/babel/javascript_extractor.py`). Obliga a normalizar el empalme en cualquier check de "el msgid sigue en el fichero".

## NEVER

- f-strings dentro de `_()` — siempre `_("texto %s") % var`
- Editar `.po` a mano: ni entradas de código (GOTCHA 16 — exporta y mergea), ni los de módulos base o enterprise (se pierden en upgrade)
- Llamar `env._()` en código que debe funcionar en v17
- Importar traducciones en producción sin probar en staging
- Dejar entradas `fuzzy` antes de importar

## ALWAYS

- Verificar la versión de Odoo antes de usar `env._()` vs `_()`
- Colocar el `.po` / `.pot` en `<módulo>/i18n/<lang>.po`
- Re-exportar `.pot` tras añadir nuevos strings
- Consultar `odoo-version-api-reference` para cambios de API v16→v18
- **Provenance marker (ALWAYS):** every generated file (except `.csv`/`.json`) ends with `Made with odoo-i18n-manager · v<version del frontmatter de esta skill>` in the file-type comment (`.po`/`.pot` → `#`), idempotent. Full rule: `references/shared/skill-provenance-marker.md`.

## Delegates to

| Skill | Suite | Cuándo |
|-------|-------|--------|
| odoo-version-api-reference | common | API de traducción v16→v18, `env._()`, `load_translations` |
| odoo-pilot | common | Instalar idioma y ejecutar `load_translations` vía API |

## Consumed by

| Skill | Suite | Para qué |
|-------|-------|----------|
| odoo-module-creator | developer | Scaffold i18n inicial en módulos nuevos |
| odoo-module-upgrader | developer | Actualizar traducciones tras migración |
| git-review-pr | developer | Criterio de catálogos al revisar un PR: cadena nueva o `msgid` reescrito |

## REFERENCES

| Archivo | Contenido | Cargar cuando |
|---------|-----------|---------------|
| [scripts/scan_i18n.py](scripts/scan_i18n.py) | **Motor** — issues Python + texto no extraíble en OWL + análisis `.po`/`.pot` + `confidence` | Paso 2 |
| [scripts/translate_batch.py](scripts/translate_batch.py) | **Motor** — `extract` (unión deduplicada + reuso TM) / `apply` (rellena `msgstr`, idempotente) | Paso 3b |
| [references/translation-mechanics.md](references/translation-mechanics.md) | `_()`, `_lt`, `env._()` v18+, placeholders, `translate=True`, contextos | Arreglar Python (y fallback) |
| [references/po-workflow.md](references/po-workflow.md) | Export/import CLI y UI, `i18n/`, `noupdate`, `fuzzy`, `load_translations`, traducción en lote | Exportar/importar `.po` |
| [references/js-web-translations.md](references/js-web-translations.md) | `_t` en OWL, qué extrae un template (texto + 9 atributos) y por qué no un `t-att-*`, frases partidas, literales unidos, qué recibe el navegador | Traducciones JS/OWL |
| [references/coverage-and-guardrails.md](references/coverage-and-guardrails.md) | Diff contra `.pot` (`missing`/`stale`/`reference_gap`), sincronizar `occurrences`, los 3 asserts del test guardarraíl | Cobertura real · test anti-regresión |
