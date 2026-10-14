# Mapeo de Campos Comunes

> Tabla de mapeos conocidos entre columnas tipicas de archivos de importacion y campos de Odoo.
> Organizado por modelo destino. Incluye sinonimos en ES y EN.

---

## res.partner (Contactos / Clientes / Proveedores)

| Columna comun (ES) | Columna comun (EN) | Campo Odoo | Tipo Odoo | Notas |
|--------------------|--------------------|------------|-----------|-------|
| Nombre, Razon social, Denominacion | Name, Company name | `name` | Char | **Requerido** |
| Email, Correo, Correo electronico, E-mail | Email, E-mail | `email` | Char | — |
| Telefono, Tel, Tlf | Phone, Telephone | `phone` | Char | Normalizar prefijo |
| Movil, Celular, Tel. movil | Mobile, Cell phone | `mobile` | Char | Normalizar prefijo |
| Direccion, Calle, Domicilio | Address, Street | `street` | Char | — |
| Direccion 2, Calle 2 | Address 2, Street 2 | `street2` | Char | — |
| Ciudad, Poblacion, Localidad | City, Town | `city` | Char | — |
| CP, Codigo postal, C.P. | Zip, Postal code, Zipcode | `zip` | Char | — |
| Provincia, Estado, Region | State, Province, Region | `state_id/id` | Many2one | `res.country.state` |
| Pais, Nacion | Country | `country_id/id` | Many2one | `res.country` |
| NIF, CIF, DNI, NIE, VAT, Id fiscal | Tax ID, VAT, TIN | `vat` | Char | Formato: ESB12345678 |
| Referencia, Ref, Codigo, Codigo cliente | Reference, Ref, Code | `ref` | Char | — |
| Sitio web, Web, Pagina web, URL | Website, URL | `website` | Char | — |
| Notas, Comentarios, Observaciones | Notes, Comments | `comment` | Html | — |
| Es empresa, Tipo, Empresa/Persona | Is company, Type | `is_company` | Boolean | True/False |
| Empresa padre, Empresa matriz | Parent company | `parent_id/id` | Many2one | `res.partner` |
| Comercial, Vendedor, Responsable | Salesperson, Sales rep | `user_id/id` | Many2one | `res.users` |
| Equipo de ventas, Equipo comercial | Sales team | `team_id/id` | Many2one | `crm.team` |
| Categoria, Etiqueta, Tags | Category, Tag | `category_id/id` | Many2many | `res.partner.category` |
| Idioma, Lengua | Language, Lang | `lang` | Selection | es_ES, en_US, etc. |
| Cargo, Puesto, Funcion | Job position, Title | `function` | Char | — |
| Titulo, Tratamiento | Title, Salutation | `title` | Many2one | `res.partner.title` |
| Cuenta bancaria, IBAN, Banco | Bank account, IBAN | *(via res.partner.bank)* | — | Importar aparte |

---

## product.template (Productos)

| Columna comun (ES) | Columna comun (EN) | Campo Odoo | Tipo Odoo | Notas |
|--------------------|--------------------|------------|-----------|-------|
| Nombre, Descripcion producto | Name, Product name | `name` | Char | **Requerido** |
| Referencia interna, Ref, SKU, Codigo | Internal reference, SKU | `default_code` | Char | — |
| Codigo de barras, EAN, Barcode | Barcode, EAN | `barcode` | Char | Unico |
| Precio de venta, PVP, Precio | Sales price, Price | `list_price` | Float | — |
| Coste, Precio coste, Coste unitario | Cost, Unit cost | `standard_price` | Float | — |
| Categoria, Categoria producto | Category, Product category | `categ_id/id` | Many2one | `product.category` |
| Tipo, Tipo producto | Product type | `type` | Selection | consu/service; Odoo 18+: `is_storable` |
| Descripcion, Descripcion larga | Description | `description` | Text | — |
| Descripcion venta | Sales description | `description_sale` | Text | — |
| Descripcion compra | Purchase description | `description_purchase` | Text | — |
| Puede venderse | Can be sold | `sale_ok` | Boolean | — |
| Puede comprarse | Can be purchased | `purchase_ok` | Boolean | — |
| Peso, Peso (kg) | Weight | `weight` | Float | En kg |
| Volumen | Volume | `volume` | Float | En m3 |
| Unidad de medida, UdM | Unit of measure, UoM | `uom_id/id` | Many2one | `uom.uom` |
| Impuesto cliente, IVA venta | Customer tax, Sales tax | `taxes_id/id` | Many2many | `account.tax` |
| Impuesto proveedor, IVA compra | Vendor tax, Purchase tax | `supplier_taxes_id/id` | Many2many | `account.tax` |
| Imagen, Foto | Image | `image_1920` | Binary | Base64 |
| Almacenable, Storable | Storable | `is_storable` | Boolean | Odoo 18+ |
| Proveedor | Vendor, Supplier | *(via product.supplierinfo)* | — | Importar aparte |

---

## account.move (Facturas)

