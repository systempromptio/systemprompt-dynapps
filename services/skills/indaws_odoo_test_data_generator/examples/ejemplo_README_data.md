# Datos de Prueba - Demo Empresa de Muebles

**Generado**: 08/02/2026
**Modo**: Demo para clientes
**Versión Odoo**: 17.0
**Prefijo External ID**: `__demo__`

---

## Descripción

Dataset de demostración para una empresa fabricante de muebles. Incluye productos típicos del sector (sofás, mesas, sillas, armarios), clientes B2B (hoteles, oficinas, restaurantes) y proveedores de materias primas.

Este dataset está diseñado para mostrar las capacidades de Odoo en los módulos de Ventas, Compras e Inventario en el contexto de una empresa de mobiliario.

---

## Contenido del Dataset

### Maestros

| Archivo | Modelo Odoo | Cantidad | Descripción |
|---------|-------------|----------|-------------|
| `res_partner.csv` | `res.partner` | 5 registros | 3 clientes B2B + 2 proveedores |
| `product_product.csv` | `product.product` | 15 registros | Productos de mobiliario (sofás, mesas, sillas, armarios) |

### Transacciones

| Archivo | Modelo Odoo | Cantidad | Descripción |
|---------|-------------|----------|-------------|
| `sale_order.csv` | `sale.order` | 8 registros | Pedidos de venta en diferentes estados |
| `sale_order_line.csv` | `sale.order.line` | 24 registros | Líneas de pedido (promedio 3 por pedido) |
| `purchase_order.csv` | `purchase.order` | 3 registros | Pedidos de compra de materias primas |

---

## Orden de Importación

⚠️ **IMPORTANTE**: Los archivos deben importarse en este orden para respetar las dependencias:

1. `res_partner.csv` - Contactos (clientes y proveedores)
2. `product_product.csv` - Productos
3. `sale_order.csv` - Pedidos de venta
4. `sale_order_line.csv` - Líneas de pedido de venta
5. `purchase_order.csv` - Pedidos de compra

---

## Cómo Importar

### Opción 1: Usando el skill odoo-importation (recomendado)

Para cada archivo CSV en el orden indicado arriba:

```bash
/odoo-importation res_partner.csv
/odoo-importation product_product.csv
/odoo-importation sale_order.csv
/odoo-importation sale_order_line.csv
/odoo-importation purchase_order.csv
```

### Opción 2: Manualmente via interfaz Odoo

1. Ve a Configuración > Técnico > Estructura de la base de datos > Importar
2. Selecciona el modelo correspondiente
3. Sube el archivo CSV
4. Mapea las columnas (debería ser automático)
5. Ejecuta la importación

---

## Nomenclatura y Coherencia

Los datos siguen las convenciones típicas del sector de mobiliario:

**Clientes**:
- Formato: [Tipo negocio] [Nombre geográfico/descriptivo]
- Ejemplos: "Hotel Plaza Mayor", "Oficinas Modernas S.L.", "Restaurante El Gourmet"

**Productos**:
- Formato: [Tipo mueble] [Modelo/Características] [Dimensiones/Material]
- Ejemplos: "Sofá Modelo Valencia 3 plazas tela gris", "Mesa comedor roble macizo 180x90 cm"

**Códigos de producto**:
- Formato: [TIPO]-[MODELO]-[VARIANTE]
- Ejemplos: SOF-VAL-3P-GRS (Sofá Valencia 3 plazas gris), MES-COM-ROB180 (Mesa comedor roble 180cm)

**Rangos de precio**:
- Sillas: 150€ - 250€
- Mesas: 800€ - 1.500€
- Sofás: 700€ - 1.200€
- Armarios: 500€ - 900€

---

## Características Especiales

- **Variedad de clientes**: Incluye hoteles (compras grandes), oficinas (pedidos medios) y restaurantes (pequeños)
- **Estados de pedidos**: Los pedidos están en diferentes estados (borrador, confirmado, enviado) para mostrar el flujo completo
- **Productos relacionados**: Las líneas de pedido usan productos coherentes con el tipo de cliente (ej: hoteles compran múltiples habitaciones de muebles)
- **Precios realistas**: Los precios están dentro de rangos de mercado para mobiliario de gama media-alta

---

## Cómo Limpiar los Datos

Para eliminar todos los registros generados (identificados por el prefijo `__demo__`):

1. Ejecuta el script `cleanup_data.sh`
2. Sigue las instrucciones en pantalla
3. El código Python eliminará todos los registros y sus external IDs

⚠️ **Advertencia**: Esta acción NO es reversible.

---

## Notas

- Los datos están optimizados para una demo de 30-45 minutos
- Se recomienda complementar con una presentación del flujo de ventas: presupuesto → pedido → entrega → factura
- Los productos tienen márgenes realistas (aproximadamente 50% sobre coste)
- Las direcciones y teléfonos son ficticios pero siguen formatos españoles válidos
