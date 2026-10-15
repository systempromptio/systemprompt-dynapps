## Subsistema: Cache cliente y parsing de barcode/GS1 (stock_barcode, Odoo 18 EE)

Fuente RELEÍDA y verificada línea a línea: `odoo_enterprise/18/stock_barcode/static/src/lazy_barcode_cache.js` y `odoo_enterprise/18/stock_barcode/static/src/barcode_object.js`. Verificados también los puntos de cableado en `models/barcode_model.js`, el contrato server en `controllers/stock_barcode.py`, y los parsers core `@barcodes/js/barcode_parser` y `@barcodes_gs1_nomenclature/js/barcode_parser` (estos últimos SIN leer enteros, solo lo necesario para el enganche de nomenclatura).

### 1. Arquitectura y cableado (quién crea qué)

El punto de ensamblaje es `BarcodeModel.setData()` en `odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js::setData-46` (verificado):

- `this.cache = new LazyBarcodeCache(data.data.records)` (`odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js::setData`) — el servidor entrega en el arranque un blob de registros pre-cacheados.
- Se reconstruye la nomenclatura **desde la cache, usando el id que da el servidor**: `getRecord('barcode.nomenclature', data.data.nomenclature_id)` (`:40`) y sus `barcode.rule` vía `nomenclature.rule_ids` (`:41-44`). **Corrección**: la nomenclatura NO se resuelve en JS desde `company.nomenclature_id`; el id llega ya calculado por el servidor en `data.data.nomenclature_id`.
- `this.parser = new BarcodeParser({ nomenclature })` (`:45`) — instancia del parser del módulo core `barcodes` (import `BarcodeParser from "@barcodes/js/barcode_parser"`, `odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js:3`).
- `BarcodeObject.setEnv(this.cache, this.parser)` (`:46`) — inyecta cache y parser como **singletons estáticos** (`__cache`/`__parser`) en la clase `BarcodeObject`.
- `FNC1_CHAR` se importa de `@barcodes_gs1_nomenclature/js/barcode_parser` (`odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js:10`) y se usa para limpiar separadores GS1 (`odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js::cleanBarcode-1565`).

`LazyBarcodeCache` NO es un servicio del registry: es una clase con `export default` (`odoo_enterprise/18/stock_barcode/static/src/lazy_barcode_cache.js:5`, `LazyBarcodeCache`), instanciada manualmente por sesión de barcode. `BarcodeObject` es `export class` (`odoo_enterprise/18/stock_barcode/static/src/barcode_object.js:3`, `BarcodeObject`).

Flujo de escaneo (bucle en `barcode_model.js` ~`:682-714`, verificados los puntos clave):
1. Por cada barcode crudo: `BarcodeObject.forBarcode(bc)` (memoizado, `:684`) → `await barcodeObject.setRecords()` (adjunta solo lo que ya está en cache, marca faltantes).
2. `await this._getMissingRecords()` (`:689`, def en `:734`) → un RPC batch para traer lo faltante. Sus params los da `_getMissingRecordsParams()` (`:727`).
3. Segunda pasada `setRecords()` para re-enlazar los objetos con `hasMissingRecords`.
4. `_processBarcode(rawValue)` dentro de un `Mutex`.

### 2. LazyBarcodeCache — estructura interna

Tres índices + dos sets de dedupe (`odoo_enterprise/18/stock_barcode/static/src/lazy_barcode_cache.js::constructor-11`):
- `dbIdCache[model][id] = record` — índice canónico por id.
- `dbBarcodeCache[model][barcode] = [id, id, ...]` — índice por barcode (varios ids posibles).
- `dbQuantCache[product_id][location_id] = [quant, ...]` — índice específico de `stock.quant`.
- `missingBarcodesCache` (Set) y `missingBarcodeKeyCache` (Set) — dedupe de fetches.

Dos mapas de configuración **hardcodeados**:
- `barcodeFieldByModel` (`:12-20`): 7 modelos. `stock.picking`→`name`, `stock.quant.package`→`name`, `stock.lot`→`name`; `stock.location`/`product.product`/`product.packaging`/`stock.package.type`→`barcode`. Espeja el servidor (§5).
- `gs1LengthsByModel` (`:21-26`): `product.product` 14, `product.packaging` 14, `stock.location` 13, `stock.quant.package` 18.

