# Cuestionario: Ventas

## 1. Clientes y Contactos

### Gestion de Clientes

1. **Como gestionan actualmente a sus clientes? (CRM, hojas de calculo, otro software)**

2. **Necesitan segmentar a sus clientes? (por ejemplo, por tipo, ubicacion, industria, nivel de compra) Cuales son los criterios de segmentacion?**

3. **Gestionan contactos dentro de cada cliente (personas de contacto, cargos, responsabilidades)?**

4. **Hay campos personalizados o informacion especifica que necesiten registrar para cada cliente que no este en Odoo por defecto?**

5. **Se tiene un control de riesgo del cliente? En caso afirmativo, explica:**
   - Como se especifica el nivel de riesgo de cada cliente
   - Se deben bloquear ciertas acciones (por ejemplo no permitir facturar a un cliente que haya superado su riesgo)?
   - Quien puede modificar el riesgo?

6. **Gestionan diferentes direcciones para los clientes (facturacion, envio)?**

### Migracion de Datos (Clientes)

7. **Disponen de una base de datos de clientes existente para migrar? Se puede exportar en Excel?**

8. **Cuantos registros de clientes hay aproximadamente?**

9. **La base de datos actual requiere limpieza o normalizacion antes de la migracion?**

## 2. Productos y Tarifas

### Gestion de Productos

10. **Venden productos fisicos, servicios o ambos?**

11. **Como clasifican sus productos? (categorias, subcategorias, familias)**

12. **Necesitan gestionar variantes de productos? (talla, color, etc.). En caso afirmativo explicar cuales:**

13. **Que atributos necesitan para describir sus productos? (descripcion, codigo interno, codigo de barras, peso, dimensiones, etc.)**

14. **Gestionan numeros de serie o lotes para sus productos? (trazabilidad)**

15. **Necesitan gestionar kits de productos o packs? (productos compuestos por otros productos)**

16. **Venden productos digitales o descargables?**

17. **Hay campos personalizados o informacion especifica que necesiten registrar para cada producto que no este en Odoo por defecto?**

### Precios y Tarifas

18. **Tienen una lista de precios unica o multiples listas de precios?**

19. **Como determinan sus precios? (coste + margen, precios fijos, descuentos por volumen, precios basados en reglas)**

20. **Ofrecen descuentos? (por cliente, por producto, por cantidad, por periodo de tiempo)**

21. **Necesitan gestionar diferentes monedas?**

22. **Como gestionan los impuestos sobre las ventas? (IVA, impuestos especiales, etc.) Varian segun el cliente o la ubicacion?**

23. **Necesitan reglas de precios avanzadas? (por ejemplo, precios que cambien automaticamente segun la fecha, descuentos condicionales)**

### Migracion de Datos (Productos)

24. **Disponen de una base de datos de productos existente para migrar?**

25. **En que formato esta esta base de datos? (CSV, Excel, otro software)**

26. **Cuantos registros de productos hay aproximadamente?**

27. **La base de datos actual requiere limpieza o normalizacion antes de la migracion?**

28. **En caso de trabajar con variantes, explicar como se podria realizar la migracion:**

## 3. Presupuestos, Pedidos y Flujos de Ventas

### Proceso de Ventas

29. **Como es su proceso de ventas actual, paso a paso? Explica lo mas detallado posible el flujo de ventas actual. En caso de haber varios flujos, detalle lo maximo posible estos flujos:**

30. **Necesitan crear presupuestos/cotizaciones? Como los gestionan actualmente?**

31. **Que informacion debe incluirse en los presupuestos? Identificar aquellos datos que no vienen en el estandar de Odoo:**

32. **Requieren aprobaciones para los presupuestos antes de enviarlos al cliente?**

33. **Como convierten un presupuesto en un pedido de venta?**

34. **Necesitan gestionar pedidos recurrentes o suscripciones?**

35. **Como gestionan las facturas? (facturacion directa, facturacion por hitos, facturacion basada en entregas). Describe el proceso de facturacion:**

36. **Como gestionan los pagos? (transferencia, tarjeta de credito, PayPal, etc.)**

37. **Necesitan integrar Odoo con pasarelas de pago online?**

