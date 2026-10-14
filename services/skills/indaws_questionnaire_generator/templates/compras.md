# Cuestionario: Compras

## 1. Proveedores y Contactos

### Gestion de Proveedores

1. **Como gestionan actualmente a sus proveedores? (ERP antiguo, hojas de calculo, otro software)**

2. **Necesitan clasificar o segmentar a sus proveedores? (Ej: por tipo de producto, ubicacion, volumen de compra, criticidad) Cuales son los criterios de segmentacion?**

3. **Hay campos personalizados o informacion especifica que necesiten registrar para cada proveedor que no este contemplada por defecto en Odoo (Ej: Certificaciones, evaluaciones de desempeno, condiciones logisticas especiales)?**

### Migracion de Datos (Proveedores)

4. **Disponen de una base de datos de proveedores existente para migrar? Se puede exportar en formato CSV/Excel?**

5. **Cuantos registros de proveedores hay aproximadamente?**

6. **La base de datos actual requiere limpieza, deduplicacion o normalizacion antes de la migracion?**

## 2. Productos y Listas de Precios de Compra

### Gestion de Productos (Perspectiva de Compra)

7. **Que tipos de articulos compran principalmente? (Productos fisicos para stock, productos para proyectos especificos, servicios, consumibles, activos fijos)**

8. **Que atributos especificos de compra necesitan para sus productos? (Ej: Codigo de producto del proveedor, Nombre del producto del proveedor, Plazo de entrega estandar del proveedor, Unidad de medida de compra si difiere de la de stock/venta, Multiplo de compra, Cantidad minima de pedido - MOQ)**

9. **Necesitan gestionar numeros de serie o lotes para los productos recibidos de proveedores? Desde que momento se necesita la trazabilidad?**

10. **Hay campos personalizados o informacion especifica que necesiten registrar para cada producto desde la perspectiva de compra?**

### Precios y Listas de Precios de Proveedor

11. **Como gestionan los precios de compra? (Precio fijo por producto/proveedor, listas de precios de proveedor, descuentos por volumen, precios negociados por contrato)**

12. **Utilizan la funcionalidad de "Listas de Precios de Proveedor" de Odoo? Describan como las gestionan actualmente:**

13. **Las listas de precios tienen fechas de validez?**

14. **Dependen de cantidades minimas?**

15. **Incluyen el plazo de entrega especifico para ese precio/producto/proveedor?**

16. **Necesitan gestionar diferentes monedas en sus compras?**

17. **Necesitan gestionar recargos de equivalencia u otros regimenes fiscales especiales en las compras?**

### Migracion de Datos (Productos y Precios de Compra)

18. **Disponen de una base de datos de productos con informacion relevante para compras (codigos de proveedor, UdM de compra, etc.)? Formato?**

19. **Disponen de listas de precios de proveedor en formato digital (CSV, Excel) para migrar?**

## 3. Flujo de Compras (Solicitudes, Pedidos)

### Proceso de Compra

20. **Describan su proceso de compra actual, paso a paso y lo mas detallado posible, desde la deteccion de la necesidad hasta la confirmacion del pedido al proveedor. Si existen diferentes flujos (ej: compras de stock vs. compras para proyecto vs. compras de servicios), detallen cada uno:**

21. **Existe un proceso formal de "Solicitud de Compra" interna antes de contactar al proveedor? (Purchase Requisition) Quien las genera? Requieren aprobacion? Basada en que criterios (importe, tipo de producto, departamento)?**

22. **Utilizan "Solicitudes de Presupuesto" (RfQ - Request for Quotation) para comparar ofertas de varios proveedores?**

23. **Como gestionan el envio de RfQs a proveedores?**

24. **Como comparan las respuestas y seleccionan al proveedor?**

25. **Que informacion debe incluirse obligatoriamente en las Solicitudes de Presupuesto y Pedidos de Compra? Identifiquen datos clave que no vengan en el estandar de Odoo:**

26. **Requieren flujos de aprobacion para los Pedidos de Compra antes de enviarlos al proveedor?**
    - Quien aprueba?
    - Cuales son los criterios de aprobacion (importe total, tipo de producto, proveedor especifico, presupuesto asociado)?
    - Necesitan multiples niveles de aprobacion?

27. **Como confirman los pedidos a los proveedores actualmente (email, portal, EDI)?**