`this.nomenclature` **solo se fija si hay EXACTAMENTE una** `barcode.nomenclature` en el blob (`:28-30`, `.length === 1`). Con 0 o >1 queda `undefined` y **el padding GS1 de la cache se desactiva silenciosamente** (guard en `:82`).

#### Métodos clave (líneas verificadas)
- `setCache(cacheData)` (`:40-89`): merge de registros; rellena los tres índices. Para `stock.quant` construye/actualiza `dbQuantCache` con dedupe por id (`:54-69`); para `product.product` con `stock.quant` presente inicializa el sub-objeto (`:70-74`). Al indexar por barcode, si `this.nomenclature.is_gs1_nomenclature` y el modelo está en `gs1LengthsByModel`, llama `_setBarcodeInCacheForGS1` (`:82-84`).
- `getRecord(model, id, raiseErrorIfMissing=true)` (`:100-115`): devuelve **copia profunda** vía `JSON.parse(JSON.stringify(record))` (`:114`). Lanza si falta salvo flag `false`.
- `getRecordByBarcode(barcode, model=false, options={})` (`:184-262`): opciones `onlyInCache`, `filters`, `fetchLater` (`:185-187`). CON `model`: si el modelo no está en cache → según flags encola en `waitingFetch`, devuelve `null`, o lanza (`:197`); si el barcode no está → encola/`null`/`_getMissingRecord` + **re-llamada recursiva con `{onlyInCache:true}`** (`:207-208`). Filtra `filters[model]` con comparación **`!=` (loose)** (`:217`). **Puede caer al final devolviendo `undefined`** si hay ids pero ninguno pasa el filtro (bucle `:211-226` sin return final). SIN `model`: itera todas las claves de `dbBarcodeCache` y devuelve un `Map{model→record}` (`:227-261`); si vacío y `onlyInCache`, devuelve un Map vacío (`:254-255`).
- `getQuants(product, location_id, params={})` (`:117-175`): resuelve quants desde `dbQuantCache`; si no hay, RPC `/stock_barcode/get_quants` (`:128`) y re-cachea. Filtra por lot/lot_name/package/owner. (Nota: en `:161` pasa `filters` como 3er argumento posicional de `getRecordByBarcode`, que espera `options` — los filtros ahí no se aplican vía `options.filters`.)
- `_getMissingRecord(barcode, model, filters={})` (`:271-311`): RPC `/stock_barcode/get_specific_barcode_data` con `barcodes_by_model` o `barcode` + `domains_by_model` construido desde filters. Dedupe por `keyCache = JSON.stringify([...arguments])` (`:272`) y variantes sin modelo/sin filtros (`:274-284`).
- `getMissingRecords(params={})` (`:313-369`): drena `waitingFetch` en **un** RPC batch (`:331`). Si `params.forceUnrestrictedSearch` (lo pone `_getMissingRecordsParams` cuando NO es GS1, `odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js::_getMissingRecordsParams`), hace una **segunda pasada** buscando los barcodes sin match sin modelo asignado (`:353-365`).
- `_setBarcodeInCacheForGS1(barcode, model, record)` (`:383-404`): si el barcode es numérico y más corto que la longitud GS1 del modelo, crea una entrada extra en `dbBarcodeCache` con el barcode **rellenado con ceros a la izquierda** (`padStart`, `:390`). Si ya existe otro registro en esa clave, emite `console.log` de conflicto (no notifica al usuario, `:398-402`).

### 3. BarcodeObject — parsing y enlace a registro

Estado por instancia (`odoo_enterprise/18/stock_barcode/static/src/barcode_object.js::constructor-12`): `rawValue`, `nomenclature` (init `undefined`), `parsedBarcode`, `parsedData` (`{product, lot, packaging, quantity}`), `isParsed`, `missingRecords`, `isURN` (regex `/^urn:.*$/`).