38. **Como gestionan las devoluciones y reembolsos? En caso de necesitar implantar un helpdesk o servicio de incidencia, indicalo:**

39. **Utilizan algun software adicional en su proceso de ventas que deba integrarse con Odoo? (ej. e-commerce, CRM externo, herramientas de marketing)**

40. **Se incluye el metodo de envio? En caso afirmativo:**
    - Cuales son los metodos de envio?
    - Se incluye un coste en los pedidos segun el envio? Es fijo o como se calcula el coste?
    - Se debe conectar con alguna empresa de transporte? En caso afirmativo, indicar con cual y si hay modulo de Odoo.

### Formatos PDF

41. **Necesitan personalizar el formato de los presupuestos, pedidos de venta y facturas? (logotipo, colores, informacion de la empresa, terminos y condiciones, etc.)**

42. **Que campos especificos deben incluirse en cada documento?**

43. **Deben tener diferentes plantillas para el presupuesto?**

44. **Adjunto los diferentes formatos de PDF que hay actualmente anotando todo aquello que es imprescindible mantener:**

### Automatizaciones

45. **Que tareas manuales en su proceso de ventas podrian automatizarse? (envio de correos, creacion de tareas, actualizacion de estados, etc.)**

46. **Necesitan reglas de negocio especificas? (por ejemplo, asignar automaticamente un vendedor a un cliente segun su ubicacion)**

### Desarrollos a Medida

47. **Tienen necesidades especificas que no esten cubiertas por la funcionalidad estandar de Odoo Ventas?**

48. **Describa cualquier desarrollo a medida que considere necesario:**

### Informes y Analisis

49. **Que informes y metricas de ventas son importantes para su negocio? (ventas por producto, ventas por vendedor, ventas por periodo, margen de beneficio, etc.)**

50. **Necesitan paneles de control personalizados para visualizar sus datos de ventas?**

<!-- VERTICAL: CONSTRUCCION - Incluir esta seccion SOLO si el proyecto usa la vertical de construccion (inBuilt) -->

## 5. Presupuestos de Obra

### Estructura y Formato

51. **Utilizan ficheros FIEBDC3 (BC3) para los presupuestos? Que programa de mediciones usan? (Presto, Arquimedes, TCQ, otro)**

52. **Como estructuran sus presupuestos de obra? (capitulos, subcapitulos, partidas, subpartidas) Cuantos niveles de jerarquia necesitan?**

53. **Las partidas llevan mediciones detalladas con dimensiones (unidades x largo x ancho x alto)? Necesitan que la cantidad se calcule automaticamente a partir de las mediciones?**

### Costes y Margenes

54. **Necesitan desglosar los costes de cada partida por categoria? (mano de obra, materiales, maquinaria, medios auxiliares, residuos) Que categorias de coste utilizan?**

55. **Aplican un porcentaje de costes indirectos (gastos generales de empresa) sobre los presupuestos? Cual es el porcentaje habitual?**

56. **Trabajan con composiciones de producto? (ejemplo: la partida "Alicatado" incluye mano de obra + material + residuos como subpartidas con cantidades vinculadas)**

### Gestion de Versiones y Visibilidad

57. **Necesitan gestionar revisiones de presupuesto? (mantener historial de versiones del presupuesto cuando hay cambios)**

58. **Hay partidas o lineas internas que no deben aparecer en el presupuesto impreso que recibe el cliente?**

### Planes de Pago y Compras

59. **Gestionan planes de cuotas o hitos de facturacion independientes de las certificaciones? (ejemplo: anticipo del 10% + cuotas mensuales)**

60. **Necesitan crear pedidos de compra o solicitudes de presupuesto a proveedores directamente desde las lineas del presupuesto de obra? Necesitan comparar ofertas de varios proveedores por partida?**

61. **Gestionan subcontrataciones desde el presupuesto? (asignar partidas completas a subcontratistas con sus costes)**

### Tipos de Pedido

62. **Diferencian tipos de pedido de venta? (ejemplo: "Obra" vs "Servicios" vs "Mantenimiento") Cada tipo tiene numeracion y reglas diferentes?**

<!-- FIN VERTICAL: CONSTRUCCION -->
