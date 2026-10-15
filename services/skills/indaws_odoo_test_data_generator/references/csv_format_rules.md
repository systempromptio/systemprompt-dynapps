# Test Data Generator - CSV Format Rules

## Mandatory Format
- Separator: comma (`,`)
- Encoding: UTF-8
- First row: Column names (Odoo API names)
- External ID: Column `id` with format `__[prefix]__descriptive_name`

## External ID Prefixes
- Modo Demo: `__demo__`
- Modo Testing: `__test__`
- Modo Personalizado: `__dev__`

## Mandatory Columns per Model

### res.partner
```csv
id,name,email,phone,street,city,zip,country_id/id,is_company,customer_rank,supplier_rank
```

### product.product
```csv
id,name,default_code,type,categ_id/id,list_price,standard_price,uom_id/id,uom_po_id/id
```

### sale.order
```csv
id,partner_id/id,date_order,user_id/id,state
```

### sale.order.line
```csv
id,order_id/id,product_id/id,product_uom_qty,price_unit
```

## Relational Fields
Use `/id` notation with the external ID of the related record.

## Example of Coherent External IDs

```csv
# res_partner.csv
id,name,is_company,customer_rank
__demo__cliente_hotel_plaza,Hotel Plaza Mayor,True,1
__demo__cliente_oficinas_modernas,Oficinas Modernas S.L.,True,1

# product.product.csv
id,name,list_price
__demo__producto_sofa_valencia,Sofa Modelo Valencia 3 plazas,899.00

# sale.order.csv
id,partner_id/id,date_order,state
__demo__pedido_001,__demo__cliente_hotel_plaza,2024-01-15,sale

# sale.order.line.csv
id,order_id/id,product_id/id,product_uom_qty,price_unit
__demo__pedido_001_linea_001,__demo__pedido_001,__demo__producto_sofa_valencia,5,899.00
```

## Data Generation Principles

1. **Coherencia de negocio**: Precios razonables, cantidades logicas, fechas coherentes
2. **Nomenclatura profesional**: Nombres como los usaria una empresa real
3. **Variedad controlada**: Sin patrones obvios pero sin exceso de variedad
4. **Validaciones Odoo**: Respetar constraints (campos requeridos, formatos, rangos)

## Output Folder Structure

```
data/
+-- README_data.md          # Documentacion de los datos generados
+-- import_data.sh          # Script para importar con odoo-importation
+-- cleanup_data.sh         # Script para limpiar datos (borrar por external ID)
+-- res_partner.csv
+-- product_category.csv
+-- product_product.csv
+-- sale_order.csv          # (si aplica)
+-- sale_order_line.csv     # (si aplica)
+-- [otros modelos segun plan]
```
