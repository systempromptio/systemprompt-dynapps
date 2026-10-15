# Website / eCommerce - Guia de Configuracion y Datos para Demo

## Modulos

| Modulo | Descripcion | Cuando instalar |
|--------|-------------|-----------------|
| website | Website builder | Siempre (si app seleccionada) |
| website_sale | eCommerce / Tienda online | Si vende online |
| website_sale_stock | Stock en tienda | Si muestra disponibilidad |
| website_blog | Blog | Si quiere contenido/SEO |
| website_crm | Formularios de contacto | Si capta leads desde web |
| website_slides | eLearning | Si tiene formacion online |

## Configuracion

### Ajustes Principales
- **Nombre de la web**: Configurar con nombre de la empresa del cliente
- **Idioma**: Espanol (instalar idioma si necesario)
- **Moneda y precios**: Visibles en tienda
- **Metodos de pago**: Configurar al menos transferencia bancaria
- **Metodos de envio**: Configurar al menos 1 (tarifa fija o por peso)
- **Impuestos en precios**: B2C con IVA incluido, B2B sin IVA

### Features Opcionales
- Comparacion de productos
- Wishlist
- Ratings y reviews
- Cupones y promociones

## Datos Maestros

### Paginas Web
- Pagina de inicio personalizada (minimo logo + titulo)
- Pagina "Sobre nosotros"
- Pagina de contacto con formulario

### Productos Publicados (8-12)
- Al menos 8 productos publicados (is_published=True)
- Todos con imagen y descripcion
- Precios de venta configurados
- Categorias de eCommerce (public_categ_ids) organizadas
- Al menos 2 productos con variantes (talla/color)
- 1 producto sin stock (para mostrar "Agotado")

### Categorias de eCommerce
- 3-5 categorias publicas con nombres claros
- Jerarquia si aplica (padre/hijo)

## Flujo de Negocio - Distribucion por Estado

### Pedidos Web (sale.order via website)
| Estado | % | Descripcion |
|--------|---|-------------|
| Carrito abandonado | 20% | Pendientes de completar |
| Presupuesto (pedido web) | 30% | Pedido realizado, pendiente confirmacion |
| Confirmado | 30% | Pagado/confirmado |
| Entregado | 20% | Completados |

## Tips para Demo Impactante
- Pagina de inicio atractiva con banner y productos destacados
- Mostrar el flujo completo: navegar tienda → anadir al carrito → checkout
- Tener categorias con productos suficientes para que el filtrado tenga sentido
- Mostrar la vista de administrador: como se publica un producto
- Si hay blog, tener 1-2 articulos publicados
- Mostrar analytics del sitio si hay datos

## Comandos API (odoo-pilot)

```bash
# Publicar producto en web
update_record.sh product.template TMPL_ID '{"is_published": true, "website_published": true}'

# Crear categoria eCommerce
create_record.sh product.public.category '{"name": "Categoria Demo"}'
```
