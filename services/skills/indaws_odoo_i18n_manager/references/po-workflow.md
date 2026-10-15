# PO Workflow — Exportar, Editar e Importar Traducciones Odoo

## Estructura `i18n/`

```
my_module/
└── i18n/
    ├── my_module.pot   # Plantilla base (todos los strings, sin traducir)
    ├── es.po           # Español — lo cargan es_ES, es_MX, es_AR...
    ├── fr.po           # Francés — lo cargan fr_FR, fr_BE, fr_CH...
    ├── es_419.po       # Solo si el español de LatAm debe decir algo distinto
    └── ...
```

**El nombre del `.po` NO es el código de idioma completo: es el que decida `get_base_langs`.**
Odoo no busca un fichero, busca **una lista** — `get_po_paths(module, lang)` itera
`get_base_langs(lang)` y carga TODOS los que existan
(`odoo19/odoo/tools/translate.py:1837-1843`, `get_po_paths`, sobre `odoo19/odoo/tools/translate.py:1823-1834`, `get_base_langs` (nuevo en 19.0), verificado en 19.0):

| idioma de la BD | ficheros que carga, en ese orden |
| --- | --- |
| `es_ES` | `es.po`, `es_ES.po` |
| `es_MX` | `es.po`, `es_419.po`, `es_MX.po` |
| `fr_BE` | `fr.po`, `fr_BE.po` |
| `nl_BE` | `nl.po`, `nl_BE.po` |
| `zh_HK` | `zh.po`, `zh_TW.po`, `zh_HK.po` |

Consecuencia práctica: **el genérico cubre todas las variantes y la variante solo cubre la suya.**
Nombrar el fichero `es_ES.po` no es "más correcto", es **menos cobertura** — deja fuera a México y
Argentina sin ganar nada. Usa la variante solo cuando ese país deba decir algo distinto, y entonces
va **encima** del genérico, con las cadenas que difieren y nada más.

**Y es lo que hace el propio Odoo:** `odoo i18n export -l es_ES` escribe `i18n/es.po`, no
`es_ES.po`. Si tu fichero se llama distinto que el que exporta Odoo, el siguiente export creará un
segundo fichero al lado y tendrás dos.

> **Ojo con `es` a secas:** en 19.0 **no existe ningún idioma con `code = es`**. La fila
> "Spanish / Español" es `code = es_ES` con `url_code = es` (`base/data/res.lang.csv`). Pasar
> `--load-language=es` a una BD nueva **revienta** con `duplicate key ... res_lang_url_code_uniq`.
> El fichero se llama `es.po`; el idioma que se instala es `es_ES`.

## Exportar desde CLI

**v19 — subcomando.** `--i18n-export` ya no existe (GOTCHA 14). El subcomando **escribe dentro del
`i18n/` de cada módulo** y elige el nombre del fichero por `get_base_langs`, así que exportando
`es_ES` obtienes `i18n/es.po`:

```bash
# Varios módulos de una vez, cada uno a su propio i18n/
odoo-bin i18n export -c <conf> -d <db> mod_a mod_b mod_c -l es_ES

# Plantilla .pot a un fichero concreto (-o: un solo idioma, un solo fichero)
odoo-bin i18n export -c <conf> -d <db> mod_a -l pot -o mod_a.pot
```

> **Los módulos van ANTES de `-l`.** `-l` es `nargs='+'`: si lo pones primero, argparse se come la
> lista de módulos como si fueran idiomas y falla con
> `the following arguments are required: MODULE`. Medido: tres intentos hasta verlo.

> **`-c <conf>` no es opcional aquí.** El subcomando no acepta `--db_host`/`--db_user`: la conexión
> sale del fichero de configuración. Sin él va a localhost y no encuentra la BD.

**v17/v18 — flag.**

```bash
# Plantilla base (sin idioma)
python odoo-bin --i18n-export=my_module/i18n/my_module.pot \
  --modules=my_module --i18n-overwrite -d <db> --stop-after-init 2>&1 | tail -20

# Con idioma (genera .po con los msgstr que Odoo ya resuelve del catálogo estándar)
python odoo-bin --i18n-export=my_module/i18n/es.po \
  --language=es_ES --modules=my_module -d <db> --stop-after-init 2>&1 | tail -20
```

## Exportar desde UI

Ajustes → Traducciones → Exportar traducción:
- Idioma: el deseado (o "Nuevo idioma / vacío" para `.pot`)
- Módulos: seleccionar el módulo
- Formato: `.po` o `.pot`

## Estructura de una entrada `.po`

```po
#. module: my_module
#: model:ir.model.fields,field_description:my_module.field_sale_order__x_notes
#, python-format
msgid "Notes for %s"
msgstr "Notas para %s"
```

- `#.` — comentario del extractor (módulo)
- `#:` — referencia (modelo, campo o archivo fuente)
- `#, python-format` — indica que hay placeholders `%s`/`%(key)s`
- `#, fuzzy` — entrada desactualizada/no verificada → **NO se importa**, resolver primero

## Entradas `fuzzy`

Las entradas marcadas `#, fuzzy` son ignoradas en la importación. Causas:
- El `msgid` cambió tras una actualización del módulo
- Poedit marcó la entrada como "necesita revisión"

**Solución:** Revisar el `msgstr`, eliminar la línea `#, fuzzy`, guardar.

## `noupdate` y traducciones de datos

Si un registro XML tiene `noupdate="1"`, sus traducciones no se actualizan al hacer upgrade. Para forzar la actualización:
1. Cambiar temporalmente a `noupdate="0"`, hacer upgrade, revertir
2. O importar manualmente el `.po` desde UI (anula `noupdate`)

