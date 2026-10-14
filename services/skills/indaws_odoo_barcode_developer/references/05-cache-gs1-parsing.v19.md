# Subsistema: Cache cliente y parsing de barcode/GS1 — stock_barcode v19 (Enterprise)

Documento de referencia (verificado contra fuente v19) para extender el escáner de códigos de barras de Odoo 19 Enterprise. Rutas: `odoo_enterprise/19/stock_barcode` y core `odoo19/addons/{barcodes,barcodes_gs1_nomenclature}`. Los cambios respecto a v18 (verificados contra `odoo_enterprise/18`) van marcados **[v18->v19]**.

## 1. Arquitectura general

El escáner es una app OWL cliente que mantiene un **cache local** de registros ORM y **parsea** cada código escaneado, resolviéndolo a registros. Tres piezas:

1. `LazyBarcodeCache` (`static/src/lazy_barcode_cache.js`, 429 líneas) — cache cliente indexado por `model+id`, `model+barcode` y quants. Lazy: si un barcode no está, dispara RPC.
2. `BarcodeParser` — vive en el **módulo core `barcodes`** (`barcodes/static/src/js/barcode_parser.js`), y es **monkey-patcheado** por **`barcodes_gs1_nomenclature`** (`barcodes_gs1_nomenclature/static/src/js/barcode_parser.js`, vía `patch()`, NO subclase). Convierte un string escaneado en objeto(s) `{type, code, base_code, value, encoding}` según la `barcode.nomenclature` activa.
3. `BarcodeObject` (`static/src/barcode_object.js`, 118 líneas) — envuelve un raw barcode: lo parsea (vía parser singleton) y adjunta los registros del cache (product/lot), registrando los que faltan para fetch diferido.

El orquestador es `BarcodeModel` (`static/src/models/barcode_model.js`) que en `setData` (`:35`) crea el cache (`:37`), construye el parser con la nomenclatura precargada (`:46`) e inyecta ambos en `BarcodeObject.setEnv` (`:47`).

### Dependencias de módulo (CORREGIDO)
- `stock_barcode.__manifest__` depende de `['stock', 'web_tour', 'web_mobile']` (`stock_barcode/__manifest__.py:12`). **No depende directamente de `barcodes` ni de `barcodes_gs1_nomenclature`.**
- La cadena real es: **`stock_barcode` → `stock` → `barcodes_gs1_nomenclature` → `barcodes` (+ `uom`)**. Verificado: `odoo19/addons/stock/__manifest__.py:9` = `['product','barcodes_gs1_nomenclature','digest']`; `odoo19/addons/barcodes_gs1_nomenclature/__manifest__.py:8` = `['barcodes','uom']`.
- **Consecuencia**: el soporte GS1 **siempre está presente** en stock_barcode (stock hard-depende de él). Por eso `odoo_enterprise/19/stock_barcode/static/src/models/barcode_model.js:8` importa `FNC1_CHAR` de `@barcodes_gs1_nomenclature/js/barcode_parser` sin condicionarlo, y `odoo_enterprise/19/stock_barcode/static/src/models/barcode_model.js:1` importa `BarcodeParser` de `@barcodes/js/barcode_parser`.
- El parser clásico (reglas por sequence) está en **`barcodes`**; el parseo **GS1** (AI-based, separador FNC1) lo añade **`barcodes_gs1_nomenclature`** mediante `patch(BarcodeParser)` (`:8`, estáticos) y `patch(BarcodeParser.prototype)` (`:22`, instancia).

## 2. LazyBarcodeCache — estructura y ciclo de vida

Clase en `:3`; constructor `:4-31`. Estructuras internas:
- `dbIdCache[model][id] = record` — índice principal.
- `dbBarcodeCache[model][barcode] = [ids]` — índice por barcode (lista de ids, puede haber colisión).
- `dbQuantCache[product_id][location_id] = [quants]` — cache específico de stock.quant. (Además, `setCache` inicializa `dbQuantCache[product.id] = {}` para cada `product.product` cuando el payload trae `stock.quant`, `:68-72`.)
- `missingBarcodesCache` (Set) y `missingBarcodeKeyCache` (Set) — memoización de barcodes ya buscados y no encontrados, para no repetir RPC.
- `nomenclature` — solo se fija si hay exactamente UNA nomenclatura en el payload (`:26-28`).
- `waitingFetch` (array) — cola de fetch diferido (`:30`).