- **Parseo en el constructor** (`:14-31`): llama `this.parser.parse_barcode(rawValue)` (parser estático). try/catch: si falla, `console.log` y fallback (parsedBarcode queda undefined, `:17-22`). Normaliza a array (`:23-27`) porque GS1 devuelve array y el resto objeto. `isParsed = Boolean(parsedBarcode?.length)` (`:28`).
- **Estáticos** (`:104-116`): `mappingRawBarcodeToObject` (memo global rawValue→instancia, **nunca se limpia**, `:104`), `setEnv(cache, parser)` (`:106-109`), `forBarcode(barcode)` (factory memoizada, `:111-116`).
- **Getters** (`:60-71`): `cache`→`__cache` (`:61-63`), `hasMissingRecords` (`:65-67`), `parser`→`__parser` (`:69-71`).
- `setRecords(options=false)` (`:41-58`): recorre `parsedBarcode`; por cada entrada, `type==="product"`→`fetchProduct(code)`, `type==="lot"`→`fetchTrackingNumber(code)`. **Solo esos dos types**; cualquier otro se ignora en silencio. Default options `{fetchLater:true, onlyInCache:true}` (`:45-48`).
- `fetchProduct(productBarcode, options)` (`:83-100`): busca `product.product`; si no, prueba `product.packaging` con `{onlyInCache:true}` (`:86-88`) y si hay packaging resuelve `product_id` y setea `parsedData.packaging`/`quantity` (`:89-93`). Faltante → `missingRecords.push({type:"product", productBarcode})` (`:98`).
- `fetchTrackingNumber(lotBarcode, options)` (`:74-81`): busca `stock.lot`; faltante → `missingRecords.push({type:"lot", lotBarcode})` (`:79`).

El `type`/`code` provienen del parser core: `parse_barcode` (`barcodes/static/src/js/barcode_parser.js:179`, `BarcodeParser.parse_barcode`) → `parseBarcodeNomenclature` devuelve `{encoding, type, code, base_code, value}` (`:187-193`); el `type` es `rules[i].type` de la `barcode.rule` que casó (`:228`).

### 4. Dependencia con los módulos core (barcodes / barcodes_gs1_nomenclature)

- **`barcodes`** (CE, `odoo18/addons/barcodes/static/src/js/barcode_parser.js`): define `class BarcodeParser` (`:3`) con `parse_barcode` (`:179`, dispatcher URI vs nomenclatura), `parseBarcodeNomenclature` (`:186`, recorre `nomenclature.rules`, aplica conversiones upc/ean y `match_pattern` `:111`). Es el parser que stock_barcode instancia.
- **`barcodes_gs1_nomenclature`** (CE, mismo path base): **parchea** `BarcodeParser` vía `patch(...)` (import `patch` `:3`):
  - `patch(BarcodeParser, {...})` añade `is_gs1_nomenclature` a `barcodeNomenclatureFields` (`:10-14`) y campos GS1 a `barcodeRuleFields` (`:16`).
  - `patch(BarcodeParser.prototype, {...})` (`:24`) con override de `parseBarcodeNomenclature` (`:149-153`): si `this.nomenclature.is_gs1_nomenclature` parsea como GS1 (**array** de segmentos AI); si no, `super(...arguments)`.
  - exporta `FNC1_CHAR = String.fromCharCode(29)` (`:8`), consumido en `odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js:10`.

  **El enganche de la nomenclatura GS1 vive en el parser CORE, no en la cache.** La cache solo lee `nomenclature.is_gs1_nomenclature` para el padding (`odoo_enterprise/18/stock_barcode/static/src/lazy_barcode_cache.js::setCache`) y `barcode_model` lo lee para `forceUnrestrictedSearch` (`odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js::_getMissingRecordsParams`).

### 5. Lado servidor (contrato de la cache)

- Route `/stock_barcode/get_specific_barcode_data` → método `get_specific_barcode_data(**kwargs)` (`odoo_enterprise/18/stock_barcode/controllers/stock_barcode.py:152-153`, `StockBarcodeController`; existe además la variante batched `get_specific_barcode_data_batch` en `:238`). Usa `request.env[model]._barcode_field` (`:185`) y `_get_barcode_field_by_model()` (`:178`).
- `_get_barcode_field_by_model()` (`:494`, `return` en `:504`): dict `{model: env[model]._barcode_field for model in list_model if hasattr(...,'_barcode_field')}` — **lista hardcodeada filtrada por hasattr**. Debe mantenerse en sync con `barcodeFieldByModel` del cliente.
- Route `/stock_barcode/get_quants` → método `get_existing_quant_and_related_data(domain)` (`:147-148`). **Corrección**: el método NO se llama `get_quants` (ese es solo el path de la ruta).
- El payload por modelo lo define `Model._get_fields_stock_barcode()` (verificados: `odoo_enterprise/18/stock_barcode/models/product_product.py:26`, `Product._get_fields_stock_barcode`, `stock_barcode/models/stock_quant_package.py:27`, `QuantPackage._get_fields_stock_barcode`, `stock_barcode/models/stock_location.py:17`, `stock_barcode/models/product_packaging.py:19`, `ProductPackaging._get_fields_stock_barcode`). Es el gancho limpio para **enriquecer** los registros cacheados.

