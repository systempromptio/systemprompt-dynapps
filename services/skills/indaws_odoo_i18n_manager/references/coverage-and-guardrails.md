# Cobertura real de traducción y guardarraíles

Todo lo de aquí sale de casos reales: `.po` que pasan cualquier check de completitud y dejan media UI
en inglés.

## "Todas las entradas traducidas" NO es cobertura

Un `.po` con 124 entradas, **todas** con `msgstr`, pasa cualquier validador de completitud y deja
**75 términos de 199 en inglés**: etiquetas de campo, nombres de modelo, `help`, nombres de grupos,
datos XML y los `_()` añadidos después de la última exportación.

El único check válido es el **diff contra el `.pot` exportado de verdad**:

| Categoría | Qué es | Consecuencia si se ignora |
|---|---|---|
| `missing` | `msgid` del `.pot` que el `.po` no tiene | UI a medio traducir con el fichero "completo" |
| `stale` | `msgid` del `.po` que el `.pot` ya no produce | ruido; traducciones de código que ya no existe |
| `reference_gap` | entrada sin la referencia `model:...` que le toca | traducción presente, registro en inglés (ver abajo) |

Exportar el `.pot` real y pasarlo al motor:

```bash
# v19
odoo i18n export -c <conf> -d <db> -l pot -o f.pot <módulo>
# v17 / v18
python odoo-bin --i18n-export=f.pot --modules=<módulo> --i18n-overwrite -d <db> --stop-after-init

python3 scripts/scan_i18n.py --module <path> --target-version v19 --pot f.pot --lang es
```

Sin ese diff la UI se ve medio traducida y los tests pasan.

## Una traducción llega al registro solo por su referencia `model:`

Caso real: dos acciones rápidas seguían en inglés en el panel **teniendo el `msgid` traducido**. El
mismo literal existía también en una plantilla OWL, así que la entrada del `.po` había quedado con
**solo** `#: code:addons/...xml:0`. `_load_module_terms` aplica traducciones de registro por la
referencia `model:<modelo>,<campo>:<xmlid>`; sin ella, el campo `ai.quick.action,name` no se traduce
nunca.

El `.pot` real trae **ambas** referencias en la misma entrada. Por eso, tras editar un `.po` a mano:

```python
# sincronizar referencias con el .pot conservando lo traducido
import polib
pot = polib.pofile("f.pot"); po = polib.pofile("i18n/es.po")
by_id = {e.msgid: e for e in pot}
for e in po:
    src = by_id.get(e.msgid)
    if src:
        e.occurrences = src.occurrences   # <- lo que hace que se aplique al registro
        e.comment = src.comment
po.save()
```

Sincroniza **referencias**, no solo `msgid`.

## Guardarraíl: el test que evita la regresión silenciosa

Barrer la fuente a inglés y no publicar el `.po` en el mismo commit es una regresión (GOTCHA 13). Un
test lo convierte en rojo en vez de en una palabra en inglés en mitad de la UI. Tres asserts:

1. **Fuente sin castellano**: prohibir tildes/`ñ`/`¿¡` en los literales **y** una lista corta de
   palabras sin tilde (`documento`, `usuario`, `fichero`, `carpeta`, `factura`…). El
   `"[documento] %s"` que se cuela no tiene ni una tilde, así que el check de caracteres solo no lo
   pilla.
2. **`.po` sano**: todo `msgstr` no vacío, sin `msgid` duplicados, sin `fuzzy`.
3. **Referencias vivas**: cada `msgid` con referencia `code:` **sigue existiendo** en su fichero,
   normalizando espacios y el empalme de literales partidos (GOTCHA 17: el extractor UNE
   `"una frase " + "continuada"` en un solo `msgid`, y `_("adyacentes" "en Python")` también).

Reformular el texto de un botón sin tocar el `.po` pasa a ser un fallo de test.

## El comentario `#. odoo-python` no es cosmético: sin él no se carga nada (GOTCHA 16b)

Una entrada `.po` de código sin ese comentario **se descarta entera**, en silencio. El cargador
filtra por él:

```python
# odoo/tools/translate.py:1854-1856 @d1d161033
def _load_python_translations(self, module_name, lang):
    def filter_func(row):
        return row.get('value') and PYTHON_TRANSLATION_COMMENT in row['comments']
```

Las constantes están en `:54` y `:57` del mismo fichero: `PYTHON_TRANSLATION_COMMENT =
'odoo-python'` y `JAVASCRIPT_TRANSLATION_COMMENT = 'odoo-javascript'`.

Un `msgid`/`msgstr` perfectos, en el fichero correcto, con las `occurrences` correctas, **no
llegan a `_()`** si falta el comentario. No hay error, no hay warning y ningún test se pone en
rojo: la pantalla sigue en inglés y parece un problema de otra cosa (idioma del usuario, módulo
sin instalar, caché).

*Cicatriz real, 2026-08-13: cuatro módulos con `.po` escritos a mano, cero traducciones de Python
cargando. Se descubrió mirando la pantalla en `es_ES`, no con una herramienta.*

**Corolario operativo:** las entradas de código no se teclean nunca. Se exporta con la herramienta
(`odoo i18n export`, GOTCHA 14) y se mergea la traducción sobre lo exportado.

### Y aun con el `.po` bien, hay que reiniciar (cola del GOTCHA 16)

`python_translations` y `web_translations` se llenan **una vez por módulo+idioma** y viven en el
proceso. Reimportar el `.po` no cambia lo que ya se está sirviendo. Reinicia antes de concluir que
la traducción no funciona: si no, se corrige un fichero que ya estaba bien.

## Reescribir un `msgid` ya traducido: la traducción se va sin avisar (GOTCHA 13b)

El `msgid` es la clave de búsqueda del catálogo, no una descripción. Cambiar una palabra del
literal en el código **no edita** la entrada existente: crea otra, sin `msgstr`, y deja la vieja
huérfana en el `.po`. La UI de ese mensaje vuelve al idioma fuente, no falla nada, y **ningún test
lo detecta** salvo que alguno cite ese texto exacto — que es raro, porque los tests afirman sobre
el comportamiento, no sobre la cadena.

Qué hacer, por orden de preferencia:

1. **No tocar el literal.** Si lo que quieres es que el mensaje diga más, mételo en el payload:
   `%(documents)s` puede pasar de nombrar el documento a nombrar el documento **y** el registro
   sin que el `msgid` cambie ni una letra.
2. Si el literal tiene que cambiar de verdad: `.pot` regenerado (`odoo i18n export`, GOTCHA 14) y
   `msgstr` traducido **en el mismo commit**, más un barrido de entradas huérfanas.
3. Y en una revisión de PR: cadenas nuevas o cambiadas con **cero ficheros bajo `i18n/`** en el
   diff es un hallazgo (lo pide `git-manager` en su acción `review`).

*Medido el 2026-09-02 en `indawsodoo/indaws-edi#30`: un `UserError` se reescribió de "This record
already has an EDI document…" a "One or more of these records…" para cubrir el caso de varios
registros. El msgid viejo seguía traducido en `indaws_edi_base/i18n/es_ES.po` y el nuevo nacía
vacío: el error habría salido en inglés a todos los usuarios. Se revirtió el literal y la mejora
se quedó en el payload.*