### Mapas de configuración (clave para extender)
```
barcodeFieldByModel = {           gs1LengthsByModel = {
  "stock.location": "barcode",      "product.product": 14,
  "product.product": "barcode",     "product.uom": 14,
  "product.uom": "barcode",         "stock.location": 13,
  "stock.package.type": "barcode",  "stock.package": 18,
  "stock.picking": "name",        }
  "stock.package": "name",
  "stock.lot": "name",
}
```
`odoo_enterprise/19/stock_barcode/static/src/lazy_barcode_cache.js::constructor-24`. **[v18->v19]** En v18 estos mapas usaban `product.packaging` (len 14) y `stock.quant.package` (len 18) — verificado en `odoo_enterprise/18/stock_barcode/static/src/lazy_barcode_cache.js:15,18,23,25`, `LazyBarcodeCache.constructor`. En v19 el empaquetado se fusionó en **`product.uom`** y `stock.quant.package` se renombró a **`stock.package`**. Es el cambio más impactante del subsistema.

### setCache (`:38-91`)
Indexa cada registro en `dbIdCache` y `dbBarcodeCache`. Caso especial `stock.quant` (`:52-67`): lo mete en `dbQuantCache[product][location]` (con dedup por id). Para modelos con nomenclatura GS1 activa y presentes en `gs1LengthsByModel`, además llama `_setBarcodeInCacheForGS1` (`:80-86`).

### _setBarcodeInCacheForGS1 (`:405-428`)
Registra una **entrada extra con el barcode rellenado con ceros a la izquierda** (`padStart`) hasta la longitud GS1 del modelo. Así un barcode corto (p.ej. product `12345`) se encuentra cuando se escanea el GTIN-14 completo. Solo padea si es numérico y más corto que la longitud objetivo (`:407-411`).
- **CORREGIDO — comportamiento ante colisión (`:415-427`):** si la clave padeada YA existe para otro registro, el método **NO sobrescribe**: solo hace `console.log`. La entrada conserva el **PRIMER/anterior** registro cacheado (`dbBarcodeCache[model][paddedBarcode][0]`), NO el último. El texto del propio log ("The last one will be used") es **engañoso y contradice el código**: en realidad gana el primero de forma determinista. Ambigüedad silenciosa (solo consola) → gotcha.

### getRecord (`:102-119`)
Devuelve una **copia profunda** (`JSON.parse(JSON.stringify(record))`) para no mutar el cache. Lanza Error si falta y `raiseErrorIfMissing` (default true).

### getRecordByBarcode (`:188-280`) — corazón de la resolución barcode->registro
- Con `model`: si el modelo/barcode no está en cache:
  - `fetchLater` -> encola en `waitingFetch` y, **[v19]** si el modelo es `product.product`, **encola también `product.uom`** (`:196-199,210-213`) para intentar resolverlo como empaquetado/uom.
  - `onlyInCache` -> devuelve null.
  - si no -> `await _getMissingRecord(...)` (RPC) y reintenta con `onlyInCache:true`.
  - Si está, itera los ids y aplica `filters[model]` (comparación campo a campo con `!=`, comparación laxa, `:232`).
- Sin `model`: busca en TODOS los modelos, devuelve un `Map(model -> record)` con la primera coincidencia por modelo (`:242-278`, filtros con `!=` en `:255`).

### _getMissingRecord (`:289-329`) y getMissingRecords (`:331-391`)
Ambos disparan RPC a `/stock_barcode/get_specific_barcode_data` y hacen `setCache(result)`. `getMissingRecords` procesa la cola `waitingFetch`; con `forceUnrestrictedSearch` (`:351`) hace un segundo RPC sin restricción de modelo para los barcodes que ninguna búsqueda por modelo encontró, memoizando en `missingBarcodesCache`/`missingBarcodeKeyCache`.

### getQuants (`:121-179`)
Devuelve quants de un producto/ubicación, con RPC a `/stock_barcode/get_quants` si no están cacheados (`:132`), y filtra por lot/lot_name/package/owner.

## 3. BarcodeObject — parseo y attach

Fichero `static/src/barcode_object.js` (118 líneas). **[v18->v19]** Idéntico funcionalmente a v18 salvo: (a) quitada cabecera `/** @odoo-module **/` (v18 la tenía en `:1`), (b) fallback a `product.uom` en vez de `product.packaging`, (c) `packaging.uom_id.factor` en vez de `packaging.qty` (v18 `:92`).

