# External IDs, noupdate y forcecreate

Fuentes de verdad: `$ODOO_SRC/odoo18/odoo/tools/convert.py` y
`$ODOO_SRC/odoo18/odoo/models.py` (`_load_records`). Lineas citadas son de v18.

## External IDs (xml_id)

Un external id mapea `modulo.identificador` -> `(modelo, id_BD)` en el modelo `ir.model.data`.

### Convencion y resolucion de modulo
- Formato: `modulo.identificador`. Si en el XML escribes un id SIN punto, convert.py le antepone
  el modulo actual: `make_xml_id` -> `"%s.%s" % (self.module, xml_id)` (convert.py:221-224).
- Por eso dentro del mismo modulo se referencia sin prefijo (`ref="mi_record"`), y a otro modulo
  con prefijo (`ref="base.group_user"`).
- `_test_xml_id` (convert.py:226-234): un id de referencia admite como maximo un punto, y si
  apunta a otro modulo, ese modulo debe estar instalado (search_count == 1).

### Como referenciarlos
- En `<field ref="modulo.xmlid"/>` -> id de BD (convert.py:402-410).
- En `eval` -> `ref('modulo.xmlid')` (funcion inyectada en safe_eval).
- En texto/eval -> `%(modulo.xmlid)d` (id numerico) o `%(modulo.xmlid)s` (convert.py:106-122).
- En `<menuitem parent= action= groups=>` -> por xmlid directamente (convert.py:280-310).

### ir.model.data
Cada record con `id` crea/actualiza una fila en `ir.model.data` con `module`, `name`, `model`,
`res_id`, `noupdate`. Es lo que permite que `-u` reencuentre el registro y que `ref` resuelva.

### Renombrar / borrar un xml_id
- **Renombrar** un xml_id ya entregado = romper toda `ref` externa y dejar huerfana la fila de
  `ir.model.data` (Odoo creara OTRO registro nuevo en el siguiente `-u` con el id nuevo, dejando
  el viejo sin tocar). Si hay que renombrar, hazlo con un script de migracion que actualice
  `ir.model.data`.
- **Borrar** el `<record>` del XML: depende **solo** de `noupdate`, y el resultado es opuesto en cada
  caso. MEDIDO en v18 y v19 (`tools/bench-orm/probe-xmlid-gc.sh`), instalando, quitando los dos
  records del fichero y haciendo `-u`:

  | Record que quitas del XML | Tras el `-u` |
  |---|---|
  | **sin** `noupdate` | **desaparece**: se borra el registro **y** su fila de `ir.model.data` |
  | **con** `noupdate="1"` | **sobrevive entero**, registro y fila, para siempre |

  La condicion esta en el SQL del recolector: `WHERE module IN %s AND res_id IS NOT NULL AND
  COALESCE(noupdate, false) != True` (`odoo/addons/base/models/ir_model.py:2653-2656`, `IrModelData._process_end` v18,
  `_process_end`). No es "modulos huerfanos": corre sobre **los modulos que se estan actualizando**.

  Las dos mitades muerden, cada una por su lado:
  - **Sin noupdate**: quitar un `<record>` del fichero **borra el dato en produccion** en el
    siguiente update. Si alguien lo estaba usando —una categoria, un tipo, una etiqueta— se va con
    todo lo que dependa de el. Un `git rm` de una linea de XML no parece una migracion de datos, y
    lo es.
  - **Con noupdate**: quitarlo **no lo apaga**. El cron, la regla o la plantilla siguen vivos en la
    base, ya no estan en ningun fichero del repositorio, y su fila de `ir.model.data` sigue ahi (de
    modo que un `<record>` futuro con ese mismo id **edita ese registro viejo** en vez de crear uno).
    Para retirarlo de verdad: `<delete>` explicito o script de migracion.

## noupdate: la verdad del loader

Dos comprobaciones en cadena.

### 1. En convert.py (`_tag_record`, convert.py:344-364)
```python
if self.noupdate and self.mode != 'init':
    if not rec_id:
        return None
    if record := env['ir.model.data']._load_xmlid(xid):
        ...
        self.idref[rec_id] = record.id
        return None                      # YA EXISTE -> no se toca, ni se calculan valores
    elif not nodeattr2bool(rec, 'forcecreate', True):
        return None                      # NO existe y forcecreate=0 -> se salta
    # else: se crea normalmente
```
Implicaciones:
- `noupdate` solo actua cuando `mode != 'init'`, es decir en `-u` (update). **En la instalacion
  inicial el record SIEMPRE se crea/escribe**, da igual noupdate.
