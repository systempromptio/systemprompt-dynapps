# Esquema YAML para Definicion de Flujos — Odoo Flow Tester

> **Version:** 1.0
> **Fecha:** 2026-03-10
> **Compatibilidad:** Odoo Flow Tester v2.1+ / Odoo 14.0-18.0

Este documento define el esquema formal YAML para que consultores definan flujos de negocio personalizados
ejecutables por el Odoo Flow Tester. El formato YAML complementa los ejemplos JSON existentes y ofrece
mayor legibilidad para definiciones complejas.

---

## Indice

1. [Esquema Completo](#1-esquema-completo)
2. [Especificacion de Campos](#2-especificacion-de-campos)
3. [Tipos de Accion](#3-tipos-de-accion)
4. [Sistema de Variables](#4-sistema-de-variables)
5. [Mapeo de Acciones a Metodos ORM](#5-mapeo-de-acciones-a-metodos-orm)
6. [Ejemplos Completos](#6-ejemplos-completos)
7. [Notas de Ejecucion](#7-notas-de-ejecucion)

---

## 1. Esquema Completo

```yaml
# ============================================================
# METADATA — Identificacion del flujo
# ============================================================
flow_id: string                    # REQUERIDO. Formato: CHAIN_<AREA>_<DESC> (UPPER_SNAKE_CASE)
flow_name: string                  # REQUERIDO. Nombre legible del flujo
description: string                # REQUERIDO. Descripcion funcional del flujo completo
version: string                    # OPCIONAL. Version del flujo (ej: "1.0", "2.1")
client: string                     # OPCIONAL. Nombre del cliente para el que se definio
target_version: string             # REQUERIDO. Version de Odoo destino (ej: "18.0")
environment: enum                  # REQUERIDO. "staging" | "production"

# ============================================================
# VARIABLES — Valores reutilizables en el flujo
# ============================================================
variables:
  <nombre>: <valor>                # Pares clave-valor referenciables como {var.<nombre>}

# ============================================================
# DATA — Datos de entrada del flujo
# ============================================================
data:
  <campo>: <valor>                 # Datos de entrada referenciables como {data.<campo>}

# ============================================================
# PRECONDITIONS — Requisitos previos a la ejecucion
# ============================================================
preconditions:
  required_modules:                # OPCIONAL. Lista de modulos que deben estar instalados
    - string

  required_data:                   # OPCIONAL. Lista de busquedas de datos existentes
    - model: string                # REQUERIDO. Modelo Odoo (ej: "res.partner")
      domain: list                 # REQUERIDO. Dominio de busqueda Odoo
      fields: list                 # REQUERIDO. Campos a leer del registro encontrado
      save_as: string              # REQUERIDO. Nombre de variable para guardar el ID
      min_records: integer         # OPCIONAL. Minimo de registros esperados (default: 1)
      description: string          # OPCIONAL. Descripcion de que se busca

# ============================================================
# STEPS — Pasos de ejecucion del flujo
# ============================================================
steps:
  - step: number                   # REQUERIDO. Numero de paso (soporta decimales: 3.1, 3.2)
    name: string                   # REQUERIDO. Descripcion legible del paso
    action: enum                   # REQUERIDO. Tipo de accion (ver seccion 3)
    model: string                  # REQUERIDO. Modelo Odoo objetivo
    data: map                      # CONDICIONAL. Valores para create/update
    method: string                 # CONDICIONAL. Nombre del metodo (para execute/custom_method)
    ids: list                      # CONDICIONAL. IDs de registros objetivo
    domain: list                   # CONDICIONAL. Dominio de busqueda (para search)
    fields: list                   # CONDICIONAL. Campos a leer (para search)
    params: map                    # OPCIONAL. Parametros adicionales del metodo
    context: map                   # OPCIONAL. Contexto Odoo para la operacion
    save_as: string                # OPCIONAL. Guardar resultado en variable
    wait_for: list                 # OPCIONAL. Lista de step numbers que deben completarse antes
    on_error: enum                 # OPCIONAL. "abort" (default) | "skip" | "retry"
    retry_count: integer           # OPCIONAL. Intentos de reintento si on_error=retry (default: 3)
    retry_delay: integer           # OPCIONAL. Segundos entre reintentos (default: 5)
    skip_if: map                   # OPCIONAL. Condicion para saltar el paso
    timeout: integer               # OPCIONAL. Timeout en segundos (default: 30)
    note: string                   # OPCIONAL. Notas tecnicas para el consultor
    detect_cascade: map            # OPCIONAL. Detectar registros creados automaticamente
    operations: list               # CONDICIONAL. Lista de operaciones (para multi_update)

    # Assertions inline del paso
    assertions:
      - field: string              # REQUERIDO. Campo a verificar
        expected: any              # CONDICIONAL. Valor esperado exacto
        expected_any: list         # CONDICIONAL. Lista de valores aceptables
        operator: enum             # OPCIONAL. "=" (default) | "!=" | ">" | "<" | "in" | "not_in"
        model: string              # OPCIONAL. Modelo alternativo (si difiere del paso)
        id: string                 # OPCIONAL. ID alternativo (si difiere del paso)

# ============================================================
# ASSERTIONS — Verificaciones finales del flujo
# ============================================================
assertions:
  - model: string                  # REQUERIDO. Modelo a verificar
    id: string                     # REQUERIDO. ID del registro (usa variables)
    field: string                  # REQUERIDO. Campo a verificar
    expected: any                  # CONDICIONAL. Valor esperado exacto
    expected_any: list             # CONDICIONAL. Lista de valores aceptables
    operator: enum                 # OPCIONAL. "=" | "!=" | ">" | "<" | "in" | "not_in"
    description: string            # OPCIONAL. Descripcion de la verificacion

# ============================================================
# REVERT — Plan de reversion (orden cronologico inverso)
# ============================================================
revert:
  - step: number                   # REQUERIDO. Referencia al step que se revierte
    action: enum                   # REQUERIDO. Accion de reversion
    model: string                  # REQUERIDO. Modelo objetivo
    method: string                 # CONDICIONAL. Metodo a ejecutar
    ids: list                      # CONDICIONAL. IDs de registros
    context: map                   # OPCIONAL. Contexto Odoo
    note: string                   # OPCIONAL. Instrucciones especiales

# ============================================================
# CLEANUP — Estrategia de limpieza
# ============================================================
cleanup:
  strategy: enum                   # REQUERIDO. "auto" | "manual" | "sql_fallback"
  neutralize_only: boolean         # OPCIONAL. Solo cancelar, no eliminar (default: false)
  recover_sequences: boolean       # OPCIONAL. Restaurar secuencias (default: false)
  prefix_field_map:                # OPCIONAL. Mapeo modelo→campo para busqueda por prefijo
    <modelo>: <campo>
```

---

## 2. Especificacion de Campos

### 2.1 Metadata

| Campo | Tipo | Requerido | Validacion | Ejemplo |
|-------|------|-----------|------------|---------|
| `flow_id` | string | Si | `^CHAIN_[A-Z][A-Z0-9_]+$` | `CHAIN_SALE_TO_CASH` |
| `flow_name` | string | Si | 5-100 caracteres | `"Venta nacional completa"` |
| `description` | string | Si | 10-500 caracteres | `"Crear pedido, confirmar, entregar, facturar y cobrar"` |
| `version` | string | No | Semver o `X.Y` | `"1.0"` |
| `client` | string | No | Texto libre | `"Muebles Valencia S.L."` |
| `target_version` | string | Si | `^(14|15|16|17|18|19)\.0$` | `"18.0"` |
| `environment` | enum | Si | `staging` o `production` | `staging` |

### 2.2 Variables

Seccion de variables reutilizables. Se referencian como `{var.<nombre>}` en cualquier campo del flujo.

```yaml
variables:
  test_qty: 5
  test_price: 450.00
  warehouse_name: "Almacen Principal"
  fiscal_position: "Regimen General"
```

**Tipos soportados:** string, integer, float, boolean, list.

### 2.3 Data

Datos de entrada del flujo. Se referencian como `{data.<campo>}`. A diferencia de `variables`, los datos
representan la informacion especifica del escenario de test.

```yaml
data:
  partner_name: "Muebles Valencia S.L."
  product_name: "Mesa Oficina Roble 160x80"
  qty: 5
  price_unit: 450.00
```

### 2.4 Preconditions

#### required_modules

Lista de technical names de modulos Odoo que deben estar instalados. El tester verifica via
`ir.module.module` con `state = 'installed'`.

```yaml
preconditions:
  required_modules:
    - sale_management
    - stock
    - account
```

#### required_data

Cada entrada busca registros existentes y guarda su ID en una variable `{pre.<save_as>}`.

| Campo | Tipo | Requerido | Descripcion |
|-------|------|-----------|-------------|
| `model` | string | Si | Modelo Odoo a buscar |
| `domain` | list | Si | Dominio de busqueda `[[campo, operador, valor], ...]` |
| `fields` | list | Si | Campos a leer (siempre incluir `id`) |
| `save_as` | string | Si | Nombre de variable (`{pre.<save_as>}` en pasos) |
| `min_records` | integer | No | Minimo de registros esperados (default: 1) |
| `description` | string | No | Descripcion legible de la precondicion |

**Busqueda con referencia (search domain):**

```yaml
required_data:
  - model: res.partner
    domain:
      - ["name", "ilike", "Muebles Valencia"]
      - ["customer_rank", ">", 0]
    fields: ["id", "name"]
    save_as: partner_id
    min_records: 1
    description: "Cliente para el pedido de venta"
```

### 2.5 Steps

Cada paso define una operacion atomica. Los campos requeridos dependen del tipo de `action`.

#### Campos comunes a todas las acciones

| Campo | Tipo | Requerido | Default | Descripcion |
|-------|------|-----------|---------|-------------|
| `step` | number | Si | — | Numero de paso. Soporta decimales para sub-pasos (3.1, 3.2) |
| `name` | string | Si | — | Descripcion legible |
| `action` | enum | Si | — | Tipo de accion (ver seccion 3) |
| `model` | string | Si | — | Modelo Odoo objetivo |
| `save_as` | string | No | — | Guardar ID del resultado en `{result.<save_as>}` |
| `wait_for` | list | No | `[]` | Steps que deben completarse antes: `[1, 2.1]` |
| `on_error` | enum | No | `abort` | Comportamiento ante error: `abort`, `skip`, `retry` |
| `retry_count` | integer | No | 3 | Intentos si `on_error: retry` |
| `retry_delay` | integer | No | 5 | Segundos entre reintentos |
| `skip_if` | map | No | — | Condicion para saltar el paso |
| `timeout` | integer | No | 30 | Timeout en segundos |
| `context` | map | No | `{}` | Contexto Odoo adicional (se fusiona con contexto base) |
| `note` | string | No | — | Notas tecnicas |

#### skip_if

Permite saltar un paso condicionalmente basandose en el estado de un registro previo:

```yaml
skip_if:
  field: state
  not_in: ["draft"]          # Saltar si el estado NO esta en esta lista
```

Operadores soportados en `skip_if`: `equals`, `not_equals`, `in`, `not_in`, `gt`, `lt`.

#### detect_cascade

Detecta registros creados automaticamente por Odoo como efecto secundario de una operacion:

```yaml
detect_cascade:
  model: account.move
  domain:
    - ["invoice_origin", "=", "{result.sale_order_name}"]
  save_as: invoice_id
```

### 2.6 Assertions

Verificaciones que se ejecutan despues de un paso o al final del flujo.

| Campo | Tipo | Requerido | Default | Descripcion |
|-------|------|-----------|---------|-------------|
| `model` | string | Si (global) / No (inline) | Modelo del paso | Modelo a verificar |
| `id` | string | Si (global) / No (inline) | ID del paso | ID del registro |
| `field` | string | Si | — | Campo a verificar |
| `expected` | any | Condicional | — | Valor esperado exacto |
| `expected_any` | list | Condicional | — | Lista de valores aceptables |
| `operator` | enum | No | `=` | Operador de comparacion |
| `description` | string | No | — | Descripcion legible |

**Operadores:**

| Operador | Descripcion | Ejemplo |
|----------|-------------|---------|
| `=` | Igualdad exacta | `expected: "sale"` |
| `!=` | Diferente de | `expected: "cancel"` |
| `>` | Mayor que | `expected: 0` |
| `<` | Menor que | `expected: 1000` |
| `in` | Contenido en lista | `expected: ["sale", "done"]` |
| `not_in` | No contenido en lista | `expected: ["cancel", "draft"]` |

**Nota:** Usar `expected` con `operator: =` o `expected_any` (sin operador) son equivalentes para
verificar que un valor esta en una lista.

### 2.7 Revert

Plan de reversion en orden cronologico **inverso** al flujo. Cada entrada deshace un paso.

```yaml
revert:
  - step: 8
    action: execute
    model: account.payment
    method: action_cancel
    ids: ["{result.payment_id}"]
  - step: 7
    action: delete
    model: account.payment
    ids: ["{result.payment_id}"]
```

**Acciones especiales de reversion:**

| Accion | Uso |
|--------|-----|
| `return_picking` | Crea devolucion via wizard `stock.return.picking` (pickings `done`) |
| `reverse_move` | Crea asiento inverso via `_reverse_moves()` (moves con hash fiscal) |

### 2.8 Cleanup

| Campo | Tipo | Requerido | Default | Descripcion |
|-------|------|-----------|---------|-------------|
| `strategy` | enum | Si | — | Estrategia de limpieza |
| `neutralize_only` | boolean | No | `false` | Solo cancelar, no eliminar registros |
| `recover_sequences` | boolean | No | `false` | Restaurar valores de secuencias `ir.sequence` |
| `prefix_field_map` | map | No | — | Mapeo modelo→campo para busqueda por prefijo |

**Estrategias:**

| Estrategia | Descripcion |
|------------|-------------|
| `auto` | Ejecutar reversion automatica usando el plan `revert` definido (Tier 1 ORM) |
| `manual` | Solo generar plan de reversion, no ejecutar automaticamente |
| `sql_fallback` | Intentar ORM primero, escalar a SQL si falla (Tier 1 → Tier 2 → Tier 3) |

---

## 3. Tipos de Accion

### 3.1 create

Crea un nuevo registro en el modelo especificado.

```yaml
- step: 1
  name: "Crear pedido de venta"
  action: create
  model: sale.order
  data:
    partner_id: "{pre.partner_id}"
    client_order_ref: "{data.test_prefix}"
    order_line:
      - [0, 0, {product_id: "{pre.product_id}", product_uom_qty: "{data.qty}", price_unit: "{data.price_unit}"}]
  save_as: sale_order_id
  assertions:
    - field: state
      expected: "draft"
```

**Campos requeridos:** `model`, `data`
**Campos opcionales:** `save_as`, `context`

### 3.2 confirm

Atajo para ejecutar el metodo de confirmacion estandar del modelo.

```yaml
- step: 2
  name: "Confirmar pedido"
  action: confirm
  model: sale.order
  ids: ["{result.sale_order_id}"]
  assertions:
    - field: state
      expected: "sale"
```

**Campos requeridos:** `model`, `ids`
**Metodo resuelto:** Ver tabla de mapeo en seccion 5.

### 3.3 cancel

Atajo para ejecutar el metodo de cancelacion estandar del modelo.

```yaml
- step: 9
  name: "Cancelar pedido"
  action: cancel
  model: sale.order
  ids: ["{result.sale_order_id}"]
  context:
    disable_cancel_warning: true
  assertions:
    - field: state
      expected: "cancel"
```

**Campos requeridos:** `model`, `ids`

### 3.4 deliver

Atajo para la secuencia completa de validacion de albaran (assign + set quantities + validate).

```yaml
- step: 4
  name: "Validar albaran de salida"
  action: deliver
  model: stock.picking
  ids: ["{result.picking_id}"]
  params:
    quantity_field: "quantity"        # "quantity" para Odoo 18+, "qty_done" para <17
    skip_backorder: true
  assertions:
    - field: state
      expected: "done"
```

**Campos requeridos:** `model`, `ids`
**Expansion interna:** El tester expande `deliver` a los sub-pasos: `action_assign` → set `move_line_ids.quantity` + `move_ids.picked` → `button_validate`

### 3.5 invoice

Atajo para crear factura desde un pedido (venta o compra).

```yaml
- step: 5
  name: "Crear factura desde pedido"
  action: invoice
  model: sale.order
  ids: ["{result.sale_order_id}"]
  detect_cascade:
    model: account.move
    domain:
      - ["invoice_origin", "=", "{result.sale_order_name}"]
    save_as: invoice_id
```

**Campos requeridos:** `model`, `ids`
**Metodo resuelto:** `_create_invoices` para `sale.order`, `action_create_invoice` para `purchase.order`

### 3.6 pay

Atajo para crear y confirmar un pago asociado a una factura.

```yaml
- step: 7
  name: "Registrar cobro"
  action: pay
  model: account.move
  ids: ["{result.invoice_id}"]
  params:
    payment_type: "inbound"
    partner_type: "customer"
    journal_id: "{pre.bank_journal_id}"
  save_as: payment_id
```

**Campos requeridos:** `model`, `ids`, `params.payment_type`, `params.partner_type`, `params.journal_id`
**Expansion interna:** Crea `account.payment` con los datos de la factura + params, luego ejecuta `action_post`.

### 3.7 custom_method

Ejecuta cualquier metodo arbitrario del modelo. Es la accion mas flexible.

```yaml
- step: 6
  name: "Publicar factura"
  action: custom_method
  model: account.move
  method: action_post
  ids: ["{result.invoice_id}"]
  params:
    args: []
    kwargs: {}
  context:
    skip_invoice_sync: true
  assertions:
    - field: state
      expected: "posted"
```

**Campos requeridos:** `model`, `method`, `ids`
**Campos opcionales:** `params`, `context`

### 3.8 Acciones adicionales (compatibilidad con JSON)

Las siguientes acciones son compatibles con el formato JSON existente y se soportan directamente:

| Accion | Uso | Campos requeridos |
|--------|-----|-------------------|
| `execute` | Ejecutar metodo (equivalente a `custom_method`) | `model`, `method`, `ids` |
| `search` | Buscar registros por dominio | `model`, `domain`, `fields` |
| `update` | Actualizar campos de un registro | `model`, `ids`, `data` |
| `delete` | Eliminar registro | `model`, `ids` |
| `multi_update` | Actualizar multiples modelos en un paso | `operations` |
| `return_picking` | Crear devolucion via wizard | `model`, `ids` |
| `scrap` | Crear y validar desguace | `model`, `ids` |

---

## 4. Sistema de Variables

### 4.1 Prefijos de variables

| Prefijo | Origen | Ejemplo | Disponible desde |
|---------|--------|---------|------------------|
| `{data.*}` | Seccion `data` del flujo | `{data.qty}` → `5` | Inicio |
| `{var.*}` | Seccion `variables` del flujo | `{var.test_qty}` → `5` | Inicio |
| `{pre.*}` | IDs de `preconditions.required_data` | `{pre.partner_id}` → `42` | Despues de precondiciones |
| `{result.*}` | IDs y campos de registros creados | `{result.sale_order_id}` → `156` | Despues del paso que usa `save_as` |
| `{result.*_name}` | Campo `name` del registro guardado | `{result.sale_order_name}` → `"S00042"` | Auto-resuelto con `save_as` |
| `{result.*_amount_total}` | Campo `amount_total` si existe | `{result.sale_order_amount_total}` → `2250.0` | Auto-resuelto |
| `{today}` | Fecha actual | `{today}` → `"2026-03-10"` | Siempre |
| `{env.*}` | Variables de entorno del sistema | `{env.ODOO_URL}` | Siempre |

### 4.2 Resolucion de variables

Las variables se resuelven en tiempo de ejecucion de forma **lazy** (solo cuando se necesitan).
El orden de resolucion es:

1. `{today}` y `{env.*}` — resueltos al inicio
2. `{data.*}` y `{var.*}` — resueltos al inicio desde las secciones del flujo
3. `{pre.*}` — resueltos despues de ejecutar precondiciones
4. `{result.*}` — resueltos conforme se ejecutan los pasos

### 4.3 Referencia por busqueda (search reference)

Dentro de los campos `data` de un paso, se puede usar una referencia de busqueda para resolver
un ID dinamicamente:

```yaml
data:
  partner_id:
    search: "res.partner"
    domain: [["name", "=", "Muebles Valencia S.L."]]
  warehouse_id:
    search: "stock.warehouse"
    domain: [["name", "ilike", "Principal"]]
```

Esto es equivalente a definir una precondicion, pero inline dentro del paso.

---

## 5. Mapeo de Acciones a Metodos ORM

### 5.1 Tabla de mapeo principal

| Accion YAML | Modelo | Metodo ORM / Script | Notas |
|-------------|--------|---------------------|-------|
| `create` | (cualquiera) | `models.execute_kw(model, 'create', [values])` / `create_record.sh` | Retorna ID |
| `confirm` | `sale.order` | `action_confirm` | — |
| `confirm` | `purchase.order` | `button_confirm` | NO `action_confirm` |
| `confirm` | `account.move` | `action_post` | "Publicar" = confirmar en contabilidad |
| `confirm` | `account.payment` | `action_post` | — |
| `confirm` | `mrp.production` | `action_confirm` | — |
| `confirm` | `hr.expense.sheet` | `action_submit_sheet` | Submit para aprobacion |
| `confirm` | `stock.picking` | `action_confirm` | Solo draft → confirmed |
| `cancel` | `sale.order` | `action_cancel` | Requiere ctx `disable_cancel_warning` |
| `cancel` | `purchase.order` | `button_cancel` | — |
| `cancel` | `account.move` | `button_draft` + `button_cancel` | Dos pasos |
| `cancel` | `account.payment` | `action_cancel` | — |
| `cancel` | `mrp.production` | `action_cancel` | auto-cancel en `unlink()` en 18.0 |
| `deliver` | `stock.picking` | `action_assign` → set qty → `button_validate` | Expansion multi-paso |
| `invoice` | `sale.order` | `_create_invoices` | Metodo privado, funcional via XML-RPC |
| `invoice` | `purchase.order` | `action_create_invoice` | Metodo publico |
| `pay` | `account.move` | Crea `account.payment` + `action_post` | Expansion multi-paso |
| `custom_method` | (cualquiera) | Metodo arbitrario | `execute_method.sh` |
| `execute` | (cualquiera) | Metodo arbitrario (alias de `custom_method`) | Compatibilidad JSON |
| `search` | (cualquiera) | `models.execute_kw(model, 'search_read', [domain])` / `search_records.sh` | — |
| `update` | (cualquiera) | `models.execute_kw(model, 'write', [ids, values])` / `update_record.sh` | — |
| `delete` | (cualquiera) | `models.execute_kw(model, 'unlink', [ids])` / `delete_record.sh` | — |
| `multi_update` | (multiples) | Multiples `write` secuenciales | — |
| `return_picking` | `stock.picking` | Wizard `stock.return.picking` | Para pickings `done` |
| `scrap` | `stock.scrap` | `create` + `action_validate` | — |

### 5.2 Diferencias por version de Odoo

| Modelo | Metodo | 14.0-16.0 | 17.0+ | 18.0 |
|--------|--------|-----------|-------|------|
| `stock.move.line` | Campo cantidad | `qty_done` | `quantity` | `quantity` |
| `stock.move` | Campo procesado | — | `picked` | `picked` |
| `product.product` | Tipo almacenable | `type='product'` | **`type='product'`** (el corte es v18, no v17) | `is_storable=True` |
| `hr.expense.sheet` | Contabilizar | `action_sheet_move_create` | — | `action_sheet_move_post` (**y en 19.0 el modelo ya no existe: `hr.expense.action_post`**) |
| `hr.leave` | Volver a borrador | `action_draft` | — | `action_reset_confirm` |
| `sale.order` (suscripcion) | Recurrencia | `recurrence_id` | — | `plan_id` |

Consultar `references/api_compatibility.md` para la matriz completa.

---

## 6. Ejemplos Completos

### 6.1 Ejemplo A — Pedido de venta simple

Flujo basico: crear pedido → confirmar → verificar.

```yaml
flow_id: CHAIN_SALE_SIMPLE
flow_name: "Pedido de venta simple"
description: "Crear y confirmar un pedido de venta con una linea de producto"
target_version: "18.0"
environment: staging

variables:
  default_qty: 3

data:
  partner_name: "Distribuciones Garcia S.L."
  product_name: "Silla Ergonomica PRO"
  qty: 3
  price_unit: 189.00

preconditions:
  required_modules:
    - sale_management

  required_data:
    - model: res.partner
      domain:
        - ["name", "ilike", "Distribuciones Garcia"]
        - ["customer_rank", ">", 0]
      fields: ["id", "name"]
      save_as: partner_id
      min_records: 1

    - model: product.product
      domain:
        - ["name", "ilike", "Silla Ergonomica"]
        - ["sale_ok", "=", true]
      fields: ["id", "name", "list_price"]
      save_as: product_id
      min_records: 1

steps:
  - step: 1
    name: "Crear pedido de venta"
    action: create
    model: sale.order
    data:
      partner_id: "{pre.partner_id}"
      client_order_ref: "{data.test_prefix}"
      order_line:
        - [0, 0, {product_id: "{pre.product_id}", product_uom_qty: "{data.qty}", price_unit: "{data.price_unit}"}]
    save_as: sale_order_id
    assertions:
      - field: state
        expected: "draft"

  - step: 2
    name: "Confirmar pedido"
    action: confirm
    model: sale.order
    ids: ["{result.sale_order_id}"]
    assertions:
      - field: state
        expected: "sale"
      - field: amount_total
        operator: ">"
        expected: 0

assertions:
  - model: sale.order
    id: "{result.sale_order_id}"
    field: invoice_status
    expected: "to invoice"
    description: "El pedido confirmado debe estar pendiente de facturar"

revert:
  - step: 2
    action: execute
    model: sale.order
    method: action_cancel
    ids: ["{result.sale_order_id}"]
    context:
      disable_cancel_warning: true
  - step: 1
    action: delete
    model: sale.order
    ids: ["{result.sale_order_id}"]

cleanup:
  strategy: auto
```

---

### 6.2 Ejemplo B — Venta completa con entrega parcial (Sale-to-Cash)

Flujo completo: pedido → confirmar → entrega parcial (3 de 5) → facturar entregado → cobrar.

```yaml
flow_id: CHAIN_SALE_PARTIAL_DELIVERY
flow_name: "Venta con entrega parcial"
description: "Pedido de venta completo con entrega parcial, facturacion de lo entregado y cobro"
version: "1.0"
client: "Muebles Valencia S.L."
target_version: "18.0"
environment: staging

variables:
  total_qty: 5
  delivered_qty: 3

data:
  partner_name: "Muebles Valencia S.L."
  product_name: "Mesa Oficina Roble 160x80"
  qty: 5
  partial_qty: 3
  price_unit: 450.00

preconditions:
  required_modules:
    - sale_management
    - stock
    - account

  required_data:
    - model: res.partner
      domain:
        - ["name", "ilike", "Muebles Valencia"]
        - ["customer_rank", ">", 0]
      fields: ["id", "name"]
      save_as: partner_id
      min_records: 1

    - model: product.product
      domain:
        - ["name", "ilike", "Mesa Oficina Roble"]
        - ["sale_ok", "=", true]
      fields: ["id", "name", "list_price"]
      save_as: product_id
      min_records: 1

    - model: account.journal
      domain: [["type", "=", "sale"]]
      fields: ["id", "name"]
      save_as: sale_journal_id
      min_records: 1

    - model: account.journal
      domain: [["type", "=", "bank"]]
      fields: ["id", "name"]
      save_as: bank_journal_id
      min_records: 1

steps:
  - step: 1
    name: "Crear pedido de venta"
    action: create
    model: sale.order
    data:
      partner_id: "{pre.partner_id}"
      client_order_ref: "{data.test_prefix}"
      order_line:
        - [0, 0, {product_id: "{pre.product_id}", product_uom_qty: "{data.qty}", price_unit: "{data.price_unit}"}]
    save_as: sale_order_id
    assertions:
      - field: state
        expected: "draft"

  - step: 2
    name: "Confirmar pedido"
    action: confirm
    model: sale.order
    ids: ["{result.sale_order_id}"]
    assertions:
      - field: state
        expected: "sale"

  - step: 3
    name: "Detectar albaran de salida"
    action: search
    model: stock.picking
    domain:
      - ["origin", "=", "{result.sale_order_name}"]
      - ["picking_type_code", "=", "outgoing"]
    fields: ["id", "name", "state", "move_ids", "move_line_ids"]
    save_as: picking_id
    assertions:
      - field: state
        expected_any: ["confirmed", "assigned"]

  - step: 3.1
    name: "Reservar stock"
    action: execute
    model: stock.picking
    method: action_assign
    ids: ["{result.picking_id}"]
    assertions:
      - field: state
        expected: "assigned"

  - step: 3.2
    name: "Establecer cantidad parcial (3 de 5)"
    action: multi_update
    operations:
      - model: stock.move.line
        ids: ["{result.picking_move_line_ids}"]
        data:
          quantity: "{data.partial_qty}"
      - model: stock.move
        ids: ["{result.picking_move_ids}"]
        data:
          picked: true
    note: "Odoo 18: campo 'quantity' en move.line (NO 'qty_done'). 'picked' en move indica procesado."

  - step: 4
    name: "Validar albaran (entrega parcial, sin backorder)"
    action: execute
    model: stock.picking
    method: button_validate
    ids: ["{result.picking_id}"]
    context:
      skip_backorder: true
      picking_ids_not_to_backorder: ["{result.picking_id}"]
    assertions:
      - field: state
        expected: "done"

  - step: 5
    name: "Crear factura de lo entregado"
    action: invoice
    model: sale.order
    ids: ["{result.sale_order_id}"]
    detect_cascade:
      model: account.move
      domain:
        - ["invoice_origin", "=", "{result.sale_order_name}"]
      save_as: invoice_id

  - step: 6
    name: "Publicar factura"
    action: custom_method
    model: account.move
    method: action_post
    ids: ["{result.invoice_id}"]
    assertions:
      - field: state
        expected: "posted"
      - field: amount_total
        expected: 1350.00
        description: "3 unidades x 450 EUR = 1350 EUR"

  - step: 7
    name: "Registrar cobro"
    action: pay
    model: account.move
    ids: ["{result.invoice_id}"]
    params:
      payment_type: "inbound"
      partner_type: "customer"
      journal_id: "{pre.bank_journal_id}"
    save_as: payment_id

assertions:
  - model: sale.order
    id: "{result.sale_order_id}"
    field: delivery_status
    expected_any: ["partial", "full"]
    description: "El pedido debe tener entrega parcial o completa"

  - model: account.payment
    id: "{result.payment_id}"
    field: state
    expected_any: ["in_process", "paid"]
    description: "El pago debe estar procesado"

revert:
  - step: 7
    action: execute
    model: account.payment
    method: action_cancel
    ids: ["{result.payment_id}"]
  - step: 7
    action: delete
    model: account.payment
    ids: ["{result.payment_id}"]
  - step: 6
    action: execute
    model: account.move
    method: button_draft
    ids: ["{result.invoice_id}"]
  - step: 5
    action: execute
    model: account.move
    method: button_cancel
    ids: ["{result.invoice_id}"]
  - step: 5
    action: delete
    model: account.move
    ids: ["{result.invoice_id}"]
  - step: 4
    action: return_picking
    model: stock.picking
    ids: ["{result.picking_id}"]
  - step: 2
    action: execute
    model: sale.order
    method: action_cancel
    ids: ["{result.sale_order_id}"]
    context:
      disable_cancel_warning: true
  - step: 1
    action: delete
    model: sale.order
    ids: ["{result.sale_order_id}"]

cleanup:
  strategy: auto
  recover_sequences: true
  prefix_field_map:
    sale.order: client_order_ref
    account.move: ref
    account.payment: ref
```

---

### 6.3 Ejemplo C — Compra completa (Purchase-to-Pay)

Flujo completo: orden de compra → confirmar → recibir mercancia → factura proveedor → pagar.

```yaml
flow_id: CHAIN_PURCHASE_TO_PAY
flow_name: "Compra nacional completa"
description: "Orden de compra, recepcion de mercancia, factura de proveedor y pago"
target_version: "18.0"
environment: staging

data:
  supplier_name: "Suministros Industriales Martinez S.L."
  product_name: "Tornillo Hexagonal M10x50 Inox"
  qty: 500
  price_unit: 0.45

preconditions:
  required_modules:
    - purchase
    - stock
    - account

  required_data:
    - model: res.partner
      domain:
        - ["name", "ilike", "Suministros Industriales Martinez"]
        - ["supplier_rank", ">", 0]
      fields: ["id", "name"]
      save_as: supplier_id
      min_records: 1

    - model: product.product
      domain:
        - ["name", "ilike", "Tornillo Hexagonal M10"]
        - ["purchase_ok", "=", true]
      fields: ["id", "name", "standard_price"]
      save_as: product_id
      min_records: 1

    - model: account.journal
      domain: [["type", "=", "purchase"]]
      fields: ["id", "name"]
      save_as: purchase_journal_id
      min_records: 1

    - model: account.journal
      domain: [["type", "=", "bank"]]
      fields: ["id", "name"]
      save_as: bank_journal_id
      min_records: 1

steps:
  - step: 1
    name: "Crear orden de compra"
    action: create
    model: purchase.order
    data:
      partner_id: "{pre.supplier_id}"
      partner_ref: "{data.test_prefix}"
      order_line:
        - [0, 0, {product_id: "{pre.product_id}", product_qty: "{data.qty}", price_unit: "{data.price_unit}"}]
    save_as: purchase_order_id
    assertions:
      - field: state
        expected: "draft"

  - step: 2
    name: "Confirmar orden de compra"
    action: confirm
    model: purchase.order
    ids: ["{result.purchase_order_id}"]
    assertions:
      - field: state
        expected: "purchase"
    note: "purchase.order usa button_confirm (NO action_confirm)"

  - step: 3
    name: "Detectar albaran de recepcion"
    action: search
    model: stock.picking
    domain:
      - ["origin", "=", "{result.purchase_order_name}"]
      - ["picking_type_code", "=", "incoming"]
    fields: ["id", "name", "state", "move_ids", "move_line_ids"]
    save_as: picking_id
    assertions:
      - field: state
        expected_any: ["confirmed", "assigned"]

  - step: 3.1
    name: "Reservar stock de recepcion"
    action: execute
    model: stock.picking
    method: action_assign
    ids: ["{result.picking_id}"]
    assertions:
      - field: state
        expected: "assigned"

  - step: 3.2
    name: "Establecer cantidades recibidas"
    action: multi_update
    operations:
      - model: stock.move.line
        ids: ["{result.picking_move_line_ids}"]
        data:
          quantity: "{data.qty}"
      - model: stock.move
        ids: ["{result.picking_move_ids}"]
        data:
          picked: true
    note: "Odoo 18: 'quantity' en move.line (NO 'qty_done'). 'picked' en move indica procesado."

  - step: 4
    name: "Validar recepcion"
    action: execute
    model: stock.picking
    method: button_validate
    ids: ["{result.picking_id}"]
    context:
      skip_backorder: true
      picking_ids_not_to_backorder: ["{result.picking_id}"]
    assertions:
      - field: state
        expected: "done"

  - step: 5
    name: "Crear factura de proveedor"
    action: invoice
    model: purchase.order
    ids: ["{result.purchase_order_id}"]
    detect_cascade:
      model: account.move
      domain:
        - ["ref", "=", "{result.purchase_order_name}"]
        - ["move_type", "=", "in_invoice"]
      save_as: invoice_id
    note: "purchase.order usa action_create_invoice (metodo publico)"

  - step: 6
    name: "Publicar factura de proveedor"
    action: custom_method
    model: account.move
    method: action_post
    ids: ["{result.invoice_id}"]
    assertions:
      - field: state
        expected: "posted"

  - step: 7
    name: "Crear pago a proveedor"
    action: create
    model: account.payment
    data:
      payment_type: "outbound"
      partner_type: "supplier"
      partner_id: "{pre.supplier_id}"
      amount: "{result.purchase_order_amount_total}"
      journal_id: "{pre.bank_journal_id}"
      ref: "{data.test_prefix}"
      date: "{today}"
    save_as: payment_id
    assertions:
      - field: state
        expected: "draft"

  - step: 8
    name: "Confirmar pago"
    action: confirm
    model: account.payment
    ids: ["{result.payment_id}"]
    assertions:
      - field: state
        expected_any: ["in_process", "paid"]

revert:
  - step: 8
    action: execute
    model: account.payment
    method: action_cancel
    ids: ["{result.payment_id}"]
  - step: 7
    action: delete
    model: account.payment
    ids: ["{result.payment_id}"]
  - step: 6
    action: execute
    model: account.move
    method: button_draft
    ids: ["{result.invoice_id}"]
  - step: 5
    action: execute
    model: account.move
    method: button_cancel
    ids: ["{result.invoice_id}"]
  - step: 5
    action: delete
    model: account.move
    ids: ["{result.invoice_id}"]
  - step: 4
    action: return_picking
    model: stock.picking
    ids: ["{result.picking_id}"]
  - step: 2
    action: execute
    model: purchase.order
    method: button_cancel
    ids: ["{result.purchase_order_id}"]
  - step: 1
    action: delete
    model: purchase.order
    ids: ["{result.purchase_order_id}"]

cleanup:
  strategy: auto
  prefix_field_map:
    purchase.order: partner_ref
    account.move: ref
    account.payment: ref
```

---

### 6.4 Ejemplo D — Flujo personalizado con pasos no estandar

Flujo custom para un cliente que requiere crear un presupuesto con descuento especial,
aprobacion manual mediante un metodo custom, y envio de confirmacion.

```yaml
flow_id: CHAIN_CUSTOM_APPROVAL_SALE
flow_name: "Venta con aprobacion de descuento"
description: "Presupuesto con descuento >20% que requiere aprobacion del director comercial via modulo custom indaws_sale_approval"
version: "1.0"
client: "Textiles Levante S.A."
target_version: "18.0"
environment: staging

variables:
  discount_threshold: 20
  approval_user_name: "Director Comercial"

data:
  partner_name: "Textiles Levante S.A."
  product_name: "Tela Algodon Premium 280g/m2"
  qty: 200
  price_unit: 12.50
  discount: 25.0
  approval_note: "Descuento autorizado por volumen >150 metros"

preconditions:
  required_modules:
    - sale_management
    - indaws_sale_approval

  required_data:
    - model: res.partner
      domain:
        - ["name", "ilike", "Textiles Levante"]
        - ["customer_rank", ">", 0]
      fields: ["id", "name"]
      save_as: partner_id
      min_records: 1

    - model: product.product
      domain:
        - ["name", "ilike", "Tela Algodon Premium"]
        - ["sale_ok", "=", true]
      fields: ["id", "name", "list_price"]
      save_as: product_id
      min_records: 1

    - model: res.users
      domain:
        - ["name", "ilike", "{var.approval_user_name}"]
        - ["active", "=", true]
      fields: ["id", "name"]
      save_as: approver_id
      min_records: 1
      description: "Usuario con permisos de aprobacion de descuentos"

    - model: account.journal
      domain: [["type", "=", "sale"]]
      fields: ["id", "name"]
      save_as: sale_journal_id
      min_records: 1

steps:
  - step: 1
    name: "Crear presupuesto con descuento alto"
    action: create
    model: sale.order
    data:
      partner_id: "{pre.partner_id}"
      client_order_ref: "{data.test_prefix}"
      order_line:
        - [0, 0, {
            product_id: "{pre.product_id}",
            product_uom_qty: "{data.qty}",
            price_unit: "{data.price_unit}",
            discount: "{data.discount}"
          }]
    save_as: sale_order_id
    assertions:
      - field: state
        expected: "draft"
      - field: x_approval_required
        expected: true
        description: "El modulo custom debe marcar aprobacion requerida por descuento >20%"

  - step: 2
    name: "Solicitar aprobacion de descuento"
    action: custom_method
    model: sale.order
    method: action_request_approval
    ids: ["{result.sale_order_id}"]
    assertions:
      - field: x_approval_state
        expected: "pending"
    note: "Metodo del modulo indaws_sale_approval. Crea actividad para el aprobador."
    on_error: abort

  - step: 3
    name: "Aprobar descuento (como director comercial)"
    action: custom_method
    model: sale.order
    method: action_approve_discount
    ids: ["{result.sale_order_id}"]
    params:
      kwargs:
        approval_note: "{data.approval_note}"
    context:
      force_user_id: "{pre.approver_id}"
    assertions:
      - field: x_approval_state
        expected: "approved"
      - field: x_approved_by
        expected: "{pre.approver_id}"
    note: "Simula aprobacion con contexto force_user_id. Verificar que el modulo soporta este contexto."

  - step: 4
    name: "Confirmar pedido aprobado"
    action: confirm
    model: sale.order
    ids: ["{result.sale_order_id}"]
    wait_for: [3]
    assertions:
      - field: state
        expected: "sale"

  - step: 5
    name: "Verificar que se creo la actividad de seguimiento"
    action: search
    model: mail.activity
    domain:
      - ["res_model", "=", "sale.order"]
      - ["res_id", "=", "{result.sale_order_id}"]
    fields: ["id", "summary", "activity_type_id", "user_id"]
    save_as: activity_id
    on_error: skip
    note: "Verificacion opcional. Algunos modulos custom crean actividades post-aprobacion."

assertions:
  - model: sale.order
    id: "{result.sale_order_id}"
    field: state
    expected: "sale"
    description: "El pedido debe estar confirmado"

  - model: sale.order
    id: "{result.sale_order_id}"
    field: amount_total
    operator: ">"
    expected: 0
    description: "El importe total debe ser positivo"

  - model: sale.order
    id: "{result.sale_order_id}"
    field: x_approval_state
    expected: "approved"
    description: "La aprobacion debe constar en el registro"

revert:
  - step: 5
    action: delete
    model: mail.activity
    ids: ["{result.activity_id}"]
    note: "Solo si la actividad existe (paso 5 puede haberse saltado)"
  - step: 4
    action: execute
    model: sale.order
    method: action_cancel
    ids: ["{result.sale_order_id}"]
    context:
      disable_cancel_warning: true
  - step: 3
    action: custom_method
    model: sale.order
    method: action_reset_approval
    ids: ["{result.sale_order_id}"]
    note: "Metodo custom para limpiar datos de aprobacion"
  - step: 1
    action: delete
    model: sale.order
    ids: ["{result.sale_order_id}"]

cleanup:
  strategy: auto
  neutralize_only: false
  prefix_field_map:
    sale.order: client_order_ref
```

---

## 7. Notas de Ejecucion

### 7.1 Como el Flow Tester interpreta un archivo YAML

1. **Parseo y validacion:** El tester lee el YAML y valida la estructura contra este esquema. Los campos requeridos se comprueban, los tipos se validan y las referencias de variables se registran.

2. **Resolucion de precondiciones:** Se ejecutan las busquedas de `required_data` en orden. Cada resultado se almacena en el namespace `{pre.*}`. Si alguna busqueda retorna menos registros que `min_records`, el flujo se aborta con error descriptivo.

3. **Verificacion de modulos:** Si `required_modules` esta definido, se consulta `ir.module.module` para confirmar que todos estan instalados (`state = 'installed'`).

4. **Ejecucion de pasos:** Los pasos se ejecutan secuencialmente por numero de `step`. Para cada paso:
   - Se resuelven todas las variables (`{pre.*}`, `{result.*}`, `{data.*}`, `{var.*}`, `{today}`)
   - Se verifica `wait_for` (todos los pasos referenciados deben haber completado)
   - Se evalua `skip_if` (si la condicion se cumple, el paso se salta)
   - Se fusiona el `context` del paso con el contexto base (`tracking_disable`, `mail_create_nolog`, `mail_notrack`)
   - Se expanden las acciones de atajo (`confirm`, `cancel`, `deliver`, `invoice`, `pay`) al metodo ORM correspondiente
   - Se ejecuta la operacion via XML-RPC usando los scripts de `odoo-pilot`
   - Si `save_as` esta definido, se guarda el ID y campos clave (`name`, `amount_total`, etc.) en `{result.*}`
   - Se ejecutan las `assertions` inline del paso
   - Si una assertion falla, se aplica la politica de `on_error`

5. **Assertions finales:** Despues de ejecutar todos los pasos, se ejecutan las assertions globales de la seccion `assertions`.

6. **Reversion:** Si `cleanup.strategy` es `auto`, se ejecuta el plan de `revert` en el orden definido (que debe ser cronologico inverso). Con `sql_fallback`, si un paso de reversion ORM falla, se escala automaticamente a SQL.

### 7.2 Contexto base automatico

El tester inyecta automaticamente el siguiente contexto en **todas** las operaciones:

```yaml
context:
  tracking_disable: true
  mail_create_nolog: true
  mail_notrack: true
```

Para operaciones contables, se anade:

```yaml
context:
  skip_invoice_sync: true
```

El contexto definido en cada paso se **fusiona** con el contexto base (el paso tiene prioridad).

### 7.3 Generacion automatica del prefijo de test

Si no se define `{data.test_prefix}` en la seccion `data`, el tester genera automaticamente el prefijo con formato `[TEST-YYYYMMDD-HHMM]` y lo inyecta como `{data.test_prefix}`.

### 7.4 Conversion JSON a YAML

Los 9 ejemplos JSON existentes en `examples/` son compatibles con este esquema. La conversion directa aplica:

| JSON | YAML equivalente |
|------|------------------|
| `"values": {...}` | `data: ...` (en pasos) |
| `"verify": {"field": "state", "expected": "sale"}` | `assertions: [{field: state, expected: "sale"}]` |
| `"verify": {"field": "state", "expected_any": [...]}` | `assertions: [{field: state, expected_any: [...]}]` |
| `"method": "action_confirm"` + `"action": "execute"` | `action: execute` + `method: action_confirm` (sin cambio) |
| `"_comment": "..."` | Comentarios YAML nativos (`# ...`) |
| `"_instructions": {...}` | No necesario (este documento cubre la especificacion) |

Los atajos YAML (`confirm`, `cancel`, `deliver`, `invoice`, `pay`) son extensiones que no existen en el formato JSON. En JSON, estos se escriben explicitamente con `"action": "execute"` + `"method": "..."`.

### 7.5 Validaciones automaticas del esquema

El tester ejecuta las siguientes validaciones antes de iniciar la ejecucion:

| Validacion | Error si falla |
|-----------|----------------|
| `flow_id` sigue formato `CHAIN_*` | `SCHEMA_ERROR: flow_id must match CHAIN_<AREA>_<DESC>` |
| `target_version` es version valida | `SCHEMA_ERROR: target_version must be 14.0-19.0` |
| Cada step tiene `action` valido | `SCHEMA_ERROR: unknown action '<action>' in step <N>` |
| Variables referenciadas existen en su namespace | `SCHEMA_ERROR: unresolved variable '{pre.xyz}' in step <N>` |
| `wait_for` referencia steps existentes | `SCHEMA_ERROR: wait_for references unknown step <N>` |
| `on_error` es valor valido | `SCHEMA_ERROR: on_error must be abort, skip, or retry` |
| `operator` en assertions es valido | `SCHEMA_ERROR: unknown operator '<op>'` |
| Plan `revert` cubre todos los steps con `save_as` | `SCHEMA_WARNING: step <N> creates record but has no revert entry` |