### Diseño singleton estático (SÍ, sigue en v19)
```js
get cache()  { return BarcodeObject.__cache; }   // :59-61
get parser() { return BarcodeObject.__parser; }  // :67-69
BarcodeObject.mappingRawBarcodeToObject = {};     // :106  (Map estático global)
BarcodeObject.setEnv = (cache, parser) => { __cache=..., __parser=... }; // :108-111
BarcodeObject.forBarcode = (barcode) => cached || new BarcodeObject(barcode); // :113-118
```
`__parser` y `__cache` son **propiedades ESTÁTICAS de la clase**, compartidas por TODAS las instancias. `setEnv` (llamado desde `BarcodeModel.setData:47`) las fija globalmente. `forBarcode` cachea instancias por raw string en `mappingRawBarcodeToObject` — **este mapa nunca se limpia** en toda la sesión.

### Constructor (`:2-33`)
Guarda `rawValue`, detecta URN (`/^urn:.*$/`, `:10`), y si hay parser intenta `parser.parse_barcode(rawValue)` dentro de try/catch (`:12-20`): **si el parser lanza, el error se traga (solo console.log) y cae al flujo clásico**. Normaliza `parsedBarcode` a array (`:21-25`). Se auto-registra en `mappingRawBarcodeToObject` (`:32`). Si no hay parser, `console.warn("No parser set !")`.

### setRecords (`:39-56`)
Itera `parsedBarcode` y despacha por `type`: `"product"` -> `fetchProduct`, `"lot"` -> `fetchTrackingNumber`. Opciones por defecto `{fetchLater:true, onlyInCache:true}` (`:43-46`). **Cualquier otro `type` se ignora** (punto de extensión).

### fetchProduct (`:81-102`)
Busca `product.product` por barcode. Si no lo encuentra, **[v19]** intenta como `product.uom` (`:88-90`, `onlyInCache:true`): si existe, obtiene el product asociado (`getRecord("product.product", packaging.product_id, false)`, `:92`) y fija `parsedData.packaging` y `parsedData.quantity = packaging.uom_id.factor` (`:93-94`). Lo no resuelto va a `missingRecords` como `{type:"product", productBarcode}`.

### fetchTrackingNumber (`:72-79`)
Busca `stock.lot` por barcode; si no, empuja `{type:"lot", lotBarcode}` a `missingRecords`.

## 4. BarcodeParser (módulos core barcodes / barcodes_gs1_nomenclature)

### parse_barcode (`barcodes/static/src/js/barcode_parser.js:177-181`, `BarcodeParser`)
```
if (barcode.match(/^urn:/)) return this.parseURI(barcode);
return this.parseBarcodeNomenclature(barcode);
```
- `parseBarcodeNomenclature` (`:184+`): itera `nomenclature.rules` (ordenadas por sequence), aplica conversiones upc<->ean, valida encoding (`check_encoding`, `:68`), y hace `match_pattern` (`:109`) que extrae valor numérico y `base_code`. Devuelve un **único objeto** `{encoding, type, code, base_code, value}`.
- `parseURI` (`:249+`): parsea URN EPC (sgtin/lgtin) a product + lote.
- `fetchNomenclature(orm, id)` es un método **estático async** (`:4`) usado solo si la nomenclatura no viene precargada.

### GS1 (barcodes_gs1_nomenclature/js/barcode_parser.js) — CORREGIDO: es un `patch`, no subclase
Usa `patch` de `@web/core/utils/patch` (`:1`): `patch(BarcodeParser, {...})` (`:8`) y `patch(BarcodeParser.prototype, {...})` (`:22`). Exporta `FNC1_CHAR = String.fromCharCode(29)` (`:6`) y `GS1BarcodeError` (`:4`). Parsea por Application Identifiers usando `parse_gs1_rule_pattern` (`:66`), convierte fechas GS1 (`gs1_date_to_date`, `:36`) y valida con `get_barcode_check_digit` (`:88`; **no** `check_digit`). El parseo GS1 devuelve un **array** de objetos (múltiples AIs en un barcode), de ahí la normalización a array en BarcodeObject.