- Bajo noupdate, si el xml_id YA existe, convert.py retorna sin calcular valores: **el XML
  editado se ignora por completo en update.**
- Bajo noupdate, si el xml_id NO existe, se crea (forcecreate por defecto True; `odoo/tools/convert.py:361`, `xml_import._tag_record`
  en v18, **`:374` en v19**). Asi puedes
  anadir nuevos records noupdate en una version posterior y se crearan en `-u`.

### 2. En models.py (`_load_records`, models.py:5486-5505)
```python
d_id, d_module, d_name, d_model, d_res_id, d_noupdate, r_id = row
...
if not (update and d_noupdate):
    to_update.append(data)
```
Esta es la segunda barrera, usando el flag `noupdate` guardado en la fila de `ir.model.data`
(no solo el del XML). En update, si la fila esta marcada noupdate, NO se reescribe.

### Resumen operativo

| Situacion | noupdate="1" | sin noupdate |
|-----------|--------------|--------------|
| Instalacion (`-i`) | se crea con valores del XML | se crea con valores del XML |
| `-u` y record sin tocar | NO se reescribe | se reescribe con el XML |
| `-u` tras editar el XML | el cambio se IGNORA | el cambio se aplica |
| `-u` tras editar en UI | tu cambio PERSISTE | tu cambio se PISA |

**Medido el 2026-08-15** con un ciclo real —instalar, editar los dos registros «desde la interfaz»,
cambiar el XML de `v1` a `v2`, y correr `-u`— (`tools/bench-orm/probe-noupdate.sh`):

```
rec_noupdate  ->  'editado-en-ui'    el cambio del usuario persiste, el XML se ignora
rec_plain     ->  'v2-plain'         el cambio del usuario se pisa, el XML se aplica
```

Las dos filas de la tabla, confirmadas en la misma pasada. Y la consecuencia operativa que suele
descubrirse tarde: **un dato sin `noupdate` es tuyo, no del cliente** — cualquier `-u` le devuelve
el valor del XML y se lleva por delante lo que hubiera tocado. Si el registro es una plantilla, una
secuencia o un parámetro que el cliente va a ajustar, tiene que ir con `noupdate="1"` desde el
primer día, porque cambiarlo después no rescata lo ya pisado.

### Los dos clasicos
- **"Edite el dato en el XML y `-u` no lo actualiza":** el bloque tiene `noupdate="1"`. Quita el
  noupdate (si el record es estructural) o cambia el valor a mano / via script.
- **"Lo edite en la UI y `-u` me lo pisa":** el bloque NO tiene noupdate. Envuelvelo en
  `<data noupdate="1">` si es configuracion que el usuario debe poder ajustar.

## Como declarar noupdate

A nivel de bloque (convert.py:587, `_tag_root` apila el flag por nodo):
```xml
<odoo noupdate="1">           <!-- todo el fichero -->
    <record .../>
</odoo>
```
```xml
<odoo>
    <data noupdate="1">       <!-- solo este bloque -->
        <record .../>
    </data>
    <record .../>             <!-- este SI se actualiza en -u -->
</odoo>
```
El flag se hereda al anidar (default = el del padre, convert.py:587).

## forcecreate

- Default **True** (`nodeattr2bool(rec, 'forcecreate', True)`).
- `forcecreate="0"`: bajo noupdate, si el record NO existe, no lo crees (convert.py:361).
  Util para "parchear si existe, ignorar si no".
- Para CREAR/actualizar un record que pertenece a OTRO modulo (xid con prefijo distinto),
  hace falta `forcecreate="1"` o el record ya debe existir, si no lanza
  "Cannot update missing record" (convert.py:366-374).

## Diffs v17 -> v18 -> v19

- El bloque `_tag_record` de noupdate/forcecreate es **identico** en las tres versiones
  (v17:361-382, v18:344-374, v19:357-383). v18/v19 usan el operador walrus `:=`; v17 escribe la
  asignacion en linea aparte, misma logica.
- `_load_records` con `if not (update and d_noupdate)` es estable en las tres.
- `ir.model.data` y la semantica de external IDs no cambian funcionalmente entre v17/v18/v19.
