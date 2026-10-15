# Test Data Generator - Modes Detail

## Mode 1: Demo para Clientes

**Objetivo**: Preparar instancia Odoo con datos realistas del sector del cliente.

### Input requerido
1. Sector de actividad
2. Version de Odoo (14.0-19.0)
3. Aplicaciones principales (Ventas, Compras, Inventario, Fabricacion, etc.)
4. Alcance de datos:
   - **Basico**: Maestros principales (10-20 productos, 5-10 clientes, 3-5 proveedores)
   - **Completo**: Maestros + transacciones (presupuestos, pedidos, facturas, albaranes)
   - **Exhaustivo**: Todo + historicos, documentos complejos
5. Informacion especifica del cliente (opcional)

### Planificacion
1. Consultar `./references/sector_knowledge.md` para info del sector
2. Identificar modelos necesarios por app:
   - Ventas: res.partner, product.product, sale.order, sale.order.line
   - Compras: res.partner, product.product, purchase.order, purchase.order.line
   - Inventario: stock.location, stock.quant, stock.picking
   - Fabricacion: mrp.bom, mrp.production, mrp.workcenter
3. Disenar nomenclatura realista del sector
4. Establecer relaciones coherentes entre registros

## Mode 2: Testing de Modulos Custom

**Objetivo**: Generar datos para probar funcionalidades de un modulo custom.

### Input requerido
1. Ruta al documento de analisis del modulo
2. Version de Odoo
3. Alcance del testing:
   - **Funcional**: Solo casos de uso principales
   - **Completo**: Casos principales + edge cases
   - **Regresion**: Todos los escenarios incluyendo casos limite

### Planificacion
1. Leer documentacion del modulo
2. Identificar modelos custom y estandar modificados
3. Identificar campos custom
4. Identificar escenarios de prueba
5. Planificar datos de test por escenario

## Mode 3: Escenarios Personalizados

**Objetivo**: Generar datos para casos especificos (POCs, demos de funcionalidades, desarrollo).

### Input requerido
1. Descripcion del escenario
2. Version de Odoo
3. Modelos involucrados
4. Requisitos especificos

### Planificacion
1. Analizar escenario descrito
2. Identificar dependencias (orden de creacion)
3. Planificar valores especificos

## Scripts Auxiliares

### import_data.sh
Guia de importacion que lista archivos CSV en orden correcto y recomienda usar `odoo-importation`.

### cleanup_data.sh
Script que elimina registros por prefijo de external ID via `ir.model.data` search + unlink. Incluye confirmacion interactiva y codigo Python para shell de Odoo.