### 6. Resolución end-to-end de un barcode

`rawValue` → `BarcodeObject.forBarcode` → `parser.parse_barcode` (core; el patch GS1 decide array vs objeto) → `parsedBarcode[{type,code}]` → `setRecords` despacha por `type` (solo `product`/`lot`) → `cache.getRecordByBarcode(code, model)` → busca en `dbBarcodeCache[model][code]` (+ variante GS1 padded) → si falta, RPC `get_specific_barcode_data` → `setCache` → copia del registro en `parsedData`.

---

### Puntos de extensión reales (verificados)

1. **Nuevo modelo indexable por barcode** — *confirmed*. Cliente: `patch` del constructor de `LazyBarcodeCache` para añadir clave a `barcodeFieldByModel` (`:12-20`). Servidor: añadir el modelo a `_get_barcode_field_by_model` (`stock_barcode/controllers/stock_barcode.py:494`, `StockBarcodeController._get_barcode_field_by_model`) + `_barcode_field` + `_get_fields_stock_barcode`. Riesgo: dos listas hardcodeadas independientes; patchear solo el cliente NO puebla `dbBarcodeCache[model]` (setCache solo crea la clave si el modelo llega en un payload, `:47-49`); `getRecordByBarcode` con modelo inexistente lanza en `:197` (rama sin onlyInCache/fetchLater).

2. **Enriquecer campos de un registro cacheado** — *confirmed*. Override Python de `Model._get_fields_stock_barcode()` (super() + append). No toca JS. Riesgo: campos JSON-safe (serialización en `:114`; fechas→string, undefined/funciones se pierden).

3. **Nuevo `type` de barcode** — *corrected*. `patch(BarcodeObject.prototype, {...})` sobre `setRecords`/`fetchX` (`:41-100`). ⚠️ `setRecords` SOLO maneja `product` y `lot` nativamente; el resto se ignora en silencio. El `type` proviene de `barcode.rule.type`, un **Selection FIJO** del core `barcodes`: para un type nuevo arbitrario NO basta con añadir un `barcode.rule` (dato) — hay que **extender el Selection server-side**. Para un type ya existente en la nomenclatura (p.ej. `package`) sí basta patch + regla.

4. **Override de resolución/caching** — *confirmed*. `patch(LazyBarcodeCache.prototype, {...})` sobre `setCache`/`getRecordByBarcode`/`_getMissingRecord`/`getQuants`. Import default `from '@stock_barcode/lazy_barcode_cache'` (confirmado en `odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js:6`). Riesgo: `getRecordByBarcode` con modelo puede devolver `undefined` (sin return final, `:211-262`); la recursión `{onlyInCache:true}` (`:208`) loopea si tu override no llena la cache.

5. **Longitudes / padding GS1** — *confirmed*. `patch` del constructor para tocar `gs1LengthsByModel` (`:21-26`) u override de `_setBarcodeInCacheForGS1` (`:383`). Riesgo: colisiones solo generan `console.log` ("el último gana", `:398-402`); todo el padding solo actúa con `this.nomenclature` fijada (una única nomenclatura, `:28-30`) e `is_gs1_nomenclature`.

6. **Parser (nomenclatura GS1/custom)** — *confirmed*. `patch(BarcodeParser.prototype, {...})` en `@barcodes/js/barcode_parser` (`parseBarcodeNomenclature` `:186`, `match_pattern` `:111`) o replicar el patrón de `barcodes_gs1_nomenclature` (`patch` en `:24`, override en `:149-153`). Riesgo: `__parser` es singleton estático (fijado en `setEnv`); el patch debe cargarse antes de `setEnv`. Errores de parseo se tragan en el constructor de `BarcodeObject` (`:15-22`) → fallback silencioso sin aviso al usuario.