| Columna comun (ES) | Columna comun (EN) | Campo Odoo | Tipo Odoo | Notas |
|--------------------|--------------------|------------|-----------|-------|
| Numero factura, Nro factura | Invoice number | `name` | Char | Autogenerado si vacio |
| Fecha factura, Fecha | Invoice date, Date | `invoice_date` | Date | YYYY-MM-DD |
| Fecha vencimiento | Due date | `invoice_date_due` | Date | YYYY-MM-DD |
| Cliente, Proveedor, Partner | Customer, Vendor, Partner | `partner_id/id` | Many2one | `res.partner` |
| Referencia, Ref | Reference | `ref` | Char | — |
| Diario | Journal | `journal_id/id` | Many2one | `account.journal` |
| Moneda | Currency | `currency_id/id` | Many2one | `res.currency` |

**Lineas de factura** (`account.move.line`):

| Columna | Campo Odoo | Tipo | Notas |
|---------|------------|------|-------|
| Producto | `product_id/id` | Many2one | `product.product` |
| Descripcion linea | `name` | Char | — |
| Cantidad | `quantity` | Float | — |
| Precio unitario | `price_unit` | Float | — |
| Impuesto | `tax_ids/id` | Many2many | `account.tax` |
| Cuenta contable | `account_id/id` | Many2one | `account.account` |

---

## sale.order (Pedidos de Venta)

| Columna comun (ES) | Columna comun (EN) | Campo Odoo | Tipo Odoo | Notas |
|--------------------|--------------------|------------|-----------|-------|
| Numero pedido | Order number | `name` | Char | Autogenerado |
| Fecha pedido | Order date | `date_order` | Datetime | — |
| Cliente | Customer | `partner_id/id` | Many2one | `res.partner` |
| Tarifa, Lista precios | Pricelist | `pricelist_id/id` | Many2one | `product.pricelist` |
| Comercial | Salesperson | `user_id/id` | Many2one | `res.users` |

**Lineas** (`sale.order.line`):

| Columna | Campo Odoo | Tipo | Notas |
|---------|------------|------|-------|
| Producto | `product_id/id` | Many2one | `product.product` |
| Descripcion | `name` | Text | — |
| Cantidad | `product_uom_qty` | Float | — |
| Precio unitario | `price_unit` | Float | — |
| Descuento (%) | `discount` | Float | — |
| Impuesto | `tax_id/id` | Many2many | `account.tax` |

---

## purchase.order (Pedidos de Compra)

| Columna comun (ES) | Columna comun (EN) | Campo Odoo | Tipo Odoo | Notas |
|--------------------|--------------------|------------|-----------|-------|
| Numero pedido | Order number | `name` | Char | Autogenerado |
| Fecha pedido | Order date | `date_order` | Datetime | — |
| Proveedor | Vendor, Supplier | `partner_id/id` | Many2one | `res.partner` |
| Fecha recepcion prevista | Expected date | `date_planned` | Datetime | — |

**Lineas** (`purchase.order.line`):

| Columna | Campo Odoo | Tipo | Notas |
|---------|------------|------|-------|
| Producto | `product_id/id` | Many2one | `product.product` |
| Descripcion | `name` | Text | — |
| Cantidad | `product_qty` | Float | — |
| Precio unitario | `price_unit` | Float | — |
| Impuesto | `taxes_id/id` | Many2many | `account.tax` |

---

## crm.lead (Oportunidades / Leads)

| Columna comun (ES) | Columna comun (EN) | Campo Odoo | Tipo Odoo | Notas |
|--------------------|--------------------|------------|-----------|-------|
| Nombre, Titulo, Asunto | Name, Title, Subject | `name` | Char | **Requerido** |
| Contacto, Cliente | Contact, Customer | `partner_id/id` | Many2one | `res.partner` |
| Email | Email | `email_from` | Char | — |
| Telefono | Phone | `phone` | Char | — |
| Ingresos esperados, Importe | Expected revenue, Amount | `expected_revenue` | Monetary | — |
| Probabilidad | Probability | `probability` | Float | 0-100 |
| Etapa | Stage | `stage_id/id` | Many2one | `crm.stage` |
| Equipo comercial | Sales team | `team_id/id` | Many2one | `crm.team` |
| Comercial | Salesperson | `user_id/id` | Many2one | `res.users` |
| Origen, Fuente | Source | `source_id/id` | Many2one | `utm.source` |

---

## Notas Generales de Mapeo

### Relaciones Many2one (campo/id)
- Siempre usar `/id` al final: `country_id/id`, `partner_id/id`
- El valor debe ser un **XML ID** valido: `base.es`, `base.EUR`
- Si no existe XML ID → buscar o crear con `__import__.*`

### Relaciones Many2many (campo/id)
- Mismo formato que Many2one: `category_id/id`, `tax_ids/id`
- Multiples valores separados por coma: `"cat_a,cat_b"`

### Campos Boolean
- Valores aceptados: `True`/`False`, `1`/`0`, `Si`/`No`, `Yes`/`No`
- Normalizar a `True`/`False` antes de importar

### Campos Date/Datetime
- Formato Odoo: `YYYY-MM-DD` (Date), `YYYY-MM-DD HH:MM:SS` (Datetime)
- Convertir desde formatos europeos (DD/MM/YYYY)

### Campos Float/Monetary
- Separador decimal: punto (`.`), NO coma
- Sin simbolo de moneda ni separador de miles
- Ejemplo: `1234.56` (no `1.234,56`)
