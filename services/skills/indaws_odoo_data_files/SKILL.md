---
name: indaws-odoo-data-files
version: 0.2.7
effort: medium
model: sonnet
description: >-
  Genera y revisa los ficheros XML/CSV de datos de un modulo Odoo: record, ir.cron, secuencias, external IDs, noupdate, data vs demo. Usala cuando toques datos de un modulo. Incluye ir.config_parameter, acciones, grupos, forcecreate y orden de carga en el manifest, y depurar «edité el XML y -u no lo actualiza».
allowed-tools: [Read, Write, Edit, Grep, Glob, Bash]
---

# Odoo Data Files

Construye la capa de datos declarativa de un modulo Odoo: ficheros XML/CSV cargados via el
manifest (`data`, `demo`) y la gestion de external IDs (`ir.model.data`). La fuente de verdad
del comportamiento es el loader `odoo/tools/convert.py`; toda afirmacion de esta skill esta
anclada ahi. NO inventes API de modelos: para signaturas de campos/metodos delega en
`odoo-version-api-reference`.

Estandares de codigo compartidos: `references/shared/coding-standards.md`.

## Workflow

1. Identifica el tipo de dato (record generico, registro de sistema como cron/sequence/param,
   menu, accion, plantilla, funcion, borrado) y su volatilidad (configuracion editable por el
   usuario vs estructura fija del modulo).
2. Decide `data` o `demo` en el manifest y si el bloque va con `noupdate`. Regla rapida: si el
   usuario puede editar el registro en runtime y NO quieres pisarlo en cada `-u`, va con
   `noupdate="1"`. Ver `references/external-ids-and-noupdate.md`.
3. Escribe el XML con la sintaxis correcta de `<record>`/`<field>` (`ref`/`eval`/`search`/
   inline/`type=`). Para la semantica exacta carga `references/record-syntax.md`.
4. Para registros de sistema (cron, sequence, config_parameter, mail.template, actions, groups)
   usa las plantillas y campos clave de `references/system-records.md`.
5. Asigna external IDs estables y descriptivos; usa `ref("modulo.xmlid")` y `%(xmlid)d` segun
   convenga. Nunca renombres un xml_id ya publicado sin un script de migracion.
6. Si son muchos registros homogeneos, evalua CSV (`references/csv-and-loading.md`).
7. Declara los ficheros en el manifest en orden de dependencia (un record que hace `ref` a otro
   debe cargarse despues). Delega el manifest/scaffolding en `odoo-module-creator`.
8. Verifica: el modulo instala y `-u` se comporta como esperas (lo editable persiste, lo
   estructural se reescribe).

## GOTCHAS

1. **noupdate solo protege en update, no en init.** En `_tag_record`, el skip por noupdate es
   `if self.noupdate and self.mode != 'init'` (**convert.py:344 en v18, :357 en v19**). En la instalacion inicial el
   registro SIEMPRE se crea/escribe; noupdate solo evita reescrituras en `-u` posteriores.
2. **"Lo edite en el XML y `-u` no lo actualiza":** el registro esta bajo `noupdate="1"`. Con
   noupdate, si el xml_id ya existe, convert.py hace `return None` y ni siquiera calcula los
   valores (convert.py:349-360 v18 / :362-373 v19). El XML editado se ignora en update.
3. **"Lo cambie en la UI y `-u` me lo pisa":** el registro NO esta bajo noupdate. Sin noupdate,
   `_load_records` reescribe los valores del XML en cada `-u` (`if not (update and d_noupdate)`
   en models.py:5497), descartando tu cambio manual.
4. **forcecreate por defecto es True.** Un record con noupdate que NO existe SE crea igualmente
   en update salvo `forcecreate="0"` (**convert.py:361 v18, :374 v19**: `elif not
   nodeattr2bool(rec, 'forcecreate', True)`). forcecreate="0" significa "si no existe, no lo crees".
   O sea: `noupdate="1"` **no es "no toques este registro"**, es "no lo ACTUALICES" — si falta, lo
   crea. El comentario de la propia fuente lo dice: *"if it doesn't exist and we shouldn't create it,
   skip it"* … *"else create it normally"*.
5. **`type="base64"` exige `file=`. Falla en las dos versiones; lo que cambia es el MENSAJE.**
   Desde v18 hay guard explícito: `case 'base64': raise ValueError("base64 type is only compatible
   with file data")` (**`odoo/tools/convert.py:165`, `def _eval_xml` v18, `:164` v19**) — dice exactamente qué falta. En **v17 no
   existe ese guard**, pero **tampoco pasa en silencio**: `data = node.text` (un `str`) llega a
   `base64.b64encode(data)` (`odoo/tools/convert.py:152-153`, `def _eval_xml`) y Python corta con
   **`TypeError: a bytes-like object is required, not 'str'`** (o `not 'NoneType'` si el nodo va
   vacío). Correcto: el guard v18 mejora el diagnóstico, no lo inventa.
   Forma correcta: `<field name="x" type="base64" file="modulo/path"/>`.