28. **Gestionan "Acuerdos de Compra" (Purchase Agreements / Blanket Orders) para compras recurrentes o volumenes pactados durante un periodo? Describan como funcionan:**

29. **Como gestionan las modificaciones o cancelaciones de pedidos de compra una vez enviados?**

### Formatos PDF

30. **Necesitan personalizar el formato de los documentos de Solicitud de Presupuesto (RfQ) y Pedido de Compra (PO)? (Logotipo, informacion de la empresa, terminos y condiciones de compra, etc.)**

31. **Que campos especificos deben incluirse en cada documento?**

32. **Adjunten ejemplos de sus formatos actuales si es posible, indicando los elementos imprescindibles:**

### Automatizaciones y Reglas de Negocio

33. **Que tareas manuales en su proceso de compra desearian automatizar? (Ej: Creacion automatica de RfQs basadas en reglas de reposicion, notificaciones de aprobacion, recordatorios de entrega)**

34. **Necesitan reglas de negocio especificas? (Ej: Asignar automaticamente un comprador segun el tipo de producto, seleccionar proveedor por defecto para ciertos productos, bloquear compras a proveedores no homologados)**

## 4. Recepcion de Mercancia y Servicios

### Costes Indirectos (Landed Costs)

35. **Necesitan anadir costes adicionales (transporte, aduanas, seguros, aranceles) al coste de los productos comprados?**

36. **Como se calculan y asignan estos costes actualmente? (Ej: por peso, volumen, valor, cantidad, manualmente)**

37. **Cuando se conocen estos costes (en el momento de la compra, en la recepcion, al recibir la factura del transportista)?**

### Devoluciones a Proveedor

38. **Cual es el proceso para devolver productos a los proveedores (RTV - Return to Vendor)?**

39. **Cuales son los motivos habituales de devolucion?**

40. **Se requiere alguna aprobacion interna o documentacion especifica (ej: nota de debito)?**

## 6. Informes y Analisis de Compras

41. **Que informes y metricas de compras son cruciales para su negocio? (Ej: Gasto por proveedor, Gasto por categoria de producto, Ahorros conseguidos (Purchase Price Variance - PPV), Evaluacion del rendimiento de proveedores (On-Time Delivery - OTD), Calidad de recepcion, Dias de pago a proveedores - DPO, Volumen de compras por comprador)**

## 7. Desarrollos a Medida e Integraciones

### Desarrollos Especificos

42. **Anticipan necesidades especificas en el flujo de compras que no esten cubiertas por la funcionalidad estandar de Odoo Compras? Describanlas:**

### Integraciones

43. **Necesitan integrar Odoo Compras con otros sistemas? (Ej: Portal de proveedores, sistema de gestion de almacenes (SGA/WMS) externo, herramientas de planificacion (MRP/APS), sistemas de e-procurement, EDI con proveedores)**

## 8. Roles, Permisos y Formacion

### Roles de Usuario

44. **Definan los diferentes roles de usuario que interactuaran con el modulo de Compras (Ej: Solicitante, Comprador, Aprobador de Compras, Receptor de Almacen, Aprobador de Facturas de Proveedor, Administrador de Compras):**

45. **Necesitan permisos especificos o restricciones de acceso mas alla de los roles estandar de Odoo (Ej: restringir que compradores pueden comprar ciertos tipos de productos, limitar la visibilidad de precios entre usuarios)?**

### Formacion

46. **Cuantos usuarios aproximadamente utilizaran la aplicacion de Compras?**

<!-- VERTICAL: CONSTRUCCION - Incluir esta seccion SOLO si el proyecto usa la vertical de construccion (inBuilt) -->

## 9. Compras de Obra

### Vinculacion a Obras

47. **Necesitan vincular las compras a una obra o proyecto especifico para controlar costes por obra?**

48. **Gestionan envio de materiales directamente a obra? Tienen varias obras simultaneas con diferentes ubicaciones de entrega?**

### Aprovisionamiento Automatico

49. **Necesitan crear pedidos de compra automaticamente a partir del listado de materiales necesarios para una tarea de obra?**

### Stock y Transferencias

50. **Gestionan stock propio que se transfiere entre almacen central y obras? O todo se compra directamente para cada obra?**

<!-- FIN VERTICAL: CONSTRUCCION -->
