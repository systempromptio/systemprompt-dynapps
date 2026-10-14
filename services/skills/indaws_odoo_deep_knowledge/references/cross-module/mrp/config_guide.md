# MRP Configuration Guide — Odoo 18.0

## MCG1. Enable Manufacturing
**Path:** `Inventario → Configuración → Almacenes → [Almacén]`
**Steps:** 1) Marcar **"Manufacture to Resupply"** 2) Seleccionar **Manufacture:** 1/2/3 pasos 3) Guardar
**Verify:** Picking type "Manufacturing" + ruta "Manufacture" creados en `Tipos de operación` y `Rutas`.

## MCG2. Configure Product for Manufacturing
**Path:** `Inventario → Productos → [Producto] → pestaña Inventario`
**Steps:** 1) Routes → activar **"Manufacture"** 2) Smart button "Bill of Materials" → Nuevo → BoM Type: "Manufacture this Product", Quantity base, añadir Components, (opcional) Operations, By-Products con `cost_share`
**Verify:** Smart button "Bill of Materials" count > 0. **Gotcha:** MG2 — BOM tipo "Kit" no crea MO.

## MCG3. Configure Kit (Phantom BOM)
**Path:** `Fabricación → Productos → Bills of Materials → Nuevo`
**Steps:** 1) Crear producto "Kit" (type=Goods, Track Inventory=off recomendado) 2) BOM: BoM Type **"Kit"**, Components con cantidades
**Verify:** SO confirmado → delivery muestra componentes, no el kit. No se crea MO. **Gotcha:** MG9 — cost share no configurado → todo el coste va a un componente. MG2 — kit se explota al confirmar move.

## MCG4. Create Manufacturing Order
**Path:** `Fabricación → Operaciones → Manufacturing Orders → Nuevo`
**Steps:** 1) Seleccionar Product (BOM auto-rellena) 2) Ajustar Quantity 3) **Confirm** 4) (Si 2/3 step) validar picking componentes 5) Registrar producción → **Mark as Done**
**Verify:** Estado Confirmed → Progress → Done. Stock producto terminado sube, componentes baja. **Gotcha:** MG13 — MO en "Waiting" = picking componentes no validado.

## MCG5. Configure Subcontracting (Basic)
**Path:** `Fabricación → Configuración → Ajustes`
**Steps:** 1) Activar **"Subcontracting"** → Guardar 2) Crear BOM: BoM Type **"Subcontracting"**, Subcontractors=[vendor], (opcional) Components 3) Producto: pestaña Purchase → vendor con precio, Routes → "Buy" 4) Crear PO al subcontratista → Confirmar → Recibir
**Verify:** PO confirmado → smart button "Manufacturing". Receipt con `is_subcontract=True`. **Gotcha:** MG7 — BOM subcontratación NO permite operaciones ni byproducts. MG8 — location cambia silenciosamente a Subcontracting Location.

## MCG6. Configure Component Supply to Subcontractor
**Path:** `Inventario → Configuración → Almacenes → [Almacén]`
**Steps:** 1) Marcar **"Resupply Subcontractors"** → Guardar 2) En componentes: Routes → activar **"Resupply Subcontractor on Order"** 3) Crear PO al subcontratista → Confirmar
**Verify:** PO confirmado → smart button "Transfers" muestra picking de componentes Stock → Subcontracting Location.

## MCG7. Configure Multi-Level BOM (Sub-assemblies)
**Path:** Producto sub-ensamblaje + producto final
**Steps:** 1) Sub-ensamblaje: crear producto + BOM tipo "Manufacture", Routes="Manufacture", reordering rule 0/0/1 (recomendado sobre MTO) 2) Producto final: BOM incluyendo sub-ensamblaje como componente 3) Confirmar MO final → si sub-ensamblaje sin stock → reordering rule crea MO automáticamente
**Verify:** MO final muestra sub-ensamblaje como componente. Si stock insuficiente → MO sub-ensamblaje creada. **Gotcha:** MG12 — con MTO, stock sub-ensamblaje queda reservado; si cancelas MO padre, queda bloqueado.

## MCG8. Configure Byproducts
**Path:** `Fabricación → Configuración → Ajustes`
**Steps:** 1) Activar **"By-Products"** → Guardar 2) En BOM → pestaña "By-Products" → añadir: Product, Quantity, UoM, (opcional) Operation, **Cost Share (%)**
**Verify:** MO done → stock byproduct aumenta. Valuation: byproduct coste = total x cost_share%. **Gotcha:** MG10 — sin cost_share → byproduct entra a coste 0. Suma cost_share principal + byproducts debe = 100%.

## MCG9. Configure BOM per Warehouse
**Path:** `Fabricación → Productos → Bills of Materials`
**Steps:** 1) BOM Almacén Madrid: Operation Type = "Madrid/Manufacturing" 2) BOM Almacén Barcelona: Operation Type = "Barcelona/Manufacturing" 3) BOM fallback: Operation Type vacío
**Verify:** MO en Madrid usa BOM de Madrid. MO en almacén sin BOM específica usa fallback. `_bom_find()` busca específica primero.

## MCG10. Unbuild (Disassembly)
**Path:** `Fabricación → Operaciones → Unbuild Orders → Nuevo`
**Steps:** 1) Product + Bill of Material (auto-rellena) 2) Quantity 3) (Opcional) Manufacturing Order original + Lot/Serial Number 4) Click **Unbuild**
**Verify:** Stock producto disminuye, stock componentes aumenta proporcionalmente. Si hay lotes: mismos lotes originales.

## MCG11. Configure Manufacturing Steps (2/3 step)
**Path:** `Inventario → Configuración → Almacenes → [Almacén]`
**Steps:** 1) Campo **Manufacture:** "Pick components and then manufacture" (2-step) o "Pick components, manufacture and then store products" (3-step)
**Verify:** MO confirmada → smart button "Transfers" muestra pickings adicionales (Pick Components, Store Finished Products). **Gotcha:** MG13 — MO en "Waiting" = picking componentes no validado.
