# Guías Específicas por Aplicación de Odoo

Este documento contiene las guías específicas para cada una de las 23 aplicaciones de Odoo reconocidas por el sistema de análisis.

**Uso**: Cuando analices una aplicación específica, consulta la sección correspondiente para obtener:
- Enfoque para la introducción
- Funcionalidades estándar clave a verificar
- Puntos críticos del cuestionario
- Guías de migración
- Tareas estándar del checklist

---

## Índice de Aplicaciones

1. [Configuraciones generales](#configuraciones-generales)
2. [CRM](#crm)
3. [Ventas](#ventas)
4. [Compras](#compras)
5. [Inventario](#inventario)
6. [Facturación](#facturación)
7. [Contabilidad](#contabilidad)
8. [Integraciones](#integraciones)
9. [Recursos humanos](#recursos-humanos)
10. [TPV](#tpv)
11. [Fabricación](#fabricación)
12. [Proyectos](#proyectos)
13. [Helpdesk](#helpdesk)
14. [Flota](#flota)
15. [Servicio externo](#servicio-externo)
16. [Firma](#firma)
17. [E-learning](#e-learning)
18. [Web](#web)
19. [E-commerce](#e-commerce)
20. [E-mail Marketing](#e-mail-marketing)
21. [Calidad](#calidad)
22. [Planificación](#planificación)
23. [App móvil](#app-móvil)

---

## Configuraciones generales

### Guía de Introducción

Aquí hablamos sobre la empresa con los resultado del apartado “Datos general de la empresa”. Incluye una descripción de la empresa, aquí puedes ver en su página web para obtener esa información. No indiques que lo has obtenido de la página web, sino escríbelo como si conocieras la empresa. En la descripción de la empresa intenta mencionar que productos venden, tipos de clientes y donde está ubicada la empresa. Debería tener unas 15 líneas de longitud.
También, cuando hables de los otros software con los que la empresa trabaja o el software que trabajaba antes de Odoo, añade algo de información sobre cada uno de ellos. Deberían ser unas 5 o 6 líneas por cada software.

### Funcionalidades Estándar a Verificar

Compañías:
En caso de haber más de 1 compañía, haz una descripción sobre como gestiona Odoo varias compañías dentro de una base de datos y las posibilidades que hay. Igual que para las reglas inter compañía. Si se ha indicado que sí que las hay, explica como lo hace Odoo. En caso que no lo sea, menciona que existe esta posibilidad por si se quiere hacer en el futuro.
En caso de haber más de una compañía, explica los siguientes apartados. Es importante que esto quede explicado de forma detallada:
Menciona en las compañías que hay datos que son compartidos y datos que están separados.
Por ejemplo, un cliente puede estar compartido entre empresas o no.
Las contabilidades son independientes por compañía
Los usuarios podrán ver una o varias compañías, según los permisos
Podremos ver información conjunta
Añade también capturas de pantalla de Odoo para mostrar estos puntos.
Si en el cuestionario se menciona que hay reglas intercompañía, hay que explicar en detalle cómo funcionan en Odoo. Se puede explicar de una forma detallada los pasos de una operación de compra-venta relacionada entre dos compañías.

Idiomas:
EN la parte de idiomas, indica todo lo que supone tener más de un idioma.
Menciona el riesgo de añadir un idioma más a Odoo:
Las traducciones de Odoo como aplicación son automáticas de lo ya existente, pero cada vez que se haga cualquier modificación se debe hacer en todos los idiomas
Nombres de productos, categorías, modos de pago, etc debe traducirse en los idiomas
Todos los informes de pdf deben hacerse también teniendo en cuenta las traducciones
plantillas de email también
Por tanto deja claro que un nuevo idioma aumenta el esfuerzo y los tiempos de implantación y trabajo diario



### Puntos Críticos del Cuestionario

Como puntos críticos de este apartado tenemos lo referente a las compañías y los idiomas

### Guía de Migración

No hay migraciones dentro de configuraciones generales. EN todo caso, habrán importaciones de usuarios pero se tendrán que crear de forma manual, igual que las compañías

### Tareas Estándar del Checklist

Estos serían unos ejemplos:
[Servidor] Alta de servidor de Odoo (Configuraciones) - Peso: 3
[Servidor] Alta de bases de datos de producción y staging (Configuraciones) - Peso: 1
[Servidor] Configuración de copias de seguridad (Configuraciones) - Peso: 1
[Correos] Configuración de servidor de correos de salida (Configuraciones) - Peso: 2
[Correos] Prueba de envío de email (Configuraciones) - Peso: 1
[Correos] Configuración de correos de entrada (Configuraciones) - Peso: 2
[Correos] Prueba de entrada de email (Configuraciones) - Peso: 1
[General] Alta de usuarios y permisos (Migraciones) - Peso: 2
[General] Configuración de idiomas (Configuraciones) - Peso: 1 - Indica aquí los idiomas que se deben configurar
[General] Ata de compañia [nombre de la compañía] (Configuraciones) - Peso: 1 - En caso de haber más compañías, añade un checklist por cada una
[General] Reglas entre compañías (Configuraciones) - Peso: 2 - solamente si hay

---

## CRM

### Guía de Introducción

Primero incluye una explicación de 15 líneas explicando lo que es un CRM y mencionando en este caso el CRM de Odoo y su conexión con otras aplicaciones. Debe quedar claro para que se utiliza un CRM.
Después incluye respuestas sobre las preguntas de si han utilizado previamente un CRM y cuantos comerciales lo van a gastar.
Viendo el resultado del cuestionar y con tu conocimiento del CRM, describe 5 objetivos que la empresa puede tener a la hora de gastar un CRM.

### Funcionalidades Estándar a Verificar

1) Gestión de clientes: solamente si hay un apartado de gestión de clientes. Explicaremos todo lo relacionado a los clientes

2) Comerciales y equipos de venta: Indica los comerciales que utilizarán el CRM y comenta sobre los permisos de estos. Si no dice nada sobre los permisos, entonces explica los permisos que vienen por defecto en Odoo en la aplicación de CRM y la de ventas.
Habla sobre los equipos de venta. Indica según las respuestas si hay equipos de ventas y cuales son. Describe también para que sirve tener en Odoo diferentes equipos de ventas (menciona que cada equipo de venta puede tener un pipeline diferentes y que es muy útil para segmentar, además de aquello que consideres importante a tener en cuenta)


3) Entrada de leads: comenta todo lo descrito sobre las vias de entrada de leads

4) Ficha de la oportunidad: explica la información que almacena Odoo, y en caso que lo haya, menciona sobre nuevos campos a crear. Si no hay, indica que se utilizará la ficha de oportunidad estándar.

5) Etapas del flujo de ventas: indica si hay varios pipeline, y si solo hay uno las etapas que se van a definir. Si no se han indicado las etapas, entonces muestra las etapas que vienen por defecto en Odoo. Indica que las etapas pueden cambiarse, y podrías mostrar ejemplos de etapas para el tipo de empresa que es el cliente.
Menciona si hay que realizar alguna automatización durante el proceso del CRM. Importante indicar que en el caso que haya automatizaciones (por ejemplo, enviar un email al cambiar de etapa), éstas deben ser implementadas durante la implantación

### Puntos Críticos del Cuestionario



### Guía de Migración



### Tareas Estándar del Checklist

Ejemplo de checklist:
Configuración de etapas del CRM (Configuraciones) - Peso 1
Alta de comerciales y configuración de permisos (Configuraciones) - Peso 1
Customización de ficha de oportunidad (Configuraciones) - Peso 1 - Solo si hemos indicado que la ficha del comercial debe cambiarse
Configuración de equipos de venta  (Configuraciones) - Peso 2
Importación de oportunidades (Migraciones) - Peso 2

---

## Ventas

### Guía de Introducción

Primero incluye una explicación de 15 líneas explicando la aplicación de ventas de Odoo y mencionando en este caso su conexión con otras aplicaciones. Menciona que desde la aplicación de ventas se van a definir datos maestros como los clientes, productos y tarifas, que son muy importantes ya que gran parte de los flujos dependen de cómo estén definidos.

### Funcionalidades Estándar a Verificar

Esta sección tendrá diferentes sub-apartados. Tendremos:

1) Clientes y contactos: En función de las respuestas sobre los clientes y contactos, explica los resultados del análisis y cómo se implementará en Odoo

2) Productos: En función de las respuestas sobre los productos, explica los resultados del análisis y cómo se implementará en Odoo.
Explica como funcionan las tarifas en Odoo en relación a las respuestas del cuestionario

3) Presupuestos, pedidos y flujo de ventas: En función de las respuestas en este apartado, explica los resultados del análisis y cómo se implementará en Odoo. 

### Puntos Críticos del Cuestionario

Da especial importancia a los formatos en pdf y lo relacionado con los presupuestos, pedidos y procesos de ventas

### Guía de Migración

Explica todo lo referente a la migración de datos según las respuestas del cuestionario. Deja claro los datos que se van a migrar y las dificultades de la migración. Si hay más de 10.000 clientes, avisa que son muchos datos para importar y que puede ser costoso ya que hay que normalizar algunos datos antes de poder importarlos.
Si se menciona que el cliente debe preparara datos antes de importarlos, menciona a esto como un riesgo, y comenta la posibilidad de hacer este proceso de limpieza una vez estén importados en Odoo.

### Tareas Estándar del Checklist

Ejemplos:
[Clientes] Crear campos personalizados (Configuraciones) - Incluye todos los campos nuevos en un único checklist
[Clientes] Configuración de control de riesgo de clientes (Configuraciones)
[Clientes] Migración de datos de clientes (Migraciones)
[Productos] Configuración de categorías de productos (Configuraciones)
[Productos] Configuración de gestión de números de serie/lotes (Configuraciones)
[Productos] Configuración de gestión de kits/packs de productos (Configuraciones)
[Productos] Migración de datos de productos (Migraciones)
[Tarifas] Configuración de tarifas - En función de las respuestas del cuestionario, crea tantos elementos del checklist como consideres necesario para cubrir todo lo descrito
[Ventas] Configuración de métodos de envío en presupuestos (Configuraciones)
[Ventas] Configuración de política de facturación (Configuraciones)
[Ventas] Configuración de gestión de devoluciones y reembolsos (Configuraciones)
[Ventas] Modificación de plantilla PDF de presupuesto (Desarrollo)

---

## Compras

### Guía de Introducción

Primero incluye una explicación de 15 líneas explicando la aplicación de compras de Odoo y mencionando en este caso su conexión con otras aplicaciones.

### Funcionalidades Estándar a Verificar

Utiliza las del cuestionario

### Puntos Críticos del Cuestionario

Da especial importancia a los flujos de compras

### Guía de Migración

Vamos a incluir la migración de proveedores y ver si hay que migrar precios de proveedores de productos

### Tareas Estándar del Checklist



---

## Inventario

### Guía de Introducción

Primero incluye una explicación de 15 líneas explicando la aplicación de ventas de Odoo y mencionando en este caso su conexión con otras aplicaciones. Explica que la apliación de inventarios puede servir para empresas que tienen un inventario y control de stock sencillo hasta una empresa con una gestión de inventario compleja (múltiples almacenes y ubicaciones, reglas automáticas de traslado de stock, operaciones de picking, etc)

### Funcionalidades Estándar a Verificar

Utiliza la información de los cuestionarios.
Si hay picking o múltiples ubicaciones recalca que son operaciones complejas

### Puntos Críticos del Cuestionario



### Guía de Migración

Estará el stock inicial siempre

### Tareas Estándar del Checklist

Genéralo en base al cuestionario

---

## Facturación

### Guía de Introducción

Primero incluye una explicación de 15 líneas explicando la aplicación de facturación de Odoo y mencionando en este caso su conexión con otras aplicaciones. Explica que está muy relacionada con la aplicación de contabilidad y en la mayoría de casos irá de la mano. Comenta también la vinculación con las aplicaciones de compras y de ventas.

### Funcionalidades Estándar a Verificar

Esta sección tendrá diferentes sub-apartados. Tendremos:

1) Facturación de ventas: Explica detalladamente como implantaremos en Odoo basado en las respuestas del cuestionario. Es muy importante dejar claros los flujos de ventas y las diferentes vías (según las respuestas del cuestionario) que tendremos para generar facturas de venta.
Comenta aquí también las facturas rectificativas.
En caso que haya facturas de anticipo de clientes, explícalo aquí también. Es importante dejar claro el flujo de facturas de anticipo.
También indica las secuencias.

2) Facturación de compras: Sigue la misma estructura que en el apartado anterior de ventas, pero en este caso sobre las compras.

3) Métodos de pago y cobro
Explica los métodos de pago y cobro que se han indicado en el cuestionario. Explica también los plazos de cobro y pago

4) Informes a medida
Explica los informes de facturación que vienen de base en Odoo y qué podemos obtener con ellos.
En caso de solicitar un nuevo informe, explica en detalle el nuevo informe y cómo se podría implantar en Odoo.

5) Otros: incluye respuestas de otras preguntas del cuestionario que no se hayan incluido en las respuestas anteriores.

### Puntos Críticos del Cuestionario



### Guía de Migración

Si no se indica nada de migración de facturas históricas entonces deja claro que no se ,migrarán.


### Tareas Estándar del Checklist

[Facturación Ventas] Configuración de política de facturación (Configuraciones) Peso: 1
[Facturación Ventas] Configuración de plantilla de envío de facturas por email (Configuraciones) Peso: 1
[Facturación Ventas] Instalación y configuración módulo facturae (Configuraciones) Peso: 4
[Facturación Ventas] Configuración de modos de pago de clientes (Configuraciones) Peso: 2
[Facturación Ventas] Configuración de plazos de pago de clientes (Configuraciones) Peso: 1
[Facturación Ventas] Configuración de impuestos de venta (Configuraciones) Peso: 1
[Facturación Ventas] Configuración de secuencia de facturas de venta (Configuraciones) Peso: 1
[Facturación Ventas] Configuración de secuencia de facturas rectificativas (Configuraciones) Peso: 1
[Facturación Ventas] Validación de flujo de facturación de ventas (Pruebas de flujo) Peso: 4 - En este caso, crea un checklist por cada flujo de ventas que se detecte
[Facturación Compras] Configuración de flujo de aprobación de facturas de compra (Configuraciones) Peso: 4
[Facturación Compras] Configuración de modos de pago de proveedores (Configuraciones) Peso: 2
[Facturación Compras] Configuración de plazos de pago de proveedores (Configuraciones) Peso: 1
[Facturación Compras] Configuración de impuestos de compra(Configuraciones) Peso: 1
[Facturación Compras] Validación de flujo de facturación de compras (Pruebas de flujo) Peso: 4 - En este caso, crea un checklist por cada flujo de ventas que se detecte (en compras siempre tiene que haber un flujo que es el de facturas de compras manuales, y si hay pedidos de compra habrá como mínimo uno para facturar compras)
[Facturación] Alta de usuarios del módulo de Facturación (Configuraciones) Peso: 1
[Facturación] Revisión del formato de factura (Formatos) Peso: 3


---

## Contabilidad

### Guía de Introducción

Primero incluye una explicación de 15 líneas explicando la aplicación de contabilidad de Odoo y explicando cómo funciona la contabilidad de Odoo y como está adaptada a la legislación española.

### Funcionalidades Estándar a Verificar

Haz un resumen del cuestionario explicando cómo se va a implantar en Odoo.


### Puntos Críticos del Cuestionario



### Guía de Migración

Explica todo lo referente a la migración de datos según las respuestas del cuestionario. 

Por un lado menciona que se migrará el plan contable actual, pero realizando ciertas adaptaciones. Por ejemplo, no habrá cuentas de clientes o proveedores específicas, y todos irán a la 43, 40 o 41. Explica por qué Odoo trabaja de esta forma.

Indica que se hará una migración de saldos contable a fecha de inicio de empezar a utilizar Odoo. Esto es necesario para tener todos los saldos iniciales.

Por último, indica si se van a migrar o no históricos de asientos contables.


### Tareas Estándar del Checklist

Revisión y adaptación del Plan Contable a Odoo (Configuraciones) - Peso:1
Configuración de cuentas bancarias (Configuraciones) - Peso:2
Configuración de diarios (Configuraciones) - Peso:1
Configuración de domiciliación bancaria SEPA (Configuraciones) - Peso:2
Prueba de generación de fichero SEPA xml y subida al banco (Pruebas de flujo) - Peso:3
Configuración de ficheros de transferencia de proveedores (Configuraciones) - Peso:1
Prueba de generación de fichero de transferencia a proveedores xml y subida al banco (Pruebas de flujo) - Peso:2
Configuración de multimoneda (Configuraciones) - Peso:2
Configuración de generación de Facturae (Configuraciones) - Peso:3
Configuración de gestión de activos fijos y amortizaciones (Configuraciones) - Peso:2
Configuración del Sistema de Información Inmediata (SII) (Configuraciones) - Peso:4
SII - Envío de prueba de factura de cliente, proveedores y rectificativas (Pruebas de flujo) - Peso:3
Configuración de modelos de impuestos (111, 115, 303, 347, 349, 390) (Configuraciones) - Peso:3
Migración del Plan Contable (Migraciones) - Peso:3
Migración de Saldos Contables Iniciales (Migraciones) - Peso:5
Formación estándar del módulo de Contabilidad (Formaciones) - Peso:3
Formación de conciliación bancaria (Formaciones) - Peso:3
Formación avanzada de contabilidad (Formaciones) - Peso:3


---

## Integraciones

### Guía de Introducción

Breve explicación del objetivo de la integración y del software al que conectamos


### Funcionalidades Estándar a Verificar

Explica lo más detalladamente posible todos los detalles de la integración que se va a llevar a cabo


### Puntos Críticos del Cuestionario



### Guía de Migración

Los empleados

### Tareas Estándar del Checklist



---

## Recursos humanos

### Guía de Introducción

Haz una introducción diciendo que la suite de recursos humanos está compuesta por diferentes aplicaciones: empleados, contratos, ausencias y vacaciones, asistencias o gastos

### Funcionalidades Estándar a Verificar

Utiliza la información y estructura de los cuestionarios.

### Puntos Críticos del Cuestionario



### Guía de Migración



### Tareas Estándar del Checklist



---

## TPV

### Guía de Introducción

Haz una introducción explicando la aplicación de Terminal de Punto de Venta (TPV)

### Funcionalidades Estándar a Verificar

Utiliza la información y estructura de los cuestionarios.

### Puntos Críticos del Cuestionario



### Guía de Migración

En principio no es necesario migrar nada

### Tareas Estándar del Checklist



---

## Fabricación

### Guía de Introducción

Haz una introducción explicando la aplicación de Fabricación en Odoo

### Funcionalidades Estándar a Verificar

Utiliza la información y estructura de los cuestionarios.

### Puntos Críticos del Cuestionario



### Guía de Migración

Posiblemente sea necesario migrar las listas de materiales de cada producto

### Tareas Estándar del Checklist



---

## Proyectos

### Guía de Introducción

Haz una introducción explicando la aplicación de gestión de proyectos en Odoo

### Funcionalidades Estándar a Verificar

Utiliza la información y estructura de los cuestionarios.

### Puntos Críticos del Cuestionario



### Guía de Migración

Revisa si hay que migrar los proyectos actuales

### Tareas Estándar del Checklist



---

## Helpdesk

### Guía de Introducción

Haz una introducción explicando la aplicación de Helpesk (Mesa de ayuda o servicio de asistencia en Odoo). Haz referencia solamente a la aplicación oficial que viene incluida en Odoo Enterprise

### Funcionalidades Estándar a Verificar

Utiliza la información y estructura de los cuestionarios.

### Puntos Críticos del Cuestionario



### Guía de Migración

En principio no es necesario migrar nada

### Tareas Estándar del Checklist



---

## Flota

### Guía de Introducción

Haz una introducción explicando la aplicación de gestión de flota en Odoo

### Funcionalidades Estándar a Verificar

Utiliza la información y estructura de los cuestionarios.

### Puntos Críticos del Cuestionario



### Guía de Migración

Migración de la flota actual

### Tareas Estándar del Checklist



---

## Servicio externo

### Guía de Introducción

Haz una introducción explicando la aplicación de servicio externo (field_service) en Odoo

### Funcionalidades Estándar a Verificar

Utiliza la información y estructura de los cuestionarios.

### Puntos Críticos del Cuestionario



### Guía de Migración

En principio no es necesario migrar nada

### Tareas Estándar del Checklist



---

## Firma

### Guía de Introducción

Haz una introducción explicando la aplicación de Firmas en Odoo

### Funcionalidades Estándar a Verificar

Utiliza la información y estructura de los cuestionarios.

### Puntos Críticos del Cuestionario



### Guía de Migración

En principio no es necesario migrar nada

### Tareas Estándar del Checklist



---

## E-learning

### Guía de Introducción

Haz una introducción explicando la aplicación de E-learning en Odoo

### Funcionalidades Estándar a Verificar

Utiliza la información y estructura de los cuestionarios.

### Puntos Críticos del Cuestionario



### Guía de Migración

En principio no es necesario migrar nada

### Tareas Estándar del Checklist



---

## Web

### Guía de Introducción

Haz una introducción explicando la aplicación de Página Web en Odoo

### Funcionalidades Estándar a Verificar

Utiliza la información y estructura de los cuestionarios.

### Puntos Críticos del Cuestionario



### Guía de Migración

Deja claro los datos que se van a migrar y las dificultades de la migración. Si hay más de 5.000 productos , avisa que son muchos datos para importar y que puede ser costoso ya que hay que normalizar algunos datos antes de poder importarlos.
Si se menciona que el cliente debe preparara datos antes de importarlos, menciona a esto como un riesgo, y comenta la posibilidad de hacer este proceso de limpieza una vez estén importados en Odoo.

### Tareas Estándar del Checklist



---

## E-commerce

### Guía de Introducción

Primero incluye una explicación de 15 líneas explicando lo que es un E-Commerce y mencionando en este caso el E-commerce de Odoo y su conexión con otras aplicaciones. Debe quedar claro para que se utiliza un E-Commerce.
Después incluye respuestas sobre las preguntas de si han utilizado previamente un E-commerce y los beneficios principales que va a proporcionar.
Viendo el resultado del cuestionario y con tu conocimiento del E-commerce, describe 5 objetivos que la empresa puede tener a la hora de gastar un  E-commerce.

### Funcionalidades Estándar a Verificar

Las funcionalidades estándar del módulo de e-commerce de Odoo se centran en ofrecer una solución integrada y completa para la gestión 

1) Gestión de la tienda online

Constructor de sitios web (Website Builder): Es una herramienta de arrastrar y soltar (drag and drop) que permite crear y personalizar la tienda sin necesidad de código. Incluye bloques prediseñados para productos, testimonios, sliders y más.

Diseño responsivo: Las tiendas creadas son compatibles con dispositivos móviles, tablets y ordenadores de escritorio.

Procesos de compra simplificados: El checkout está diseñado para ser rápido y sencillo, reduciendo la fricción para el cliente. Los clientes pueden comprar sin crear una cuenta.

Integración con medios de pago: Soporta múltiples pasarelas de pago, como PayPal, Stripe, Authorize.Net y Mercado Pago, facilitando las transacciones.

Gestión de productos y catálogo
Gestión de productos centralizada: Permite gestionar el inventario, las variantes de productos (talla, color) y las categorías desde un solo lugar.

Fichas de producto detalladas: Puedes añadir descripciones completas, especificaciones técnicas, múltiples imágenes, videos y reseñas de clientes.

Up-selling y cross-selling: El sistema sugiere productos complementarios o de mayor valor en la ficha de producto y en el carrito para aumentar el valor medio del pedido.

2) Marketing y SEO
Herramientas de SEO: Incluye herramientas para optimizar la tienda en buscadores, como la edición de metadatos (títulos y descripciones) y URLs amigables.

Promociones y cupones: Permite crear campañas de marketing con códigos de descuento, cupones y ofertas especiales.

Recuperación de carritos abandonados: Envía correos electrónicos automáticos a los clientes que han añadido productos al carrito pero no han completado la compra.

Integración con marketing automation: El módulo de e-commerce se conecta con el módulo de marketing de Odoo para automatizar campañas de email, SMS y otras comunicaciones.

3) Gestión de pedidos y logística

Gestión de pedidos automática: Cuando un cliente realiza una compra, el pedido se crea automáticamente en el sistema de ventas de Odoo, y el inventario se actualiza en tiempo real.

Cálculo de impuestos y gastos de envío: Calcula de forma automática los impuestos (IVA) y los costes de envío basados en la dirección del cliente y las reglas de envío configuradas.

Integración con transportistas: Se integra con los principales transportistas para gestionar l

### Puntos Críticos del Cuestionario



### Guía de Migración



### Tareas Estándar del Checklist

Configuración de la tienda online (Configuraciones) - Peso 1

Definir el nombre de la tienda, datos de contacto y la moneda principal.

Establecer las reglas básicas de impuestos y unidades de medida.

Configurar los transportistas y las tarifas de envío por peso, zona o precio.

Diseño y personalización de la interfaz (Configuraciones) - Peso 2

Utilizar el constructor de sitios web para diseñar la página de inicio, las páginas de producto y otras páginas estáticas (Ej. 'Sobre Nosotros', 'Política de Privacidad').

Personalizar la paleta de colores, fuentes y el logo para que coincidan con la identidad de la marca.

Ajustar el menú de navegación y el pie de página.

Configuración de métodos de pago (Configuraciones) - Peso 2

Integrar y configurar las pasarelas de pago online (como Stripe, PayPal, etc.).

Configurar opciones de pago offline si es necesario (ej. transferencia bancaria).

Configuración de variantes de producto (Configuraciones) - Peso 1

Crear los atributos de producto (talla, color, material) y sus valores.

Configurar las variantes que se mostrarán en la ficha de producto.

Tareas de Gestión de Contenido y Catálogo
Creación de categorías de producto (Contenido) - Peso 1

Diseñar y crear la estructura de categorías y subcategorías que se utilizará en la tienda.

Alta de productos (Contenido) - Peso 2

Crear las fichas de los productos principales, con descripciones, imágenes y precios.

Optimización SEO de fichas de producto y páginas (Contenido) - Peso 1

Añadir metadatos (título y descripción) y palabras clave relevantes a cada producto y página para mejorar el posicionamiento en Google.

Generar URLs amigables.

Tareas de Integración y Marketing
Configuración de promociones y descuentos (Marketing) - Peso 1

Crear reglas de precios y promociones como cupones de descuento, "2x1", etc.

Configuración de email marketing (Marketing) - Peso 2

Diseñar plantillas para correos de confirmación de pedido, envío y recuperación de carritos abandonados.

Automatizar el envío de estos correos.

Integración con plataformas de redes sociales y Google Analytics (Marketing) - Peso 1

Configurar los píxeles de seguimiento para analizar el comportamiento del usuario y medir el rendimiento de las campañas.

Tareas de Migración (si aplica)
Migración de productos e inventario (Migraciones) - Peso 3

Importar masivamente el catálogo de productos, junto con sus variantes, precios y stock.

Este proceso es crítico y puede requerir scripts o herramientas de importación especializadas.

Migración de clientes (Migraciones) - Peso 2

Importar la base de datos de clientes existente.

Migración de pedidos históricos (Migraciones) - Peso 3



---

## E-mail Marketing

### Guía de Introducción

Haz una introducción explicando la aplicación de email marketing en Odoo

### Funcionalidades Estándar a Verificar

Utiliza la información y estructura de los cuestionarios.

### Puntos Críticos del Cuestionario



### Guía de Migración

En principio no es necesario migrar nada

### Tareas Estándar del Checklist



---

## Calidad

### Guía de Introducción

Haz una introducción explicando la aplicación de calidad en Odoo

### Funcionalidades Estándar a Verificar

Utiliza la información y estructura de los cuestionarios.

### Puntos Críticos del Cuestionario



### Guía de Migración

En principio no es necesario migrar nada

### Tareas Estándar del Checklist



---

## Planificación

### Guía de Introducción

Haz una introducción explicando la aplicación de planificación en Odoo

### Funcionalidades Estándar a Verificar

Utiliza la información y estructura de los cuestionarios.

### Puntos Críticos del Cuestionario



### Guía de Migración

En principio no es necesario migrar nada

### Tareas Estándar del Checklist



---

## App móvil

### Guía de Introducción

Explica la aplicación móvil con un texto similar al siguiente, que puedes mejorar con los resultados del cuestionario: 

Explicación de la Aplicación (App)
Una aplicación de gestión de operaciones es el centro de control digital para equipos que trabajan fuera de la oficina. Su propósito es reemplazar procesos manuales como partes en papel, llamadas y mensajes, centralizando toda la información en un solo lugar. Permite asignar tareas con instrucciones claras y checklists, asegurando que el trabajo se realice siempre con el mismo estándar de calidad. Además, facilita el control horario de los trabajadores directamente desde su móvil, registrando su jornada de forma precisa y legal.

Aplicaciones profesionales como INdaws o la que se propone para Minusval no son islas; su verdadero poder reside en su capacidad para conectarse con otros sistemas. Por ejemplo, los datos del control horario se integran con el software de nóminas para automatizar el cálculo de salarios, las tareas finalizadas se conectan con el sistema de facturación para generar facturas a clientes, y el módulo de materiales se enlaza directamente con el e-commerce interno. De este modo, la app se convierte en el motor que digitaliza y optimiza toda la operación de la empresa, desde el trabajador en la calle hasta la administración en la oficina.

Después incluye respuestas sobre las preguntas de si han utilizado previamente un APP móvil y cuantos usuarios lo van a gastar.
Viendo el resultado del cuestionario y con tu conocimiento del APPs, describe 5 objetivos que la empresa puede tener a la hora de gastar un APP

### Funcionalidades Estándar a Verificar

Funcionalidades Estándar de la Aplicación
Módulo 1: Gestión de Tareas y Partes de Trabajo Digitales
Asignación de Tareas y Planificación:

Permite a los administradores o supervisores crear servicios de limpieza, añadir toda la información relevante (cliente, ubicación, instrucciones) y asignarlos a un trabajador o equipo específico.

Checklists (Listas de Verificación) por Tarea:

Cada tarea incluye una lista de pasos o puntos a verificar por el operario. Esto estandariza la calidad del servicio y asegura que no se olvide ninguna labor.

Reporte de Estado y Finalización:

El trabajador puede iniciar, pausar y finalizar tareas desde el móvil. Al terminar, puede adjuntar evidencias como fotografías (del antes y después), añadir comentarios y capturar la firma digital del cliente.

Gestión de Incidencias:

Una función para que el operario pueda notificar en tiempo real cualquier problema o imprevisto durante el servicio (ej. falta de material, un equipo averiado, etc.), adjuntando fotos si es necesario.

Panel de Supervisión:

Una vista para los supervisores que muestra el estado de todas las tareas en tiempo real (pendientes, en progreso, completadas), permitiendo un control total de las operaciones diarias.

Módulo 2: Control Horario y Gestión de Asistencia
Fichaje de Entrada y Salida con Geolocalización:

El trabajador registra el inicio y fin de su jornada laboral con un solo clic. El sistema guarda la hora y la ubicación GPS para verificar la presencia en el centro de trabajo.

Gestión de Pausas:

Permite registrar las pausas reglamentarias durante la jornada, asegurando un cómputo preciso de las horas trabajadas.

Solicitud de Ausencias:

Los empleados pueden solicitar días de vacaciones, permisos o notificar bajas médicas directamente desde la app. Los supervisores reciben la solicitud para su aprobación.

Generación de Informes de Jornada:

El sistema recopila todos los fichajes y genera automáticamente los informes de registro horario exigidos por la legislación laboral, listos para ser consultados o exportados.



### Puntos Críticos del Cuestionario



### Guía de Migración



### Tareas Estándar del Checklist

Fase 1: Planificación y Diseño UX/UI
Definición del Alcance Mínimo Viable (MVP) y Fases del Proyecto (Gestión) - Peso 4

Creación de Wireframes y Flujos de Navegación de la App (Diseño UX/UI) - Peso 4

Diseño Visual de la Interfaz (Mockups) para la App y la Plataforma Web (Diseño UX/UI) - Peso 4

Fase 2: Configuración e Infraestructura (Backend)
Diseño de la Arquitectura de la Base de Datos (Backend) - Peso 4

Configuración del Servidor y despliegue de la API (Backend) - Peso 5

Implementación del Sistema de Roles y Permisos (Operario, Supervisor, Admin) (Backend) - Peso 3

Fase 3: Desarrollo de Funcionalidades Principales
Módulo de Tareas:

Desarrollo del listado y detalle de tareas en la App (Desarrollo App) - Peso 3

Implementación de Checklists interactivas (Desarrollo App) - Peso 3

Funcionalidad para adjuntar Fotos y capturar Firma del cliente (Desarrollo App) - Peso 3

Creación y gestión completa de Tareas desde la Plataforma Web (Desarrollo Web) - Peso 4

Panel de Supervisión en tiempo real para el Supervisor (Desarrollo Web) - Peso 4

Módulo de Asistencia:

Lógica de Fichaje (Entrada/Salida/Pausa) con Geolocalización (Desarrollo App) - Peso 4

Módulo de Solicitud de Ausencias (Vacaciones/Bajas) en la App (Desarrollo App) - Peso 3

Sistema de Aprobación de Ausencias para el Supervisor (Desarrollo Web) - Peso 3

Generación de Informes de Jornada Laboral (PDF/Excel) (Desarrollo Web) - Peso 4

Funcionalidades Generales:

Sistema de Login y autenticación de usuarios (Desarrollo App & Web) - Peso 3

Implementación de Notificaciones Push para eventos clave (Desarrollo App) - Peso 3

Desarrollo del Modo Offline y lógica de sincronización (Desarrollo App) - Peso 5 - Nota: Es una de las funcionalidades más complejas y de mayor impacto en el coste.

Integración con E-commerce mediante Vista Web y SSO (Single Sign-On) (Desarrollo App) - Peso 3

Fase 4: Pruebas, Despliegue y Formación
Creación y ejecución del Plan de Pruebas Funcionales (Calidad/QA) - Peso 4

Pruebas de Usabilidad con un grupo piloto de trabajadores de Minusval (Calidad/QA) - Peso 3

Publicación de la App en Google Play Store y Apple App Store (Despliegue) - Peso 3 - Nota: Depende de los tiempos de revisión de las tiendas.

Creación de la documentación y manuales de usuario (Documentación) - Peso 2

Sesiones de Formación para los equipos (Operarios y Supervisore

---