## Importar `.po` desde CLI

```bash
# v19
odoo-bin i18n import -c <conf> -d <db> my_module -l es_ES

# v17/v18
python odoo-bin --i18n-import=my_module/i18n/es.po \
  --language=es_ES --modules=my_module -d <db> --stop-after-init 2>&1 | tail -20
```

**El import de verdad es `-u`.** Un `.po` que ya vive en `i18n/` del módulo lo carga Odoo al
actualizar el módulo, sin más ceremonia — y ese es el camino que hay que usar para *verificar* que
la traducción entra: `-u <módulos>` y luego leer un campo con `env(context={'lang': ...})`. Si
`msgfmt` pasa pero el `-u` no cambia nada, el fichero está bien escrito y mal nombrado.

## Importar desde UI

Ajustes → Traducciones → Importar traducción:
- Idioma: seleccionar o crear
- Archivo: subir el `.po`
- ¿Sobrescribir traducciones existentes? → marcar si se está actualizando

## `load_translations` vía RPC

**v17** — método en `ir.translation`:
```
model: ir.translation
method: load_translations
args: [{'lang': 'es_ES', 'modules': ['my_module']}]
```

**v18** — el modelo `ir.translation` fue refactorizado a `ir.module.module`:
```
# Alternativa segura en v18: reinstalar/actualizar el módulo con el idioma activo
model: ir.module.module
method: button_immediate_upgrade
# Con contexto lang=es_ES activo
```

Consultar `odoo-version-api-reference` para la API exacta según versión.

## Override de términos de módulos base

Para sobrescribir una traducción de un módulo Odoo/OCA sin tocar su `.po`:

1. Crear archivo `i18n/es_ES.po` en **tu módulo**
2. Añadir la entrada con el mismo `msgid` que el original y el `msgstr` deseado
3. El módulo propio debe cargarse DESPUÉS del original (`depends = ['base']`)

```po
# Override de término de base
msgid "Quotation"
msgstr "Presupuesto"   # en lugar de "Cotización" que usa base
```

## Traducción en lote (multi-módulo) — sin fundir tokens

Traducir varios módulos "a la vez" pasando cada `.po` entero al LLM (o un agente por módulo)
quema tokens ×N: cada `.po` tiene cientos de entradas y los `msgid` se **repiten muchísimo**
entre módulos (`Company`, `Reference`, `Date`, labels de campo). La extracción es mecánica (CLI);
lo caro es sólo traducir los `msgstr` nuevos. El motor `scripts/translate_batch.py` lo acota:

1. **Exporta** los `.pot`/`.po` por CLI (sección arriba). Multi-módulo: repite el CLI por módulo — nunca por LLM.
2. **Extrae la lista única pendiente**, reusando como memoria de traducción lo que Odoo/OCA ya tradujeron:
   ```bash
   python3 scripts/translate_batch.py extract --lang es_ES \
     --modules mod_a mod_b mod_c \
     --tm $ODOO_SRC/odoo18/addons/*/i18n/es_ES.po $OCA_SRC/v18/*/i18n/es_ES.po --md
   ```
   Devuelve `to_translate` (los strings ÚNICOS que faltan) + `reuse` (resueltos automáticamente
   desde la TM, **0 LLM**) + `stats` con el factor de ahorro. Ignora `fuzzy` y lo ya traducido;
   las entradas con `msgid_plural` se dejan aparte (revisar a mano).
3. **Traduce sólo `to_translate`** y escribe `map.json = {msgid: msgstr}` (fusiónalo con `reuse`).
   Conserva los placeholders (`%s`, `%(name)s`) exactamente igual que en el `msgid`.
4. **Rellena** cada `.po`:
   ```bash
   python3 scripts/translate_batch.py apply --lang es_ES --modules mod_a mod_b mod_c --map map.json
   ```
   Idempotente (re-ejecutar = 0 cambios), respeta `fuzzy` y lo ya traducido, reporta `filled`/`still_empty` por módulo.

### Con qué obtener las traducciones (mejor → peor)

1. **TM oficial de Odoo/OCA (`--tm`)** — lo primero SIEMPRE. Los `.po` que Odoo/OCA ya envían son
   la traducción canónica; reusarlos sale gratis y deja la terminología **consistente con el resto
   de la UI**. En Odoo el `msgid` es inglés (idioma fuente), así que para inglés muchos términos
   estándar no se traducen: se toman del propio fuente / del `en_US.po` de base.
2. **El propio LLM (Claude)** para los strings **propios del módulo** — domina la jerga ERP/contable
   y preserva los placeholders. Ya va acotado a la lista única deduplicada, así que es barato.
3. **DeepL / Google Translate API** sólo si el volumen de strings propios es enorme y prima el coste:
   requiere **glosario** de términos Odoo y un *guard* que verifique que no se pierden los `%s`; suele
   quedar peor en jerga contable. Para unos cientos de strings no compensa frente al LLM.
4. **Poedit** (con su TM y "pretraducir") para el **repaso humano** final antes de cerrar — no para el pipeline.

El pipeline es agnóstico del motor: `extract` da la lista, cualquier fuente rellena `map.json`, `apply` lo escribe.

## Sincronización con Poedit

1. Abrir el `.po` existente en Poedit
2. Catálogo → Actualizar desde código (si se tiene el fuente) o cargar el `.pot` nuevo
3. Resolver entradas `fuzzy` y nuevas entradas sin traducir
4. Guardar → sube el `.po` actualizado al módulo