6. **`eval` corre en safe_eval con idref:** dispones de `ref(...)`, `Command`, `obj()`,
   `time`, `datetime`, etc., no de Python arbitrario. `%(xmlid)d`/`%(xmlid)s` solo funcionan en
   contenido de texto/`eval`, resueltos por `_process` (convert.py:106-122).
7. **`<function>` tambien respeta noupdate:** no se ejecuta en update bajo noupdate
   (convert.py:258). Util para post-procesos solo-en-init.
8. **CSV: el nombre del fichero es el modelo.** `res.partner.csv` -> modelo `res.partner`
   (`model = filename.split('-')[0]`, convert.py:679). La columna `id` es el external id.
9. **Quitar un `<record>` del XML hace cosas OPUESTAS segun `noupdate` — MEDIDO en v18 y v19**
   (`tools/bench-orm/probe-xmlid-gc.sh`). El recolector filtra por
   `COALESCE(noupdate,false) != True` (`odoo/addons/base/models/ir_model.py:2653-2656`, `IrModelData._process_end`, `_process_end`):
   **sin noupdate el registro DESAPARECE** en el siguiente `-u` (borra el dato en produccion: un
   `git rm` de una linea de XML es una migracion de datos sin querer); **con `noupdate="1"`
   SOBREVIVE entero** — el cron o la regla siguen vivos, ya no estan en ningun fichero, y su fila de
   `ir.model.data` sigue ahi, asi que un `<record>` futuro con ese id **edita el viejo**. Para
   retirarlo de verdad: `<delete>` o script de migracion. → `references/external-ids-and-noupdate.md`.

## NEVER

- NUNCA renombres o muevas de modulo un xml_id ya entregado sin migracion: rompe `ref` ajenos y
  deja huerfano el `ir.model.data`.
- NUNCA pongas datos de configuracion editables por el usuario sin `noupdate="1"`: cada `-u` los
  pisara.
- NUNCA escribas codigo Python/SQL ad-hoc para sembrar datos cuando un fichero declarativo basta.
- NUNCA inventes nombres/parametros de campos: confirma contra `odoo-version-api-reference`.
- NUNCA uses texto sin tildes ni identificadores/comentarios fuera de ingles (los `string=`/
  labels si pueden ir traducidos).

## ALWAYS

- SIEMPRE da a cada record un xml_id estable y semantico (`modulo.proposito_modelo`).
- SIEMPRE decide conscientemente data-vs-demo y noupdate-si-o-no antes de escribir.
- SIEMPRE ordena los ficheros en el manifest segun sus dependencias de `ref`.
- SIEMPRE prefiere `ref="..."`/`%(xmlid)d` a IDs numericos hardcodeados.
- **Provenance marker (ALWAYS):** every generated file (except `.csv`/`.json` — no comment syntax) ends with `Made with odoo-data-files · v<version del frontmatter de esta skill>`, wrapped in the file-type comment, idempotent. Full rule: `references/shared/skill-provenance-marker.md`.

## Delegates to

- `odoo-version-api-reference`: signaturas y campos exactos por version (no inline a ciegas).
- `odoo-module-creator`: manifest, declaracion de `data`/`demo` y scaffolding del modulo.
- `odoo-server-action-creator`: si el dato es una `ir.actions.server` o una automatizacion.

## Consumed by

- `odoo-module-creator`: genera la capa de datos del modulo que esta scaffoldeando.
- `odoo-mail-integration` (developer): declarar subtypes, activity types y `mail.template` como data records.

## REFERENCES

- `references/record-syntax.md` — `<record>`/`<field>`/`<function>`/`<delete>`/`<menuitem>`,
  formas de field (`ref`/`eval`/`search`/inline/`type`), anclado a convert.py.
- `references/external-ids-and-noupdate.md` — external IDs, `ir.model.data`, `noupdate`,
  `forcecreate`, semantica update-vs-init, los dos clasicos del `-u`.
- `references/system-records.md` — cron, sequence, config_parameter, mail.template, actions,
  groups: campos clave y plantillas.
- `references/csv-and-loading.md` — ficheros CSV de datos, `field/id`, `field:id`, orden de
  carga y data-vs-demo en el manifest.
- Cada reference cierra con diffs confirmados v17 -> v18 -> v19.