La nomenclatura se elige por compañía: servidor `request.env.company.nomenclature_id`; `is_gs1_nomenclature` gobierna el padding cliente (`_setBarcodeInCacheForGS1`) y el dominio servidor (`ilike`).

## 5. Lado servidor (controller) — cómo llegan los datos

`controllers/stock_barcode.py`. **[v18->v19] todas las rutas pasan de `type='json'` a `type='jsonrpc'`** (verificado: v18 usaba `type='json'` en `:15,81,89,117,147,152`).
- `/stock_barcode/get_barcode_data` (`:92-93`) — payload inicial completo (records + nomenclature_id + config + groups). Alimenta `new LazyBarcodeCache(...)`.
- `/stock_barcode/get_specific_barcode_data` (`:143-222`) — fetch lazy. **[v19]** usa la nueva API `odoo.fields.Domain` (`from odoo.fields import Domain`, `:10`; `Domain(...)`, `Domain.OR`, `Domain.TRUE`, `&=`, `|=`, en `:161-201`). El campo barcode se lee de `request.env[model]._barcode_field` (`:179`). En GS1 construye dominios `ilike` sobre el barcode sin padding (`:190`, combinados con `Domain.OR` en `:195`).
- `/stock_barcode/get_quants` (`:232-233`, método `get_existing_quant_and_related_data`).
- Hook por modelo: `_get_fields_stock_barcode()` (campos a enviar) y opcional `_get_stock_barcode_specific_data()` (registros satélite), recolectados en `_get_records_fields_stock_barcode` (`:223-230`) — detectado por `hasattr` (`:226`). Helper `_get_barcode_field_by_model` (`:454-465`) también deriva el campo de `_barcode_field`.

Ejemplo `product.uom` (`models/product_uom.py`): `_inherit='product.uom'` (`:5`), `_barcode_field='barcode'` (`:6`), `_get_fields_stock_barcode` (`:9`), y `_get_stock_barcode_specific_data` (`:16-19`) adjunta `product.product` + `uom.uom`.

## 6. Flujo completo de un escaneo (BarcodeModel)
`barcode_model.js` (~`:700-760`):
1. Filtra URIs ya escaneadas (`uriInCache`, `:660`, usado en `:705`).
2. Por cada barcode: `BarcodeObject.forBarcode(bc)` (`:717`, reusa instancia si ya existe) + `await barcodeObject.setRecords()` (`:718`, attach solo-cache + encola faltantes).
3. `await this._getMissingRecords()` (`:722`; def en `:770`) — vacía `waitingFetch` con un RPC batch a `getMissingRecords`.
4. Segunda pasada (`:727-741`): para objetos con `hasMissingRecords`, re-`setRecords()`; si es URN y sigue faltando el product, se ignora el barcode (TODO en código: Barcode Lookup pendiente).
5. `actionMutex.exec` (`:744`) -> `_processBarcode(rawValue)` (`:747`; def en `:1300`) por cada barcode válido.

## 7. Gotchas críticos para extender
- **Singleton estático**: `__parser`/`__cache`/`mappingRawBarcodeToObject` son de clase, compartidos globalmente. Dos apps de barcode simultáneas se pisan. `forBarcode` no re-parsea un raw ya visto aunque cambie la nomenclatura.
- **Fuga**: `mappingRawBarcodeToObject` crece sin límite durante la sesión.
- **Errores de parser silenciados**: try/catch en BarcodeObject (`:12-20`) solo hace console.log.
- **Colisiones GS1 silenciosas (CORREGIDO)**: `_setBarcodeInCacheForGS1` ante conflicto **NO sobrescribe**; conserva el PRIMER record cacheado y solo avisa por consola. El mensaje del log dice "the last one will be used" pero es falso: gana el primero.
- **Modelos renombrados**: cualquier override heredado de v18 que mencione `product.packaging` o `stock.quant.package` está roto en v19; usar `product.uom` / `stock.package`.
- **`type='jsonrpc'`**: overrides de controller de v18 con `type='json'` no interceptan las rutas v19.
- **Domain API v19**: código servidor que construya dominios como listas debe migrar a `odoo.fields.Domain` para encajar con el patrón del controller.
- **Comparación laxa**: `getRecordByBarcode` filtra con `!=` (no `!==`); tipos distintos (`'5'` vs `5`) pueden colar/descartar por coerción.
- **Copias defensivas**: `getRecord`/`getRecordByBarcode` devuelven deep-copies; mutar el resultado NO afecta al cache (hay que re-`setCache`).
- **GS1 se extiende por `patch`, no por subclase**: `BarcodeModel` instancia la clase `BarcodeParser` importada de `@barcodes`; una subclase propia no se usaría. Para añadir comportamiento hay que parchear el prototype (como hace GS1) o añadir `barcode.rule`.