### Gotchas (verificados)
- `BarcodeObject.mappingRawBarcodeToObject` (`:104`) **nunca se limpia**: `forBarcode` devuelve la misma instancia para un rawValue repetido durante toda la vida de la página → `parsedData` puede quedar obsoleto si el registro cambió en servidor. `__cache`/`__parser` son estáticos compartidos, re-seteados en cada `setData` (última sesión gana).
- `this.nomenclature` en la cache solo se fija con **exactamente una** `barcode.nomenclature` en el blob (`:28-30`). Con 0/>1 el padding GS1 de la cache no se aplica, aunque el parser (construido desde la nomenclatura que da el servidor en `data.data.nomenclature_id`, NO desde `company.nomenclature_id` en JS) sí parsee GS1 → posible desalineación cache↔parser.
- Filtros comparados con `!=` loose (`:217`, `:240`): `1 == "1"`. Cuidado con numérico vs string.
- `getRecord` devuelve **copia** (`:114`): mutar `parsedData.product` NO actualiza la cache; hay que volver a `setCache`. Campos no-JSON se degradan.
- `getRecordByBarcode` con modelo puede devolver `undefined` (no null) si existen ids pero ninguno pasa el filtro (bucle `:211-226` sin return final).
- `forceUnrestrictedSearch` solo es true cuando la nomenclatura NO es GS1 (`odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js::_getMissingRecordsParams`): en GS1 no hay segunda pasada de búsqueda sin modelo.
- Errores de `parse_barcode` en el constructor de `BarcodeObject` se capturan y solo se loguean (`:15-22`): fallback silencioso.
- El enganche real de la nomenclatura GS1 está en el parser core (`barcodes_gs1_nomenclature` patchea `BarcodeParser.prototype.parseBarcodeNomenclature` `:149`), NO en la cache.
- `barcodeFieldByModel` (cliente, `:12`) y `_get_barcode_field_by_model` (servidor, `stock_barcode/controllers/stock_barcode.py:494`, `StockBarcodeController._get_barcode_field_by_model`) son dos listas hardcodeadas independientes que hay que sincronizar a mano.


## Correcciones aplicadas en verificación

- GOTCHA nomenclatura↔parser: el draft decía que el parser 'se construye desde company.nomenclature_id'. FALSO en el lado JS. barcode_model.setData() construye el parser desde la nomenclatura que el SERVIDOR le pasa en el blob: getRecord('barcode.nomenclature', data.data.nomenclature_id) (odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js::setData) y luego new BarcodeParser({nomenclature}) (:45). El JS no referencia company.nomenclature_id; esa resolución (probablemente desde la compañía) ocurre server-side. El resto del gotcha (cache exige nomenclatura única :28, parser no) es correcto.
- RPC get_quants: el draft daba el 'serverModel' como '(get_quants)'. El ROUTE es /stock_barcode/get_quants (stock_barcode.py:147) pero el MÉTODO Python se llama get_existing_quant_and_related_data (:148), no get_quants. Corregido.
- Controller get_specific_barcode_data: el rango '152-236' es aproximado. El @http.route está en :152 y la def en :153; además existe una variante batched get_specific_barcode_data_batch en :238 (no mencionada). El uso de env[model]._barcode_field está en :185 y _get_barcode_field_by_model en :494 (return con hasattr en :504) — todo confirmado.
- EXTENSION POINT 3 (nuevo 'type'): corregido/matizado. setRecords SOLO despacha type==='product' y type==='lot' (odoo_enterprise/18/stock_barcode/static/src/barcode_object.js:52-56); ni siquiera maneja 'package'/'location'/'owner' de forma nativa. Además el 'type' proviene de barcode.rule.type, que es un campo Selection con valores FIJOS en el core barcodes: NO basta con 'añadir un barcode.rule (dato)' para inventar un type nuevo arbitrario — hay que extender la Selection server-side si el valor no existe. Para types estándar ya soportados por la nomenclatura (p.ej. 'package') basta con patchear setRecords + añadir la regla.
- EXTENSION POINT 1 (matiz añadido, no error): patchear solo barcodeFieldByModel en cliente NO puebla dbBarcodeCache[model]; setCache solo crea la clave del modelo si el modelo aparece en algún payload (odoo_enterprise/18/stock_barcode/static/src/lazy_barcode_cache.js::setCache-49). El throw ':197' ('Model doesn't exist in the cache') solo salta en la rama sin onlyInCache/sin fetchLater. La indexación por barcode en setCache usa _getBarcodeField (:51), así que añadir el mapping cliente sí hace que setCache indexe el modelo cuando llega.
