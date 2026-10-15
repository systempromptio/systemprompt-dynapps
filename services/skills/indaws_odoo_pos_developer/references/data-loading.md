# Carga de datos al POS (eje critico)

> Citas verificadas contra v18/v19. Resolver fuente via
> `references/shared/odoo-source.md`.

Al abrir una sesion, `pos.session` recopila los datos de todos los modelos relevantes
y los envia al navegador. Cada modelo que viaja al POS hereda `pos.load.mixin`
(`models/pos_load_mixin.py`) y define que dominio/campos exporta. El orquestador es
`pos.session._load_data_params` / la cadena `_load_pos_data*` en
`models/pos_session.py`.

## FIRMAS: v18 vs v19 (DIFERENCIA CRITICA)

v19 renombra el metodo de lectura y cambia la firma de casi todos: pasa de recibir
`config_id` (int) a recibir `config` (recordset), y anade `config` a los que no lo
tenian.

| Concepto | v18 (`pos_session.py`) | v19 (`pos_session.py`) |
|----------|------------------------|------------------------|
| Lectura de datos del modelo | `_load_pos_data(self, data)` `:156` | `_load_pos_data_search_read(self, data, config)` `:159` (llamada) |
| Campos a exportar | `_load_pos_data_fields(self, config_id)` `:150` | `_load_pos_data_fields(self, config)` `:151` |
| Dominio de busqueda | `_load_pos_data_domain(self, data)` `:146` | `_load_pos_data_domain(self, data, config)` `:147` |
| Lista de modelos a cargar | `_load_pos_data_models(self, config_id)` `:138` | `_load_pos_data_models(self, config)` `:139` |
| Relaciones | `_load_pos_data_relations(self, model, response)` `:105` | `_load_pos_data_relations(self, model, fields)` `:105` |

Un override v18 portado a v19 sin ajustar la firma o no se llama o revienta por
argumentos. Confirma SIEMPRE la version primero.

## Anadir un CAMPO a un modelo ya cargado

`_load_pos_data_fields` actua de **whitelist**: solo viajan los campos que devuelve.
Anadir el campo en el modelo Python NO basta; aparece `undefined` en JS hasta que lo
incluyas aqui.

### v18

```python
class ProductTemplate(models.Model):
    _inherit = "product.template"

    def _load_pos_data_fields(self, config_id):
        fields = super()._load_pos_data_fields(config_id)
        fields += ["mi_campo"]
        return fields
```

### v19 (la firma recibe `config`, no `config_id`)

```python
    def _load_pos_data_fields(self, config):
        fields = super()._load_pos_data_fields(config)
        fields += ["mi_campo"]
        return fields
```

En JS el campo queda accesible en la instancia del modelo (`product.mi_campo`).

## Anadir un MODELO nuevo al POS (DOS sitios)

### 1. Python — que el servidor lo envie

a) El modelo hereda `pos.load.mixin` y define dominio/campos:

```python
class MiModelo(models.Model):
    _name = "mi.modelo"
    _inherit = ["mi.modelo", "pos.load.mixin"]  # o solo pos.load.mixin si nuevo

    @api.model
    def _load_pos_data_domain(self, data):            # v18; v19: (self, data, config)
        return [("active", "=", True)]

    @api.model
    def _load_pos_data_fields(self, config_id):       # v18; v19: (self, config)
        return ["id", "name", "mi_campo"]
```

b) Anadirlo a la lista de modelos a cargar (override de `pos.session`):

```python
class PosSession(models.Model):
    _inherit = "pos.session"

    def _load_pos_data_models(self, config_id):        # v18; v19: (self, config)
        models = super()._load_pos_data_models(config_id)
        models += ["mi.modelo"]
        return models
```

La lista base en v18 esta en `point_of_sale/models/pos_session.py:138-142`, `PosSession._load_pos_data_models` (incluye `pos.config`,
`pos.order`, `product.product`, `res.partner`, etc.).

### 2. JS — que el cliente lo instancie

Ver `architecture-and-store.md`: clase con `static pythonModel = "mi.modelo"` +
`registry.category("pos_available_models").add(...)`. Si falta cualquiera de los dos
lados, el modelo simplemente no existe en el cliente (silencioso).

### Qué hay registrado, por versión — medido 2026-08-17

El censo del registry **casi se duplica en v19** (10 clases → 18, y de 10 ficheros a 18):

| | Registrados |
|---|---|
| **v18** | `AccountTax` · `LoyaltyCard` · `PosCategory` · `PosOrder` · `PosOrderline` · `PosPayment` · `ProductProduct` · `ResPartner` · `RestaurantFloor` · `RestaurantTable` |
| **v19** | los mismos **menos `AccountTax`**, más **9 nuevos**: `AccountCashRounding` · `DecimalPrecision` · `PosConfig` · `PosPreset` · `ProductPricelist` · `ProductTag` · `ProductTemplate` · `ResCurrency` · `UomUom` |

> **La baja importa más que las altas: `AccountTax` desaparece del registry en v19.** Su fichero
> `point_of_sale/static/src/app/models/account_tax.js` —que en v18 hace el `add` en `:21`— **no
> existe en v19**. Si extiendes el modelo JS de impuestos por esa vía, en v19 **no hay a qué
> engancharse**, y el fallo es el de siempre en POS: **silencioso**.
>
> Y de las altas, la que más cambia el diseño es **`ProductTemplate`**
> (`app/models/product_template.js`, **inexistente en v18**): v19 lleva plantilla y variante al
> cliente, así que un código que asumía *"en POS solo hay `product.product`"* deja de ser cierto.

## AccessError se traga en SILENCIO

En v18 `point_of_sale/models/pos_session.py:182`, `PosSession.load_data`, durante la carga de cada modelo:

```python
try:
    response[model] = self.env[model]._load_pos_data(response)
except AccessError as e:
    response[model] = {'data': [], 'fields': ..., 'error': e.args[0]}
```

Si el usuario del POS no tiene permiso de lectura sobre el modelo, NO hay error
visible: llega lista vacia. Sintoma tipico: "mi modelo nuevo no carga nada". Revisa
`ir.model.access` y record rules para el grupo del operador POS.

## Delta-loading / carga incremental

Para no reenviar todo en cada apertura, el POS usa carga incremental basada en
`pos_last_server_date` (marca temporal del ultimo dato recibido): al reabrir solo se
piden los registros modificados despues. v19 endurece esto y anade `_filtered_access`
para filtrar registros por permisos antes de enviarlos. Si tu modelo cambia mucho y
quieres que el cliente vea actualizaciones, asegurate de que respeta el campo de
fecha que usa el delta-loading o se quedara con datos cacheados.

## Guardar desde el cliente

Al validar un pedido, el cliente llama `pos.order.sync_from_ui(orders)`
(`point_of_sale/models/pos_order.py:1110`, `PosOrder.sync_from_ui` v18 / `:1238` v19). Es el hook Python para post-procesar pedidos
(crear facturas, integraciones, etc.). Overridealo ahi, NO en `pos.session`.