## Correcciones aplicadas en verificación (v19)

- CRÍTICA — _setBarcodeInCacheForGS1 (odoo_enterprise/19/stock_barcode/static/src/lazy_barcode_cache.js::_setBarcodeInCacheForGS1-427): el borrador afirma que ante colisión 'se queda con el ÚLTIMO registro'. FALSO. Releído el código v19: en la rama else-if de conflicto el método SOLO hace console.log y NO reasigna dbBarcodeCache[model][paddedBarcode]; la entrada conserva el PRIMER/anterior record ya cacheado (dbBarcodeCache[model][paddedBarcode][0]). Gana el PRIMERO, no el último. El texto del propio console.log ('The last one will be used') es engañoso y contradice lo que hace el código. Corregido en sección 2, gotchas y extension-point risk.
- Dependencia de módulo: el borrador dice que stock_barcode obtiene barcodes 'transitivamente vía stock (que depende de barcodes)'. Impreciso. Verificado: odoo19/addons/stock/__manifest__.py:9 depende de ['product','barcodes_gs1_nomenclature','digest'] — NO de 'barcodes' directamente. odoo19/addons/barcodes_gs1_nomenclature/__manifest__.py:8 depende de ['barcodes','uom']. Cadena real: stock_barcode → stock → barcodes_gs1_nomenclature → barcodes(+uom). Consecuencia: el soporte GS1 SIEMPRE está presente en stock_barcode (no es opcional), lo que justifica el import de FNC1_CHAR.
- Mecanismo del override GS1: el borrador dice 'override JS de BarcodeParser' y en el extension point sugiere 'subclasar el parser'. Corregido: barcodes_gs1_nomenclature/static/src/js/barcode_parser.js NO subclasa; hace monkey-patch con patch() de @web/core/utils/patch — patch(BarcodeParser, {...}) (:8, estáticos) y patch(BarcodeParser.prototype, {...}) (:22, instancia). Para extender: añadir barcode.rule (nomenclatura) o parchear el prototype, NO subclasar (una subclase no se usaría, ya que BarcodeModel instancia la clase BarcodeParser importada de @barcodes).
- Nombre de método de dígito de control GS1: el borrador dice 'usa check_digit'. El método real en v19 es get_barcode_check_digit (barcodes_gs1_nomenclature/js/barcode_parser.js:88). Corregido.
- barcode_object.js tiene 118 líneas, no 119 (dato menor del borrador).
- Nits de file:line (±1-2, no afectan a la sustancia): el cuerpo del constructor de LazyBarcodeCache es :4-31 (la clase abre en :3), no ':3-31'. Controller _get_records_fields_stock_barcode: def en :223 (el borrador citaba :224-230). request.env.company.nomenclature_id en get_specific_barcode_data está en :163 (borrador decía :164). product_uom._get_stock_barcode_specific_data en :16-19 (borrador citaba product_uom.py:15). Todo lo demás (setData:35/cache:37/setEnv:47, getRecordByBarcode:188-280, fetchLater dual-enqueue :196-199/:210-213, _getMissingRecord:289-329, getMissingRecords:331-391, getQuants:121-179, barcode_object getters :59-61/:67-69, setEnv :108-111, forBarcode :113-118, try/catch :12-20, fetchProduct :81-102 con uom_id.factor :94) CONFIRMADO contra v19.
- Comparaciones v18→v19 CONFIRMADAS contra fuente v18 real: v18 usaba product.packaging (len 14) y stock.quant.package (len 18) en ambos mapas (18/stock_barcode/lazy_barcode_cache.js:15,18,23,25); v18 controller usaba type='json' en TODAS las rutas /stock_barcode/* (18/.../controllers/stock_barcode.py:15,81,89,117,147,152); v18 barcode_object.js tenía cabecera /** @odoo-module **/ (:1), usaba product.packaging (:86) y packaging.qty (:92). Todos los cambios [v18->v19] del borrador son correctos.
